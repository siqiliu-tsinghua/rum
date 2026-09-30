//! Command-line entry point for `rum`.
//!
//! The `rum` executable: evaluates expressions and scripts (from arguments,
//! files, or stdin) and offers `--parse` / `--format` modes, all over a
//! [`rum_host`] session. It is a pure `rum-host` consumer with no direct
//! dependency on the kernel or standard library.

mod input_retirement;

use input_retirement::{CancellationOutput, retire_input_request};
use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use rum_host::{
    ClosurePrintSink, HostConfig, HostEvalOutcome, HostEvalSuccess, HostFailure, HostFailureCode,
    HostInputRequest, HostInputSuspension, HostSession,
};
use rum_repl::Repl;
use rum_syntax::{format_parsed_source, parse_format_cli_args, parse_with_diagnostics};
use rum_terminal::{render_host_failure, render_host_message, render_host_messages};

// The allocation-counter shim (Layer B B0 measurement) is compiled in ONLY under
// the `alloc-count` feature. The default build uses bare mimalloc with no wrapper
// so there is zero per-allocation cost on the production path (a global-allocator
// wrapper regresses every allocation 4-19% even when its flag is off, because it
// blocks the allocator fast-path inlining — measured 2026-06-22).
#[cfg(not(feature = "alloc-count"))]
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "alloc-count")]
mod alloc_count;

fn main() -> ExitCode {
    #[cfg(feature = "alloc-count")]
    alloc_count::init();

    let code = match run() {
        Ok(code) => code,
        Err(err) if is_broken_pipe_error(err.as_ref()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    };

    #[cfg(feature = "alloc-count")]
    print_alloc_profile();
    code
}

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let Invocation {
        command,
        script_args,
    } = parse_invocation(env::args().skip(1).collect())?;
    let stdin_is_tty = io::stdin().is_terminal();

    match command {
        CommandMode::ReplOrFilter if stdin_is_tty => {
            let mut session = build_host_session();
            run_repl(&mut session)?;
        }
        CommandMode::Parse(input) => match parse_with_diagnostics(input) {
            Ok(parsed) => write_stdout(parsed.debug_dump().as_bytes())?,
            Err(diagnostic) => {
                eprintln!("{}", diagnostic.render());
                return Err(io::Error::other("parse failed").into());
            }
        },
        CommandMode::Format(raw_args) => {
            let (input, options) = parse_format_cli_args(&raw_args, "--format")
                .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))?;

            match parse_with_diagnostics(input) {
                Ok(parsed) => {
                    let formatted = format_parsed_source(&parsed, options);
                    write_stdout_line(&formatted)?;
                }
                Err(diagnostic) => {
                    eprintln!("{}", diagnostic.render());
                    return Err(io::Error::other("format failed").into());
                }
            }
        }
        CommandMode::Help => print_usage()?,
        command => return run_non_interactive(command, script_args, stdin_is_tty),
    }

    Ok(ExitCode::SUCCESS)
}

fn run_non_interactive(
    command: CommandMode,
    script_args: Option<Vec<String>>,
    stdin_is_tty: bool,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut session = build_host_session();

    match command {
        CommandMode::ReplOrFilter | CommandMode::StdinProgram => {
            bind_script_args_if_present(&mut session, script_args)?;
            let input = read_stdin_to_string()?;
            eval_script_source(&mut session, &input)
        }
        CommandMode::Eval(input) => {
            if !stdin_is_tty {
                bind_script_input(&mut session, read_stdin_to_string()?)?;
            }
            bind_script_args_if_present(&mut session, script_args)?;
            eval_source(&mut session, &input)
        }
        CommandMode::File(path) => {
            if !stdin_is_tty {
                bind_script_input(&mut session, read_stdin_to_string()?)?;
            }
            bind_script_args_if_present(&mut session, script_args)?;
            eval_file(&mut session, &path)
        }
        CommandMode::Parse(_) | CommandMode::Format(_) | CommandMode::Help => {
            unreachable!("non-interactive evaluation received a non-evaluation command")
        }
    }
}

struct Invocation {
    command: CommandMode,
    script_args: Option<Vec<String>>,
}

enum CommandMode {
    ReplOrFilter,
    StdinProgram,
    Parse(String),
    Format(Vec<String>),
    Eval(String),
    File(String),
    Help,
}

fn parse_invocation(raw_args: Vec<String>) -> Result<Invocation, Box<dyn std::error::Error>> {
    match raw_args.first().map(String::as_str) {
        Some("--parse") => {
            let input = raw_args.get(1).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--parse expects an expression")
            })?;
            Ok(Invocation {
                command: CommandMode::Parse(input.clone()),
                script_args: None,
            })
        }
        Some("--format") => Ok(Invocation {
            command: CommandMode::Format(raw_args.into_iter().skip(1).collect()),
            script_args: None,
        }),
        _ => parse_program_invocation(raw_args),
    }
}

