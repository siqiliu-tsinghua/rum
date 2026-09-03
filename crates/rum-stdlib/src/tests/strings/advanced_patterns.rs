use super::*;

#[test]
fn repeated_anchors_and_shortest_longest_extend_string_patterns() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\"aaa\", Repeated[\"a\"]]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\"\", RepeatedNull[\"a\"]]"),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[\"warn\", StringExpression[StartOfString, \"wa\", Repeated[\"r\"], \"n\", EndOfString]]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringContainsQ[\"baaa\", StringExpression[StartOfString, \"a\"]]"
        ),
        "False"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"aaa\", Repeated[\"a\"]]"),
        "List[List[1, 3], List[2, 3], List[3, 3]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"aaab\", Shortest[Repeated[\"a\"]]]"
        ),
        "List[\"a\", \"a\", \"a\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"aaab\", Longest[Repeated[\"a\"]]]"
        ),
        "List[\"aaa\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"\", RepeatedNull[\"a\"]]"),
        "List[\"\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"\", RepeatedNull[\"a\"]]"),
        "List[List[1, 0]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"abc\", EndOfString]"),
        "List[List[4, 3]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"abc\", StartOfString]"),
        "List[List[1, 0]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringReplace[\"abc\", EndOfString -> \"$\"]"),
        "\"abc$\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "{StringCases[\"\", \"a\"...], StringReplace[\"\", \"a\"... -> \"x\"]}"
        ),
        "List[List[\"\"], \"x\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "{StringMatchQ[\"aaa\", Shortest[\"a\"..]], StringMatchQ[\"aaa\", Longest[\"a\"..]]}"
        ),
        "List[True, True]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCount[\"abc\", RepeatedNull[\"a\"]]"),
        "4"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringContainsQ[\"abc\", EndOfString]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringFreeQ[\"abc\", EndOfString]"),
        "False"
    );
}

#[test]
fn bounded_repetition_and_except_extend_string_patterns() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"aaaa\", Repeated[\"a\", {2}]]"),
        "List[\"aa\", \"aa\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"aaaa\", Repeated[\"a\", 2]]"),
        "List[\"aa\", \"aa\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"aa\", RepeatedNull[\"a\", 2]]"),
        "List[\"aa\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"aaaa\", Repeated[\"a\", {2, 3}]]"
        ),
        "List[\"aaa\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"abc\", Except[\"b\"]]"),
        "List[\"a\", \"c\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"abc\", Except[\"b\", \"c\"]]"),
        "List[\"c\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"abc\", Repeated[Except[\"b\"]]]"
        ),
        "List[\"a\", \"c\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"aaaa\", Repeated[\"a\", {2, Infinity}]]"
        ),
        "List[\"aaaa\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[\"aaa\", Repeated[\"a\", {2, Infinity}]]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"aaa\", RepeatedNull[\"a\", {1, Infinity}]]"
        ),
        "List[\"aaa\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\"\", Repeated[\"a\", {0}]]"),
        "List[\"\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"a\", RepeatedNull[\"a\", {0, 2}]]"
        ),
        "List[\"a\", \"\"]"
    );
}

#[test]
fn string_pattern_consumers_reject_malformed_repetition_bounds() {
    let mut session = core_session();

    for (input, symbol, expected) in [
        (
            "StringCases[\"a\", Repeated[\"a\", -1]]",
            "Repeated",
            "StringCases[\"a\", Repeated[\"a\", -1]]",
        ),
        (
            "StringMatchQ[\"a\", RepeatedNull[\"a\", {2, 1}]]",
            "RepeatedNull",
            "StringMatchQ[\"a\", RepeatedNull[\"a\", List[2, 1]]]",
        ),
        (
            "StringReplace[\"a\", Repeated[\"a\", x] -> \"b\"]",
            "Repeated",
            "StringReplace[\"a\", Rule[Repeated[\"a\", x], \"b\"]]",
        ),
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{input}");
        assert!(outcome.messages.iter().any(|message| {
            message.symbol == symbol
                && message.tag == "range"
                && message.text.contains("0 <= min <= max")
        }));
    }
}

#[test]
fn character_class_pattern_atoms_extend_the_current_string_pattern_subset() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"a1_ \", {LetterCharacter, DigitCharacter, WordCharacter, WhitespaceCharacter}]"
        ),
        "List[\"a\", \"1\", \"_\", \" \"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[\"a1_\", StringExpression[LetterCharacter, DigitCharacter, WordCharacter]]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringReplace[\"ab12\", DigitCharacter -> \"#\"]"
        ),
        "\"ab##\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringContainsQ[\"abc\", WordCharacter]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringCases[\" \\t\", WhitespaceCharacter]"),
        "List[\" \", \"\\t\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"ab, cd! ef\", PunctuationCharacter]"
        ),
        "List[\",\", \"!\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"x\\ny\", StartOfLine]"),
        "List[List[1, 0], List[3, 2]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPosition[\"x\\ny\", EndOfLine]"),
        "List[List[2, 1], List[4, 3]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringReplace[\"x\\ny\", StartOfLine -> \"> \"]"
        ),
        "\"> x\\n> y\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"one two\", WordBoundary ~~ WordCharacter]"
        ),
        "List[\"o\", \"t\"]"
    );
}

#[test]
fn regular_expression_patterns_extend_the_current_string_pattern_subset() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"ab12cd345\", RegularExpression[\"\\\\d+\"]]"
        ),
        "List[\"12\", \"345\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[\"ab12\", RegularExpression[\"[a-z]+\\\\d+\"]]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringReplace[\"ab12cd34\", RegularExpression[\"\\\\d+\"] -> \"#\"]"
        ),
        "\"ab#cd#\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"foo bar\", RegularExpression[\"(foo|bar)\"]]"
        ),
        "List[\"foo\", \"bar\"]"
    );
}

#[test]
fn regular_expression_group_nesting_is_capped_and_far_past_shapes_stay_inert() {
    let mut session = core_session();

    fn nested_group_pattern(levels: usize) -> String {
        format!("{}a{}", "(".repeat(levels), ")".repeat(levels))
    }

    // At the cap the regex still compiles and evaluates.
    assert_eq!(
        eval_to_string(
            &mut session,
            &format!(
                "StringMatchQ[\"a\", RegularExpression[\"{}\"]]",
                nested_group_pattern(256)
            )
        ),
        "True"
    );

    // One past the cap and far past it, the compile fails in band the same
    // way any invalid regular expression does: the call stays inert and the
    // session keeps evaluating.
    for levels in [257, 20_000] {
        let source = format!(
            "StringMatchQ[\"a\", RegularExpression[\"{}\"]]",
            nested_group_pattern(levels)
        );
        assert_eq!(eval_to_string(&mut session, &source), source);
    }

    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[\"ab12\", RegularExpression[\"\\\\d+\"]]"
        ),
        "False"
    );
}

#[test]
fn symbolic_blank_patterns_extend_the_current_string_pattern_subset() {
    let mut session = core_session();
    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\"a\", _]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\"ab\", __]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\"\", ___]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringContainsQ[\"abc\", __]"),
        "True"
    );
}
