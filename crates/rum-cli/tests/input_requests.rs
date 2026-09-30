//! Script modes never answer guest input, including from nested files.
//! Contract: docs/guide/getting-started.md, non-interactive input and bindings.
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

struct File(PathBuf);

impl File {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "rum-cli-input-{}-{}.rum",
            std::process::id(),
            FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, source).unwrap();
        Self(path)
    }

    fn path(&self) -> &str {
        self.0.to_str().unwrap()
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn run(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rum-cli"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn modes(source: &str, prompt: &str, expected_stdout: &str) {
    let file = File::new(source);
    for (label, args, stdin) in [
        (
            "eval",
            vec!["--eval", source, "--", "arg1", "arg2"],
            "answer\nsecond\n",
        ),
        (
            "file",
            vec!["--file", file.path(), "--", "arg1", "arg2"],
            "answer\nsecond\n",
        ),
        (
            "positional",
            vec![file.path(), "--", "arg1", "arg2"],
            "answer\nsecond\n",
        ),
        ("stdin", vec!["-", "--", "arg1", "arg2"], source),
        ("filter", vec!["--", "arg1", "arg2"], source),
    ] {
        let output = run(&args, stdin);
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(
            output.status.code(),
            Some(1),
            "{label}: {stdout:?} {stderr:?}"
        );
        assert_eq!(stdout, expected_stdout, "{label}");
        assert_eq!(
            stderr,
            format!("! input required: {prompt}\nerror: input required: {prompt}\n"),
            "{label}"
        );
    }
}

#[test]
fn all_script_modes_report_input_and_skip_tail() {
    modes(
        "Print[\"prefix\"]; InputString[\"p: \"]; Print[\"tail\"]",
        "p: ",
        "prefix\n",
    );
}

#[test]
fn all_script_modes_stop_at_first_of_two_inputs() {
    modes(
        "Print[\"prefix\"]; InputString[\"first: \"]; InputString[\"second: \"]; Print[\"tail\"]",
        "first: ",
        "prefix\n",
    );
}

#[test]
fn all_script_modes_report_nested_get_input() {
    let inner = File::new("InputString[\"nested: \"]; Print[\"inner-tail\"]");
    let middle = File::new(&format!("Get[{:?}]; Print[\"middle-tail\"]", inner.path()));
    modes(
        &format!(
            "Print[\"prefix\"]; Get[{:?}]; Print[\"tail\"]",
            middle.path()
        ),
        "nested: ",
        "prefix\n",
    );
}

#[test]
fn all_script_modes_deliver_cleanup_once_without_cancel_diagnostic() {
    modes(
        "Print[\"prefix\"]; WithCleanup[InputString[\"p: \"], Print[\"cleanup\"]]; Print[\"tail\"]",
        "p: ",
        "prefix\ncleanup\n",
    );
}

#[test]
fn program_modes_consume_stdin_and_arguments_as_binding_data() {
    let source = "Print[$ScriptInput]; Print[$ScriptArgs]; InputString[\"after bindings: \"]; Print[\"tail\"]";
    let file = File::new(source);
    for args in [
        vec!["--eval", source, "--", "arg1", "InputString[]"],
        vec!["--file", file.path(), "--", "arg1", "InputString[]"],
        vec![file.path(), "--", "arg1", "InputString[]"],
    ] {
        let output = run(&args, "answer\n");
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "answer\n\n{\"arg1\", \"InputString[]\"}\n"
        );
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "! input required: after bindings: \nerror: input required: after bindings: \n"
        );
    }
}

#[test]
fn stdin_program_modes_bind_only_arguments() {
    for args in [vec!["-", "--", "data"], vec!["--", "data"]] {
        let output = run(
            &args,
            "Print[$ScriptInput]; Print[$ScriptArgs]; InputString[\"p: \"]",
        );
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "$ScriptInput\n{\"data\"}\n"
        );
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "! input required: p: \nerror: input required: p: \n"
        );
    }
}
