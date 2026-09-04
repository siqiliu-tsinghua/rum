use crate::install_core;
use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::system_expr;
use rum_runtime::{RuntimeMessage, Session, native_stack_limit_for_declared_stack};

const TOML_DEPTH_ACCEPTANCE_STACK_BYTES: usize = 16 * 1024 * 1024;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn eval_string_atom(session: &mut Session, source: &str) -> String {
    match session.eval_text(source).unwrap().value {
        Expr::Atom(Atom::String(value)) => value,
        other => panic!("expected string atom, got {other}"),
    }
}

fn from_toml_call(source: &str) -> Expr {
    Expr::call(system_expr("FromToml"), vec![Expr::string(source)])
}

fn from_toml_to_string(session: &mut Session, source: &str) -> String {
    let value = session.evaluate(from_toml_call(source)).unwrap();
    let messages = session.take_messages();
    assert!(messages.is_empty(), "{source}: {messages:?}");
    value.to_string()
}

fn assert_failed_with_message(
    value: Expr,
    messages: Vec<RuntimeMessage>,
    symbol: &str,
    tag: &str,
    needle: &str,
) {
    assert_eq!(value.to_string(), "$Failed");
    assert!(
        messages.iter().any(|message| {
            message.symbol == symbol && message.tag == tag && message.text.contains(needle)
        }),
        "missing {symbol}::{tag} containing {needle:?}; messages: {messages:?}"
    );
}

const CODEC_DEPTH_LIMIT: usize = 256;

fn nested_toml_arrays(levels: usize) -> String {
    format!("a = {}{}\n", "[".repeat(levels), "]".repeat(levels))
}

fn nested_toml_inline_tables(levels: usize) -> String {
    format!("a = {}0{}\n", "{ k = ".repeat(levels), " }".repeat(levels))
}

fn dotted_toml_path(parts: usize) -> String {
    std::iter::repeat_n("k", parts)
        .collect::<Vec<_>>()
        .join(".")
}

fn assert_toml_depth_failure(session: &mut Session, source: &str, offset: usize) {
    let value = session.evaluate(from_toml_call(source)).unwrap();
    let messages = session.take_messages();
    assert_eq!(value.to_string(), "$Failed");
    assert_eq!(messages.len(), 1, "{source}: {messages:?}");
    assert_eq!(messages[0].symbol, "FromToml");
    assert_eq!(messages[0].tag, "depth");
    assert_eq!(
        messages[0].text,
        format!(
            "TOML nesting exceeds the fixed depth limit of {CODEC_DEPTH_LIMIT} at byte offset {offset}."
        )
    );
}

fn assert_toml_depth_success(session: &mut Session, source: &str) {
    let value = session.evaluate(from_toml_call(source)).unwrap();
    let messages = session.take_messages();
    assert_ne!(value.to_string(), "$Failed");
    assert!(messages.is_empty(), "{source}: {messages:?}");
}

#[test]
fn from_toml_parses_realistic_config() {
    let mut session = core_session();
    let source = r#"
title = "rum service"

[server]
owner.name = "ops"
ports = [8000, 8001]
meta = { role = "primary", weight = 0.5 }
note = """
line\nwrap \"ok\""""
when = 1970-01-01T01:00:00+01:00
date = 2026-07-06

[[servers]]
name = "edge"
port = 8080

[[servers]]
name = "batch"
port = 9090
"#;

    assert_eq!(
        from_toml_to_string(&mut session, source),
        "Association[Rule[\"title\", \"rum service\"], Rule[\"server\", Association[Rule[\"owner\", Association[Rule[\"name\", \"ops\"]]], Rule[\"ports\", List[8000, 8001]], Rule[\"meta\", Association[Rule[\"role\", \"primary\"], Rule[\"weight\", 1/2]]], Rule[\"note\", \"line\\nwrap \\\"ok\\\"\"], Rule[\"when\", 0], Rule[\"date\", List[2026, 7, 6]]]], Rule[\"servers\", List[Association[Rule[\"name\", \"edge\"], Rule[\"port\", 8080]], Association[Rule[\"name\", \"batch\"], Rule[\"port\", 9090]]]]]"
    );
}

