use crate::install_core;
use rum_runtime::{
    Session,
    kernel_api::{
        BoundedDestruction, EntropyError, EntropyProvider, EntropySource, expr_as_string,
    },
};
use std::collections::VecDeque;
use std::sync::Mutex;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn core_session_with_entropy(bytes: &[u8]) -> Session {
    let mut session = core_session();
    session.set_entropy_source(Some(SequentialEntropy::provider(bytes)));
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn eval_string_value(session: &mut Session, source: &str) -> String {
    let value = session.eval_text(source).unwrap().value;
    expr_as_string(&value)
        .expect("expression should evaluate to a string")
        .to_string()
}

fn assert_password_failed(source: &str, expected_message: &str) {
    let mut session = core_session();
    let response = session.eval_text(source).unwrap();

    assert_eq!(response.value.to_string(), "$Failed");
    assert!(
        response.messages.iter().any(|message| {
            message.symbol == "RandomPassword"
                && message.tag == "password"
                && message.text.contains(expected_message)
        }),
        "messages={:?}",
        response.messages
    );
}

fn assert_random_choice_weights_failed(source: &str, expected_value: &str, expected_message: &str) {
    let mut session = core_session();
    let response = session.eval_text(source).unwrap();

    assert_eq!(response.value.to_string(), expected_value);
    assert!(
        response.messages.iter().any(|message| {
            message.symbol == "RandomChoice"
                && message.tag == "weights"
                && message.text.contains(expected_message)
        }),
        "messages={:?}",
        response.messages
    );
}

#[derive(Debug)]
struct SequentialEntropy {
    bytes: Mutex<VecDeque<u8>>,
}

impl SequentialEntropy {
    fn provider(bytes: &[u8]) -> EntropyProvider {
        EntropyProvider::new(Self {
            bytes: Mutex::new(bytes.iter().copied().collect()),
        })
    }
}

/// The mutex owns a flat byte deque whose destruction has no recursive ownership chain.
impl BoundedDestruction for SequentialEntropy {}

impl EntropySource for SequentialEntropy {
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError> {
        let mut bytes = self.bytes.lock().unwrap();
        for slot in buf {
            let Some(byte) = bytes.pop_front() else {
                return Err(EntropyError::new("fixed entropy exhausted"));
            };
            *slot = byte;
        }
        Ok(())
    }
}

#[test]
fn character_class_constants_have_pinned_values_and_compose() {
    let mut session = core_session();

    assert_eq!(
        eval_string_value(&mut session, "UppercaseLetters"),
        crate::character_classes::UPPERCASE_LETTERS
    );
    assert_eq!(
        eval_string_value(&mut session, "LowercaseLetters"),
        crate::character_classes::LOWERCASE_LETTERS
    );
    assert_eq!(
        eval_string_value(&mut session, "DigitCharacters"),
        crate::character_classes::DIGIT_CHARACTERS
    );
    assert_eq!(
        eval_string_value(&mut session, "PunctuationCharacters"),
        crate::character_classes::PUNCTUATION_CHARACTERS
    );
    assert_eq!(
        eval_string_value(&mut session, "Alphanumeric"),
        crate::character_classes::ALPHANUMERIC
    );
    assert_eq!(
        eval_string_value(&mut session, "WhitespaceCharacters"),
        crate::character_classes::WHITESPACE_CHARACTERS
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Alphanumeric === UppercaseLetters <> LowercaseLetters <> DigitCharacters"
        ),
        "True"
    );

    let punctuation = format!(
        "PunctuationCharacters === StringJoin[StringCases[{:?}, PunctuationCharacter]]",
        crate::character_classes::PUNCTUATION_CHARACTERS
    );
    assert_eq!(eval_to_string(&mut session, &punctuation), "True");
}

#[test]
fn whitespace_character_constant_has_pinned_value_and_composes() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "WhitespaceCharacters === \" \\t\\n\\r\""),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringLength[WhitespaceCharacters]"),
        "4"
    );
    assert_eq!(
        eval_to_string(&mut session, "Attributes[WhitespaceCharacters]"),
        "List[Protected]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringLength[RandomString[WhitespaceCharacters, 5]]"
        ),
        "5"
    );
}

#[test]
fn character_class_constants_are_protected() {
    let mut session = core_session();

    for (name, value) in crate::character_classes::CHARACTER_CLASS_CONSTANTS {
        let outcome = session
            .eval_text(&format!("{name} = \"x\""))
            .expect("assignment should evaluate");
        assert_eq!(outcome.value.to_string(), "\"x\"");
        assert!(
            outcome.messages.iter().any(|message| {
                message.symbol == "Set"
                    && message.tag == "wrsym"
                    && message.text.contains("Protected")
            }),
            "{name} messages={:?}",
            outcome.messages
        );
        assert_eq!(eval_string_value(&mut session, name), *value);
    }
}

