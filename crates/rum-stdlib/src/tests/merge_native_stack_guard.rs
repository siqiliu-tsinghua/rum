#[cfg(not(debug_assertions))]
use std::thread;

use rum_core::Expr;
use rum_runtime::Session;
#[cfg(not(debug_assertions))]
use rum_runtime::kernel_api::{render_association, system_expr};

use crate::install_core;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[cfg(not(debug_assertions))]
fn expected_deep_merge() -> Expr {
    render_association(vec![(
        Expr::string("deep"),
        Expr::call(system_expr("List"), vec![Expr::small_integer(7)]),
    )])
}

#[cfg(not(debug_assertions))]
fn assert_merge_at_physical_height(height: usize) {
    let mut session = core_session();
    let source =
        format!("c = {{\"deep\" -> 7}}; Do[c = {{c}}, {{i, {height}}}]; Merge[c, Identity]");
    let outcome = session
        .eval_text(&source)
        .expect("the Merge certificate should evaluate");
    assert_eq!(outcome.value, expected_deep_merge());
}

#[cfg(not(debug_assertions))]
fn run_on_certificate_thread(name: &str, action: impl FnOnce() + Send + 'static) {
    let joined = thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("the 2 MiB certificate thread should start")
        .join();
    assert!(joined.is_ok(), "certificate thread {name:?} panicked");
}

#[cfg(not(debug_assertions))]
#[test]
fn merge_propagates_the_unique_deep_entry_at_physical_height_4097() {
    run_on_certificate_thread("merge-native-stack-height-4097", || {
        assert_merge_at_physical_height(4_097);
    });
}

#[cfg(not(debug_assertions))]
#[test]
fn merge_shallow_companion_preserves_three_level_flattening() {
    run_on_certificate_thread("merge-native-stack-height-3", || {
        assert_merge_at_physical_height(3);
    });
}

#[test]
#[ignore = "requires rum-runtime/test-native-stack-guard-zero"]
fn merge_zero_native_stack_boundary_fails_in_band_and_session_recovers() {
    let mut session = core_session();
    let exhausted = session
        .eval_text("c = {\"deep\" -> 7}; Do[c = {c}, {i, 12}]; Merge[c, Identity]")
        .expect("guard exhaustion should be returned in band");

    assert_eq!(
        exhausted.value.to_string(),
        "TerminatedEvaluation[\"StackLimit\"]"
    );
    assert!(exhausted.messages.iter().any(|message| {
        message.symbol == "General"
            && message.tag == "reslim"
            && message.text == "evaluation exceeded native stack limit (0 bytes)"
    }));

    let recovered = session
        .eval_text("2 + 5")
        .expect("the session should remain usable after guard exhaustion");
    assert_eq!(recovered.value, Expr::small_integer(7));
}
