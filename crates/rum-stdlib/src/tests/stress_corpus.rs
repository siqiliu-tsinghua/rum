use std::{collections::BTreeMap, thread};

use crate::install_core;
use rum_core::{Atom, Expr, IntValue};
use rum_runtime::{ResourceLimits, Session, SessionConfig};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn core_session_with_limits(limits: ResourceLimits) -> Session {
    let mut session = Session::with_config(SessionConfig::default().with_resource_limits(limits));
    install_core(&mut session);
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn assert_termination(session: &mut Session, source: &str, expected: &str, expected_tag: &str) {
    let outcome = session.eval_text(source).unwrap();
    assert_eq!(outcome.value.to_string(), expected);
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == expected_tag),
        "missing General::{expected_tag} in {:?}",
        outcome.messages
    );
}

fn nested_wrap(depth: usize) -> String {
    let mut expr = "done".to_string();
    for _ in 0..depth {
        expr = format!("wrap[{expr}]");
    }
    expr
}

#[test]
fn guard_adjacent_user_recursion_below_limit_succeeds() {
    let mut session = core_session();
    session.set_recursion_limit(96);
    session.eval_text("peel[done] := done").unwrap();
    session.eval_text("peel[wrap[x_]] := peel[x]").unwrap();

    assert_eq!(
        eval_to_string(&mut session, &format!("peel[{}]", nested_wrap(72))),
        "done"
    );
}

#[test]
fn guard_adjacent_user_recursion_above_limit_terminates_and_session_recovers() {
    let mut session = core_session();
    session.set_recursion_limit(96);
    session.eval_text("peel[done] := done").unwrap();
    session.eval_text("peel[wrap[x_]] := peel[x]").unwrap();

    assert_termination(
        &mut session,
        &format!("peel[{}]", nested_wrap(120)),
        "TerminatedEvaluation[\"RecursionLimit\"]",
        "reclim",
    );
    assert_eq!(eval_to_string(&mut session, "2 + 5"), "7");
}

#[test]
fn reduced_native_stack_limit_returns_stack_limit_and_session_recovers() {
    let mut session = core_session();
    session.set_recursion_limit(4096);
    session.set_native_stack_limit(512 * 1024);

    session
        .eval_text("stackGrow[n_] := stackGrow[n + 1]")
        .unwrap();
    assert_termination(
        &mut session,
        "stackGrow[0]",
        "TerminatedEvaluation[\"StackLimit\"]",
        "reslim",
    );
    assert_eq!(eval_to_string(&mut session, "3 + 4"), "7");
}

fn differentiator_source(term_count: usize) -> String {
    let rules = [
        "d[n_Integer, x] := 0",
        "d[x, x] := 1",
        "d[y_Symbol, x] := 0",
        "d[Plus[a_, rest__], x] := d[a, x] + d[Plus[rest], x]",
        "d[Plus[a_], x] := d[a, x]",
        "d[Times[a_, rest__], x] := d[a, x]*Times[rest] + a*d[Times[rest], x]",
        "d[Times[a_], x] := d[a, x]",
        "d[Power[x, n_Integer], x] := n*x^(n - 1)",
        "d[Power[y_Symbol, n_Integer], x] := 0",
    ];
    let terms = (2..term_count + 2)
        .map(|power| format!("{power}*x^{power}"))
        .collect::<Vec<_>>()
        .join(" + ");

    format!("{}; d[{terms}, x]", rules.join("; "))
}

fn expected_derivative(term_count: usize) -> String {
    let terms = (2..term_count + 2)
        .map(|power| {
            let coefficient = power * power;
            if power == 2 {
                format!("Times[{coefficient}, x]")
            } else {
                format!("Times[{coefficient}, Power[x, {}]]", power - 1)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("Plus[{terms}]")
}

#[test]
fn rule_based_differentiator_handles_generated_polynomial() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, &differentiator_source(8)),
        expected_derivative(8)
    );
}

/// Run with:
/// `cargo test -p rum-stdlib rule_based_differentiator_handles_heavy_generated_polynomial -- --ignored`
#[test]
#[ignore]
fn rule_based_differentiator_handles_heavy_generated_polynomial() {
    thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(run_rule_based_differentiator_handles_heavy_generated_polynomial)
        .expect("differentiator stress thread should start")
        .join()
        .expect("differentiator stress should finish");
}

fn run_rule_based_differentiator_handles_heavy_generated_polynomial() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, &differentiator_source(10)),
        expected_derivative(10)
    );
}

fn peano(n: usize) -> String {
    let mut expr = "z".to_string();
    for _ in 0..n {
        expr = format!("s[{expr}]");
    }
    expr
}

fn peano_multiply_source(left: usize, right: usize) -> String {
    format!(
        "add[z, y_] := y; add[s[x_], y_] := s[add[x, y]]; \
         mul[z, y_] := z; mul[s[x_], y_] := add[y, mul[x, y]]; \
         mul[{}, {}]",
        peano(left),
        peano(right)
    )
}

#[test]
fn peano_arithmetic_multiplies_through_pure_rewriting() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, &peano_multiply_source(12, 13)),
        peano(156)
    );
}

/// Run with:
/// `cargo test -p rum-stdlib peano_arithmetic_heavy_multiplication -- --ignored`
#[test]
#[ignore]
fn peano_arithmetic_heavy_multiplication() {
    thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(run_peano_arithmetic_heavy_multiplication)
        .expect("peano stress thread should start")
        .join()
        .expect("peano stress should finish");
}

fn run_peano_arithmetic_heavy_multiplication() {
    let mut session = core_session();
    session.set_recursion_limit(10_000);

    assert_eq!(
        eval_to_string(&mut session, &peano_multiply_source(30, 52)),
        peano(1_560)
    );
}

