use std::{cell::RefCell, collections::VecDeque};

use rum_host::HostSession;

use crate::{
    KernelBootstrapConfig, KernelChannel, KernelQueuedTransport, KernelSocketDriver,
    KernelSocketSession, KernelSocketSessionConfig, KernelTransport, KernelTransportBackend,
    KernelTransportLayout, KernelWireError, KernelWireMessage,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZeroMqBackendError {
    SocketCreation {
        kind: &'static str,
        message: String,
    },
    SocketSetup {
        endpoint: String,
        operation: &'static str,
        message: String,
    },
    InvalidWireFrames {
        channel: KernelChannel,
        message: String,
    },
}

pub trait ZeroMqSocket {
    fn bind(&mut self, endpoint: &str) -> Result<(), String>;
    fn bind_resolved(&mut self, endpoint: &str) -> Result<String, String> {
        self.bind(endpoint)?;
        Ok(endpoint.to_string())
    }
    fn connect(&mut self, endpoint: &str) -> Result<(), String>;
    fn send_multipart(&mut self, frames: Vec<Vec<u8>>);
    fn try_recv_multipart(&mut self) -> Option<Vec<Vec<u8>>>;
}

pub trait ZeroMqSocketFactory {
    type Socket: ZeroMqSocket;

    fn router_socket(&mut self) -> Result<Self::Socket, String>;
    fn pub_socket(&mut self) -> Result<Self::Socket, String>;
    fn rep_socket(&mut self) -> Result<Self::Socket, String>;
}

struct ZeroMqSocketState<S> {
    shell: S,
    control: S,
    stdin: S,
    iopub: S,
    heartbeat: S,
    incoming_shell: VecDeque<KernelWireMessage>,
    incoming_control: VecDeque<KernelWireMessage>,
    incoming_stdin: VecDeque<KernelWireMessage>,
    pending_errors: VecDeque<KernelWireError>,
    heartbeat_incoming: VecDeque<Vec<u8>>,
}

impl<S> ZeroMqSocketState<S> {
    fn incoming_queue_mut(
        &mut self,
        channel: KernelChannel,
    ) -> Option<&mut VecDeque<KernelWireMessage>> {
        match channel {
            KernelChannel::Shell => Some(&mut self.incoming_shell),
            KernelChannel::Control => Some(&mut self.incoming_control),
            KernelChannel::IoPub => None,
            KernelChannel::Stdin => Some(&mut self.incoming_stdin),
        }
    }

    fn has_buffered_channel(&self, channel: KernelChannel) -> bool {
        if !self.pending_errors.is_empty() {
            return true;
        }

        match channel {
            KernelChannel::Shell => !self.incoming_shell.is_empty(),
            KernelChannel::Control => !self.incoming_control.is_empty(),
            KernelChannel::IoPub => false,
            KernelChannel::Stdin => !self.incoming_stdin.is_empty(),
        }
    }
}

pub struct ZeroMqSocketSession<S> {
    session_config: KernelSocketSessionConfig,
    delimiter: String,
    state: RefCell<ZeroMqSocketState<S>>,
}

impl<S: ZeroMqSocket> ZeroMqSocketSession<S> {
    pub fn from_bootstrap<F>(
        bootstrap_config: &KernelBootstrapConfig,
        factory: &mut F,
    ) -> Result<Self, ZeroMqBackendError>
    where
        F: ZeroMqSocketFactory<Socket = S>,
    {
        let mut session_config = bootstrap_config.socket_session_config();
        let mut shell =
            factory
                .router_socket()
                .map_err(|message| ZeroMqBackendError::SocketCreation {
                    kind: "router",
                    message,
                })?;
        let mut control =
            factory
                .router_socket()
                .map_err(|message| ZeroMqBackendError::SocketCreation {
                    kind: "router",
                    message,
                })?;
        let mut stdin =
            factory
                .router_socket()
                .map_err(|message| ZeroMqBackendError::SocketCreation {
                    kind: "router",
                    message,
                })?;
        let mut iopub =
            factory
                .pub_socket()
                .map_err(|message| ZeroMqBackendError::SocketCreation {
                    kind: "pub",
                    message,
                })?;
        let mut heartbeat =
            factory
                .rep_socket()
                .map_err(|message| ZeroMqBackendError::SocketCreation {
                    kind: "rep",
                    message,
                })?;

        setup_socket(&mut shell, &mut session_config.shell)?;
        setup_socket(&mut control, &mut session_config.control)?;
        setup_socket(&mut stdin, &mut session_config.stdin)?;
        setup_socket(&mut iopub, &mut session_config.iopub)?;
        setup_socket(&mut heartbeat, &mut session_config.heartbeat)?;

        Ok(Self {
            session_config,
            delimiter: bootstrap_config.wire_config().delimiter,
            state: RefCell::new(ZeroMqSocketState {
                shell,
                control,
                stdin,
                iopub,
                heartbeat,
                incoming_shell: VecDeque::new(),
                incoming_control: VecDeque::new(),
                incoming_stdin: VecDeque::new(),
                pending_errors: VecDeque::new(),
                heartbeat_incoming: VecDeque::new(),
            }),
        })
    }

    fn poll_channel_into_buffer(&self, channel: KernelChannel) -> bool {
        let mut state = self.state.borrow_mut();
        if state.has_buffered_channel(channel) {
            return true;
        }

        let frames = match channel {
            KernelChannel::Shell => state.shell.try_recv_multipart(),
            KernelChannel::Control => state.control.try_recv_multipart(),
            KernelChannel::IoPub => None,
            KernelChannel::Stdin => state.stdin.try_recv_multipart(),
        };

        if let Some(frames) = frames {
            match frames_to_wire_message(channel, &self.delimiter, frames) {
                Ok(message) => {
                    state
                        .incoming_queue_mut(channel)
                        .expect("channel-backed incoming queues exist")
                        .push_back(message);
                }
                Err(ZeroMqBackendError::InvalidWireFrames { channel, message }) => {
                    state
                        .pending_errors
                        .push_back(KernelWireError::InvalidWireFrames { channel, message });
                }
                Err(other) => {
                    state
                        .pending_errors
                        .push_back(KernelWireError::InvalidWireFrames {
                            channel,
                            message: format!("{other:?}"),
                        });
                }
            }
            true
        } else {
            false
        }
    }
}

impl<S: ZeroMqSocket> KernelTransport for ZeroMqSocketSession<S> {
    fn send(&mut self, message: KernelWireMessage) {
        let frames = wire_message_to_frames(message);
        let mut state = self.state.borrow_mut();
        match frames.channel {
            KernelChannel::Shell => state.shell.send_multipart(frames.frames),
            KernelChannel::Control => state.control.send_multipart(frames.frames),
            KernelChannel::IoPub => state.iopub.send_multipart(frames.frames),
            KernelChannel::Stdin => state.stdin.send_multipart(frames.frames),
        }
    }
}

impl<S: ZeroMqSocket> KernelQueuedTransport for ZeroMqSocketSession<S> {
    fn pop_incoming(&mut self, channel: KernelChannel) -> Option<KernelWireMessage> {
        self.state
            .borrow_mut()
            .incoming_queue_mut(channel)
            .and_then(VecDeque::pop_front)
    }

    fn has_pending_incoming(&self) -> bool {
        if !self.state.borrow().pending_errors.is_empty() {
            return true;
        }

        self.poll_channel_into_buffer(KernelChannel::Control)
            || self.poll_channel_into_buffer(KernelChannel::Shell)
            || self.poll_channel_into_buffer(KernelChannel::Stdin)
    }

    fn take_pending_error(&mut self) -> Option<KernelWireError> {
        self.state.borrow_mut().pending_errors.pop_front()
    }
}

impl<S: ZeroMqSocket> KernelSocketSession for ZeroMqSocketSession<S> {
    fn session_config(&self) -> &KernelSocketSessionConfig {
        &self.session_config
    }

    fn has_pending_channel(&self, channel: KernelChannel) -> bool {
        self.poll_channel_into_buffer(channel)
    }

    fn has_pending_heartbeat(&self) -> bool {
        let mut state = self.state.borrow_mut();
        if !state.heartbeat_incoming.is_empty() {
            return true;
        }

        if let Some(frames) = state.heartbeat.try_recv_multipart() {
            if let Some(frame) = frames.into_iter().next() {
                state.heartbeat_incoming.push_back(frame);
                return true;
            }
        }

        false
    }

    fn pop_heartbeat(&mut self) -> Option<Vec<u8>> {
        self.state.borrow_mut().heartbeat_incoming.pop_front()
    }

    fn send_heartbeat(&mut self, payload: Vec<u8>) {
        self.state
            .borrow_mut()
            .heartbeat
            .send_multipart(vec![payload]);
    }
}

pub struct ZeroMqTransportBackend<S> {
    bootstrap_config: KernelBootstrapConfig,
    transport_layout: KernelTransportLayout,
    socket_session_config: KernelSocketSessionConfig,
    transport: ZeroMqSocketSession<S>,
}

impl<S: ZeroMqSocket> ZeroMqTransportBackend<S> {
    pub fn from_factory<F>(
        mut bootstrap_config: KernelBootstrapConfig,
        factory: &mut F,
    ) -> Result<Self, ZeroMqBackendError>
    where
        F: ZeroMqSocketFactory<Socket = S>,
    {
        let transport = ZeroMqSocketSession::from_bootstrap(&bootstrap_config, factory)?;
        let socket_session_config = transport.session_config().clone();
        let transport_layout = transport_layout_from_session_config(&socket_session_config);
        bootstrap_config.connection.shell_port = transport_layout.shell.port;
        bootstrap_config.connection.control_port = transport_layout.control.port;
        bootstrap_config.connection.stdin_port = transport_layout.stdin.port;
        bootstrap_config.connection.iopub_port = transport_layout.iopub.port;
        bootstrap_config.connection.hb_port = transport_layout.heartbeat.port;

        Ok(Self {
            bootstrap_config,
            transport_layout,
            socket_session_config,
            transport,
        })
    }

    pub fn event_pump(&self, session: HostSession) -> crate::KernelEventPump {
        self.bootstrap_config.event_pump(session)
    }

    pub fn socket_driver(&self, session: HostSession) -> KernelSocketDriver {
        self.bootstrap_config.socket_driver(session)
    }

    pub fn socket_session_config(&self) -> &KernelSocketSessionConfig {
        &self.socket_session_config
    }
}

impl<S> KernelTransportBackend for ZeroMqTransportBackend<S> {
    type Transport = ZeroMqSocketSession<S>;

    fn bootstrap_config(&self) -> &KernelBootstrapConfig {
        &self.bootstrap_config
    }

    fn transport_layout(&self) -> &KernelTransportLayout {
        &self.transport_layout
    }

    fn transport(&self) -> &Self::Transport {
        &self.transport
    }

    fn transport_mut(&mut self) -> &mut Self::Transport {
        &mut self.transport
    }
}

struct MultipartFrames {
    channel: KernelChannel,
    frames: Vec<Vec<u8>>,
}

fn wire_message_to_frames(message: KernelWireMessage) -> MultipartFrames {
    let mut frames = message.routing_ids;
    frames.push(message.delimiter.into_bytes());
    frames.push(message.signature.unwrap_or_default().into_bytes());
    frames.push(message.header_json.into_bytes());
    frames.push(message.parent_header_json.into_bytes());
    frames.push(message.metadata_json.into_bytes());
    frames.push(message.content_json.into_bytes());
    frames.extend(message.buffers);
    MultipartFrames {
        channel: message.channel,
        frames,
    }
}

fn frames_to_wire_message(
    channel: KernelChannel,
    delimiter: &str,
    frames: Vec<Vec<u8>>,
) -> Result<KernelWireMessage, ZeroMqBackendError> {
    let delimiter_bytes = delimiter.as_bytes();
    let Some(delimiter_index) = frames.iter().position(|frame| frame == delimiter_bytes) else {
        return Err(ZeroMqBackendError::InvalidWireFrames {
            channel,
            message: "missing delimiter frame".to_string(),
        });
    };

    if frames.len() < delimiter_index + 5 {
        return Err(ZeroMqBackendError::InvalidWireFrames {
            channel,
            message: "missing required signed payload frames".to_string(),
        });
    }

    let routing_ids = frames[..delimiter_index].to_vec();
    let payload_frames = &frames[delimiter_index + 1..];
    let signature = bytes_to_string(channel, "signature", &payload_frames[0])?;
    let header_json = bytes_to_string(channel, "header", &payload_frames[1])?;
    let parent_header_json = bytes_to_string(channel, "parent_header", &payload_frames[2])?;
    let metadata_json = bytes_to_string(channel, "metadata", &payload_frames[3])?;
    let content_json = bytes_to_string(channel, "content", &payload_frames[4])?;
    let buffers = payload_frames[5..].to_vec();

    Ok(KernelWireMessage {
        channel,
        routing_ids,
        delimiter: delimiter.to_string(),
        signature: if signature.is_empty() {
            None
        } else {
            Some(signature)
        },
        header_json,
        parent_header_json,
        metadata_json,
        content_json,
        buffers,
    })
}

fn bytes_to_string(
    channel: KernelChannel,
    frame_name: &str,
    bytes: &[u8],
) -> Result<String, ZeroMqBackendError> {
    String::from_utf8(bytes.to_vec()).map_err(|error| ZeroMqBackendError::InvalidWireFrames {
        channel,
        message: format!("{frame_name} frame was not valid utf-8: {error}"),
    })
}

fn setup_socket<S: ZeroMqSocket>(
    socket: &mut S,
    spec: &mut crate::KernelSocketSpec,
) -> Result<(), ZeroMqBackendError> {
    let endpoint = spec.uri();
    match spec.bind_mode {
        crate::KernelSocketBindMode::Bind => {
            let resolved = socket.bind_resolved(&endpoint).map_err(|message| {
                ZeroMqBackendError::SocketSetup {
                    endpoint: endpoint.clone(),
                    operation: "bind",
                    message,
                }
            })?;
            spec.endpoint =
                resolved_transport_endpoint(&spec.endpoint, &resolved).map_err(|message| {
                    ZeroMqBackendError::SocketSetup {
                        endpoint,
                        operation: "bind",
                        message,
                    }
                })?;
            Ok(())
        }
        crate::KernelSocketBindMode::Connect => {
            socket
                .connect(&endpoint)
                .map_err(|message| ZeroMqBackendError::SocketSetup {
                    endpoint,
                    operation: "connect",
                    message,
                })
        }
    }
}

fn resolved_transport_endpoint(
    requested: &crate::KernelTransportEndpoint,
    resolved: &str,
) -> Result<crate::KernelTransportEndpoint, String> {
    let (transport, address) = resolved
        .split_once("://")
        .ok_or_else(|| format!("bind returned an invalid resolved endpoint: {resolved}"))?;
    if transport != requested.transport.as_str() {
        return Err(format!(
            "bind returned transport {transport:?}, expected {:?}",
            requested.transport.as_str()
        ));
    }

    let (ip, port) = address
        .rsplit_once(':')
        .ok_or_else(|| format!("bind returned an invalid resolved endpoint: {resolved}"))?;
    let port = port
        .parse::<u16>()
        .map_err(|_| format!("bind returned an invalid resolved port: {resolved}"))?;
    if port == 0 {
        return Err(format!("bind did not resolve port zero: {resolved}"));
    }

    Ok(crate::KernelTransportEndpoint {
        transport: requested.transport,
        ip: ip
            .strip_prefix('[')
            .and_then(|ip| ip.strip_suffix(']'))
            .unwrap_or(ip)
            .to_string(),
        port,
    })
}

fn transport_layout_from_session_config(
    config: &KernelSocketSessionConfig,
) -> KernelTransportLayout {
    KernelTransportLayout {
        shell: config.shell.endpoint.clone(),
        control: config.control.endpoint.clone(),
        stdin: config.stdin.endpoint.clone(),
        iopub: config.iopub.endpoint.clone(),
        heartbeat: config.heartbeat.endpoint.clone(),
    }
}

#[path = "zeromq_native.rs"]
mod zeromq_native;
pub use self::zeromq_native::{
    NativeZeroMqFactory, NativeZeroMqSocket, NativeZeroMqTransportBackend,
};
#[cfg(test)]
use self::zeromq_native::{frames_to_zmq_message, zmq_message_to_frames};

#[cfg(test)]
#[cfg(test)]
mod tests;
