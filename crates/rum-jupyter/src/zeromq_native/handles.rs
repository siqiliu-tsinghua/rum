use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, mpsc::sync_channel},
    thread::JoinHandle,
};

use futures::channel::mpsc::{UnboundedSender, unbounded};

use super::{
    commands::{
        NativePubSocketCommand, NativeRepSocketCommand, NativeRouterSocketCommand,
        ZeroMqLifecycleCommand,
    },
    threading::{lifecycle_reply_channel, spawn_zeromq_worker},
    workers::{run_pub_socket_worker, run_rep_socket_worker, run_router_socket_worker},
};

pub(super) struct NativeRouterSocketHandle {
    incoming: Arc<Mutex<VecDeque<Vec<Vec<u8>>>>>,
    command_tx: Option<UnboundedSender<NativeRouterSocketCommand>>,
    join_handle: Option<JoinHandle<()>>,
}

pub(super) struct NativePubSocketHandle {
    command_tx: Option<UnboundedSender<NativePubSocketCommand>>,
    join_handle: Option<JoinHandle<()>>,
}

pub(super) struct NativeRepSocketHandle {
    incoming: Arc<Mutex<VecDeque<Vec<Vec<u8>>>>>,
    command_tx: Option<UnboundedSender<NativeRepSocketCommand>>,
    join_handle: Option<JoinHandle<()>>,
}

impl NativeRouterSocketHandle {
    pub(super) fn spawn() -> Self {
        let incoming = Arc::new(Mutex::new(VecDeque::new()));
        let (command_tx, command_rx) = unbounded();
        let incoming_clone = incoming.clone();
        let join_handle = Some(spawn_zeromq_worker("rum-zmq-router", async move {
            run_router_socket_worker(command_rx, incoming_clone).await;
        }));

        Self {
            incoming,
            command_tx: Some(command_tx),
            join_handle,
        }
    }

    pub(super) fn bind(&self, endpoint: &str) -> Result<String, String> {
        let (reply, receiver) = sync_channel(1);
        self.command_tx
            .as_ref()
            .expect("router worker sender should be present")
            .unbounded_send(NativeRouterSocketCommand::Lifecycle(
                ZeroMqLifecycleCommand::Bind {
                    endpoint: endpoint.to_string(),
                    reply,
                },
            ))
            .map_err(|_| "router worker is no longer running".to_string())?;
        receiver
            .recv()
            .map_err(|_| "router worker did not acknowledge bind".to_string())?
    }

    pub(super) fn connect(&self, endpoint: &str) -> Result<(), String> {
        let (reply, receiver) = lifecycle_reply_channel();
        self.command_tx
            .as_ref()
            .expect("router worker sender should be present")
            .unbounded_send(NativeRouterSocketCommand::Lifecycle(
                ZeroMqLifecycleCommand::Connect {
                    endpoint: endpoint.to_string(),
                    reply,
                },
            ))
            .map_err(|_| "router worker is no longer running".to_string())?;
        receiver
            .recv()
            .map_err(|_| "router worker did not acknowledge connect".to_string())?
    }

    pub(super) fn start(&self) {
        let (reply, receiver) = sync_channel(1);
        if self
            .command_tx
            .as_ref()
            .expect("router worker sender should be present")
            .unbounded_send(NativeRouterSocketCommand::Lifecycle(
                ZeroMqLifecycleCommand::Start { reply },
            ))
            .is_ok()
        {
            let _ = receiver.recv();
        }
    }

    pub(super) fn send(&self, frames: Vec<Vec<u8>>) {
        if let Some(sender) = &self.command_tx {
            let _ = sender.unbounded_send(NativeRouterSocketCommand::Send(frames));
        }
    }

    pub(super) fn try_recv(&self) -> Option<Vec<Vec<u8>>> {
        self.incoming
            .lock()
            .expect("router incoming queue should not be poisoned")
            .pop_front()
    }
}

