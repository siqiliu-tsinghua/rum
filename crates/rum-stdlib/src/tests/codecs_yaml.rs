use crate::install_core;
use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::system_expr;
use rum_runtime::{RuntimeMessage, Session, native_stack_limit_for_declared_stack};

const YAML_DEPTH_ACCEPTANCE_STACK_BYTES: usize = 16 * 1024 * 1024;

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

fn from_yaml_call(source: &str) -> Expr {
    Expr::call(system_expr("FromYaml"), vec![Expr::string(source)])
}

fn from_yaml_to_string(session: &mut Session, source: &str) -> String {
    let value = session.evaluate(from_yaml_call(source)).unwrap();
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

const K8SISH: &str = r#"
# service config
svc:
  name: web # plain scalar comment
  replicas: 2
  enabled: true
  pi: 3.14
  hosts:
    - a
    - b
  containers:
    - name: web
      image: "nginx:1.25"
      env:
        - name: COUNTRY
          value: 'no'
  note: |-
    line1
    line2
"#;

const CODEC_DEPTH_LIMIT: usize = 256;

fn nested_yaml_blocks(levels: usize, marker: &str) -> (String, Option<usize>) {
    let mut source = String::new();
    let mut first_past_limit_offset = None;
    for indent in 0..levels {
        let content_offset = source.len() + indent;
        if indent == CODEC_DEPTH_LIMIT {
            first_past_limit_offset = Some(content_offset);
        }
        source.push_str(&" ".repeat(indent));
        source.push_str(marker);
        source.push('\n');
    }
    (source, first_past_limit_offset)
}

fn nested_yaml_sequence_leaf(sequence_levels: usize, leaf: &str) -> (String, usize) {
    let mut source = String::new();
    for indent in 0..sequence_levels - 1 {
        source.push_str(&" ".repeat(indent));
        source.push_str("-\n");
    }
    let indent = sequence_levels - 1;
    let item_offset = source.len() + indent + 2;
    source.push_str(&" ".repeat(indent));
    source.push_str("- ");
    source.push_str(leaf);
    source.push('\n');
    (source, item_offset)
}

fn assert_yaml_depth_failure(session: &mut Session, source: &str, offset: usize) {
    let value = session.evaluate(from_yaml_call(source)).unwrap();
    let messages = session.take_messages();
    assert_eq!(value.to_string(), "$Failed");
    assert_eq!(messages.len(), 1, "{source}: {messages:?}");
    assert_eq!(messages[0].symbol, "FromYaml");
    assert_eq!(messages[0].tag, "depth");
    assert_eq!(
        messages[0].text,
        format!(
            "YAML nesting exceeds the fixed depth limit of {CODEC_DEPTH_LIMIT} at byte offset {offset}."
        )
    );
}

fn assert_yaml_depth_success(session: &mut Session, source: &str) {
    let value = session.evaluate(from_yaml_call(source)).unwrap();
    let messages = session.take_messages();
    assert_ne!(value.to_string(), "$Failed");
    assert!(messages.is_empty(), "{source}: {messages:?}");
}

#[test]
fn from_yaml_parses_k8s_flavored_config() {
    let mut session = core_session();

    assert_eq!(
        from_yaml_to_string(&mut session, K8SISH),
        "Association[Rule[\"svc\", Association[Rule[\"name\", \"web\"], Rule[\"replicas\", 2], Rule[\"enabled\", True], Rule[\"pi\", 157/50], Rule[\"hosts\", List[\"a\", \"b\"]], Rule[\"containers\", List[Association[Rule[\"name\", \"web\"], Rule[\"image\", \"nginx:1.25\"], Rule[\"env\", List[Association[Rule[\"name\", \"COUNTRY\"], Rule[\"value\", \"no\"]]]]]]], Rule[\"note\", \"line1\\nline2\"]]]]"
    );
}

#[test]
fn from_yaml_uses_norway_safe_typing() {
    let mut session = core_session();
    let source = "\
country: no
no: 1
yes: yes
on: on
off: off
No: No
";

    assert_eq!(
        from_yaml_to_string(&mut session, source),
        "Association[Rule[\"country\", \"no\"], Rule[\"no\", 1], Rule[\"yes\", \"yes\"], Rule[\"on\", \"on\"], Rule[\"off\", \"off\"], Rule[\"No\", \"No\"]]"
    );
}

#[test]
fn from_yaml_numbers_and_nulls_are_exact() {
    let mut session = core_session();
    let big = "1234567890".repeat(12);
    let source = format!(
        "\
decimal: 0.1
pi: 3.14
exp: 1.5e3
neg: -2
big: {big}
tilde: ~
empty:
"
    );

    assert_eq!(
        from_yaml_to_string(&mut session, &source),
        format!(
            "Association[Rule[\"decimal\", 1/10], Rule[\"pi\", 157/50], Rule[\"exp\", 1500], Rule[\"neg\", -2], Rule[\"big\", {big}], Rule[\"tilde\", Null], Rule[\"empty\", Null]]"
        )
    );
}

#[test]
fn from_yaml_literal_blocks_pin_chomping() {
    let mut session = core_session();
    let source = "\
clip: |
  line1
  line2
strip: |-
  line1
  line2
";

    assert_eq!(
        from_yaml_to_string(&mut session, source),
        "Association[Rule[\"clip\", \"line1\\nline2\\n\"], Rule[\"strip\", \"line1\\nline2\"]]"
    );
}

#[test]
fn from_yaml_rejects_unsupported_yaml_fail_closed() {
    let mut session = core_session();

    for (source, needle) in [
        ("a: &x 1", "anchors"),
        ("a: *x", "aliases"),
        ("a: !str x", "tags"),
        ("%YAML 1.2\n", "directives"),
        ("---\na: 1\n", "document markers"),
        ("<<: 1", "merge keys"),
        ("a: [1, 2]", "non-empty flow"),
        ("a: >\n  x\n", "folded block"),
        ("a:\n\tb: 1\n", "tab indentation"),
        ("a: 1\na: 2\n", "duplicate mapping key"),
        ("- a\nb: c\n", "same indentation"),
        ("a:\n    b: 1\n  c: 2\n", "inconsistent indentation"),
    ] {
        let value = session.evaluate(from_yaml_call(source)).unwrap();
        let messages = session.take_messages();
        assert_failed_with_message(value, messages, "FromYaml", "yaml", needle);
    }
}

/// Golden corpus captured from the unmodified decoder before depth limits
/// were added (brief-329): 7 output shapes plus 7 error shapes. The expected
/// strings pin decoded structure, key order, diagnostics, and byte offsets.
#[test]
fn from_yaml_golden_corpus_is_byte_stable() {
    let mut session = core_session();

    for (source, expected) in [
        (
            "z: 1\na: 2\nm: 3\n",
            "Association[Rule[\"z\", 1], Rule[\"a\", 2], Rule[\"m\", 3]]",
        ),
        (
            "a:\n  b:\n    c: 1\n  d: 2\n",
            "Association[Rule[\"a\", Association[Rule[\"b\", Association[Rule[\"c\", 1]]], Rule[\"d\", 2]]]]",
        ),
        (
            "items:\n  - 1\n  - 2\n",
            "Association[Rule[\"items\", List[1, 2]]]",
        ),
        (
            "items:\n  - name: first\n    values:\n      - 1\n      - 2\n  - name: second\n",
            "Association[Rule[\"items\", List[Association[Rule[\"name\", \"first\"], Rule[\"values\", List[1, 2]]], Association[Rule[\"name\", \"second\"]]]]]",
        ),
        (
            "emptyMap: {}\nemptyList: []\nnullish:\n",
            "Association[Rule[\"emptyMap\", Association[]], Rule[\"emptyList\", List[]], Rule[\"nullish\", Null]]",
        ),
        (
            "clip: |\n  line1\n  line2\nstrip: |-\n  line1\n  line2\n",
            "Association[Rule[\"clip\", \"line1\\nline2\\n\"], Rule[\"strip\", \"line1\\nline2\"]]",
        ),
        (
            "\"q:key\": 'no'\nnum: 0.125\nbool: true\n",
            "Association[Rule[\"q:key\", \"no\"], Rule[\"num\", 1/8], Rule[\"bool\", True]]",
        ),
    ] {
        assert_eq!(
            from_yaml_to_string(&mut session, source),
            expected,
            "golden corpus case: {source:?}"
        );
    }
}

#[test]
fn from_yaml_golden_error_corpus_is_byte_stable() {
    let mut session = core_session();

    for (source, message) in [
        (
            "a: 1\na: 2\n",
            "Invalid YAML at byte offset 5: duplicate mapping key \"a\".",
        ),
        (
            "a:\n    b: 1\n  c: 2\n",
            "Invalid YAML at byte offset 14: inconsistent indentation.",
        ),
        (
            "a: [1, 2]\n",
            "Invalid YAML at byte offset 3: non-empty flow collections are outside rum's YAML subset.",
        ),
        (
            "a: *x\n",
            "Invalid YAML at byte offset 3: YAML aliases are unsupported.",
        ),
        (
            "- a\nb: c\n",
            "Invalid YAML at byte offset 4: sequence items and mapping keys cannot appear at the same indentation.",
        ),
        (
            "a: \"unterminated\n",
            "Invalid YAML at byte offset 3: unterminated double-quoted string.",
        ),
        (
            "  a: 1\n",
            "Invalid YAML at byte offset 2: top-level YAML content must start at indentation 0.",
        ),
    ] {
        let value = session.evaluate(from_yaml_call(source)).unwrap();
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
fn from_yaml_depth_limit_accepts_every_collection_descent_at_the_limit() {
    std::thread::Builder::new()
        .name("yaml-depth-acceptance".to_string())
        .stack_size(YAML_DEPTH_ACCEPTANCE_STACK_BYTES)
        .spawn(|| {
            let mut session = core_session();
            session.set_native_stack_limit(
                native_stack_limit_for_declared_stack(YAML_DEPTH_ACCEPTANCE_STACK_BYTES)
                    .expect("YAML acceptance stack should satisfy the supported minimum"),
            );

            let (mappings, _) = nested_yaml_blocks(CODEC_DEPTH_LIMIT, "k:");
            assert_yaml_depth_success(&mut session, &mappings);

            let (sequences, _) = nested_yaml_blocks(CODEC_DEPTH_LIMIT, "-");
            assert_yaml_depth_success(&mut session, &sequences);

            let (sequence_mapping, _) = nested_yaml_sequence_leaf(CODEC_DEPTH_LIMIT - 1, "k: 0");
            assert_yaml_depth_success(&mut session, &sequence_mapping);

            let (sequence_flow, _) = nested_yaml_sequence_leaf(CODEC_DEPTH_LIMIT - 1, "{}");
            assert_yaml_depth_success(&mut session, &sequence_flow);
        })
        .expect("YAML depth-acceptance thread should start")
        .join()
        .expect("YAML depth acceptance should finish");
}

#[test]
fn from_yaml_depth_limit_rejects_every_collection_descent_one_past() {
    let mut session = core_session();

    let (mappings, mapping_offset) = nested_yaml_blocks(CODEC_DEPTH_LIMIT + 1, "k:");
    assert_yaml_depth_failure(&mut session, &mappings, mapping_offset.unwrap());

    let (sequences, sequence_offset) = nested_yaml_blocks(CODEC_DEPTH_LIMIT + 1, "-");
    assert_yaml_depth_failure(&mut session, &sequences, sequence_offset.unwrap());

    let (sequence_mapping, mapping_item_offset) =
        nested_yaml_sequence_leaf(CODEC_DEPTH_LIMIT, "k: 0");
    assert_yaml_depth_failure(&mut session, &sequence_mapping, mapping_item_offset);

    let (sequence_flow, flow_item_offset) = nested_yaml_sequence_leaf(CODEC_DEPTH_LIMIT, "{}");
    assert_yaml_depth_failure(&mut session, &sequence_flow, flow_item_offset);
}

#[test]
fn from_yaml_depth_limit_rejects_the_far_past_stack_exhaustion_shape() {
    let mut session = core_session();
    let (source, first_past_limit_offset) = nested_yaml_blocks(8_001, "k:");

    assert_yaml_depth_failure(&mut session, &source, first_past_limit_offset.unwrap());
}

#[test]
fn to_yaml_round_trips_adversarial_strings_and_empties() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SameQ[FromYaml[ToYaml[<|\"a\" -> {1, \"no\", \"3.14\", \"null\", \"x y\"}, \"b\" -> <||>, \"c\" -> {}, \"m\" -> \"line1\\nline2\", \"t\" -> \"line1\\n\"|>]], <|\"a\" -> {1, \"no\", \"3.14\", \"null\", \"x y\"}, \"b\" -> <||>, \"c\" -> {}, \"m\" -> \"line1\\nline2\", \"t\" -> \"line1\\n\"|>]"
        ),
        "True"
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToYaml[<|\"a\" -> {1, \"no\", \"3.14\"}, \"b\" -> <||>|>]"
        ),
        "a:\n  - 1\n  - \"no\"\n  - \"3.14\"\nb: {}"
    );

    let outcome = session.eval_text("ToYaml[<|x -> 1|>]").unwrap();
    assert_failed_with_message(
        outcome.value,
        outcome.messages,
        "ToYaml",
        "yaml",
        "mapping keys must be Strings",
    );

    let outcome = session.eval_text("ToYaml[<|\"x\" -> 1/3|>]").unwrap();
    assert_failed_with_message(
        outcome.value,
        outcome.messages,
        "ToYaml",
        "yaml",
        "non-terminating Rational",
    );
}

#[test]
fn yaml_wide_mapping_index_preserves_round_trip() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "value = AssociationThread[Map[ToString, Range[20]], Range[20]]; SameQ[FromYaml[ToYaml[value]], value]",
        ),
        "True"
    );
}

#[test]
fn yaml_codecs_compose_with_json() {
    let mut session = core_session();

    assert_eq!(
        eval_string_atom(&mut session, &format!("ToJson[FromYaml[{:?}]]", K8SISH)),
        "{\"svc\":{\"name\":\"web\",\"replicas\":2,\"enabled\":true,\"pi\":3.14,\"hosts\":[\"a\",\"b\"],\"containers\":[{\"name\":\"web\",\"image\":\"nginx:1.25\",\"env\":[{\"name\":\"COUNTRY\",\"value\":\"no\"}]}],\"note\":\"line1\\nline2\"}}"
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToYaml[FromJson[\"{\\\"srv\\\":{\\\"port\\\":8080,\\\"hosts\\\":[\\\"a\\\",\\\"b\\\"]}}\"]]"
        ),
        "srv:\n  port: 8080\n  hosts:\n    - a\n    - b"
    );
}
