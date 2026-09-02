use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use futures::{FutureExt, StreamExt, channel::mpsc::UnboundedReceiver, pin_mut, select};
use zeromq::{PubSocket, RepSocket, RouterSocket, ZmqMessage, prelude::*};

use super::{
    commands::{
        NativePubSocketCommand, NativeRepSocketCommand, NativeRouterSocketCommand,
        ZeroMqLifecycleCommand,
    },
    message::{frames_to_zmq_message, zmq_message_to_frames},
};

pub(super) async fn run_router_socket_worker(
    mut command_rx: UnboundedReceiver<NativeRouterSocketCommand>,
    incoming: Arc<Mutex<VecDeque<Vec<Vec<u8>>>>>,
) {
    let mut socket = RouterSocket::new();
    if !prepare_router_socket(&mut socket, &mut command_rx).await {
        return;
    }
    let (mut send_half, mut recv_half) = socket.split();

    loop {
        let next_command = command_rx.next().fuse();
        let next_message = recv_half.recv().fuse();
        pin_mut!(next_command, next_message);

        select! {
            command = next_command => {
                match command {
                    Some(NativeRouterSocketCommand::Send(frames)) => {
                        let mut delivered = false;
                        for _ in 0..50 {
                            if send_half
                                .send(frames_to_zmq_message(frames.clone()))
                                .await
                                .is_ok()
                            {
                                delivered = true;
                                break;
                            }
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        }
                        if !delivered {
                            break;
                        }
                    }
                    Some(NativeRouterSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Start { reply })) => {
                        let _ = reply.send(());
                    }
                    Some(NativeRouterSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Bind { reply, .. })) => {
                        let _ = reply.send(Err("router socket cannot be rebound after it starts receiving".to_string()));
                    }
                    Some(NativeRouterSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Connect { reply, .. })) => {
                        let _ = reply.send(Err("router socket cannot be rebound after it starts receiving".to_string()));
                    }
                    None => break,
                }
            }
            message = next_message => {
                match message {
                    Ok(message) => {
                        incoming
                            .lock()
                            .expect("router incoming queue should not be poisoned")
                            .push_back(zmq_message_to_frames(message));
                    }
                    Err(_) => break,
                }
            }
        }
    }
}

async fn prepare_router_socket(
    socket: &mut RouterSocket,
    command_rx: &mut UnboundedReceiver<NativeRouterSocketCommand>,
) -> bool {
    while let Some(command) = command_rx.next().await {
        match command {
            NativeRouterSocketCommand::Lifecycle(control) => {
                if handle_lifecycle_command(socket, control).await {
                    return true;
                }
            }
            NativeRouterSocketCommand::Send(_) => {}
        }
    }

    false
}

pub(super) async fn run_pub_socket_worker(
    mut command_rx: UnboundedReceiver<NativePubSocketCommand>,
) {
    let mut socket = PubSocket::new();
    if !prepare_pub_socket(&mut socket, &mut command_rx).await {
        return;
    }

    while let Some(command) = command_rx.next().await {
        match command {
            NativePubSocketCommand::Send(frames) => {
                if socket.send(frames_to_zmq_message(frames)).await.is_err() {
                    break;
                }
            }
            NativePubSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Start { reply }) => {
                let _ = reply.send(());
            }
            NativePubSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Bind { reply, .. }) => {
                let _ = reply.send(Err(
                    "pub socket cannot be rebound after it starts sending".to_string()
                ));
            }
            NativePubSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Connect {
                reply, ..
            }) => {
                let _ = reply.send(Err(
                    "pub socket cannot be rebound after it starts sending".to_string()
                ));
            }
        }
    }
}

async fn prepare_pub_socket(
    socket: &mut PubSocket,
    command_rx: &mut UnboundedReceiver<NativePubSocketCommand>,
) -> bool {
    while let Some(command) = command_rx.next().await {
        match command {
            NativePubSocketCommand::Lifecycle(control) => {
                if handle_lifecycle_command(socket, control).await {
                    return true;
                }
            }
            NativePubSocketCommand::Send(_) => {}
        }
    }

    false
}

