use crate::install_core;
use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::{render_association, system_expr};
use rum_runtime::{RuntimeMessage, Session};

const REALISTIC_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<cfg env='dev' q="a &amp; b">
  <db host="h" port='5432'/>
  <!-- dropped -->
  <note>a &amp; b<![CDATA[<raw>]]></note>
  <ns:item ns:id="1">pre <b/> tail</ns:item>
</cfg>"#;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    let outcome = session.eval_text(source).unwrap();
    assert!(
        outcome.messages.is_empty(),
        "{source}: {:?}",
        outcome.messages
    );
    outcome.value.to_string()
}

fn eval_string_atom(session: &mut Session, source: &str) -> String {
    let outcome = session.eval_text(source).unwrap();
    assert!(
        outcome.messages.is_empty(),
        "{source}: {:?}",
        outcome.messages
    );
    match outcome.value {
        Expr::Atom(Atom::String(value)) => value,
        other => panic!("expected string atom, got {other}"),
    }
}

fn from_xml_call(source: &str) -> Expr {
    Expr::call(system_expr("FromXml"), vec![Expr::string(source)])
}

fn from_xml_to_string(session: &mut Session, source: &str) -> String {
    let value = session.evaluate(from_xml_call(source)).unwrap();
    let messages = session.take_messages();
    assert!(messages.is_empty(), "{source}: {messages:?}");
    value.to_string()
}

fn eval_without_messages(session: &mut Session, expr: Expr) -> Expr {
    let value = session.evaluate(expr).unwrap();
    let messages = session.take_messages();
    assert!(messages.is_empty(), "{messages:?}");
    value
}

fn xml_element(tag: &str, attrs: Vec<(&str, &str)>, children: Vec<Expr>) -> Expr {
    Expr::call(
        system_expr("XmlElement"),
        vec![
            Expr::string(tag),
            render_association(
                attrs
                    .into_iter()
                    .map(|(key, value)| (Expr::string(key), Expr::string(value)))
                    .collect(),
            ),
            Expr::call(system_expr("List"), children),
        ],
    )
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

#[test]
fn from_xml_parses_realistic_document_to_symbolic_tree() {
    let mut session = core_session();

    assert_eq!(
        from_xml_to_string(&mut session, REALISTIC_XML),
        "XmlElement[\"cfg\", Association[Rule[\"env\", \"dev\"], Rule[\"q\", \"a & b\"]], List[XmlElement[\"db\", Association[Rule[\"host\", \"h\"], Rule[\"port\", \"5432\"]], List[]], XmlElement[\"note\", Association[], List[\"a & b<raw>\"]], XmlElement[\"ns:item\", Association[Rule[\"ns:id\", \"1\"]], List[\"pre \", XmlElement[\"b\", Association[], List[]], \" tail\"]]]]"
    );
}

#[test]
fn from_xml_decodes_unicode_and_numeric_entities() {
    let mut session = core_session();

    assert_eq!(
        from_xml_to_string(
            &mut session,
            "<x attr=\"héllo, 世界\">Hi &#x4E16;&#30028;</x>"
        ),
        "XmlElement[\"x\", Association[Rule[\"attr\", \"héllo, 世界\"]], List[\"Hi 世界\"]]"
    );
    assert_eq!(
        from_xml_to_string(&mut session, "<x> </x>"),
        "XmlElement[\"x\", Association[], List[\" \"]]"
    );
}

#[test]
fn from_xml_rejects_invalid_xml_fail_closed() {
    let mut session = core_session();

    for (source, needle) in [
        ("<!DOCTYPE x><x/>", "DTD/DOCTYPE"),
        ("<a><b></a>", "mismatched closing tag"),
        ("<a x=\"1\" x='2'/>", "duplicate attribute"),
        ("<a>&bogus;</a>", "unknown named entity"),
        ("<a x=1/>", "single- or double-quoted"),
        ("<a>a & b</a>", "bare '&'"),
        ("<a>a < b</a>", "bare '<'"),
        (
            "<?xml-stylesheet href=\"x\"?><a/>",
            "processing instructions",
        ),
    ] {
        let value = session.evaluate(from_xml_call(source)).unwrap();
        let messages = session.take_messages();
        assert_failed_with_message(value, messages, "FromXml", "xml", needle);
    }
}

#[test]
fn to_xml_encodes_compact_escaped_and_pretty_forms() {
    let mut session = core_session();

    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToXml[XmlElement[\"a\", <|\"x\" -> \"1<2\", \"q\" -> \"a\\\"b&c\"|>, {XmlElement[\"b\", <||>, {}], \"t&t\"}]]"
        ),
        "<a x=\"1&lt;2\" q=\"a&quot;b&amp;c\"><b/>t&amp;t</a>"
    );
    let pretty_tree = xml_element(
        "r",
        Vec::new(),
        vec![
            xml_element("a", Vec::new(), Vec::new()),
            xml_element(
                "b",
                vec![("x", "1")],
                vec![xml_element("c", Vec::new(), Vec::new())],
            ),
        ],
    );
    let pretty = eval_without_messages(
        &mut session,
        Expr::call(
            system_expr("ToXml"),
            vec![
                pretty_tree,
                Expr::call(
                    system_expr("Rule"),
                    vec![Expr::string("Indent"), Expr::small_integer(2)],
                ),
            ],
        ),
    );
    assert_eq!(
        expr_string(&pretty),
        "<r>\n  <a/>\n  <b x=\"1\">\n    <c/>\n  </b>\n</r>"
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToXml[XmlElement[\"r\", <||>, {\"pre \", XmlElement[\"b\", <||>, {}], \" post\"}], \"Indent\" -> 2]"
        ),
        "<r>pre <b/> post</r>"
    );
}

#[test]
fn to_xml_rejects_non_xml_trees_fail_closed() {
    let mut session = core_session();

    for (source, needle) in [
        ("ToXml[\"x\"]", "XmlElement"),
        ("ToXml[XmlElement[\"1a\", <||>, {}]]", "name subset"),
        (
            "ToXml[XmlElement[\"a\", <|\"bad name\" -> \"1\"|>, {}]]",
            "name subset",
        ),
        (
            "ToXml[XmlElement[\"a\", <||>, {1}]]",
            "Strings or XmlElement",
        ),
        ("ToXml[XmlElement[\"a\", <||>, {\"\"}]]", "empty text nodes"),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_failed_with_message(outcome.value, outcome.messages, "ToXml", "xml", needle);
    }
}

#[test]
fn xml_round_trips_are_pinned() {
    let mut session = core_session();
    let tree = xml_element(
        "r",
        vec![("a", "b")],
        vec![
            Expr::string("mixed "),
            xml_element("i", Vec::new(), vec![Expr::string("x")]),
            Expr::string(" tail"),
        ],
    );

    let encoded = eval_without_messages(
        &mut session,
        Expr::call(system_expr("ToXml"), vec![tree.clone()]),
    );
    let reparsed = eval_without_messages(&mut session, from_xml_call(expr_string(&encoded)));
    assert_eq!(reparsed, tree);

    let parsed = eval_without_messages(&mut session, from_xml_call(REALISTIC_XML));
    let encoded = eval_without_messages(
        &mut session,
        Expr::call(system_expr("ToXml"), vec![parsed.clone()]),
    );
    let reparsed = eval_without_messages(&mut session, from_xml_call(expr_string(&encoded)));
    assert_eq!(reparsed, parsed);
}

#[test]
fn xml_wide_attribute_index_preserves_round_trip() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "names = Map[Function[StringJoin[\"k\", ToString[Slot[1]]]], Range[20]]; attrs = AssociationThread[names, Map[ToString, Range[20]]]; xml = XmlElement[\"r\", attrs, {}]; SameQ[FromXml[ToXml[xml]], xml]",
        ),
        "True"
    );
}

