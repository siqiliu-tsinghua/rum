//! S4 argument and traversal resolution, with harness frame attribution.
use super::super::descriptors::{Checks, accept};

fn equality(id: &str, source: &str, pauses: &[(&str, &str, &str)], expected: &str) {
    let fixture = super::make(id, source, pauses, Some(expected), "List[]");
    let result = accept(&fixture, Checks::ALL);
    super::super::support::faults::assert_all();
    let accepted = result.unwrap_or_else(|error| panic!("{id}: {error:?}"));
    println!(
        "{} {id} strict equality PASS: {:?}; log={}",
        super::typed_slots::TYPING_RECEIPT,
        accepted.observation,
        accepted.log
    );
}

// harness-frame-census-final.log: F0247 and F0248 each resume once per iteration;
// F0568 finishes after the second pause and proves accumulator uniqueness.
#[test]
fn s4_argument_twice() {
    equality(
        "s4.argument-twice",
        r#"hv={{1,2},{3,4}};hv[[(InputString["first"];1),(InputString["later"];2)]]+=10;hv"#,
        &[("InputString", "first", "1"), ("InputString", "later", "2")],
        "List[List[1, 12], List[3, 4]]",
    );
}

// harness-frame-census-final.log: F0250 and F0251 each resume once.
#[test]
fn s4_rewrite_exprs() {
    equality(
        "s4.rewrite-exprs",
        r#"Replace[1,(InputString["rules"];1->2)]"#,
        &[("InputString", "rules", "1")],
        "2",
    );
}

// harness-frame-census-final.log: F0253 resumes once.
#[test]
fn s4_rewrite_compiled() {
    equality(
        "s4.rewrite-compiled",
        r#"ReplaceAll[1,(InputString["rules"];1->2)]"#,
        &[("InputString", "rules", "1")],
        "2",
    );
}

// harness-frame-census-final.log: F0249 resumes once.
#[test]
fn s4_cases_spec() {
    equality(
        "s4.cases-spec",
        r#"Cases[{2},Rule[(InputString["spec"];2),9]]"#,
        &[("InputString", "spec", "1")],
        "List[9]",
    );
}

// harness-frame-census-final.log: F0247 resumes once, F0254 resumes once, F0568 finishes.
#[test]
fn s4_cases_traversal() {
    equality(
        "s4.cases-traversal",
        r#"Cases[{1},1,(InputString["level"];{1})]"#,
        &[("InputString", "level", "{1}")],
        "List[1]",
    );
}

// harness-frame-census-final.log: F0247 resumes once, F0255 resumes once, F0568 finishes.
#[test]
fn s4_position_traversal() {
    equality(
        "s4.position-traversal",
        r#"Position[{1},1,(InputString["level"];{1})]"#,
        &[("InputString", "level", "{1}")],
        "List[List[1]]",
    );
}

// harness-frame-census-final.log: F0247 resumes once, F0256 resumes once, F0568 finishes.
#[test]
fn s4_count_traversal() {
    equality(
        "s4.count-traversal",
        r#"Count[{1},1,(InputString["level"];{1})]"#,
        &[("InputString", "level", "{1}")],
        "1",
    );
}

// harness-frame-census-final.log: F0249/F0247/F0254 each resume once per iteration;
// this is the shared-from M0249/M0257 route, and F0568 finishes after the level pause.
#[test]
fn s4_cases_shared() {
    equality(
        "s4.cases-shared",
        r#"Cases[{2},Rule[(InputString["spec"];2),9],(InputString["level"];{1})]"#,
        &[
            ("InputString", "spec", "1"),
            ("InputString", "level", "{1}"),
        ],
        "List[9]",
    );
}
