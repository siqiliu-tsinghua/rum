//! Extra real-PTY coverage for unwinding, prompt errors and buffered read-ahead.
use super::*;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

static SERIAL: Mutex<()> = Mutex::new(());

fn pty() -> (File, File) {
    use rustix::pty::{OpenptFlags, grantpt, openpt, ptsname, unlockpt};
    let master = openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).expect("real PTY required");
    grantpt(&master).unwrap();
    unlockpt(&master).unwrap();
    let name = ptsname(&master, Vec::new()).unwrap();
    let slave = rustix::fs::open(
        name,
        OFlags::RDWR | OFlags::NOCTTY,
        rustix::fs::Mode::empty(),
    )
    .unwrap();
    (master.into(), slave.into())
}

fn assert_modes(fd: &impl AsFd, before: &Termios) {
    // Termios has no PartialEq. Its Debug includes every exposed flag, speed,
    // special character and (where present) line discipline.
    assert_eq!(
        format!("{:?}", tcgetattr(fd).unwrap()),
        format!("{before:?}")
    );
}

struct PanicReader(BufReader<File>);
impl Read for PanicReader {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        panic!("use buffered reader");
    }
}
impl BufRead for PanicReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let mode = tcgetattr(self.0.get_ref()).unwrap();
        assert!(
            !mode
                .local_modes
                .intersects(LocalModes::ECHO | LocalModes::ECHONL)
        );
        panic!("synthetic reader panic after echo suppression");
    }
    fn consume(&mut self, amount: usize) {
        self.0.consume(amount);
    }
}

#[test]
fn masked_panic_restores_terminal_and_descriptor() {
    let _serial = SERIAL.lock().unwrap();
    let (_master, slave) = pty();
    let before = tcgetattr(&slave).unwrap();
    let flags = fcntl_getfl(&slave).unwrap();
    let terminal = Terminal::new(&slave).unwrap().unwrap();
    let mut input = PanicReader(BufReader::new(slave));
    let result = catch_unwind(AssertUnwindSafe(|| {
        terminal.read_answer(&mut input, true, || Ok(()))
    }));
    assert!(result.is_err());
    assert_modes(input.0.get_ref(), &before);
    assert_eq!(fcntl_getfl(input.0.get_ref()).unwrap(), flags);
    assert!(outside_wait().unwrap().load(Ordering::SeqCst));
}

#[test]
fn masked_prompt_error_restores_terminal_and_descriptor() {
    let _serial = SERIAL.lock().unwrap();
    let (_master, slave) = pty();
    let before = tcgetattr(&slave).unwrap();
    let flags = fcntl_getfl(&slave).unwrap();
    let terminal = Terminal::new(&slave).unwrap().unwrap();
    let mut input = BufReader::new(slave);
    let result = terminal.read_answer(&mut input, true, || {
        assert!(
            !tcgetattr(&terminal.fd)
                .unwrap()
                .local_modes
                .intersects(LocalModes::ECHO | LocalModes::ECHONL)
        );
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "synthetic prompt failure",
        ))
    });
    assert!(matches!(result, Err(e) if e.kind() == io::ErrorKind::BrokenPipe));
    assert_modes(input.get_ref(), &before);
    assert_eq!(fcntl_getfl(input.get_ref()).unwrap(), flags);
    assert!(outside_wait().unwrap().load(Ordering::SeqCst));
}

#[test]
fn answers_use_the_source_readers_existing_buffer() {
    let _serial = SERIAL.lock().unwrap();
    let (mut master, slave) = pty();
    let before = tcgetattr(&slave).unwrap();
    let mut raw = before.clone();
    raw.make_raw();
    tcsetattr(&slave, OptionalActions::Now, &raw).unwrap();
    master
        .write_all(b"source\n  buffered answer\r\nnext source\n")
        .unwrap();
    let mut input = BufReader::new(slave);
    let mut source = String::new();
    input.read_line(&mut source).unwrap();
    assert_eq!(source, "source\n");
    assert!(input.buffer().starts_with(b"  buffered answer\r\n"));
    tcsetattr(input.get_ref(), OptionalActions::Now, &before).unwrap();
    let terminal = Terminal::new(input.get_ref()).unwrap().unwrap();
    let answer = terminal.read_answer(&mut input, true, || Ok(())).unwrap();
    assert!(matches!(answer, InputEvent::Answer(text) if text == "  buffered answer"));
    source.clear();
    input.read_line(&mut source).unwrap();
    assert_eq!(source, "next source\n");
    assert_modes(input.get_ref(), &before);
}
