use super::*;

#[test]
fn message_name_and_message_emit_runtime_messages() {
    let mut session = session_with_test_control_builtins();

    let named = session
        .eval_text("MessageName[FindFile, \"notfound\"]")
        .unwrap()
        .value;
    assert_eq!(named.to_string(), "FindFile::notfound");

    let outcome = session.eval_text("Message[FindFile::notfound, 7]").unwrap();
    assert_eq!(outcome.value.to_string(), "Null");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "FindFile" && message.tag == "notfound")
    );
    assert!(outcome.messages.iter().any(|message| message.text == "7"));
}

#[test]
fn message_family_accepts_symbol_name_strings_and_messagename_targets() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f::argx = \"bad arg `1`\"").unwrap();
    session.eval_text("f::usage = \"usage text\"").unwrap();

    let emitted = session
        .eval_text("Message[MessageName[\"f\", \"argx\"], 42]")
        .unwrap();
    assert_eq!(emitted.value.to_string(), "Null");
    assert!(emitted.messages.iter().any(|message| {
        message.symbol == "f" && message.tag == "argx" && message.text == "bad arg 42"
    }));

    assert_eq!(
        session
            .eval_text("Messages[\"f\"]")
            .unwrap()
            .value
            .to_string(),
        "List[f::argx, f::usage]"
    );
    assert_eq!(
        session
            .eval_text("MessageDefinitions[MessageName[\"f\", \"argx\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[f::argx, \"bad arg `1`\"]]"
    );
    assert_eq!(
        session
            .eval_text("MessageTags[MessageName[\"f\", \"argx\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"argx\"]"
    );
}

#[test]
fn usage_message_assignment_holds_symbol_with_existing_own_value() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("a = 5").unwrap();
    session.eval_text("a::usage = \"doc-a\"").unwrap();
    session.eval_text("b::usage = \"doc-b\"").unwrap();
    session.eval_text("b = 5").unwrap();

    assert_eq!(
        session.eval_text("Messages[a]").unwrap().value.to_string(),
        "List[a::usage]"
    );
    assert_eq!(
        session.eval_text("Messages[b]").unwrap().value.to_string(),
        "List[b::usage]"
    );
}

#[test]
fn direct_set_and_setdelayed_calls_share_message_assignment_normalization() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("Set[MessageName[\"f\", \"argx\"], \"bad arg `1`\"]")
        .unwrap();
    session
        .eval_text("SetDelayed[MessageName[\"f\", \"usage\"], suffix]")
        .unwrap();
    session.eval_text("suffix = 7").unwrap();

    assert_eq!(
        session.eval_text("Message[f::argx, 42]").unwrap().messages[0].text,
        "bad arg 42"
    );
    assert_eq!(
        session.eval_text("Message[f::usage]").unwrap().messages[0].text,
        "7"
    );
}

#[test]
fn message_templates_can_be_defined_and_formatted() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f::argx = \"bad arg `1`\"").unwrap();
    let outcome = session.eval_text("Message[f::argx, 42]").unwrap();

    assert_eq!(outcome.value.to_string(), "Null");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "f" && message.tag == "argx")
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.text == "bad arg 42")
    );
}
#[test]
fn message_templates_support_message_name_and_all_args_placeholders() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text("f::argx = \"`0`: first=`1`; all=`*`; implicit=``/``\"")
        .unwrap();
    let outcome = session.eval_text("Message[f::argx, 1, 2]").unwrap();

    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "f"
            && message.tag == "argx"
            && message.text == "f::argx: first=1; all=1, 2; implicit=1/2"
    }));

    session
        .eval_text("f::str = \"string=`1`; all=`*`\"")
        .unwrap();
    let outcome = session.eval_text("Message[f::str, \"x\", \"y\"]").unwrap();
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "f" && message.tag == "str" && message.text == "string=x; all=x, y"
    }));
}
#[test]
fn message_templates_support_richer_format_specifiers() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text(
            "f::argx = \"sym=`0:symbol`; tag=`0:tag`; count=`*:count`; head=`1:head`; name=`2:name`; ctx=`2:context`; full=`2:full`; quoted=`3:quoted`; argc=`3:count`\"",
        )
        .unwrap();
    let outcome = session
        .eval_text("Message[f::argx, 7, Global`x, g[1, 2]]")
        .unwrap();

    assert!(outcome.messages.iter().any(|message| {
        message.text
            == "sym=f; tag=argx; count=3; head=Integer; name=x; ctx=Global`; full=Global`x; quoted=\"g[1, 2]\"; argc=2"
    }));
}

#[test]
fn message_templates_leave_unknown_placeholders_unchanged() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text("f::argx = \"missing=`3`; literal=`name`\"")
        .unwrap();
    let outcome = session.eval_text("Message[f::argx, 1]").unwrap();

    assert!(
        outcome
            .messages
            .iter()
            .any(|message| { message.text == "missing=`3`; literal=`name`" })
    );
}

