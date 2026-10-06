use super::*;
use std::{cell::Cell, fs, path::Path, process::Command};

#[derive(FrameRootTrace)]
struct Frozen;
impl FrozenOptionReadContinuation<bool> for Frozen {
    fn finish(&self, _session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        Ok(Expr::integer(BigInt::from(if value { 7 } else { 9 })))
    }
}
struct Working<'a> {
    finished: &'a Cell<usize>,
    frozen: &'a Cell<usize>,
}
impl OptionReadContinuation<bool> for Working<'_> {
    type Frozen = Frozen;
    fn finish(self, _session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        self.finished.set(self.finished.get() + 1);
        Ok(Expr::integer(BigInt::from(if value { 7 } else { 9 })))
    }
    fn freeze(self) -> Self::Frozen {
        self.frozen.set(self.frozen.get() + 1);
        Frozen
    }
}

#[test]
fn read_phase_success_fallback_and_ordinary_error_do_not_freeze() {
    let mut session = Session::new();
    let finished = Cell::new(0);
    let frozen = Cell::new(0);
    let next = || Working {
        finished: &finished,
        frozen: &frozen,
    };
    assert_eq!(
        session
            .system_option_bool("Absent577", "Heads", true, next())
            .unwrap(),
        Expr::integer(BigInt::from(7))
    );
    session.set_options_for_symbol(
        Symbol::system("Reader577"),
        SymbolOptions::from_entries(vec![SymbolOptionEntry {
            name: Symbol::system("Heads"),
            value: system_expr("False"),
            delayed: false,
        }]),
    );
    assert_eq!(
        session
            .system_option_bool("Reader577", "Heads", true, next())
            .unwrap(),
        Expr::integer(BigInt::from(9))
    );
    assert_eq!(finished.get(), 2);
    let error = EvalError::Return {
        value: Expr::integer(BigInt::from(11)),
    };
    let result =
        read_phase::suspend_option_read(&mut session, error, read_phase::BoolOptionDecode, next());
    assert!(matches!(result, Err(EvalError::Return { .. })));
    assert_eq!(finished.get(), 2);
    assert_eq!(frozen.get(), 0);
    assert!(session.pending_input_request().is_none());
}

#[test]
fn read_phase_compile_controls_pin_missing_private_and_unrooted_errors() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let scratch = workspace.join(".temp/brief-578/compile-controls");
    fs::create_dir_all(&scratch).unwrap();
    fs::write(scratch.join("Cargo.toml"), format!(
        "[package]\nname = \"option-read-proof\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n[dependencies]\nrum-runtime = {{ path = {:?} }}\nrum-core = {{ path = {:?} }}\n[[bin]]\nname = \"option-read-proof\"\npath = \"fixture.rs\"\n",
        workspace.join("crates/rum-runtime"), workspace.join("crates/rum-core")
    )).unwrap();
    let generic = r#"
use rum_runtime::kernel_api::{Session, OptionReadContinuation};
fn consume<C: OptionReadContinuation<bool>>(s: &mut Session, next: C) {
    let _ = s.system_option_bool("Depth", "Heads", false, next);
}
fn main() {}
"#;
    let rooted = r#"
use rum_runtime::kernel_api::{Session, EvalError, FrameRootTrace, FrameRootVisitor, FrozenOptionReadContinuation};
use rum_core::Expr;
#[derive(FrameRootTrace)]
struct Capture { value: Expr }
#[derive(FrameRootTrace)]
struct Frozen { capture: Capture }
impl FrozenOptionReadContinuation<bool> for Frozen {
    fn finish(&self, _: &mut Session, _: bool) -> Result<Expr, EvalError> { Ok(self.capture.value.clone()) }
}
fn main() {}
"#;
    let private = r#"