#[test]
fn random_family_heads_report_declared_attributes() {
    let mut session = core_session();

    let cases = [
        ("Attributes[RandomInteger]", "List[Protected]"),
        ("Attributes[RandomPassword]", "List[Protected]"),
        ("Attributes[BlockRandom]", "List[HoldAll, Protected]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn random_text_composes_with_character_class_constants() {
    let mut password_session = core_session_with_entropy(&[
        0, 26, 52, 62, 1, 27, 53, 63, 2, 28, 54, 64, 3, 29, 55, 65, 4, 30, 56, 66,
    ]);
    let password_outcome = password_session
        .eval_text(
            "RandomPassword[Alphanumeric <> PunctuationCharacters, 20, \"Require\" -> {UppercaseLetters, LowercaseLetters, DigitCharacters, PunctuationCharacters}]",
        )
        .expect("RandomPassword should evaluate");
    let password = expr_as_string(&password_outcome.value)
        .expect("RandomPassword should return a string")
        .to_string();

    assert_eq!(password.chars().count(), 20);
    assert!(
        password
            .chars()
            .any(|ch| crate::character_classes::UPPERCASE_LETTERS.contains(ch))
    );
    assert!(
        password
            .chars()
            .any(|ch| crate::character_classes::LOWERCASE_LETTERS.contains(ch))
    );
    assert!(
        password
            .chars()
            .any(|ch| crate::character_classes::DIGIT_CHARACTERS.contains(ch))
    );
    assert!(
        password
            .chars()
            .any(crate::character_classes::is_punctuation_character)
    );

    let mut string_session = core_session();
    let digits = eval_string_value(
        &mut string_session,
        "SeedRandom[42]; RandomString[DigitCharacters, 6]",
    );
    assert_eq!(digits.chars().count(), 6);
    assert!(digits.chars().all(|ch| ch.is_ascii_digit()));
}

#[test]
fn random_integer_default_seed_sequence_is_pinned() {
    let mut session = core_session();
    let mut zero_seed_session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "{RandomInteger[100], RandomInteger[100], RandomInteger[100], RandomInteger[100], RandomInteger[100]}"
        ),
        "List[52, 42, 96, 44, 89]"
    );
    assert_eq!(
        eval_to_string(
            &mut zero_seed_session,
            "SeedRandom[0]; {RandomInteger[100], RandomInteger[100], RandomInteger[100], RandomInteger[100], RandomInteger[100]}"
        ),
        "List[52, 42, 96, 44, 89]"
    );
}

#[test]
fn random_string_default_seed_sequence_is_pinned() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "{RandomString[8], RandomString[\"abc\", 5]}"),
        "List[\"0qgsZKYx\", \"babaa\"]"
    );
}

#[test]
fn random_string_seed_random_reproducibility_is_pinned() {
    let mut first = core_session();
    let mut second = core_session();
    let source = "SeedRandom[42]; RandomString[8]";

    assert_eq!(eval_to_string(&mut first, source), "\"7nKuuJoy\"");
    assert_eq!(
        eval_to_string(&mut first, source),
        eval_to_string(&mut second, source)
    );
}

#[test]
fn random_string_alphabet_duplicates_weight_draws() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "SeedRandom[42]; RandomString[\"aaab\", 16]"),
        "\"bbaaaaaaaaaaaaab\""
    );
}

#[test]
fn random_string_draws_from_unicode_code_points() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "SeedRandom[42]; RandomString[\"日本語\", 12]"),
        "\"語語語本日語語日日日本語\""
    );
}

#[test]
fn random_string_invalid_count_or_empty_alphabet_stays_inert() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "{RandomString[-1], RandomString[\"\", 3]}"),
        "List[RandomString[-1], RandomString[\"\", 3]]"
    );
}

#[test]
fn seed_random_integer_sequence_is_pinned() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[42]; {RandomInteger[100], RandomInteger[100], RandomInteger[100], RandomInteger[100], RandomInteger[100]}"
        ),
        "List[39, 10, 46, 9, 40]"
    );
}

