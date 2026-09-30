//! Interactive shell (REPL) layer for `rum`.
//!
//! Drives a read-eval-print loop over a [`rum_host`] session, including the
//! `:`-prefixed shell commands (`:help`, `:trace`, `:parse`, `:format`, and the
//! rest) and resumable input. Like the other front ends, it is a pure
//! `rum-host` consumer with no direct dependency on the kernel or standard
//! library.
//!
//! A source line or trace command owns its evaluation until completion. When
//! the guest requests input, the same reader supplies one answer at a time:
//! the guest prompt is shown verbatim (or `input> ` when empty) and flushed
//! after pending output. Answers bypass source completeness, shell commands
//! and history; only the line terminator is removed. `InputString` receives
//! literal text, while `Input` parsing belongs to `rum-host`.
//!
//! EOF during an answer cancels the request and exits; a reader's
//! [`io::ErrorKind::Interrupted`] cancels and returns to the primary prompt.
//! Other I/O errors retire the request before returning the error. Cancellation
//! runs guest cleanup once and preserves earlier definitions. A borrowed
//! session never retains input work started here; a rejected overlap leaves
//! the caller's existing request alone.
//!
//! On Unix, [`Repl::with_terminal_input`] enables hidden password entry and
//! Ctrl-C cancellation during answers, using the same buffered input. Echo and
//! descriptor flags are restored on return, EOF, interruption, I/O error and
//! panic unwinding. Answers never enter history or reader diagnostics. Masking
//! protects entry only; guest output may print the answer afterwards.
//!
//! Plain [`BufRead`] readers cannot guarantee hidden entry. Masked requests
//! report "password input unavailable" and are abandoned without reading an
//! answer. Terminal input is an explicit opt-in to the CLI's SIGINT policy;
//! plain readers do not change the embedding process's signal handlers.

use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use rum_host::{HostCompletenessStatus, HostSession};
use rum_terminal::render_host_eval_response;

mod commands;
mod completeness;
mod request;
use self::commands::*;
use self::completeness::{CompletenessHint, IncrementalCompleteness};
use self::request::{AnswerInput, RequestExit};
#[cfg(unix)]
mod terminal;

const HISTORY_ENTRY_PREFIX: &str = ":rum-history-v1:";

pub struct Repl<R, W> {
    input: R,
    output: W,
    prompt: String,
    continuation_prompt: String,
    history: Vec<String>,
    history_path: Option<PathBuf>,
    #[cfg(unix)]
    terminal: Option<terminal::Terminal>,
}

