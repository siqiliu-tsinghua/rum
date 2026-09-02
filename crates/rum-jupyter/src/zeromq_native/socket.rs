use super::{
    super::*,
    handles::{NativePubSocketHandle, NativeRepSocketHandle, NativeRouterSocketHandle},
};

#[derive(Default)]
pub struct NativeZeroMqFactory;

pub type NativeZeroMqTransportBackend = ZeroMqTransportBackend<NativeZeroMqSocket>;

impl ZeroMqTransportBackend<NativeZeroMqSocket> {
    pub fn new_native(bootstrap_config: KernelBootstrapConfig) -> Result<Self, ZeroMqBackendError> {
        let mut factory = NativeZeroMqFactory;
        Self::from_factory(bootstrap_config, &mut factory)
    }
}

pub struct NativeZeroMqSocket(NativeZeroMqSocketInner);

enum NativeZeroMqSocketInner {
    Router(NativeRouterSocket),
    Pub(NativePubSocket),
    Rep(NativeRepSocket),
}

impl ZeroMqSocketFactory for NativeZeroMqFactory {
    type Socket = NativeZeroMqSocket;

    fn router_socket(&mut self) -> Result<Self::Socket, String> {
        Ok(NativeZeroMqSocket(NativeZeroMqSocketInner::Router(
            NativeRouterSocket::new(),
        )))
    }

    fn pub_socket(&mut self) -> Result<Self::Socket, String> {
        Ok(NativeZeroMqSocket(NativeZeroMqSocketInner::Pub(
            NativePubSocket::new(),
        )))
    }

    fn rep_socket(&mut self) -> Result<Self::Socket, String> {
        Ok(NativeZeroMqSocket(NativeZeroMqSocketInner::Rep(
            NativeRepSocket::new(),
        )))
    }
}

impl ZeroMqSocket for NativeZeroMqSocket {
    fn bind(&mut self, endpoint: &str) -> Result<(), String> {
        self.bind_resolved(endpoint)?;
        Ok(())
    }

    fn bind_resolved(&mut self, endpoint: &str) -> Result<String, String> {
        match &mut self.0 {
            NativeZeroMqSocketInner::Router(socket) => socket.bind(endpoint),
            NativeZeroMqSocketInner::Pub(socket) => socket.bind(endpoint),
            NativeZeroMqSocketInner::Rep(socket) => socket.bind(endpoint),
        }
    }

    fn connect(&mut self, endpoint: &str) -> Result<(), String> {
        match &mut self.0 {
            NativeZeroMqSocketInner::Router(socket) => socket.connect(endpoint),
            NativeZeroMqSocketInner::Pub(socket) => socket.connect(endpoint),
            NativeZeroMqSocketInner::Rep(socket) => socket.connect(endpoint),
        }
    }

    fn send_multipart(&mut self, frames: Vec<Vec<u8>>) {
        match &mut self.0 {
            NativeZeroMqSocketInner::Router(socket) => socket.send_multipart(frames),
            NativeZeroMqSocketInner::Pub(socket) => socket.send_multipart(frames),
            NativeZeroMqSocketInner::Rep(socket) => socket.send_multipart(frames),
        }
    }

    fn try_recv_multipart(&mut self) -> Option<Vec<Vec<u8>>> {
        match &mut self.0 {
            NativeZeroMqSocketInner::Router(socket) => socket.try_recv_multipart(),
            NativeZeroMqSocketInner::Pub(socket) => socket.try_recv_multipart(),
            NativeZeroMqSocketInner::Rep(socket) => socket.try_recv_multipart(),
        }
    }
}

struct NativeRouterSocket {
    started: bool,
    handle: NativeRouterSocketHandle,
}

impl NativeRouterSocket {
    fn new() -> Self {
        Self {
            started: false,
            handle: NativeRouterSocketHandle::spawn(),
        }
    }

    fn bind(&mut self, endpoint: &str) -> Result<String, String> {
        self.handle.bind(endpoint)
    }

    fn connect(&mut self, endpoint: &str) -> Result<(), String> {
        self.handle.connect(endpoint)
    }

    fn send_multipart(&mut self, frames: Vec<Vec<u8>>) {
        self.ensure_started();
        self.handle.send(frames);
    }

    fn try_recv_multipart(&mut self) -> Option<Vec<Vec<u8>>> {
        self.ensure_started();
        self.handle.try_recv()
    }

    fn ensure_started(&mut self) {
        if !self.started {
            self.handle.start();
            self.started = true;
        }
    }
}

struct NativePubSocket {
    started: bool,
    handle: NativePubSocketHandle,
}

impl NativePubSocket {
    fn new() -> Self {
        Self {
            started: false,
            handle: NativePubSocketHandle::spawn(),
        }
    }

    fn bind(&mut self, endpoint: &str) -> Result<String, String> {
        self.handle.bind(endpoint)
    }

    fn connect(&mut self, endpoint: &str) -> Result<(), String> {
        self.handle.connect(endpoint)
    }

    fn send_multipart(&mut self, frames: Vec<Vec<u8>>) {
        self.ensure_started();
        self.handle.send(frames);
    }

    fn try_recv_multipart(&mut self) -> Option<Vec<Vec<u8>>> {
        None
    }

    fn ensure_started(&mut self) {
        if !self.started {
            self.handle.start();
            self.started = true;
        }
    }
}

struct NativeRepSocket {
    started: bool,
    handle: NativeRepSocketHandle,
}

impl NativeRepSocket {
    fn new() -> Self {
        Self {
            started: false,
            handle: NativeRepSocketHandle::spawn(),
        }
    }

    fn bind(&mut self, endpoint: &str) -> Result<String, String> {
        self.handle.bind(endpoint)
    }

    fn connect(&mut self, endpoint: &str) -> Result<(), String> {
        self.handle.connect(endpoint)
    }

    fn send_multipart(&mut self, frames: Vec<Vec<u8>>) {
        self.ensure_started();
        self.handle.send(frames);
    }

    fn try_recv_multipart(&mut self) -> Option<Vec<Vec<u8>>> {
        self.ensure_started();
        self.handle.try_recv()
    }

    fn ensure_started(&mut self) {
        if !self.started {
            self.handle.start();
            self.started = true;
        }
    }
}