#[test]
fn seed_random_is_deterministic_only_and_reseed_random_has_exact_arity() {
    let mut session = core_session();

    let seed_empty = session.eval_text("SeedRandom[]").unwrap();
    assert_eq!(seed_empty.value.to_string(), "SeedRandom[]");
    assert!(
        seed_empty
            .messages
            .iter()
            .any(|message| message.symbol == "SeedRandom" && message.tag == "argx"),
        "messages={:?}",
        seed_empty.messages
    );

    let reseed_args = session.eval_text("ReseedRandom[5]").unwrap();
    assert_eq!(reseed_args.value.to_string(), "ReseedRandom[5]");
    assert!(
        reseed_args
            .messages
            .iter()
            .any(|message| message.symbol == "ReseedRandom" && message.tag == "argx"),
        "messages={:?}",
        reseed_args.messages
    );
}

#[test]
fn reseed_random_uses_entropy_source_in_unrestricted_session() {
    let bytes = (0_u8..32).collect::<Vec<_>>();
    let mut session = core_session_with_entropy(&bytes);

    let outcome = session.eval_text("ReseedRandom[]").unwrap();
    assert_eq!(outcome.value.to_string(), "Null");
    assert!(
        outcome.messages.is_empty(),
        "messages={:?}",
        outcome.messages
    );

    let mut first = core_session_with_entropy(&bytes);
    let mut second = core_session_with_entropy(&bytes);
    let source =
        "ReseedRandom[]; {RandomInteger[1000000], RandomInteger[1000000], RandomInteger[1000000]}";

    assert_eq!(
        eval_to_string(&mut first, source),
        eval_to_string(&mut second, source)
    );
}

#[test]
fn block_random_restores_the_random_string_stream() {
    let mut with_block = core_session();
    let mut without_block = core_session();

    assert_eq!(
        eval_to_string(
            &mut with_block,
            "SeedRandom[123]; {RandomString[4], BlockRandom[RandomString[4]], RandomString[4]}"
        ),
        "List[\"laD2\", \"jatK\", \"jatK\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut without_block,
            "SeedRandom[123]; {RandomString[4], RandomString[4]}"
        ),
        "List[\"laD2\", \"jatK\"]"
    );
}

#[test]
fn random_password_fixed_entropy_uses_rejection_sampling() {
    let mut session = core_session_with_entropy(&[248, 0, 249, 61, 250, 62, 63]);

    assert_eq!(
        eval_to_string(&mut session, "RandomPassword[4]"),
        "\"A9AB\""
    );
}

#[test]
fn random_password_respects_custom_alphabet_membership() {
    let mut session = core_session_with_entropy(&[0, 1, 2, 3, 4, 5]);

    assert_eq!(
        eval_to_string(&mut session, "RandomPassword[\"abc\", 6]"),
        "\"abcabc\""
    );
}

#[test]
fn random_password_policy_options_are_discoverable_without_changing_draws() {
    let mut options_session = core_session();

    assert_eq!(
        eval_to_string(&mut options_session, "Options[RandomPassword]"),
        "List[Rule[Require, List[]], Rule[NoAdjacentRepeats, False], Rule[Exclude, List[]], Rule[IgnoreCase, False]]"
    );

    let mut password_session = core_session_with_entropy(&[0, 0, 1, 0, 0, 1, 0, 1]);
    assert_eq!(
        eval_to_string(
            &mut password_session,
            "RandomPassword[\"ab\", 4, \"NoAdjacentRepeats\" -> True]"
        ),
        "\"abab\""
    );
}

#[test]
fn random_password_require_policy_resamples_until_groups_are_present() {
    let mut single = core_session_with_entropy(&[0, 0, 0, 1]);
    let mut multiple = core_session_with_entropy(&[0, 0, 0, 0, 2, 0, 5, 1]);

    assert_eq!(
        eval_to_string(
            &mut single,
            "RandomPassword[\"ab\", 2, \"Require\" -> {\"b\"}]"
        ),
        "\"ab\""
    );
    assert_eq!(
        eval_to_string(
            &mut multiple,
            "RandomPassword[\"abc123\", 4, \"Require\" -> {\"c\", \"3\"}]"
        ),
        "\"ca3b\""
    );
}

#[test]
fn random_password_require_policy_rejects_malformed_groups() {
    assert_password_failed(
        "RandomPassword[\"abc\", 8, \"Require\" -> {\"xyz\"}]",
        "required group \"xyz\" is not a subset",
    );
    assert_password_failed(
        "RandomPassword[8, \"Require\" -> {\"!\"}]",
        "required group \"!\" is not a subset",
    );
    assert_password_failed(
        "RandomPassword[\"abc\", 8, \"Require\" -> {\"\"}]",
        "required groups cannot be empty",
    );
}