fn parse_program_invocation(
    raw_args: Vec<String>,
) -> Result<Invocation, Box<dyn std::error::Error>> {
    let (args, script_args) = split_script_args(raw_args);
    let command = match args.first().map(String::as_str) {
        None => CommandMode::ReplOrFilter,
        Some("--eval") => {
            let input = args.get(1).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--eval expects an expression")
            })?;
            CommandMode::Eval(input.clone())
        }
        Some("--file") => {
            let path = args.get(1).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--file expects a path")
            })?;
            CommandMode::File(path.clone())
        }
        Some("--help") | Some("-h") => CommandMode::Help,
        Some(flag) if flag.starts_with("--") => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown flag `{flag}`"),
            )));
        }
        Some("-") => CommandMode::StdinProgram,
        Some(path) => CommandMode::File(path.to_string()),
    };

    Ok(Invocation {
        command,
        script_args,
    })
}

fn split_script_args(raw_args: Vec<String>) -> (Vec<String>, Option<Vec<String>>) {
    let Some(separator) = raw_args.iter().position(|arg| arg == "--") else {
        return (raw_args, None);
    };

    let mut args = raw_args;
    let script_args = args.split_off(separator + 1);
    args.pop();
    (args, Some(script_args))
}

fn build_host_session() -> HostSession {
    HostSession::builder()
        .with_config(HostConfig::scripting())
        .build()
}

fn run_repl(session: &mut HostSession) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let repl = Repl::new(stdin.lock(), stdout.lock()).with_default_history();
    #[cfg(unix)]
    let repl = repl.with_terminal_input()?;
    let mut repl = repl;
    repl.run(session)
}

fn eval_source(
    session: &mut HostSession,
    input: &str,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    eval_request(
        session,
        "evaluation failed",
        CancellationOutput::Cumulative,
        |session| session.eval(input),
    )
}

fn eval_script_source(
    session: &mut HostSession,
    input: &str,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    eval_request(
        session,
        "evaluation failed",
        CancellationOutput::CleanupOnly,
        |session| session.eval_script(input),
    )
}

fn eval_file(
    session: &mut HostSession,
    path: &str,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    eval_request(
        session,
        "file evaluation failed",
        CancellationOutput::CleanupOnly,
        |session| session.eval_file(path),
    )
}

fn eval_request(
    session: &mut HostSession,
    error_context: &str,
    cancellation_output: CancellationOutput,
    request: impl FnOnce(&mut HostSession) -> HostEvalOutcome,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let cancel = session.cancel_handle();
    let broken_pipe = install_stdout_print_sink(session);

    match request(session) {
        HostEvalOutcome::Success(response) => {
            if broken_pipe.load(Ordering::Relaxed) {
                return Ok(ExitCode::SUCCESS);
            }
            print_outcome(&response)?;
            Ok(ExitCode::SUCCESS)
        }
        HostEvalOutcome::Failure(failure) => {
            if broken_pipe.load(Ordering::Relaxed) {
                return Ok(ExitCode::SUCCESS);
            }
            if let Some(code) = exit_code_from_failure(&failure) {
                print_failure_side_effects(&failure);
                return Ok(code);
            }

            Err(host_failure_to_cli_error(failure, error_context))
        }
        HostEvalOutcome::InputRequired(mut suspension) => {
            // Only an admitted request can suspend. Retire before any output
            // error or broken-pipe return; a rejected admission owns nothing.
            retire_input_request(session, &cancel, &mut suspension, cancellation_output);
            if broken_pipe.load(Ordering::Relaxed) {
                return Ok(ExitCode::SUCCESS);
            }

            Err(input_suspension_to_cli_error(suspension))
        }
    }
}

fn bind_script_input(
    session: &mut HostSession,
    input: String,
) -> Result<(), Box<dyn std::error::Error>> {
    session.queue_input_response(input);
    eval_setup_binding(session, "Global`$ScriptInput = InputString[]")
}

fn bind_script_args_if_present(
    session: &mut HostSession,
    args: Option<Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(args) = args else {
        return Ok(());
    };

    for arg in args.iter().cloned() {
        session.queue_input_response(arg);
    }

    let placeholders = std::iter::repeat_n("InputString[]", args.len())
        .collect::<Vec<_>>()
        .join(", ");
    eval_setup_binding(session, &format!("Global`$ScriptArgs = {{{placeholders}}}"))
}