#[test]
fn message_templates_leave_unsupported_format_specs_unchanged() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text("f::argx = \"ctx=`1:context`; bad=`*:head`; nope=`1:nope`\"")
        .unwrap();
    let outcome = session.eval_text("Message[f::argx, 7]").unwrap();

    assert!(
        outcome
            .messages
            .iter()
            .any(|message| { message.text == "ctx=`1:context`; bad=`*:head`; nope=`1:nope`" })
    );
}

#[test]
fn messages_builtin_lists_defined_message_names() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f::argx = \"bad arg\"").unwrap();
    session.eval_text("f::usage = \"usage text\"").unwrap();
    session.eval_text("g::tag = \"other\"").unwrap();

    assert_eq!(
        session.eval_text("Messages[f]").unwrap().value.to_string(),
        "List[f::argx, f::usage]"
    );
    let all_messages = session.eval_text("Messages[]").unwrap().value;
    let Expr::Call { args, .. } = all_messages else {
        panic!("Messages[] should return a list")
    };
    let message_names = args.iter().map(ToString::to_string).collect::<Vec<_>>();
    assert_eq!(message_names.len(), 332);
    for expected in [
        "f::argx",
        "f::usage",
        "g::tag",
        "MissingQ::argx",
        "Trace::argtu",
        "Function::argx",
        "Get::syntax",
        "Part::partw",
        "Input::optx",
        "RepeatedNull::range",
        "DefinePackage::failed",
        "RestoreSealedReferences::mode",
        "Save::sealedref",
        "DumpSave::sealedref",
    ] {
        assert!(message_names.iter().any(|name| name == expected));
    }
    assert_eq!(
        session
            .eval_text("Messages[f::argx]")
            .unwrap()
            .value
            .to_string(),
        "List[f::argx]"
    );
}

#[test]
fn inert_message_carriers_list_seeded_tags_and_reject_assignment() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("Messages[General]")
            .unwrap()
            .value
            .to_string(),
        "List[General::cancel, General::iterlim, General::matchreclim, General::memlim, General::noentropy, General::noio, General::normal, General::optreentry, General::passlim, General::reclim, General::reslim, General::sandbox, General::timelim]"
    );
    assert_eq!(
        session
            .eval_text("Messages[UpTo]")
            .unwrap()
            .value
            .to_string(),
        "List[UpTo::argx, UpTo::innf]"
    );

    for (input, expected_text) in [
        ("General = 5", "Symbol General is Protected."),
        ("UpTo = 5", "Symbol UpTo is Protected."),
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert_eq!(outcome.value.to_string(), "5");
        assert_eq!(outcome.messages.len(), 1);
        assert_eq!(outcome.messages[0].symbol, "Set");
        assert_eq!(outcome.messages[0].tag, "wrsym");
        assert_eq!(outcome.messages[0].text, expected_text);
    }
    assert_eq!(
        session.eval_text("General").unwrap().value.to_string(),
        "General"
    );
    assert_eq!(session.eval_text("UpTo").unwrap().value.to_string(), "UpTo");
}

#[test]
fn upto_stays_inert_and_selector_diagnostic_stays_exact() {
    let mut session = session_with_test_control_builtins();

    let inert = session.eval_text("UpTo[3]").unwrap();
    assert_eq!(inert.value.to_string(), "UpTo[3]");
    assert!(inert.messages.is_empty());

    let invalid = session
        .eval_text("Part[{1,2,3}, Span[1, UpTo[-1]]]")
        .unwrap();
    assert_eq!(
        invalid.value.to_string(),
        "Part[List[1, 2, 3], Span[1, UpTo[-1]]]"
    );
    assert_eq!(invalid.messages.len(), 2);
    assert_eq!(invalid.messages[0].symbol, "UpTo");
    assert_eq!(invalid.messages[0].tag, "innf");
    assert_eq!(
        invalid.messages[0].text,
        "Non-negative integer or Infinity expected at position 1 in UpTo[-1]."
    );
    assert_eq!(invalid.messages[1].symbol, "Part");
    assert_eq!(invalid.messages[1].tag, "span");
    assert_eq!(
        invalid.messages[1].text,
        "1 ;; UpTo[-1] is not a valid Span specification. A Span specification should be 1, 2 or 3 machine-sized integers separated by ;;. (Any of the integers can be omitted or replaced with All.)"
    );
}