#[test]
fn xml_values_compose_with_json() {
    let mut session = core_session();

    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToJson[<|\"port\" -> FromXml[\"<c><p>8080</p></c>\"][[3, 1, 3, 1]]|>]"
        ),
        "{\"port\":\"8080\"}"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "FromXml[ToXml[XmlElement[\"r\", <||>, {\"mixed \", XmlElement[\"i\", <||>, {\"x\"}], \" tail\"}]]]"
        ),
        "XmlElement[\"r\", Association[], List[\"mixed \", XmlElement[\"i\", Association[], List[\"x\"]], \" tail\"]]"
    );
}

const CODEC_DEPTH_LIMIT: usize = 256;

fn nested_xml_elements(levels: usize) -> String {
    format!("{}{}", "<a>".repeat(levels), "</a>".repeat(levels))
}

fn assert_xml_depth_failure(session: &mut Session, source: &str, offset: usize) {
    let value = session.evaluate(from_xml_call(source)).unwrap();
    let messages = session.take_messages();
    assert_eq!(value.to_string(), "$Failed");
    assert_eq!(messages.len(), 1, "{source}: {messages:?}");
    assert_eq!(messages[0].symbol, "FromXml");
    assert_eq!(messages[0].tag, "depth");
    assert_eq!(
        messages[0].text,
        format!(
            "XML nesting exceeds the fixed depth limit of {CODEC_DEPTH_LIMIT} at byte offset {offset}."
        )
    );
}

fn assert_xml_depth_success(session: &mut Session, source: &str) {
    let value = session.evaluate(from_xml_call(source)).unwrap();
    let messages = session.take_messages();
    assert_ne!(value.to_string(), "$Failed");
    assert!(messages.is_empty(), "{source}: {messages:?}");
}

#[test]
fn from_xml_depth_limit_accepts_deep_element_nesting() {
    let mut session = core_session();

    // 64 rather than CODEC_DEPTH_LIMIT: evaluating a 256-deep XmlElement tree
    // costs two kernel evaluator cycles per level, which exceeds the default
    // ~2 MiB debug test-thread stack (JSON/TOML List chains fit; this tree
    // shape does not). Release builds accept the full 256 (verified at the
    // brief-422 gate); this pins the accept direction within harness limits.
    assert_xml_depth_success(&mut session, &nested_xml_elements(64));
}

#[test]
fn from_xml_depth_limit_rejects_element_nesting_one_past() {
    let mut session = core_session();

    let source = nested_xml_elements(CODEC_DEPTH_LIMIT + 1);
    assert_xml_depth_failure(&mut session, &source, 3 * CODEC_DEPTH_LIMIT);
}

#[test]
fn from_xml_depth_limit_rejects_the_far_past_stack_exhaustion_shape() {
    let mut session = core_session();
    let source = nested_xml_elements(15_000);

    assert_xml_depth_failure(&mut session, &source, 3 * CODEC_DEPTH_LIMIT);

    assert_eq!(
        eval_to_string(&mut session, "FromXml[\"<r x=\\\"1\\\"><c/></r>\"]"),
        "XmlElement[\"r\", Association[Rule[\"x\", \"1\"]], List[XmlElement[\"c\", Association[], List[]]]]"
    );
}

fn expr_string(expr: &Expr) -> &str {
    let Expr::Atom(Atom::String(value)) = expr else {
        panic!("expected string atom, got {expr}");
    };
    value
}