fn eval_setup_binding(
    session: &mut HostSession,
    source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let cancel = session.cancel_handle();
    let rendered = match session.eval(source) {
        HostEvalOutcome::Success(_) => return Ok(()),
        HostEvalOutcome::Failure(failure) => render_host_failure(&failure),
        HostEvalOutcome::InputRequired(mut suspension) => {
            retire_input_request(
                session,
                &cancel,
                &mut suspension,
                CancellationOutput::Cumulative,
            );
            render_input_suspension(&suspension)
        }
    };

    Err(io::Error::other(format!("failed to bind script variable: {rendered}")).into())
}

fn read_stdin_to_string() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

fn install_stdout_print_sink(session: &mut HostSession) -> Arc<AtomicBool> {
    let broken_pipe = Arc::new(AtomicBool::new(false));
    let sink_broken_pipe = Arc::clone(&broken_pipe);
    let cancel = session.cancel_handle();

    session.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |text| {
        if sink_broken_pipe.load(Ordering::Relaxed) {
            return;
        }

        if let Err(err) = write_stdout_line(text) {
            if err.kind() == io::ErrorKind::BrokenPipe {
                sink_broken_pipe.store(true, Ordering::Relaxed);
                cancel.cancel();
            }
        }
    }));

    broken_pipe
}

fn print_outcome(outcome: &HostEvalSuccess) -> io::Result<()> {
    for message in &outcome.messages {
        eprintln!("{}", render_host_message(message));
    }

    write_stdout_line(&outcome.value.text)
}

fn print_failure_side_effects(failure: &HostFailure) {
    for message in &failure.messages {
        eprintln!("{}", render_host_message(message));
    }
}

fn render_input_suspension(suspension: &HostInputSuspension) -> String {
    let mut output = render_host_messages(&suspension.messages);
    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str("! ");
    output.push_str(&input_required_message(&suspension.request));
    output
}

fn input_required_message(request: &HostInputRequest) -> String {
    if request.prompt.is_empty() {
        "input required".to_string()
    } else {
        format!("input required: {}", request.prompt)
    }
}

#[cfg(feature = "alloc-count")]
fn print_alloc_profile() {
    if !alloc_count::enabled() {
        return;
    }

    let profile = alloc_count::snapshot();
    eprintln!(
        "rum-profile alloc count={} bytes_total={} peak_live_bytes={}",
        profile.count, profile.bytes_total, profile.peak_live_bytes
    );
}

fn host_failure_to_cli_error(
    failure: HostFailure,
    syntax_context: &str,
) -> Box<dyn std::error::Error> {
    let rendered = render_host_failure(&failure);
    eprintln!("{rendered}");

    if failure.code == HostFailureCode::Syntax {
        Box::new(io::Error::other(syntax_context.to_string()))
    } else {
        Box::new(io::Error::other(failure.message))
    }
}

fn input_suspension_to_cli_error(suspension: HostInputSuspension) -> Box<dyn std::error::Error> {
    let message = input_required_message(&suspension.request);
    eprintln!("{}", render_input_suspension(&suspension));
    Box::new(io::Error::other(message))
}

fn exit_code_from_failure(failure: &HostFailure) -> Option<ExitCode> {
    if failure.code == HostFailureCode::Exit {
        let status = failure
            .exit_status
            .expect("Exit host failures carry a requested status");
        Some(ExitCode::from(status as u8))
    } else {
        None
    }
}

fn write_stdout(bytes: &[u8]) -> io::Result<()> {
    let mut out = io::stdout().lock();
    out.write_all(bytes)?;
    out.flush()
}

fn write_stdout_line(line: &str) -> io::Result<()> {
    let mut out = io::stdout().lock();
    writeln!(out, "{line}")?;
    out.flush()
}

fn is_broken_pipe_error(err: &(dyn std::error::Error + 'static)) -> bool {
    err.downcast_ref::<io::Error>()
        .is_some_and(|err| err.kind() == io::ErrorKind::BrokenPipe)
}

fn print_usage() -> io::Result<()> {
    write_stdout(
        b"rum [--parse <expr>]\n\
rum [--format <expr> [--line-width <columns>]]\n\
rum [--eval <expr> [-- <arg>...]]\n\
rum [--file <path> [-- <arg>...]]\n\
rum [- [-- <arg>...]]\n\
rum [--help]\n\
rum [<path> [-- <arg>...]]\n\
rum\n\n\
With non-TTY stdin and no program argument, stdin is evaluated as a script.\n\
With a program argument and non-TTY stdin, stdin is bound as Global`$ScriptInput.\n\
Arguments after -- are bound as Global`$ScriptArgs.\n\
Exit codes: 0 on success, requested Exit[n] status for Exit, 1 on syntax/load/evaluation failure.\n",
    )
}

#[cfg(test)]
mod eval_tests;