#[test]
fn builtin_message_registry_populates_listings_and_information() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("Messages[Set]")
            .unwrap()
            .value
            .to_string(),
        "List[Set::argform, Set::argx, Set::kval, Set::pkspec1, Set::shape, Set::write, Set::wrsym]"
    );
    assert_eq!(
        session
            .eval_text("MessageTags[Set]")
            .unwrap()
            .value
            .to_string(),
        "List[\"argform\", \"argx\", \"kval\", \"pkspec1\", \"shape\", \"write\", \"wrsym\"]"
    );

    let information = session
        .eval_text("Information[Set]")
        .unwrap()
        .value
        .to_string();
    assert!(information.contains(
        "Rule[\"Messages\", List[\"argform\", \"argx\", \"kval\", \"pkspec1\", \"shape\", \"write\", \"wrsym\"]]"
    ));
}

#[test]
fn push_message_allows_open_user_symbol_families() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f = 1").unwrap();
    session.push_message("f", "custom", "open family");

    assert!(session.messages.iter().any(|message| {
        message.symbol == "f" && message.tag == "custom" && message.text == "open family"
    }));
}

#[test]
fn push_message_allows_test_local_builtins_without_declared_messages() {
    let mut session = Session::new();
    session.register_builtin(BuiltinSpec::new(
        "MessageProbe",
        "Test-local builtin with no declared messages.",
        MemoryGrowthContract::NonAmplifying,
    ));

    session.push_message("MessageProbe", "custom", "undeclared family");

    assert!(session.messages.iter().any(|message| {
        message.symbol == "MessageProbe"
            && message.tag == "custom"
            && message.text == "undeclared family"
    }));
}

#[test]
fn push_message_allows_user_defined_builtin_message_names() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("Unprotect[Set]").unwrap();
    session
        .eval_text("Set::custom = \"custom message\"")
        .unwrap();
    session.eval_text("Protect[Set]").unwrap();
    session.push_message("Set", "custom", "user-defined builtin tag");

    assert!(session.messages.iter().any(|message| {
        message.symbol == "Set"
            && message.tag == "custom"
            && message.text == "user-defined builtin tag"
    }));
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "unregistered builtin message Set::bogus")]
fn push_message_rejects_unregistered_builtin_message_tags() {
    let mut session = Session::new();

    session.push_message("Set", "bogus", "missing registry entry");
}

#[test]
fn message_definitions_builtin_lists_immediate_and_delayed_templates() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f::argx = \"bad arg `1`\"").unwrap();
    session.eval_text("f::usage := suffix").unwrap();

    assert_eq!(
        session
            .eval_text("MessageDefinitions[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[f::argx, \"bad arg `1`\"], RuleDelayed[f::usage, suffix]]"
    );
    assert_eq!(
        session
            .eval_text("MessageDefinitions[f::argx]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[f::argx, \"bad arg `1`\"]]"
    );
}

#[test]
fn message_tags_builtin_lists_global_symbol_and_exact_message_tags() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f::argx = \"bad arg\"").unwrap();
    session.eval_text("f::usage := suffix").unwrap();
    session.eval_text("g::argx = \"other\"").unwrap();

    assert_eq!(
        session
            .eval_text("MessageTags[f]")
            .unwrap()
            .value
            .to_string(),
        "List[\"argx\", \"usage\"]"
    );
    let all_tags = session.eval_text("MessageTags[]").unwrap().value;
    let Expr::Call { args, .. } = all_tags else {
        panic!("MessageTags[] should return a list")
    };
    let tags = args.iter().map(ToString::to_string).collect::<Vec<_>>();
    assert_eq!(tags.len(), 62);
    for expected in [
        "\"argform\"",
        "\"argx\"",
        "\"decl\"",
        "\"failed\"",
        "\"mode\"",
        "\"notfound\"",
        "\"range\"",
        "\"sealedref\"",
        "\"setval\"",
        "\"unit\"",
        "\"usage\"",
        "\"wrsym\"",
    ] {
        assert!(tags.iter().any(|tag| tag == expected));
    }
    assert_eq!(
        session
            .eval_text("MessageTags[f::argx]")
            .unwrap()
            .value
            .to_string(),
        "List[\"argx\"]"
    );
}

#[test]
fn inspect_reports_message_definition_presence() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f::argx = \"bad arg\"").unwrap();

    let info = session.inspect("f").unwrap();
    assert_eq!(info.message_count, 1);
    assert_eq!(info.message_tags, vec!["argx".to_string()]);
}

#[test]
fn inspect_message_catalog_exposes_templates_and_delayed_flags() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f::argx = \"bad arg\"").unwrap();
    session.eval_text("f::usage := suffix").unwrap();

    let entries = session.inspect_message_catalog(Some("f")).unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].message_name, "f::argx");
    assert!(!entries[0].delayed);
    assert_eq!(entries[0].template.to_string(), "\"bad arg\"");
    assert_eq!(entries[1].message_name, "f::usage");
    assert!(entries[1].delayed);
    assert_eq!(entries[1].template.to_string(), "suffix");

    let exact = session.inspect_message_catalog(Some("f::usage")).unwrap();
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0].message_name, "f::usage");
}

