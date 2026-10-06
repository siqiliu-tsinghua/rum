//! Boundary witnesses observe the actual child InputRequired before the sink
//! retires it. Guest-state rollback alone cannot witness ValueQ's attempted input.
use super::super::descriptors::{Fixture, Variant};
use rum_core::{Expr, SharedArgs};
use rum_runtime::kernel_api::{BuiltinSpec, HoldAttribute, MemoryGrowthContract};
use rum_runtime::{EvalError, RuntimeInputRequest, Session};
use std::cell::RefCell;

std::thread_local! {
    static ATTEMPTS: RefCell<Vec<RuntimeInputRequest>> = const { RefCell::new(Vec::new()) };
}
fn attempt(s: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let output = s.evaluate(args[0].clone());
    if let Err(EvalError::InputRequired { request }) = &output {
        ATTEMPTS.with(|rows| rows.borrow_mut().push(request.clone()));
    }
    // A real tail: no guest work follows this child's outcome. The enclosing
    // repository boundary remains responsible for retiring its suspension.
    output
}
fn evidence(_: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    Ok(ATTEMPTS.with(|rows| {
        Expr::call(
            Expr::system_symbol("List"),
            rows.borrow()
                .iter()
                .map(|r| {
                    Expr::call(
                        Expr::system_symbol("List"),
                        vec![
                            Expr::string(format!("{:?}", r.kind)),
                            Expr::string(r.prompt.clone()),
                        ],
                    )
                })
                .collect(),
        )
    }))
}
fn install(s: &mut Session) {
    ATTEMPTS.with(|rows| rows.borrow_mut().clear());
    s.register_builtin(BuiltinSpec::ordinary_with_hold(
        "OracleBoundaryAttempt",
        "Test-only observation of retired input",
        HoldAttribute::HoldAll,
        attempt,
        MemoryGrowthContract::Delegated,
    ));
    s.register_builtin(BuiltinSpec::ordinary(
        "OracleBoundaryEvidence",
        "Test-only typed request transcript projection",
        evidence,
        MemoryGrowthContract::Delegated,
    ));
}

pub fn fixture(entry: &str, variant: Variant) -> Fixture {
    let (prompt, source, value) = match entry {
        "ValueQ.trial" => (
            "retired trial",
            "ValueQ[(AppendTo[Global`log,1];INPUT;AppendTo[Global`log,2])]".to_owned(),
            "True",
        ),
        "Monitor.display" => (
            "retired display",
            "Monitor[77,(AppendTo[Global`log,1];INPUT;AppendTo[Global`log,2])]".to_owned(),
            "77",
        ),
        "Monitor.evaluationMonitor" | "Monitor.stepMonitor" => {
            let option = if entry.ends_with("evaluationMonitor") {
                "EvaluationMonitor"
            } else {
                "StepMonitor"
            };
            (
                "retired callback",
                format!(
                    "Monitor[1+2,0,{option}:>(AppendTo[Global`log,1];INPUT;AppendTo[Global`log,2])]"
                ),
                "3",
            )
        }
        _ => panic!("unknown boundary {entry}"),
    };
    let source = source.replace(
        "INPUT",
        &format!("OracleBoundaryAttempt[InputString[\"{prompt}\"]]"),
    );
    let mut f = Fixture::new(entry, &source);
    f.configure = install;
    f.expected_value = Some(value.into());
    f.projections
        .push("DeleteDuplicates[OracleBoundaryEvidence[]]".into());
    f.expected_projections
        .push(format!("List[List[\"InputString\", \"{prompt}\"]]"));
    if entry == "ValueQ.trial" {
        f.expected_log = Some("List[]".into());
        f.projections
            .push("Length[OracleBoundaryEvidence[]]".into());
        f.expected_projections
            .push(if variant == Variant::Twice { "2" } else { "1" }.into());
    } else {
        f.projections
            .push("{MemberQ[Global`log,1],FreeQ[Global`log,2]}".into());
        f.expected_projections.push("List[True, True]".into());
    }
    if variant == Variant::Twice {
        f.source = format!("({});({})", f.source, f.source);
    }
    f
}
