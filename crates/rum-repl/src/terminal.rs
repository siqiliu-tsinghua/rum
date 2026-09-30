//! Unix answer waits. The descriptor controls modes; only the REPL's existing
//! buffered reader consumes input. No terminal or signal change in `new`.

use std::io::{self, BufRead};
use std::os::fd::{AsFd, OwnedFd};
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};

use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
use rustix::io::{FdFlags, fcntl_setfd};
use rustix::termios::{
    LocalModes, OptionalActions, QueueSelector, Termios, tcflush, tcgetattr, tcsetattr,
};
use signal_hook::{SigId, consts::SIGINT};

use super::request::InputEvent;

pub(super) struct Terminal {
    fd: OwnedFd,
}

impl Terminal {
    pub fn new(input: &impl AsFd) -> io::Result<Option<Self>> {
        if !rustix::termios::isatty(input) {
            return Ok(None);
        }
        Ok(Some(Self {
            fd: input.as_fd().try_clone_to_owned()?,
        }))
    }

    pub fn read_answer(
        &self,
        input: &mut impl BufRead,
        password: bool,
        prompt: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<InputEvent> {
        let mut wait = Wait::new(&self.fd, password).map_err(|error| {
            if password {
                io::Error::new(error.kind(), "password input unavailable")
            } else {
                error
            }
        })?;
        // Echo must already be off when a password prompt becomes visible.
        let result = (|| {
            prompt()?;
            // stdin and stdout may be duplicates of one open description.
            // Flush the prompt before NONBLOCK can affect its writes.
            fcntl_setfl(&self.fd, wait.flags | OFlags::NONBLOCK)?;
            read_line(input, &wait)
        })();
        let restored = wait.restore();
        // Preserve the original I/O error, without including any answer bytes.
        let event = result?;
        restored?;
        Ok(event)
    }
}

// signal-hook's unregister removes an action, not the OS handler. Keep one
// conditional default action for the CLI's lifetime so that SIGINT still has
// its original default effect outside waits (including after unwinding).
fn outside_wait() -> io::Result<Arc<AtomicBool>> {
    static POLICY: OnceLock<Result<Arc<AtomicBool>, (io::ErrorKind, String)>> = OnceLock::new();
    POLICY
        .get_or_init(|| {
            let outside = Arc::new(AtomicBool::new(true));
            signal_hook::flag::register_conditional_default(SIGINT, outside.clone())
                .map(|_| outside)
                .map_err(|e| (e.kind(), e.to_string()))
        })
        .clone()
        .map_err(|(kind, message)| io::Error::new(kind, message))
}

struct Wait<'a> {
    fd: &'a OwnedFd,
    saved: Termios,
    flags: OFlags,
    wake: OwnedFd,
    signal: SigId,
    outside: Arc<AtomicBool>,
    restored: bool,
}

impl<'a> Wait<'a> {
    fn new(fd: &'a OwnedFd, password: bool) -> io::Result<Self> {
        let saved = tcgetattr(fd)?;
        let flags = fcntl_getfl(fd)?;
        let (wake, write) = rustix::pipe::pipe()?;
        fcntl_setfd(&wake, FdFlags::CLOEXEC)?;
        fcntl_setfd(&write, FdFlags::CLOEXEC)?;
        let outside = outside_wait()?;
        let signal = signal_hook::low_level::pipe::register(SIGINT, write)?;
        // Arm the wakeup before intercepting SIGINT; no lost-signal window.
        // Only one process-wide answer wait may own SIGINT at a time.
        if outside
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            signal_hook::low_level::unregister(signal);
            return Err(io::Error::other("terminal answer wait already active"));
        }
        let wait = Self {
            fd,
            saved,
            flags,
            wake,
            signal,
            outside,
            restored: false,
        };
        // From here all exits, including setup failure and panic, restore modes.
        if password {
            let mut hidden = wait.saved.clone();
            hidden
                .local_modes
                .remove(LocalModes::ECHO | LocalModes::ECHONL);
            tcsetattr(fd, OptionalActions::Now, &hidden)?;
        }
        Ok(wait)
    }

    fn restore(&mut self) -> io::Result<()> {
        if self.restored {
            return Ok(());
        }
        let modes = retry(|| tcsetattr(self.fd, OptionalActions::Now, &self.saved));
        let flags = retry(|| fcntl_setfl(self.fd, self.flags));
        self.restored = modes.is_ok() && flags.is_ok();
        modes.and(flags).map_err(Into::into)
    }

    fn interrupted(&self, timeout: Option<&Timespec>) -> io::Result<bool> {
        let mut descriptors = [PollFd::new(&self.wake, PollFlags::IN)];
        retry(|| poll(&mut descriptors, timeout))?;
        Ok(!descriptors[0].revents().is_empty())
    }

    fn ready(&self) -> io::Result<()> {
        let mut descriptors = [
            PollFd::new(&self.wake, PollFlags::IN),
            PollFd::new(self.fd, PollFlags::IN),
        ];
        // Darwin poll does not support terminal descriptors. Bound that input
        // retry there; the signal pipe still wakes immediately on every Unix.
        #[cfg(target_vendor = "apple")]
        let descriptors = &mut descriptors[..1];
        #[cfg(not(target_vendor = "apple"))]
        let descriptors = &mut descriptors[..];
        #[cfg(target_vendor = "apple")]
        let timeout = Some(&Timespec {
            tv_sec: 0,
            tv_nsec: 20_000_000,
        });
        #[cfg(not(target_vendor = "apple"))]
        let timeout = None;
        retry(|| poll(descriptors, timeout))?;
        Ok(())
    }
}

impl Drop for Wait<'_> {
    fn drop(&mut self) {
        let _ = self.restore();
        self.outside.store(true, Ordering::SeqCst);
        signal_hook::low_level::unregister(self.signal);
    }
}

fn retry<T>(mut operation: impl FnMut() -> rustix::io::Result<T>) -> rustix::io::Result<T> {
    loop {
        match operation() {
            Err(rustix::io::Errno::INTR) => continue,
            result => return result,
        }
    }
}

fn read_line(input: &mut impl BufRead, wait: &Wait<'_>) -> io::Result<InputEvent> {
    let mut bytes = Vec::new();
    loop {
        if wait.interrupted(Some(&Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        }))? {
            // A directly sent SIGINT need not flush the terminal's partial line.
            retry(|| tcflush(wait.fd, QueueSelector::IFlush))?;
            if let Ok(buffer) = input.fill_buf() {
                let length = buffer.len();
                input.consume(length);
            }
            return Ok(InputEvent::Interrupted);
        }
        // Drain read-ahead before polling the descriptor. NONBLOCK ensures a
        // restarted read cannot hide a signal or a spurious readiness event.
        match input.fill_buf() {
            Ok([]) if bytes.is_empty() => return Ok(InputEvent::Eof),
            Ok([]) => break,
            Ok(buffer) => {
                let length = buffer
                    .iter()
                    .position(|b| *b == b'\n')
                    .map_or(buffer.len(), |i| i + 1);
                let complete = buffer[length - 1] == b'\n';
                bytes.extend_from_slice(&buffer[..length]);
                input.consume(length);
                if complete {
                    break;
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => wait.ready()?,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    String::from_utf8(bytes)
        .map(InputEvent::Answer)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "input was not valid UTF-8"))
}

#[cfg(test)]
mod tests;