impl Drop for NativeRouterSocketHandle {
    fn drop(&mut self) {
        self.command_tx.take();
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

impl NativePubSocketHandle {
    pub(super) fn spawn() -> Self {
        let (command_tx, command_rx) = unbounded();
        let join_handle = Some(spawn_zeromq_worker("rum-zmq-pub", async move {
            run_pub_socket_worker(command_rx).await;
        }));

        Self {
            command_tx: Some(command_tx),
            join_handle,
        }
    }

    pub(super) fn bind(&self, endpoint: &str) -> Result<String, String> {
        let (reply, receiver) = sync_channel(1);
        self.command_tx
            .as_ref()
            .expect("pub worker sender should be present")
            .unbounded_send(NativePubSocketCommand::Lifecycle(
                ZeroMqLifecycleCommand::Bind {
                    endpoint: endpoint.to_string(),
                    reply,
                },
            ))
            .map_err(|_| "pub worker is no longer running".to_string())?;
        receiver
            .recv()
            .map_err(|_| "pub worker did not acknowledge bind".to_string())?
    }

    pub(super) fn connect(&self, endpoint: &str) -> Result<(), String> {
        let (reply, receiver) = lifecycle_reply_channel();
        self.command_tx
            .as_ref()
            .expect("pub worker sender should be present")
            .unbounded_send(NativePubSocketCommand::Lifecycle(
                ZeroMqLifecycleCommand::Connect {
                    endpoint: endpoint.to_string(),
                    reply,
                },
            ))
            .map_err(|_| "pub worker is no longer running".to_string())?;
        receiver
            .recv()
            .map_err(|_| "pub worker did not acknowledge connect".to_string())?
    }

    pub(super) fn start(&self) {
        let (reply, receiver) = sync_channel(1);
        if self
            .command_tx
            .as_ref()
            .expect("pub worker sender should be present")
            .unbounded_send(NativePubSocketCommand::Lifecycle(
                ZeroMqLifecycleCommand::Start { reply },
            ))
            .is_ok()
        {
            let _ = receiver.recv();
        }
    }

    pub(super) fn send(&self, frames: Vec<Vec<u8>>) {
        if let Some(sender) = &self.command_tx {
            let _ = sender.unbounded_send(NativePubSocketCommand::Send(frames));
        }
    }
}

impl Drop for NativePubSocketHandle {
    fn drop(&mut self) {
        self.command_tx.take();
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

impl NativeRepSocketHandle {
    pub(super) fn spawn() -> Self {
        let incoming = Arc::new(Mutex::new(VecDeque::new()));
        let (command_tx, command_rx) = unbounded();
        let incoming_clone = incoming.clone();
        let join_handle = Some(spawn_zeromq_worker("rum-zmq-rep", async move {
            run_rep_socket_worker(command_rx, incoming_clone).await;
        }));

        Self {
            incoming,
            command_tx: Some(command_tx),
            join_handle,
        }
    }

    pub(super) fn bind(&self, endpoint: &str) -> Result<String, String> {
        let (reply, receiver) = sync_channel(1);
        self.command_tx
            .as_ref()
            .expect("rep worker sender should be present")
            .unbounded_send(NativeRepSocketCommand::Lifecycle(
                ZeroMqLifecycleCommand::Bind {
                    endpoint: endpoint.to_string(),
                    reply,
                },
            ))
            .map_err(|_| "rep worker is no longer running".to_string())?;
        receiver
            .recv()
            .map_err(|_| "rep worker did not acknowledge bind".to_string())?
    }

    pub(super) fn connect(&self, endpoint: &str) -> Result<(), String> {
        let (reply, receiver) = lifecycle_reply_channel();
        self.command_tx
            .as_ref()
            .expect("rep worker sender should be present")
            .unbounded_send(NativeRepSocketCommand::Lifecycle(
                ZeroMqLifecycleCommand::Connect {
                    endpoint: endpoint.to_string(),
                    reply,
                },
            ))
            .map_err(|_| "rep worker is no longer running".to_string())?;
        receiver
            .recv()
            .map_err(|_| "rep worker did not acknowledge connect".to_string())?
    }

    pub(super) fn start(&self) {
        let (reply, receiver) = sync_channel(1);
        if self
            .command_tx
            .as_ref()
            .expect("rep worker sender should be present")
            .unbounded_send(NativeRepSocketCommand::Lifecycle(
                ZeroMqLifecycleCommand::Start { reply },
            ))
            .is_ok()
        {
            let _ = receiver.recv();
        }
    }

    pub(super) fn send(&self, frames: Vec<Vec<u8>>) {
        if let Some(sender) = &self.command_tx {
            let _ = sender.unbounded_send(NativeRepSocketCommand::Send(frames));
        }
    }

    pub(super) fn try_recv(&self) -> Option<Vec<Vec<u8>>> {
        self.incoming
            .lock()
            .expect("rep incoming queue should not be poisoned")
            .pop_front()
    }
}

impl Drop for NativeRepSocketHandle {
    fn drop(&mut self) {
        self.command_tx.take();
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}