use rum_runtime::kernel_api::Session;
use rum_core::Symbol;
fn consume(s: &mut Session) {
    let _ = s.option_value_for_symbol(&Symbol::system("Depth"), &Symbol::system("Heads"));
}
fn main() {}
"#;
    for (name, source, error) in [
        ("missing-pass", generic.to_string(), None),
        (
            "missing-fail",
            generic.replace("false, next", "false"),
            Some("E0061"),
        ),
        ("private-pass", generic.to_string(), None),
        ("private-fail", private.to_string(), Some("E0624")),
        ("root-pass", rooted.to_string(), None),
        ("runtime-root-pass", rooted.replace("<bool>", "<Option<Expr>>").replace("_: bool", "_: Option<Expr>"), None),
        ("syntax-root-pass", rooted.replace("<bool>", "<Expr>").replace("_: bool", "_: Expr"), None),
        ("runtime-root-fail", rooted.replace("<bool>", "<Option<Expr>>").replace("_: bool", "_: Option<Expr>").replacen("#[derive(FrameRootTrace)]", "", 1), Some("E0277")),
        ("syntax-root-fail", rooted.replace("<bool>", "<Expr>").replace("_: bool", "_: Expr").replacen("#[derive(FrameRootTrace)]", "", 1), Some("E0277")),
        ("runtime-private-fail", private.replace("option_value_for_symbol(&Symbol::system(\"Depth\"), &Symbol::system(\"Heads\"))", "runtime_option_expr_for_symbol(None, &Symbol::system(\"Monitor\"), \"StepMonitor\")"), Some("E0624")),
        ("syntax-private-fail", private.replace("option_value_for_symbol(&Symbol::system(\"Depth\"), &Symbol::system(\"Heads\"))", "eval_option_syntax(rum_core::Expr::small_integer(1))"), Some("E0624")),
        (
            "root-fail",
            rooted.replacen("#[derive(FrameRootTrace)]", "", 1),
            Some("E0277"),
        ),
    ] {
        let fixture = scratch.join(format!("{name}.rs"));
        fs::write(&fixture, source).unwrap();
        fs::copy(&fixture, scratch.join("fixture.rs")).unwrap();
        let output = Command::new("python3")
            .arg(workspace.join("scripts/run-bounded.py"))
            .args([
                "--timeout",
                "60",
                "--",
                "cargo",
                "check",
                "--offline",
                "--manifest-path",
            ])
            .arg(scratch.join("Cargo.toml"))
            .arg("--target-dir")
            .arg(scratch.join("target"))
            .output()
            .unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        fs::write(scratch.join(format!("{name}.stderr")), &stderr).unwrap();
        fs::write(scratch.join(format!("{name}.stdout")), &output.stdout).unwrap();
        assert_eq!(output.status.success(), error.is_none(), "{name}: {stderr}");
        if let Some(error) = error {
            assert!(
                stderr.contains(&format!("error[{error}]")),
                "{name}: {stderr}"
            );
            assert_eq!(
                stderr.matches("error[E").count(),
                1,
                "no unrelated failure: {stderr}"
            );
        }
        println!("{name}: success={}\n{stderr}", output.status.success());
    }
}

impl FrozenOptionReadContinuation<Option<Expr>> for Frozen {
    fn finish(&self, _: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        Ok(value.unwrap_or_else(|| system_expr("Absent578")))
    }
}
impl FrozenOptionReadContinuation<Expr> for Frozen {
    fn finish(&self, _: &mut Session, value: Expr) -> Result<Expr, EvalError> {
        Ok(value)
    }
}
impl OptionReadContinuation<Option<Expr>> for Working<'_> {
    type Frozen = Frozen;
    fn finish(self, _: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        self.finished.set(self.finished.get() + 1);
        Ok(value.unwrap_or_else(|| system_expr("Absent578")))
    }
    fn freeze(self) -> Self::Frozen {
        self.frozen.set(self.frozen.get() + 1);
        Frozen
    }
}
impl OptionReadContinuation<Expr> for Working<'_> {
    type Frozen = Frozen;
    fn finish(self, _: &mut Session, value: Expr) -> Result<Expr, EvalError> {
        self.finished.set(self.finished.get() + 1);
        Ok(value)
    }
    fn freeze(self) -> Self::Frozen {
        self.frozen.set(self.frozen.get() + 1);
        Frozen
    }
}
#[test]
fn runtime_and_syntax_reads_complete_without_ordinary_capture() {
    let mut session = Session::new();
    let finished = Cell::new(0);
    let frozen = Cell::new(0);
    let next = || Working {
        finished: &finished,
        frozen: &frozen,
    };
    let target = Symbol::system("Reader578");
    assert_eq!(
        session
            .runtime_option_expr_for_symbol(None, &target, "Value", next())
            .unwrap(),
        system_expr("Absent578")
    );
    for delayed in [false, true] {
        let value = Expr::small_integer(23);
        session.set_options_for_symbol(
            target,
            SymbolOptions::from_entries(vec![SymbolOptionEntry {
                name: Symbol::system("Value"),
                value: value.clone(),
                delayed,
            }]),
        );
        assert_eq!(
            session
                .runtime_option_expr_for_symbol(None, &target, "Value", next())
                .unwrap(),
            value
        );
    }
    let value = Expr::small_integer(31);
    assert_eq!(
        session.eval_option_syntax(value.clone(), next()).unwrap(),
        value
    );
    assert_eq!(finished.get(), 4);
    for syntax in [false, true] {
        let error = EvalError::Return {
            value: Expr::small_integer(41),
        };
        let result = if syntax {
            read_phase::suspend_option_read(
                &mut session,
                error,
                read_phase::OptionSyntaxDecode,
                next(),
            )
        } else {
            read_phase::suspend_option_read(
                &mut session,
                error,
                read_phase::RuntimeOptionDecode,
                next(),
            )
        };
        assert!(matches!(result, Err(EvalError::Return { .. })));
    }
    assert_eq!(finished.get(), 4);
    assert_eq!(frozen.get(), 0);
    assert!(session.pending_input_request().is_none());
}
