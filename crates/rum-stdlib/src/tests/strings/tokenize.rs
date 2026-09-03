use rum_core::{Atom, IntValue};

use super::*;

#[test]
fn tokenize_with_reports_total_ascending_spans_across_inputs() {
    let mut session = core_session();

    for (source, expr) in [
        ("", "TokenizeWith[{}, \"\"]"),
        ("abc", "TokenizeWith[{}, \"abc\"]"),
        (
            "日本 x1🙂",
            "TokenizeWith[{\"word\" -> (WordCharacter..)}, \"日本 x1🙂\"]",
        ),
        (
            "a🙂b",
            "TokenizeWith[{\"letter\" -> LetterCharacter}, \"a🙂b\"]",
        ),
    ] {
        let spans = eval_token_spans(&mut session, expr);
        assert_total_coverage(source, &spans);
    }
}

#[test]
fn tokenize_with_uses_longest_match_then_earliest_rule() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "TokenizeWith[{\"a\" -> \"aa\", \"b\" -> \"a\"}, \"aaa\"]"
        ),
        "List[List[\"a\", List[1, 2]], List[\"b\", List[3, 3]]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "TokenizeWith[{\"a\" -> \"a\", \"b\" -> \"a\"}, \"a\"]"
        ),
        "List[List[\"a\", List[1, 1]]]"
    );
}

#[test]
fn tokenize_with_skips_zero_width_rules_and_coalesces_text() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "TokenizeWith[{\"start\" -> StartOfString, \"word\" -> (WordCharacter..)}, \"ab -- cd\"]"
        ),
        "List[List[\"word\", List[1, 2]], List[\"text\", List[3, 6]], List[\"word\", List[7, 8]]]"
    );
}

#[test]
fn tokenize_with_fails_closed_for_invalid_inputs() {
    let mut session = core_session();

    for expr in [
        "TokenizeWith[{\"n\" -> NumberString}, 10]",
        "TokenizeWith[{n -> NumberString}, \"1\"]",
        "TokenizeWith[{\"x\" -> {}}, \"x\"]",
    ] {
        let result = session.eval_text(expr).unwrap();
        assert_eq!(result.value.to_string(), "$Failed");
        assert!(
            result
                .messages
                .iter()
                .any(|message| message.symbol == "TokenizeWith" && message.tag == "tok")
        );
    }
}

#[test]
fn tokenize_with_ini_grammar_reports_exact_spans() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "TokenizeWith[{\"comment\" -> (\";\" ~~ Except[\"\\n\"]...), \"section\" -> (\"[\" ~~ Except[\"]\"]... ~~ \"]\"), \"key\" -> (LetterCharacter ~~ WordCharacter... ~~ \"=\"), \"number\" -> NumberString, \"value\" -> (WordCharacter..)}, \";top\\n[main]\\nname=rum\\nport=42\"]"
        ),
        "List[List[\"comment\", List[1, 4]], List[\"text\", List[5, 5]], List[\"section\", List[6, 11]], List[\"text\", List[12, 12]], List[\"key\", List[13, 17]], List[\"value\", List[18, 20]], List[\"text\", List[21, 21]], List[\"key\", List[22, 26]], List[\"number\", List[27, 28]]]"
    );
}

#[test]
fn tokenize_with_spans_round_trip_through_string_take() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "StringJoin[Map[StringTake[\"日本 x1\", #[[2]]] &, TokenizeWith[{\"word\" -> (WordCharacter..)}, \"日本 x1\"]]]"
        ),
        "\"日本 x1\""
    );
}

#[test]
fn rum_tokens_reports_total_ascending_spans_across_inputs() {
    let mut session = core_session();

    for (source, expr) in [
        ("", "RumTokens[\"\"]"),
        ("f[x]", "RumTokens[\"f[x]\"]"),
        ("日本 x1", "RumTokens[\"日本 x1\"]"),
        ("x 🙂 y", "RumTokens[\"x 🙂 y\"]"),
        ("\"unclosed", "RumTokens[\"\\\"unclosed\"]"),
    ] {
        let spans = eval_token_spans(&mut session, expr);
        assert_total_coverage(source, &spans);
    }
}

