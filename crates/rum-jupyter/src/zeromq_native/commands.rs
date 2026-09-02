use std::sync::mpsc::SyncSender;

pub(super) enum ZeroMqLifecycleCommand {
    Bind {
        endpoint: String,
        reply: SyncSender<Result<String, String>>,
    },
    Connect {
        endpoint: String,
        reply: SyncSender<Result<(), String>>,
    },
    Start {
        reply: SyncSender<()>,
    },
}

pub(super) enum NativeRouterSocketCommand {
    Lifecycle(ZeroMqLifecycleCommand),
    Send(Vec<Vec<u8>>),
}

pub(super) enum NativePubSocketCommand {
    Lifecycle(ZeroMqLifecycleCommand),
    Send(Vec<Vec<u8>>),
}

pub(super) enum NativeRepSocketCommand {
    Lifecycle(ZeroMqLifecycleCommand),
    Send(Vec<Vec<u8>>),
}