#[test]
fn inspect_message_catalog_merges_builtin_templates_and_user_overrides() {
    let mut session = session_with_test_control_builtins();

    let entries = session.inspect_message_catalog(Some("Set")).unwrap();
    let expected = [
        (
            "argform",
            "\"Reports an invalid assignment target or assigned metadata value.\"",
        ),
        ("argx", "\"expected exactly `1` argument(s), got `2`\""),
        ("kval", "\"The value for the key `1` does not exist.\""),
        (
            "pkspec1",
            "\"The expression `1` cannot be used as a part specification.\"",
        ),
        ("shape", "\"Lists `1` and `2` are not the same shape.\""),
        ("write", "\"Tag `1` in `2` is Protected.\""),
        ("wrsym", "\"Symbol `1` is Protected.\""),
    ];

    assert_eq!(entries.len(), expected.len());
    for (entry, (tag, template)) in entries.iter().zip(expected) {
        assert_eq!(entry.message_name, format!("Set::{tag}"));
        assert_eq!(entry.qualified_symbol_name, "System`Set");
        assert_eq!(entry.tag, tag);
        assert_eq!(entry.template.to_string(), template);
        assert!(!entry.delayed);
    }

    let exact = session.inspect_message_catalog(Some("Set::argx")).unwrap();
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0], entries[1]);

    session.eval_text("Unprotect[Set]").unwrap();
    session.eval_text("Set::argx = \"custom\"").unwrap();
    session.eval_text("Protect[Set]").unwrap();

    let overridden = session.inspect_message_catalog(Some("Set")).unwrap();
    assert_eq!(overridden.len(), expected.len());
    let argx = overridden
        .iter()
        .filter(|entry| entry.tag == "argx")
        .collect::<Vec<_>>();
    assert_eq!(argx.len(), 1);
    assert_eq!(argx[0].template.to_string(), "\"custom\"");
    assert!(!argx[0].delayed);

    session.eval_text("f::tag = \"user-only\"").unwrap();
    assert_eq!(
        session.eval_text("Messages[f]").unwrap().value.to_string(),
        "List[f::tag]"
    );
}

#[test]
fn delayed_message_templates_are_evaluated_when_emitted() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("suffix = 7").unwrap();
    session.eval_text("f::argx := suffix").unwrap();
    let outcome = session.eval_text("Message[f::argx]").unwrap();

    assert!(outcome.messages.iter().any(|message| message.text == "7"));
}

#[test]
fn message_list_reports_messages_emitted_so_far_in_current_input() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text("Message[FindFile::notfound]; $MessageList")
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[HoldForm[FindFile::notfound]]"
    );
    assert_eq!(outcome.messages.len(), 1);
}

#[test]
fn request_message_list_reports_messages_emitted_so_far_in_current_input() {
    let mut session = session_with_test_control_builtins();

    let response = session
        .eval_request("Message[FindFile::notfound]; $MessageList")
        .unwrap();

    assert_eq!(
        response.value.to_string(),
        "List[HoldForm[FindFile::notfound]]"
    );
    assert_eq!(response.messages.len(), 1);
}

#[test]
fn quiet_keeps_suppressed_messages_out_of_message_list() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text("Quiet[Message[FindFile::notfound]]")
        .unwrap();

    assert_eq!(
        session.eval_text("$MessageList").unwrap().value.to_string(),
        "List[]"
    );
}

#[test]
fn invalid_message_name_form_emits_runtime_message() {
    let mut session = session_with_test_control_builtins();

    let outcome = session.eval_text("MessageName[1, 2]").unwrap();

    assert_eq!(outcome.value.to_string(), "MessageName[1, 2]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "MessageName" && message.tag == "argform")
    );
}

#[test]
fn invalid_builtin_argument_shapes_emit_runtime_messages() {
    let mut session = session_with_test_control_builtins();

    let with_outcome = session.eval_text("With[x, x]").unwrap();
    assert_eq!(with_outcome.value.to_string(), "With[x, x]");
    assert!(
        with_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "With" && message.tag == "argform" })
    );

    let replace_outcome = session.eval_text("7 /. 1").unwrap();
    assert_eq!(replace_outcome.value.to_string(), "ReplaceAll[7, 1]");
    assert!(
        replace_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "ReplaceAll" && message.tag == "argform" })
    );

    let get_outcome = session.eval_text("Get[1]").unwrap();
    assert_eq!(get_outcome.value.to_string(), "Get[1]");
    assert!(
        get_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "Get" && message.tag == "argform" })
    );
}