impl<R, W> Repl<R, W>
where
    R: BufRead,
    W: Write,
{
    pub fn new(input: R, output: W) -> Self {
        Self {
            input,
            output,
            prompt: "rum> ".to_string(),
            continuation_prompt: "...> ".to_string(),
            history: Vec::new(),
            history_path: None,
            #[cfg(unix)]
            terminal: None,
        }
    }

    /// Enable Unix terminal answers on the same buffered input.
    ///
    /// This opts into the CLI's process-wide SIGINT policy: cancel during an
    /// answer wait, default SIGINT termination otherwise. Use only in a process
    /// that owns its SIGINT policy, without other SIGINT handlers or readers of
    /// this terminal. Plain `new` leaves the embedding process's policy alone.
    /// Non-terminal input retains the plain reader's password refusal.
    #[cfg(unix)]
    pub fn with_terminal_input(mut self) -> io::Result<Self>
    where
        R: std::os::fd::AsFd,
    {
        self.terminal = terminal::Terminal::new(&self.input)?;
        Ok(self)
    }

    pub fn with_default_history(mut self) -> Self {
        if let Some(path) = default_history_path() {
            self.history_path = Some(path);
            self.load_history();
        }

        self
    }

    pub fn with_history_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.history_path = Some(path.into());
        self.load_history();
        self
    }

    pub fn run(&mut self, session: &mut HostSession) -> io::Result<()> {
        let mut buffer = String::new();
        let mut completeness = IncrementalCompleteness::default();

        loop {
            let prompt = if buffer.trim().is_empty() {
                &self.prompt
            } else {
                &self.continuation_prompt
            };

            self.output.write_all(prompt.as_bytes())?;
            self.output.flush()?;

            let mut line = String::new();
            let bytes_read = self.input.read_line(&mut line)?;

            if bytes_read == 0 {
                writeln!(self.output)?;

                let pending = buffer.trim();

                if !pending.is_empty() {
                    let input = pending.to_string();
                    self.record_history(&input);
                    self.evaluate_input(session, &input)?;
                }

                return Ok(());
            }

            let line = line.trim_end_matches(['\r', '\n']);
            let trimmed = line.trim();

            if buffer.trim().is_empty() && trimmed.is_empty() {
                continue;
            }

            if buffer.trim().is_empty() {
                if let Some(execution) = self.try_handle_command(session, trimmed)? {
                    if matches!(execution, RequestExit::Eof) {
                        self.record_history(trimmed);
                        return Ok(());
                    }

                    self.record_history(trimmed);
                    continue;
                }
            }

            if !buffer.is_empty() {
                buffer.push('\n');
                completeness.append("\n");
            }

            buffer.push_str(line);
            completeness.append(line);

            let pending = buffer.trim();

            match resolve_completeness(&completeness, pending, |pending| {
                session.is_complete(pending).status
            }) {
                HostCompletenessStatus::Complete | HostCompletenessStatus::Invalid => {
                    let input = pending.to_string();
                    buffer.clear();
                    completeness.reset();
                    self.record_history(&input);
                    if self.evaluate_input(session, &input)? == RequestExit::Eof {
                        return Ok(());
                    }
                }
                HostCompletenessStatus::Incomplete => continue,
            }
        }
    }

    fn evaluate_input(
        &mut self,
        session: &mut HostSession,
        input: &str,
    ) -> io::Result<RequestExit> {
        request::drive(
            session,
            &mut AnswerInput {
                reader: &mut self.input,
                #[cfg(unix)]
                terminal: self.terminal.as_ref(),
            },
            &mut self.output,
            input,
            render_host_eval_response,
        )
    }

    fn load_history(&mut self) {
        let Some(path) = &self.history_path else {
            return;
        };

        let Ok(contents) = fs::read_to_string(path) else {
            return;
        };

        self.history = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(decode_history_line)
            .collect();
    }

    fn record_history(&mut self, entry: &str) {
        let entry = entry.trim();

        if entry.is_empty() {
            return;
        }

        self.history.push(entry.to_string());
        let _ = self.persist_history();
    }

    fn persist_history(&self) -> io::Result<()> {
        let Some(path) = &self.history_path else {
            return Ok(());
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut contents = self
            .history
            .iter()
            .map(|entry| encode_history_entry(entry))
            .collect::<Vec<_>>()
            .join("\n");

        if !contents.is_empty() {
            contents.push('\n');
        }

        fs::write(path, contents)
    }

    fn try_handle_command(
        &mut self,
        session: &mut HostSession,
        line: &str,
    ) -> io::Result<Option<RequestExit>> {
        let Some(command) = (match parse_repl_command(line) {
            Ok(command) => command,
            Err(message) => {
                writeln!(self.output, "! {message}")?;
                return Ok(Some(RequestExit::Continue));
            }
        }) else {
            return Ok(None);
        };

        let execution = execute_repl_command(session, &self.history, command);

        match &execution {
            CommandExecution::Exit => return Ok(Some(RequestExit::Eof)),
            CommandExecution::Output(rendered) if rendered.is_empty() => {}
            CommandExecution::Output(rendered) => {
                writeln!(self.output, "{rendered}")?;
            }
            CommandExecution::Trace {
                source,
                full,
                form_notice,
            } => {
                return request::drive(
                    session,
                    &mut AnswerInput {
                        reader: &mut self.input,
                        #[cfg(unix)]
                        terminal: self.terminal.as_ref(),
                    },
                    &mut self.output,
                    source,
                    |response| render_trace_response(response, *full, form_notice.as_deref()),
                )
                .map(Some);
            }
        }

        Ok(Some(RequestExit::Continue))
    }
}

fn resolve_completeness(
    completeness: &IncrementalCompleteness,
    pending: &str,
    full_check: impl FnOnce(&str) -> HostCompletenessStatus,
) -> HostCompletenessStatus {
    match completeness.hint() {
        CompletenessHint::DefinitelyIncomplete => HostCompletenessStatus::Incomplete,
        CompletenessHint::NeedsFullCheck => full_check(pending),
    }
}

fn default_history_path() -> Option<PathBuf> {
    env::var_os("RUM_HISTORY_PATH")
        .map(PathBuf::from)
        .or_else(|| env::current_dir().ok().map(|dir| dir.join(".rum_history")))
}

fn encode_history_entry(entry: &str) -> String {
    if !history_entry_needs_encoding(entry) {
        return entry.to_string();
    }

    let mut encoded = String::with_capacity(HISTORY_ENTRY_PREFIX.len() + entry.len());
    encoded.push_str(HISTORY_ENTRY_PREFIX);

    for ch in entry.chars() {
        match ch {
            '\\' => encoded.push_str("\\\\"),
            '\n' => encoded.push_str("\\n"),
            '\r' => encoded.push_str("\\r"),
            other => encoded.push(other),
        }
    }

    encoded
}

fn history_entry_needs_encoding(entry: &str) -> bool {
    entry.starts_with(HISTORY_ENTRY_PREFIX) || entry.contains(['\n', '\r'])
}

fn decode_history_line(line: &str) -> String {
    let Some(encoded) = line.strip_prefix(HISTORY_ENTRY_PREFIX) else {
        return line.to_string();
    };

    let mut decoded = String::new();
    let mut chars = encoded.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            decoded.push(ch);
            continue;
        }

        match chars.next() {
            Some('\\') => decoded.push('\\'),
            Some('n') => decoded.push('\n'),
            Some('r') => decoded.push('\r'),
            Some(other) => {
                decoded.push('\\');
                decoded.push(other);
            }
            None => decoded.push('\\'),
        }
    }

    decoded
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