pub(super) async fn run_rep_socket_worker(
    mut command_rx: UnboundedReceiver<NativeRepSocketCommand>,
    incoming: Arc<Mutex<VecDeque<Vec<Vec<u8>>>>>,
) {
    let mut socket = RepSocket::new();
    if !prepare_rep_socket(&mut socket, &mut command_rx).await {
        return;
    }

    loop {
        enum RepPhase1 {
            Continue,
            Stop,
            Request(ZmqMessage),
        }

        let phase = {
            let next_command = command_rx.next().fuse();
            let next_message = socket.recv().fuse();
            pin_mut!(next_command, next_message);

            select! {
                command = next_command => {
                    match command {
                        Some(NativeRepSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Start { reply })) => {
                            let _ = reply.send(());
                            RepPhase1::Continue
                        }
                        Some(NativeRepSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Bind { reply, .. })) => {
                            let _ = reply.send(Err("rep socket cannot be rebound after it starts receiving".to_string()));
                            RepPhase1::Continue
                        }
                        Some(NativeRepSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Connect { reply, .. })) => {
                            let _ = reply.send(Err("rep socket cannot be rebound after it starts receiving".to_string()));
                            RepPhase1::Continue
                        }
                        Some(NativeRepSocketCommand::Send(_)) => RepPhase1::Continue,
                        None => RepPhase1::Stop,
                    }
                }
                message = next_message => {
                    match message {
                        Ok(message) => RepPhase1::Request(message),
                        Err(_) => RepPhase1::Stop,
                    }
                }
            }
        };

        let request = match phase {
            RepPhase1::Continue => continue,
            RepPhase1::Stop => return,
            RepPhase1::Request(request) => request,
        };

        incoming
            .lock()
            .expect("rep incoming queue should not be poisoned")
            .push_back(zmq_message_to_frames(request));

        loop {
            match command_rx.next().await {
                Some(NativeRepSocketCommand::Send(frames)) => {
                    if socket.send(frames_to_zmq_message(frames)).await.is_err() {
                        return;
                    }
                    break;
                }
                Some(NativeRepSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Start {
                    reply,
                })) => {
                    let _ = reply.send(());
                }
                Some(NativeRepSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Bind {
                    reply,
                    ..
                })) => {
                    let _ = reply.send(Err(
                        "rep socket cannot be rebound after it starts receiving".to_string(),
                    ));
                }
                Some(NativeRepSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Connect {
                    reply,
                    ..
                })) => {
                    let _ = reply.send(Err(
                        "rep socket cannot be rebound after it starts receiving".to_string(),
                    ));
                }
                None => return,
            }
        }
    }
}

async fn prepare_rep_socket(
    socket: &mut RepSocket,
    command_rx: &mut UnboundedReceiver<NativeRepSocketCommand>,
) -> bool {
    while let Some(command) = command_rx.next().await {
        match command {
            NativeRepSocketCommand::Lifecycle(control) => {
                if handle_lifecycle_command(socket, control).await {
                    return true;
                }
            }
            NativeRepSocketCommand::Send(_) => {}
        }
    }

    false
}

async fn handle_lifecycle_command<S: zeromq::Socket>(
    socket: &mut S,
    command: ZeroMqLifecycleCommand,
) -> bool {
    match command {
        ZeroMqLifecycleCommand::Bind { endpoint, reply } => {
            let _ = reply.send(
                socket
                    .bind(&endpoint)
                    .await
                    .map(|endpoint| endpoint.to_string())
                    .map_err(|error| error.to_string()),
            );
            false
        }
        ZeroMqLifecycleCommand::Connect { endpoint, reply } => {
            let _ = reply.send(
                socket
                    .connect(&endpoint)
                    .await
                    .map_err(|error| error.to_string()),
            );
            false
        }
        ZeroMqLifecycleCommand::Start { reply } => {
            let _ = reply.send(());
            true
        }
    }
}