#[test]
fn rum_tokens_matches_pre_change_offset_goldens() {
    let mut session = core_session();

    for (input, expected) in [
        ("RumTokens[\"\"]", "List[]"),
        (
            "RumTokens[\"f[x]\"]",
            "List[List[\"symbol\", List[1, 1]], List[\"[\", List[2, 2]], List[\"symbol\", List[3, 3]], List[\"]\", List[4, 4]]]",
        ),
        (
            "RumTokens[\"f[\\\"中文\\\"]\"]",
            "List[List[\"symbol\", List[1, 1]], List[\"[\", List[2, 2]], List[\"string\", List[3, 6]], List[\"]\", List[7, 7]]]",
        ),
        (
            "RumTokens[\"a (* c *) + b\"]",
            "List[List[\"symbol\", List[1, 1]], List[\"whitespace\", List[2, 2]], List[\"comment\", List[3, 9]], List[\"whitespace\", List[10, 10]], List[\"+\", List[11, 11]], List[\"whitespace\", List[12, 12]], List[\"symbol\", List[13, 13]]]",
        ),
        (
            "RumTokens[\"identifier\"]",
            "List[List[\"symbol\", List[1, 10]]]",
        ),
    ] {
        assert_eq!(eval_to_string(&mut session, input), expected);
    }
}

#[test]
fn rum_tokens_preserves_trivia_and_operator_coverage() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "RumTokens[\"f[x_] := x + 1 (* c *)\"]"),
        "List[List[\"symbol\", List[1, 1]], List[\"[\", List[2, 2]], List[\"symbol\", List[3, 3]], List[\"_\", List[4, 4]], List[\"]\", List[5, 5]], List[\"whitespace\", List[6, 6]], List[\":=\", List[7, 8]], List[\"whitespace\", List[9, 9]], List[\"symbol\", List[10, 10]], List[\"whitespace\", List[11, 11]], List[\"+\", List[12, 12]], List[\"whitespace\", List[13, 13]], List[\"integer\", List[14, 14]], List[\"whitespace\", List[15, 15]], List[\"comment\", List[16, 22]]]"
    );
}

#[test]
fn rum_tokens_recovers_malformed_input_with_error_spans() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "RumTokens[\"\\\"unclosed\"]"),
        "List[List[\"error\", List[1, 9]]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "RumTokens[\"x 🙂 y\"]"),
        "List[List[\"symbol\", List[1, 1]], List[\"whitespace\", List[2, 2]], List[\"error\", List[3, 3]], List[\"whitespace\", List[4, 4]], List[\"symbol\", List[5, 5]]]"
    );
}

#[test]
fn rum_tokens_reports_multibyte_code_point_positions() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "RumTokens[\"日本 x1\"]"),
        "List[List[\"symbol\", List[1, 2]], List[\"whitespace\", List[3, 3]], List[\"symbol\", List[4, 5]]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringJoin[Map[StringTake[\"日本 x1\", #[[2]]] &, RumTokens[\"日本 x1\"]]]"
        ),
        "\"日本 x1\""
    );
}

fn eval_token_spans(session: &mut Session, input: &str) -> Vec<(String, usize, usize)> {
    let expr = session.eval_text(input).unwrap().value;
    token_spans(&expr)
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

fn assert_total_coverage(source: &str, spans: &[(String, usize, usize)]) {
    let mut expected_start = 1usize;

    for (_, start, end) in spans {
        assert_eq!(*start, expected_start);
        assert!(*start <= *end);
        expected_start = end + 1;
    }

    assert_eq!(expected_start, source.chars().count() + 1);
}

fn expect_list(expr: &Expr) -> &[Expr] {
    let Expr::Call { head, args } = expr else {
        panic!("expected List, got {expr}");
    };
    assert_eq!(head.as_ref(), &system_expr("List"));
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
