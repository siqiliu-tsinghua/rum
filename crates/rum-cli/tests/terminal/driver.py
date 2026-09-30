"""Real PTY assertions for the REPL contract in getting-started.md / brief-554.

Each invocation owns its PTY and subprocess; failure still kills and reaps it.
Only synthetic secrets are used. No process touches the runner's terminal.
"""
import fcntl
import os
from pathlib import Path
import select
import signal
import subprocess
import sys
import termios
import time

BINARY, CASE, DIRECTORY = sys.argv[1:]
DIRECTORY = Path(DIRECTORY)
DIRECTORY.mkdir(parents=True, exist_ok=True)
SECRET = b"synthetic-secret-554"


class Terminal:
    def __enter__(self):
        self.master, self.slave = os.openpty()
        self.before = termios.tcgetattr(self.slave)
        self.flags = fcntl.fcntl(self.slave, fcntl.F_GETFL)
        self.history = DIRECTORY / (CASE + "-history")
        self.history.write_bytes(b"")
        self.process = subprocess.Popen(
            [BINARY], stdin=self.slave, stdout=self.slave, stderr=self.slave,
            env={**os.environ, "RUM_HISTORY_PATH": str(self.history)},
        )
        self.output = b""
        self.until(b"rum> ")
        return self

    def __exit__(self, *_):
        if self.process.poll() is None:
            self.process.kill()
        self.process.wait(timeout=3)
        self.drain()
        (DIRECTORY / (CASE + "-transcript")).write_bytes(self.output)
        os.close(self.slave)
        os.close(self.master)

    def send(self, data):
        os.write(self.master, data)

    def until(self, marker):
        result = b""
        deadline = time.monotonic() + 3
        while marker not in result:
            assert time.monotonic() < deadline, ("timed out waiting for", marker, result)
            if select.select([self.master], [], [], 0.05)[0]:
                data = os.read(self.master, 65536)
                result += data
                self.output += data
            else:
                assert self.process.poll() is None, ("process exited", self.process.returncode, result)
        return result

    def drain(self):
        result = b""
        while select.select([self.master], [], [], 0.02)[0]:
            data = os.read(self.master, 65536)
            result += data
            self.output += data
        return result

    def request(self, masked=False, cleanup=False):
        source = b'received=InputString["answer: "' + (b', FieldMasked->True' if masked else b'') + b']; Null'
        if cleanup:
            source = b'before=1; after=0; count=0; WithCleanup[(' + source + b'; after=1), (count=count+1; Print["cleaned"])]; Print["remainder"]'
        self.send(source + b"\n")
        # First answer: occurrence is in source echo, second is the guest prompt.
        seen = self.until(b"answer: ")
        if seen.count(b"answer: ") < 2:
            seen += self.until(b"answer: ")
        assert b"password input unavailable" not in seen, "terminal reader refused masked input"
        if masked:
            attrs = termios.tcgetattr(self.slave)
            assert not attrs[3] & (termios.ECHO | termios.ECHONL), "password prompt visible before echo disabled"

    def restored(self):
        assert termios.tcgetattr(self.slave) == self.before, "terminal attributes were not restored"
        assert fcntl.fcntl(self.slave, fcntl.F_GETFL) == self.flags, "descriptor flags were not restored"

    def quit(self):
        self.send(b":quit\n")
        assert self.process.wait(timeout=3) == 0


def masked_answer(t):
    t.request(masked=True)
    t.send(SECRET + b"\n")
    reply = t.until(b"rum> ")
    assert SECRET not in reply, "secret appeared during entry"
    assert b"*" not in reply, "password placeholders appeared"
    t.restored()
    t.send(b"received\n")
    assert b'"' + SECRET + b'"' in t.until(b"rum> "), "guest did not receive exact answer"
    t.quit()
    assert SECRET not in t.history.read_bytes(), "secret entered history"


def ordinary_answer(t):
    t.request()
    t.send(b"ordinary answer\n")
    assert b"ordinary answer" in t.until(b"rum> "), "ordinary answer did not echo"
    t.restored()
    t.quit()


def masked_eof(t):
    t.request(masked=True, cleanup=True)
    t.send(b"\x04")
    reply = t.until(b"cleaned")
    assert t.process.wait(timeout=3) == 0
    assert reply.count(b"input cancelled") == 1
    assert reply.count(b"cleaned") == 1
    assert b"remainder" not in reply
    t.restored()


def masked_io_error(t):
    t.request(masked=True, cleanup=True)
    t.send(SECRET + b"\xff\n")
    reply = t.until(b"error:")
    assert t.process.wait(timeout=3) != 0
    reply += t.drain()
    assert SECRET not in reply, "secret appeared in a read-error diagnostic"
    assert SECRET not in t.history.read_bytes(), "failed answer entered history"
    assert reply.count(b"cleaned") == 1
    assert b"remainder" not in reply
    t.restored()


def interrupt(t, masked=False):
    t.request(masked=masked, cleanup=True)
    # Real SIGINT during the read, without Enter or a synthetic read error.
    t.send(b"discarded partial answer")
    time.sleep(0.05)
    t.process.send_signal(signal.SIGINT)
    reply = t.until(b"rum> ")
    assert reply.count(b"input cancelled") == 1
    assert reply.count(b"cleaned") == 1
    assert b"remainder" not in reply
    assert b"TerminatedEvaluation" not in reply
    t.restored()
    t.send(b"{before,after,count}\n")
    assert b"List[1, 0, 1]" in t.until(b"rum> ")
    t.send(b"1+1\n")
    assert b"2\r\nrum> " in t.until(b"rum> ")
    t.quit()


def signal_outside_wait(t, after=False, evaluation=False):
    if after:
        t.request()
        t.send(b"answered\n")
        t.until(b"rum> ")
        t.restored()
    if evaluation:
        t.send(b'Print["running"]; Pause[30]\n')
        seen = t.until(b"Pause[30]\r\n")
        assert b"rum> " not in seen
        # Guest Print is delivered only after evaluation returns; Pause holds it.
        time.sleep(0.1)
    t.process.send_signal(signal.SIGINT)
    assert t.process.wait(timeout=3) == -signal.SIGINT, "SIGINT outside wait changed"


with Terminal() as terminal:
    {
        "masked_answer": masked_answer,
        "ordinary_answer": ordinary_answer,
        "masked_eof": masked_eof,
        "masked_io_error": masked_io_error,
        "interrupt": interrupt,
        "masked_interrupt": lambda t: interrupt(t, masked=True),
        "primary_sigint": signal_outside_wait,
        "primary_sigint_after_answer": lambda t: signal_outside_wait(t, after=True),
        "evaluation_sigint_after_answer": lambda t: signal_outside_wait(t, after=True, evaluation=True),
    }[CASE](terminal)
print("PASS:", CASE)