#[test]
fn random_password_no_adjacent_repeats_policy_resamples() {
    let mut session = core_session_with_entropy(&[0, 0, 1, 0, 0, 1, 0, 1]);

    assert_eq!(
        eval_to_string(
            &mut session,
            "RandomPassword[\"ab\", 4, \"NoAdjacentRepeats\" -> True]"
        ),
        "\"abab\""
    );
}

#[test]
fn random_password_exclude_policy_resamples_and_can_ignore_case() {
    let mut exact = core_session_with_entropy(&[0, 1, 2, 2, 0, 2, 0, 2]);
    let mut ignore_case = core_session_with_entropy(&[0, 1, 2, 3, 2, 3, 0, 2]);

    assert_eq!(
        eval_to_string(
            &mut exact,
            "RandomPassword[\"abc\", 4, \"Exclude\" -> {\"ab\"}]"
        ),
        "\"acac\""
    );
    assert_eq!(
        eval_to_string(
            &mut ignore_case,
            "RandomPassword[\"aBcd\", 4, \"Exclude\" -> {\"ab\"}, \"IgnoreCase\" -> True]"
        ),
        "\"cdac\""
    );
}

#[test]
fn random_password_exclude_policy_rejects_empty_substrings() {
    assert_password_failed(
        "RandomPassword[\"abc\", 8, \"Exclude\" -> {\"\"}]",
        "excluded substrings cannot be empty",
    );
}

#[test]
fn random_password_combined_policy_uses_whole_password_rejection() {
    let mut session = core_session_with_entropy(&[0, 3, 1, 2, 0, 0, 3, 1, 0, 2, 1, 3]);

    assert_eq!(
        eval_to_string(
            &mut session,
            "RandomPassword[\"ab12\", 4, \"Require\" -> {\"a\", \"2\"}, \"NoAdjacentRepeats\" -> True, \"Exclude\" -> {\"a2\"}]"
        ),
        "\"a1b2\""
    );
}

#[test]
fn random_password_policy_cap_exhaustion_fails_closed() {
    assert_password_failed(
        "RandomPassword[\"a\", 2, \"NoAdjacentRepeats\" -> True]",
        "could not satisfy the password policy within 10000 attempts",
    );
}

#[test]
fn random_password_policy_rejects_non_positive_length() {
    assert_password_failed(
        "RandomPassword[0, \"Require\" -> {\"A\"}]",
        "requires a positive length",
    );
}

#[test]
fn random_password_does_not_advance_the_prng_stream() {
    let mut session = core_session_with_entropy(&[0, 1, 2, 3, 4, 5, 6, 7]);

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[7]; first = RandomString[8]; password = RandomPassword[8]; after = RandomString[8]; SeedRandom[7]; {first, password, after, RandomString[8], RandomString[8]}"
        ),
        "List[\"XVoZyDqB\", \"ABCDEFGH\", \"t0CnBKji\", \"XVoZyDqB\", \"t0CnBKji\"]"
    );
}

#[test]
fn block_random_does_not_restore_random_password_entropy() {
    let mut session = core_session_with_entropy(&[0, 1, 2, 3, 4, 5, 6, 7]);

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[123]; {BlockRandom[RandomPassword[4]], RandomPassword[4], RandomString[4], SeedRandom[123]; RandomString[4]}"
        ),
        "List[\"ABCD\", \"EFGH\", \"laD2\", \"laD2\"]"
    );
}

#[test]
fn random_integer_respects_large_exact_ranges() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[7]; r = RandomInteger[{2^80, 2^80 + 1000}]; {r >= 2^80, r <= 2^80 + 1000}"
        ),
        "List[True, True]"
    );
}

