//! S3 builtin completion: If selects a held branch that pauses.
//! InputString -> CompoundExpression continuation (Expr) -> F0278 (Some)
//! -> dispatch consumer -> fixed point -> F0278 for If -> dispatch consumer.
//! Both builtin maps belong to S3; no Function frame is involved.
use super::super::descriptors::{Checks, accept, run};

fn divergence(id: &str, source: &str, expected: &str) {
    let fixture = super::make(
        id,
        source,
        &[("InputString", "sweep", "answer")],
        Some(expected),
        "List[]",
    );
    let result = accept(&fixture, Checks::ALL);
    if result.is_err() {
        println!("{id} queued: {:?}", run(&fixture, true, Checks::ALL));
        println!("{id} resumed: {:?}", run(&fixture, false, Checks::ALL));
    }
    // This check is unconditional, including a failing equality comparison.
    super::super::support::faults::assert_all();
    let accepted = result.unwrap_or_else(|error| panic!("{id}: {error:?}"));
    println!(
        "{} {id} strict equality PASS: {:?}; log={}",
        super::typed_slots::TYPING_RECEIPT,
        accepted.observation,
        accepted.log
    );
}

#[test]
fn s3_builtin_none() {
    divergence(
        "builtin.none",
        r#"If[True, (InputString["sweep"];System`InternalContinuationNone[]), 0]"#,
        "InternalContinuationNone[]",
    );
}

#[test]
fn s3_builtin_some() {
    divergence(
        "builtin.some",
        r#"If[True, (InputString["sweep"];System`InternalContinuationSome[5]), 0]"#,
        "InternalContinuationSome[5]",
    );
}

#[test]
fn s3_builtin_empty_some() {
    divergence(
        "builtin.empty_some",
        r#"If[True, (InputString["sweep"];System`InternalContinuationSome[]), 0]"#,
        "InternalContinuationSome[]",
    );
}

// The List definition is installed only after the scalar callback pauses.
// Its ordinary replacement makes the completed threaded List itself guest data.
#[test]
fn s3_threaded_none() {
    divergence(
        "threaded.none",
        r#"SetAttributes[f,Listable]; f[x_] := (InputString["sweep"]; Unprotect[List]; DownValues[List]={HoldPattern[List[1]] :> System`InternalContinuationNone[]}; 1); f[{1}]"#,
        "InternalContinuationNone[]",
    );
}

// The List definition is installed only after the scalar callback pauses.
// Its ordinary replacement makes the completed threaded List itself guest data.
#[test]
fn s3_threaded_some() {
    divergence(
        "threaded.some",
        r#"SetAttributes[f,Listable]; f[x_] := (InputString["sweep"]; Unprotect[List]; DownValues[List]={HoldPattern[List[1]] :> System`InternalContinuationSome[5]}; 1); f[{1}]"#,
        "InternalContinuationSome[5]",
    );
}

// The List definition is installed only after the scalar callback pauses.
// Its ordinary replacement makes the completed threaded List itself guest data.
#[test]
fn s3_threaded_empty_some() {
    divergence(
        "threaded.empty_some",
        r#"SetAttributes[f,Listable]; f[x_] := (InputString["sweep"]; Unprotect[List]; DownValues[List]={HoldPattern[List[1]] :> System`InternalContinuationSome[]}; 1); f[{1}]"#,
        "InternalContinuationSome[]",
    );
}

#[test]
fn prepared_twice() {
    let mut fixture = super::make(
        "prepared.twice",
        r#"f[InputString["p1"],Unevaluated[kept],(hits++;InputString["p2"]),7]"#,
        &[
            ("InputString", "p1", "answer1"),
            ("InputString", "p2", "answer2"),
        ],
        Some(r#"List["answer1", "answer2", 7]"#),
        "List[]",
    );
    fixture.setup = "f[x_,y_,z_,w_]:={x,z,w};hits=0".into();
    fixture.projections.push("hits".into());
    fixture.expected_projections.push("1".into());
    let result = accept(&fixture, Checks::ALL);
    super::super::support::faults::assert_all();
    result.unwrap();
}

#[test]
fn head_map_twice() {
    let mut fixture = super::make(
        "head.map.twice",
        r#"(AppendTo[Global`log,1];InputString["head1"];AppendTo[Global`log,2];InputString["head2"];f)[7]"#,
        &[
            ("InputString", "head1", "answer1"),
            ("InputString", "head2", "answer2"),
        ],
        Some("8"),
        "List[1, 2]",
    );
    fixture.setup = "f[x_]:=x+1".into();
    let result = accept(&fixture, Checks::ALL);
    super::super::support::faults::assert_all();
    result.unwrap();
}