#[test]
fn from_toml_numbers_are_exact() {
    let mut session = core_session();
    let source = "\
decimal = 0.1
million = 1_000_000
hex = 0x10
oct = 0o10
bin = 0b1010
exp = 1.5e3
neg = -2
";

    assert_eq!(
        from_toml_to_string(&mut session, source),
        "Association[Rule[\"decimal\", 1/10], Rule[\"million\", 1000000], Rule[\"hex\", 16], Rule[\"oct\", 8], Rule[\"bin\", 10], Rule[\"exp\", 1500], Rule[\"neg\", -2]]"
    );
}

#[test]
fn from_toml_maps_datetime_forms() {
    let mut session = core_session();
    let source = "\
offset = 1970-01-01T01:00:00+01:00
localDateTime = 2026-07-06T12:13:14.5
localDate = 2026-07-06
localTime = 12:13:14.5
";

    assert_eq!(
        from_toml_to_string(&mut session, source),
        "Association[Rule[\"offset\", 0], Rule[\"localDateTime\", List[2026, 7, 6, 12, 13, 29/2]], Rule[\"localDate\", List[2026, 7, 6]], Rule[\"localTime\", List[12, 13, 29/2]]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "List[Lookup[FromToml[\"offset = 1970-01-01T01:00:00+01:00\"], \"offset\"], FromDateString[\"1970-01-01T00:00:00Z\"]]"
        ),
        "List[0, 0]"
    );
}

#[test]
fn from_toml_multiline_strings_trim_and_continue_lines() {
    let mut session = core_session();
    let source = "a = \"\"\"\nfirst\\\n    second\"\"\"\nb = '''\ntrimmed'''\n";

    assert_eq!(
        from_toml_to_string(&mut session, source),
        "Association[Rule[\"a\", \"firstsecond\"], Rule[\"b\", \"trimmed\"]]"
    );
}

#[test]
fn from_toml_rejects_invalid_input_fail_closed() {
    let mut session = core_session();

    for (source, needle) in [
        ("a = 1\na = 2\n", "duplicate key"),
        ("a = 1\na.b = 2\n", "dotted-key conflict"),
        ("a = inf\n", "inf and nan"),
        ("d = 2026-02-30\n", "day must be"),
        ("s = \"\\x\"\n", "invalid string escape"),
        ("[a]\n[a]\n", "duplicate table"),
        ("a = { b = 1 }\na.c = 2\n", "inline table"),
    ] {
        let value = session.evaluate(from_toml_call(source)).unwrap();
        let messages = session.take_messages();
        assert_failed_with_message(value, messages, "FromToml", "toml", needle);
    }
}

#[test]
fn to_toml_selects_tables_quotes_keys_and_preserves_assignment_order() {
    let mut session = core_session();

    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToToml[<|\"root\" -> 1, \"a.b\" -> 2, \"child\" -> <|\"x\" -> True|>, \"servers\" -> {<|\"name\" -> \"edge\"|>, <|\"name\" -> \"batch\"|>}|>]"
        ),
        "root = 1\n\"a.b\" = 2\n\n[child]\nx = true\n\n[[servers]]\nname = \"edge\"\n\n[[servers]]\nname = \"batch\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "FromToml[ToToml[<|\"k\" -> <|\"x\" -> 1|>, \"arr\" -> {1, 1/4, True, \"z\"}|>]]"
        ),
        "Association[Rule[\"k\", Association[Rule[\"x\", 1]]], Rule[\"arr\", List[1, 1/4, True, \"z\"]]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "SameQ[FromToml[ToToml[<|\"k\" -> <|\"x\" -> 1|>, \"arr\" -> {1, 1/4, True, \"z\"}|>]], <|\"k\" -> <|\"x\" -> 1|>, \"arr\" -> {1, 1/4, True, \"z\"}|>]"
        ),
        "True"
    );

    let outcome = session.eval_text("ToToml[<|\"x\" -> 1/3|>]").unwrap();
    assert_failed_with_message(
        outcome.value,
        outcome.messages,
        "ToToml",
        "toml",
        "non-terminating Rational",
    );
}

/// Golden corpus captured from the release CLI before the linear-encoding
/// rework (brief-310): 18 output shapes plus 8 error shapes. The expected
/// strings are byte-for-byte what the encoder emitted at capture time; any
/// drift is a behavior change in emitted TOML, not a formatting nit.
#[test]
fn to_toml_golden_corpus_is_byte_stable() {
    let mut session = core_session();

    for (source, expected) in [
        (
            r#"<|"a" -> 1, "b" -> "x", "c" -> True, "d" -> False, "e" -> 1/4, "f" -> {1, 2, 3}|>"#,
            "a = 1\nb = \"x\"\nc = true\nd = false\ne = 0.25\nf = [1, 2, 3]",
        ),
        (
            r#"<|"t1" -> <|"x" -> 1|>, "t2" -> <|"y" -> 2, "z" -> 3|>|>"#,
            "[t1]\nx = 1\n\n[t2]\ny = 2\nz = 3",
        ),
        (
            r#"<|"t1" -> <|"x" -> 1|>, "t2" -> <|"x" -> 2|>, "t3" -> <|"x" -> 3|>, "z" -> 0|>"#,
            "t1 = {x = 1}\nt2 = {x = 2}\nt3 = {x = 3}\nz = 0",
        ),
        (
            r#"<|"t1" -> <|"x" -> 1|>, "z" -> 0, "t2" -> <|"y" -> 2|>|>"#,
            "t1 = {x = 1}\nz = 0\n\n[t2]\ny = 2",
        ),
        (
            r#"<|"a" -> <|"b" -> <|"c" -> 1|>|>|>"#,
            "[a]\n\n[a.b]\nc = 1",
        ),
        (
            r#"<|"a" -> <|"b" -> <|"c" -> 1|>, "p" -> 2|>|>"#,
            "[a]\nb = {c = 1}\np = 2",
        ),
        (
            r#"<|"servers" -> {<|"n" -> "edge"|>, <|"n" -> "batch"|>}|>"#,
            "[[servers]]\nn = \"edge\"\n\n[[servers]]\nn = \"batch\"",
        ),
        (r#"<||>"#, ""),
        (r#"<|"e" -> <||>|>"#, "[e]\n"),
        (r#"<|"arr" -> {<||>, <||>}|>"#, "[[arr]]\n\n[[arr]]\n"),
        (
            r#"<|"servers" -> {<|"n" -> 1|>, <|"n" -> 2|>}, "z" -> 0|>"#,
            "servers = [{n = 1}, {n = 2}]\nz = 0",
        ),
        (
            r#"<|"t" -> <|"x" -> 1|>, "z" -> 0, "arr" -> {<|"n" -> 1|>, <|"n" -> 2|>}|>"#,
            "t = {x = 1}\nz = 0\n\n[[arr]]\nn = 1\n\n[[arr]]\nn = 2",
        ),
        (
            r#"<|"a.b" -> 1, "" -> 2, "with space" -> 3, "q\"uote" -> 4, "tab\tkey" -> 5|>"#,
            "\"a.b\" = 1\n\"\" = 2\n\"with space\" = 3\n\"q\\\"uote\" = 4\n\"tab\\tkey\" = 5",
        ),
        (
            r#"<|"nl" -> "a\nb", "tab" -> "a\tb", "q" -> "say \"hi\"", "bs" -> "a\\b", "uni" -> "héllo→"|>"#,
            "nl = \"a\\nb\"\ntab = \"a\\tb\"\nq = \"say \\\"hi\\\"\"\nbs = \"a\\\\b\"\nuni = \"héllo→\"",
        ),
        (
            r#"<|"neg" -> -5, "big" -> 123456789012345678901234567890, "rat" -> 3/8, "rat2" -> -1/20, "zero" -> 0|>"#,
            "neg = -5\nbig = 123456789012345678901234567890\nrat = 0.375\nrat2 = -0.05\nzero = 0",
        ),
        (
            r#"<|"m" -> {{1, 2}, {3, 4}}, "empty" -> {}, "mixed" -> {1, "s", True, {}}|>"#,
            "m = [[1, 2], [3, 4]]\nempty = []\nmixed = [1, \"s\", true, []]",
        ),
        (
            r#"<|"arr" -> {<|"i" -> 1|>, <|"i" -> 2|>, 3}|>"#,
            "arr = [{i = 1}, {i = 2}, 3]",
        ),
        (
            r#"<|"svc" -> <|"name" -> "rum", "opts" -> <|"debug" -> True, "level" -> 2|>, "ports" -> {8080, 9090}|>, "flag" -> False|>"#,
            "svc = {name = \"rum\", opts = {debug = true, level = 2}, ports = [8080, 9090]}\nflag = false",
        ),
    ] {
        assert_eq!(
            eval_string_atom(&mut session, &format!("ToToml[{source}]")),
            expected,
            "golden corpus case: {source}"
        );
    }
}

#[test]
fn to_toml_golden_error_corpus_is_byte_stable() {
    let mut session = core_session();

    for (source, message) in [
        (r#"<|1 -> "a"|>"#, "ToToml keys must be Strings."),
        (
            r#"<|"t" -> <|"x" -> 1, 2 -> "y"|>|>"#,
            "ToToml keys must be Strings.",
        ),
        (
            r#"<|"t" -> <|"x" -> 1, 2 -> "y"|>, "z" -> 0|>"#,
            "ToToml inline table keys must be Strings.",
        ),
        (
            r#"<|"s" -> wumbo|>"#,
            "Cannot encode expression wumbo as TOML.",
        ),
        (
            r#"<|"k" :> 2 + 3|>"#,
            "Cannot encode expression Plus[2, 3] as TOML.",
        ),
        (
            r#"<|"v" -> {"a" -> 1}|>"#,
            "Cannot encode expression Rule[\"a\", 1] as TOML.",
        ),
        (
            r#"<|"r" -> 1/3|>"#,
            "Cannot encode non-terminating Rational 1/3 as TOML; convert it explicitly before calling ToToml.",
        ),
        (
            r#"42"#,
            "ToToml expects a top-level Association with String keys.",
        ),
    ] {
        let outcome = session.eval_text(&format!("ToToml[{source}]")).unwrap();
        assert_failed_with_message(outcome.value, outcome.messages, "ToToml", "toml", message);
    }
}

/// Golden corpus captured from the unmodified decoder before the adaptive
/// entry-index rework (brief-312): 7 output shapes plus 7 error shapes. The
/// expected strings are byte-for-byte what the decoder emitted at capture
/// time — including the byte offsets inside error messages; any drift is a
/// behavior change in decoded structure or diagnostics, not a formatting
/// nit. The corpus ran green against the unmodified decoder first, so the
/// goldens cannot have been fitted to the new code.
#[test]
fn from_toml_golden_corpus_is_byte_stable() {
    let mut session = core_session();

    for (source, expected) in [
        (
            "z = 1\na = 2\nm = 3\n",
            "Association[Rule[\"z\", 1], Rule[\"a\", 2], Rule[\"m\", 3]]",
        ),
        (
            "[a.b]\ny = 2\n\n[a]\nx = 1\n",
            "Association[Rule[\"a\", Association[Rule[\"b\", Association[Rule[\"y\", 2]]], Rule[\"x\", 1]]]]",
        ),
        (
            "a.b = 1\na.c = 2\n",
            "Association[Rule[\"a\", Association[Rule[\"b\", 1], Rule[\"c\", 2]]]]",
        ),
        (
            "[[servers]]\nname = \"edge\"\n\n[[servers]]\nname = \"batch\"\n",
            "Association[Rule[\"servers\", List[Association[Rule[\"name\", \"edge\"]], Association[Rule[\"name\", \"batch\"]]]]]",
        ),
        (
            "[[a.b]]\nx = 1\n",
            "Association[Rule[\"a\", Association[Rule[\"b\", List[Association[Rule[\"x\", 1]]]]]]]",
        ),
        (
            "meta = { role = \"primary\", weight = 0.5, nested = { k = 1 } }\n",
            "Association[Rule[\"meta\", Association[Rule[\"role\", \"primary\"], Rule[\"weight\", 1/2], Rule[\"nested\", Association[Rule[\"k\", 1]]]]]]",
        ),
        (
            "title = \"x\"\n[server]\nports = [1, 2]\n",
            "Association[Rule[\"title\", \"x\"], Rule[\"server\", Association[Rule[\"ports\", List[1, 2]]]]]",
        ),
    ] {
        assert_eq!(
            from_toml_to_string(&mut session, source),
            expected,
            "golden corpus case: {source:?}"
        );
    }
}

#[test]
fn from_toml_golden_error_corpus_is_byte_stable() {
    let mut session = core_session();

    for (source, message) in [
        (
            "a = 1\na = 2\n",
            "Invalid TOML at byte offset 6: duplicate key \"a\".",
        ),
        (
            "[t]\nx = 1\nx = 2\n",
            "Invalid TOML at byte offset 10: duplicate key \"x\".",
        ),
        (
            "[t]\nx = 1\n[t]\ny = 2\n",
            "Invalid TOML at byte offset 10: duplicate table \"t\".",
        ),
        (
            "m = { x = 1, x = 2 }\n",
            "Invalid TOML at byte offset 13: duplicate key \"x\".",
        ),
        (
            "a =\n",
            "Invalid TOML at byte offset 3: expected a TOML value.",
        ),
        ("= 1\n", "Invalid TOML at byte offset 0: expected TOML key."),
        (
            "[a\n",
            "Invalid TOML at byte offset 2: expected ']' to close table header.",
        ),
    ] {
        let value = session.evaluate(from_toml_call(source)).unwrap();
        let messages = session.take_messages();
        assert_eq!(
            value.to_string(),
            "$Failed",
            "golden error case: {source:?}"
        );
        assert_eq!(
            messages
                .iter()
                .map(|message| message.text.as_str())
                .collect::<Vec<_>>(),
            vec![message],
            "golden error case: {source:?}"
        );
    }
}

#[test]
fn from_toml_depth_limit_accepts_each_descent_at_the_limit() {
    std::thread::Builder::new()
        .name("toml-depth-acceptance".to_string())
        .stack_size(TOML_DEPTH_ACCEPTANCE_STACK_BYTES)
        .spawn(|| {
            let mut session = core_session();
            session.set_native_stack_limit(
                native_stack_limit_for_declared_stack(TOML_DEPTH_ACCEPTANCE_STACK_BYTES)
                    .expect("TOML acceptance stack should satisfy the supported minimum"),
            );

            assert_toml_depth_success(&mut session, &nested_toml_arrays(CODEC_DEPTH_LIMIT));
            assert_toml_depth_success(&mut session, &nested_toml_inline_tables(CODEC_DEPTH_LIMIT));

            let header = format!("[{}]\nx = 0\n", dotted_toml_path(CODEC_DEPTH_LIMIT));
            assert_toml_depth_success(&mut session, &header);

            let dotted_assignment = format!("{} = 0\n", dotted_toml_path(CODEC_DEPTH_LIMIT + 1));
            assert_toml_depth_success(&mut session, &dotted_assignment);

            let inline_dotted = format!("a = {{ {} = 0 }}\n", dotted_toml_path(CODEC_DEPTH_LIMIT));
            assert_toml_depth_success(&mut session, &inline_dotted);
        })
        .expect("TOML depth-acceptance thread should start")
        .join()
        .expect("TOML depth acceptance should finish");
}

#[test]
fn from_toml_depth_limit_rejects_each_descent_one_past() {
    let mut session = core_session();

    let arrays = nested_toml_arrays(CODEC_DEPTH_LIMIT + 1);
    assert_toml_depth_failure(&mut session, &arrays, 4 + CODEC_DEPTH_LIMIT);

    let inline_tables = nested_toml_inline_tables(CODEC_DEPTH_LIMIT + 1);
    assert_toml_depth_failure(&mut session, &inline_tables, 4 + 6 * CODEC_DEPTH_LIMIT);

    let header = format!("[{}]\nx = 0\n", dotted_toml_path(CODEC_DEPTH_LIMIT + 1));
    assert_toml_depth_failure(&mut session, &header, 1 + 2 * CODEC_DEPTH_LIMIT);

    let dotted_assignment = format!("{} = 0\n", dotted_toml_path(CODEC_DEPTH_LIMIT + 2));
    assert_toml_depth_failure(
        &mut session,
        &dotted_assignment,
        2 * (CODEC_DEPTH_LIMIT + 1),
    );

    let inline_dotted = format!(
        "a = {{ {} = 0 }}\n",
        dotted_toml_path(CODEC_DEPTH_LIMIT + 1)
    );
    assert_toml_depth_failure(&mut session, &inline_dotted, 6 + 2 * CODEC_DEPTH_LIMIT);
}

#[test]
fn from_toml_depth_limit_rejects_the_far_past_stack_exhaustion_shape() {
    let mut session = core_session();
    let source = nested_toml_arrays(15_000);

    assert_toml_depth_failure(&mut session, &source, 4 + CODEC_DEPTH_LIMIT);
}

#[test]
fn toml_codecs_compose_with_json() {
    let mut session = core_session();

    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToJson[FromToml[\"title = \\\"x\\\"\\n[server]\\nports = [1, 2]\\n\"]]"
        ),
        "{\"title\":\"x\",\"server\":{\"ports\":[1,2]}}"
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToToml[FromJson[\"{\\\"srv\\\":{\\\"port\\\":8080}}\"]]"
        ),
        "[srv]\nport = 8080"
    );
}