#[test]
fn random_integer_and_choice_accept_nested_output_dimensions() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[7]; grid = RandomInteger[9, {2, 3}]; choices = RandomChoice[{a, b, c}, {2, 2}]; {grid, Length[grid], Length /@ grid, ContainsOnly[Flatten[grid], Range[0, 9]], choices, Length[choices], Length /@ choices, ContainsOnly[Flatten[choices], {a, b, c}]}"
        ),
        "List[List[List[7, 5, 8], List[9, 2, 3]], 2, List[3, 3], True, List[List[c, b], List[b, a]], 2, List[2, 2], True]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[7]; first = RandomInteger[9, {2, 3}]; SeedRandom[7]; second = RandomInteger[9, {2, 3}]; SeedRandom[99]; flatInts = RandomInteger[9, 3]; SeedRandom[99]; dimInts = RandomInteger[9, {3}]; SeedRandom[99]; flatChoices = RandomChoice[{left, right}, 2]; SeedRandom[99]; dimChoices = RandomChoice[{left, right}, {2}]; {SameQ[first, second], SameQ[flatInts, dimInts], SameQ[flatChoices, dimChoices], flatInts, flatChoices, RandomInteger[9, {0, 5}], IntegerQ[RandomInteger[9, {}]]}"
        ),
        "List[True, True, True, List[4, 7, 2], List[right, left], List[], True]"
    );

    let mut limited = core_session();
    limited.set_generated_element_limit(3);
    let too_many = limited.resource_limits().generated_element_limit + 1;
    let limited_outcome = limited
        .eval_text(&format!("RandomInteger[9, {{{too_many}, 0}}]"))
        .unwrap();
    assert_eq!(
        limited_outcome.value.to_string(),
        "TerminatedEvaluation[\"GeneratedElementLimit\"]"
    );
    assert!(
        limited_outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim"),
        "messages={:?}",
        limited_outcome.messages
    );
}

#[test]
fn random_choice_and_sample_cover_current_list_contracts() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[42]; {RandomChoice[{a,b,c}, 5], RandomSample[Range[10]]}"
        ),
        "List[List[c, c, c, b, a], List[3, 4, 1, 8, 5, 7, 9, 10, 2, 6]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Sort[RandomSample[Range[10]]]"),
        "List[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"
    );

    let empty_choice = session.eval_text("RandomChoice[{}]").unwrap();
    assert_eq!(empty_choice.value.to_string(), "RandomChoice[List[]]");
    assert!(
        empty_choice
            .messages
            .iter()
            .any(|message| message.symbol == "RandomChoice" && message.tag == "empty")
    );
}

#[test]
fn random_choice_weighted_forms_use_exact_weights_and_fail_closed() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "{RandomChoice[{1, 0, 0} -> {a, b, c}], RandomChoice[{0, 1, 0} -> {a, b, c}, 5]}"
        ),
        "List[a, List[b, b, b, b, b]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[7]; ContainsOnly[RandomChoice[{1/2, 1/2} -> {a, b}, 20], {a, b}]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[19]; first = RandomChoice[{3, 1} -> {a, b}, 12]; SeedRandom[19]; second = RandomChoice[{3, 1} -> {a, b}, 12]; SameQ[first, second]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[23]; grid = RandomChoice[{1, 1} -> {a, b}, {2, 2}]; {Length[grid], Length /@ grid, ContainsOnly[Flatten[grid], {a, b}]}"
        ),
        "List[2, List[2, 2], True]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "ContainsOnly[{RandomChoice[{a, b, c}]}, {a, b, c}]"
        ),
        "True"
    );

    assert_random_choice_weights_failed(
        "RandomChoice[{1, 2} -> {a, b, c}]",
        "RandomChoice[Rule[List[1, 2], List[a, b, c]]]",
        "equal, nonzero length",
    );
    assert_random_choice_weights_failed(
        "RandomChoice[{-1, 2} -> {a, b}]",
        "RandomChoice[Rule[List[-1, 2], List[a, b]]]",
        "non-negative",
    );
    assert_random_choice_weights_failed(
        "RandomChoice[{0, 0} -> {a, b}]",
        "RandomChoice[Rule[List[0, 0], List[a, b]]]",
        "at least one weight",
    );
}

#[test]
fn block_random_restores_the_prng_stream() {
    let mut with_block = core_session();
    let mut without_block = core_session();

    assert_eq!(
        eval_to_string(
            &mut with_block,
            "SeedRandom[123]; {RandomInteger[1000], BlockRandom[RandomInteger[1000]], RandomInteger[1000]}"
        ),
        "List[997, 474, 474]"
    );
    assert_eq!(
        eval_to_string(
            &mut without_block,
            "SeedRandom[123]; {RandomInteger[1000], RandomInteger[1000]}"
        ),
        "List[997, 474]"
    );
}

#[test]
fn random_uuid_and_uuid_q_cover_format_boundaries() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[99]; {RandomUuid[], UuidQ[RandomUuid[]], UuidQ[\"00000000-0000-4000-8000-000000000000\"], UuidQ[\"00000000-0000-5000-8000-000000000000\"], UuidQ[\"00000000-0000-4000-7000-000000000000\"], UuidQ[\"not-a-uuid\"]}"
        ),
        "List[\"cb520df9-5923-46de-ba5d-3c9f7a9cb464\", True, True, False, False, False]"
    );
}
