//! Test-only completion work after the last child evaluation has returned.
use super::super::super::{
    descriptors::Fixture,
    support::{drive, observe::Observation, session},
};
use rum_core::{Expr, SharedArgs};
use rum_runtime::kernel_api::{
    AbandonmentContext, BuiltinSpec, FrameAbandon, FrameAbandonAction, FrameAction, FrameRootTrace,
    FrameRootVisitor, MemoryGrowthContract, PureFrameAbandon, SuspensionFrame,
};
use rum_runtime::{EvalError, Session};
use std::{sync::Arc, time::Duration};

std::thread_local! { static DONE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }

#[derive(FrameRootTrace, PureFrameAbandon)]
struct SameAfterInput {
    original: Expr,
}

fn finish(s: &mut Session, original: Expr) -> Result<Expr, EvalError> {
    if original.to_string() == "G14LegacyFirst[]" {
        return Ok(Expr::small_integer(7));
    }
    DONE.with(|done| done.set(true));
    if original.to_string() == "G14Grow[]" {
        s.install_system_own_value("G14Payload", Expr::string("x".repeat(8_000_000)));
    } else {
        // 1.5 times the 100 ms limit; both arms cross it at this level's completion.
        std::thread::sleep(Duration::from_millis(150));
    }
    Ok(original)
}
impl SuspensionFrame for SameAfterInput {
    fn resume(self: Arc<Self>, s: &mut Session, outcome: Result<Expr, EvalError>) -> FrameAction {
        FrameAction::complete(outcome.and_then(|_| finish(s, self.original.clone())))
    }
}
fn handler(s: &mut Session, head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    if DONE.with(|done| done.get()) {
        return Ok(Expr::small_integer(77));
    }
    let original = Expr::call(head.clone(), args.to_vec());
    let input = Expr::call(
        Expr::system_symbol("InputString"),
        vec![Expr::string("epilogue")],
    );
    match s.evaluate(input) {
        Ok(_) => finish(s, original),
        Err(error @ EvalError::InputRequired { .. }) => {
            s.insert_suspension_frame(SameAfterInput { original });
            Err(error)
        }
        Err(error) => Err(error),
    }
}
pub(super) fn configure(s: &mut Session) {
    DONE.with(|done| done.set(false));
    for name in ["G14Same", "G14Grow"] {
        s.register_builtin(BuiltinSpec::ordinary(
            name,
            "test-only evaluation completion witness",
            handler,
            MemoryGrowthContract::Delegated,
        ));
    }
}

/// Preserve the full evidence even if strict acceptance rejects an early pause count.
pub(super) fn diagnose(f: &Fixture) {
    let answers: Vec<_> = f.pauses.iter().map(|p| p.answer.as_str()).collect();
    println!(
        "G14 SOURCE {} setup={:?} source={:?}",
        f.id, f.setup, f.source
    );
    let mut sides = Vec::new();
    for queued in [false, true] {
        let mut s = f.session();
        let run = drive::execute(&mut s, &f.source, &answers, queued).unwrap();
        let observation = Observation::from_output(&run.output);
        let projections: Vec<_> = f
            .projections
            .iter()
            .map(|p| s.eval_request(p).unwrap().value)
            .collect();
        println!(
            "G14 OBS {} queued={queued} planned={} actual={} observation={observation:?} projections={projections:?}",
            f.id,
            f.pauses.len(),
            run.requests.len()
        );
        session::fresh(&mut s).unwrap();
        sides.push((observation, projections));
    }
    println!("G14 PAIR {} equal={}", f.id, sides[0] == sides[1]);
}

/// Invoke the installed InputString producer directly so a legacy continuation
/// is inserted before the surrounding evaluator installs any typed consumer.
/// This tests the existing InputString role through the public builtin catalog.
fn legacy_first_handler(
    s: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let input = s
        .registered_builtin_specs()
        .find(|spec| spec.name == "InputString")
        .unwrap()
        .handler
        .unwrap();
    let arguments = SharedArgs::from(vec![Expr::string("widen")]);
    match input(s, &Expr::system_symbol("InputString"), &arguments) {
        Ok(_) => Ok(Expr::small_integer(7)),
        Err(error @ EvalError::InputRequired { .. }) => {
            s.insert_suspension_frame(SameAfterInput {
                original: Expr::call(head.clone(), args.to_vec()),
            });
            Err(error)
        }
        Err(error) => Err(error),
    }
}
pub(super) fn configure_legacy_first(s: &mut Session) {
    s.register_builtin(BuiltinSpec::ordinary(
        "G14LegacyFirst",
        "test-only populated queue widening",
        legacy_first_handler,
        MemoryGrowthContract::Delegated,
    ));
}
