use crate::install_core;
use rum_runtime::{EvalOutcome, Session};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn assert_message(outcome: &EvalOutcome, symbol: &str, tag: &str) {
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == symbol && message.tag == tag),
        "missing {symbol}::{tag}; messages: {:?}",
        outcome.messages
    );
}

#[test]
fn d_handles_polynomial_and_rational_function_rules() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("D[x^3 + y^3, x]")
            .unwrap()
            .value
            .to_string(),
        session.eval_text("3*x^2").unwrap().value.to_string()
    );
    assert_eq!(
        session
            .eval_text("D[1/x + 1/y, x]")
            .unwrap()
            .value
            .to_string(),
        session.eval_text("-1*x^-2").unwrap().value.to_string()
    );
    assert_eq!(
        session
            .eval_text("Together[D[(x + 1)/(x - 1), x]]")
            .unwrap()
            .value
            .to_string(),
        session
            .eval_text("-2*(1 + x^2 - 2*x)^-1")
            .unwrap()
            .value
            .to_string()
    );
}

#[test]
fn d_supports_indexed_variables_and_higher_order_specs() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("D[(u[1] + u)^2, u[1]]")
            .unwrap()
            .value
            .to_string(),
        session.eval_text("2*(u + u[1])").unwrap().value.to_string()
    );
    assert_eq!(
        session
            .eval_text("D[f[x], {x, 2}]")
            .unwrap()
            .value
            .to_string(),
        "Derivative[2][f][x]"
    );
    assert_eq!(
        session
            .eval_text("D[f[x, y], x, y]")
            .unwrap()
            .value
            .to_string(),
        "Derivative[1, 1][f][x, y]"
    );
}

#[test]
fn d_uses_abstract_derivative_forms_for_symbol_headed_calls() {
    let mut session = core_session();

    assert_eq!(
        session.eval_text("D[f[x], x]").unwrap().value.to_string(),
        "Derivative[1][f][x]"
    );
    assert_eq!(
        session
            .eval_text("D[f[g[x], y], x]")
            .unwrap()
            .value
            .to_string(),
        session
            .eval_text("Derivative[1, 0][f][g[x], y] * Derivative[1][g][x]")
            .unwrap()
            .value
            .to_string()
    );
}

#[test]
fn d_preserves_nested_chain_rule_factor_order() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("D[Nest[Sin, q, 3], q]")
            .unwrap()
            .value
            .to_string(),
        "Times[Derivative[1][Sin][q], Derivative[1][Sin][Sin[q]], Derivative[1][Sin][Sin[Sin[q]]]]"
    );
}

#[test]
fn d_preserves_plus_zero_elision_and_combination() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("D[x + f[y] + g[x], x]")
            .unwrap()
            .value
            .to_string(),
        "Plus[1, Derivative[1][g][x]]"
    );
}

#[test]
fn d_preserves_times_term_and_factor_order() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("D[f[x] * g[x] * h[y], x]")
            .unwrap()
            .value
            .to_string(),
        "Plus[Times[f[x], h[y], Derivative[1][g][x]], Times[g[x], h[y], Derivative[1][f][x]]]"
    );
}

#[test]
fn d_preserves_rational_and_abstract_power_rules() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("D[x^(3/2), x]")
            .unwrap()
            .value
            .to_string(),
        "Times[3/2, Power[x, 1/2]]"
    );
    assert_eq!(
        session.eval_text("D[x^y, x]").unwrap().value.to_string(),
        "Derivative[1, 0][Power][x, y]"
    );
}

#[test]
fn d_preserves_abstract_call_fallback_and_variable_free_prepass() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("D[f[x][x], x]")
            .unwrap()
            .value
            .to_string(),
        "D[f[x][x], x]"
    );
    assert_eq!(
        session.eval_text("D[f[y], x]").unwrap().value.to_string(),
        "0"
    );
}

#[test]
fn d_keeps_current_variable_boundary_for_pattern_or_non_symbol_heads() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("D[(u[i_] + 1)^2, u[i_]]")
            .unwrap()
            .value
            .to_string(),
        "D[Power[Plus[1, u[Pattern[i, Blank[]]]], 2], u[Pattern[i, Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("D[x^2, h[x][1]]")
            .unwrap()
            .value
            .to_string(),
        "D[Power[x, 2], h[x][1]]"
    );
}

#[test]
fn d_reports_concrete_invalid_variable_and_negative_order_specs() {
    let mut session = core_session();

    let invalid_variable = session.eval_text("D[x^2, 1]").unwrap();
    assert_eq!(invalid_variable.value.to_string(), "D[Power[x, 2], 1]");
    assert_message(&invalid_variable, "D", "dvar");

    let negative_order = session.eval_text("D[x^2, {x, -1}]").unwrap();
    assert_eq!(
        negative_order.value.to_string(),
        "D[Power[x, 2], List[x, -1]]"
    );
    assert_message(&negative_order, "D", "dvord");
}

#[test]
fn d_symbolic_order_and_valid_derivatives_stay_message_free() {
    let mut session = core_session();

    let symbolic_order = session.eval_text("D[x^2, {x, a}]").unwrap();
    assert_eq!(
        symbolic_order.value.to_string(),
        "D[Power[x, 2], List[x, a]]"
    );
    assert!(
        symbolic_order.messages.is_empty(),
        "{:?}",
        symbolic_order.messages
    );

    let polynomial = session.eval_text("D[x^3, {x, 2}]").unwrap();
    assert_eq!(
        polynomial.value.to_string(),
        session.eval_text("6*x").unwrap().value.to_string()
    );
    assert!(polynomial.messages.is_empty(), "{:?}", polynomial.messages);

    let abstract_call = session.eval_text("D[f[x], x]").unwrap();
    assert_eq!(abstract_call.value.to_string(), "Derivative[1][f][x]");
    assert!(
        abstract_call.messages.is_empty(),
        "{:?}",
        abstract_call.messages
    );
}