#[test]
fn list_pipeline_processes_large_range_with_exact_result() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "Fold[Plus, 0, Map[Length, GatherBy[Select[Range[500], Function[EvenQ[#]]], Function[Mod[#, 10]]]]]"
        ),
        "250"
    );
}

#[test]
fn bug_list_pipeline_range_50000_overflows_native_stack() {
    let mut session = core_session();
    let expected = (1..=50_000)
        .filter(|value| value % 2 == 0)
        .count()
        .to_string();

    assert_eq!(
        eval_to_string(
            &mut session,
            "Fold[Plus, 0, Map[Length, GatherBy[Select[Range[50000], Function[EvenQ[#]]], Function[Mod[#, 10]]]]]"
        ),
        expected
    );
}

fn generated_ini(min_bytes: usize) -> String {
    let mut source = String::from("; generated stress corpus\n");
    let mut section = 0usize;
    while source.len() < min_bytes {
        source.push_str(&format!("[section{section}]\n"));
        source.push_str(&format!("name{section}=value{section}\n"));
        source.push_str(&format!("port{section}={}\n", 10_000 + section));
        section += 1;
    }
    source
}

fn escape_rum_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn ini_token_source(input: &str) -> String {
    format!(
        "TokenizeWith[{{\
         \"comment\" -> (\";\" ~~ Except[\"\\n\"]...), \
         \"section\" -> (\"[\" ~~ Except[\"]\"]... ~~ \"]\"), \
         \"key\" -> (LetterCharacter ~~ WordCharacter... ~~ \"=\"), \
         \"number\" -> NumberString, \
         \"value\" -> (WordCharacter..)}}, \"{}\"]",
        escape_rum_string(input)
    )
}

fn token_spans(expr: &Expr) -> Vec<(String, usize, usize)> {
    expect_list(expr)
        .iter()
        .map(|item| {
            let fields = expect_list(item);
            assert_eq!(fields.len(), 2);
            let class = expect_string(&fields[0]).to_string();
            let span = expect_list(&fields[1]);
            assert_eq!(span.len(), 2);
            (class, expect_usize(&span[0]), expect_usize(&span[1]))
        })
        .collect()
}

fn token_class_counts(tokens: &[(String, usize, usize)]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for (class, _, _) in tokens {
        *counts.entry(class.clone()).or_insert(0) += 1;
    }
    counts
}

fn assert_ini_tokens(input: &str, tokens: &[(String, usize, usize)]) {
    let block_count = input.matches("[section").count();
    let expected_total = 2 + block_count * 8;
    assert_eq!(tokens.len(), expected_total);

    let counts = token_class_counts(tokens);
    assert_eq!(counts.get("comment").copied(), Some(1));
    assert_eq!(counts.get("section").copied(), Some(block_count));
    assert_eq!(counts.get("key").copied(), Some(block_count * 2));
    assert_eq!(counts.get("value").copied(), Some(block_count));
    assert_eq!(counts.get("number").copied(), Some(block_count));
    assert_eq!(counts.get("text").copied(), Some(1 + block_count * 3));

    let comment_len = "; generated stress corpus".chars().count();
    assert_eq!(tokens[0], ("comment".to_string(), 1, comment_len));
    assert_eq!(
        tokens[1],
        ("text".to_string(), comment_len + 1, comment_len + 1)
    );

    let first_section_start = comment_len + 2;
    let first_section_end = first_section_start + "[section0]".chars().count() - 1;
    assert_eq!(
        tokens[2],
        (
            "section".to_string(),
            first_section_start,
            first_section_end
        )
    );

    let source_chars = input.chars().count();
    assert_eq!(
        tokens.last(),
        Some(&("text".to_string(), source_chars, source_chars))
    );
}

#[test]
fn tokenize_with_ini_grammar_handles_generated_100kb_config() {
    let input = generated_ini(100 * 1024);
    let mut limits = ResourceLimits::default();
    limits.generated_element_limit = 200_000;
    limits.iteration_work_limit = 400_000;
    let mut session = core_session_with_limits(limits);

    let value = session.eval_text(&ini_token_source(&input)).unwrap().value;
    let tokens = token_spans(&value);
    assert_ini_tokens(&input, &tokens);
}

/// Run with:
/// `cargo test -p rum-stdlib tokenize_with_ini_grammar_handles_heavy_generated_config -- --ignored`
#[test]
#[ignore]
fn tokenize_with_ini_grammar_handles_heavy_generated_config() {
    let input = generated_ini(1024 * 1024);
    let mut limits = ResourceLimits::default();
    limits.generated_element_limit = 2_000_000;
    limits.iteration_work_limit = 4_000_000;
    let mut session = core_session_with_limits(limits);

    let value = session.eval_text(&ini_token_source(&input)).unwrap().value;
    let tokens = token_spans(&value);
    assert_ini_tokens(&input, &tokens);
}

fn expect_list(expr: &Expr) -> &[Expr] {
    let Expr::Call { head, args, .. } = expr else {
        panic!("expected List, got {expr}");
    };
    assert_eq!(head.as_ref().to_string(), "List");
    args.as_ref()
}

fn expect_string(expr: &Expr) -> &str {
    let Expr::Atom(Atom::String(value)) = expr else {
        panic!("expected String, got {expr}");
    };
    value
}

fn expect_usize(expr: &Expr) -> usize {
    let Expr::Atom(Atom::Integer(value)) = expr else {
        panic!("expected Integer, got {expr}");
    };

    match value {
        IntValue::Small(value) => usize::try_from(*value).expect("span must be non-negative"),
        IntValue::Big(value) => usize::try_from(value.as_ref()).expect("span must fit usize"),
    }
}
