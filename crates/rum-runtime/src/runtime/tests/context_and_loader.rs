use super::*;

mod file_loading;
mod script_forms;

#[test]
fn context_variables_and_builtin_inspection_reflect_session_contexts() {
    let mut session = session_with_test_control_builtins();
    session.set_current_context("Rules");

    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Rules`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"System`\", \"Global`\"]"
    );

    let info = session.inspect("Identity").unwrap();
    assert_eq!(info.qualified_name, "System`Identity");
    assert_eq!(info.context, "System`");
    assert_eq!(info.display_name, "Identity");
    assert!(info.is_builtin);
}

#[test]
fn context_path_and_packages_assignments_update_session_fields() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("Rules`x = 10").unwrap();
    session.eval_text("Global`x = 20").unwrap();

    assert_eq!(
        session
            .eval_text("$ContextPath = {\"Rules\", \"System`\", \"Rules`\"}")
            .unwrap()
            .value
            .to_string(),
        "List[\"Rules\", \"System`\", \"Rules`\"]"
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Rules`\", \"System`\"]"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");

    session
        .eval_text("$Packages = {\"Pkg\", \"Pkg`\", \"Other`\"}")
        .unwrap();
    assert_eq!(
        session.eval_text("$Packages").unwrap().value.to_string(),
        "List[\"Pkg`\", \"Other`\"]"
    );
}

#[test]
fn explicitly_protected_session_configuration_variables_reject_assignments() {
    let mut session = session_with_test_control_builtins();

    let context_path = session.eval_text("$ContextPath").unwrap().value.to_string();
    let packages = session.eval_text("$Packages").unwrap().value.to_string();
    let path = session.eval_text("$Path").unwrap().value.to_string();

    session
        .eval_text("Protect[$ContextPath, $Packages, $Path]")
        .unwrap();

    for (source, target, expected) in [
        (
            "$ContextPath = {\"System`\", \"Leak`\"}",
            "$ContextPath",
            context_path,
        ),
        ("$Packages = {\"Leak`\"}", "$Packages", packages),
        ("$Path = {\"/tmp/rum-protected-path\"}", "$Path", path),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == "Set" && message.tag == "wrsym"),
            "{source}: {:?}",
            outcome.messages
        );
        assert_eq!(
            session.eval_text(target).unwrap().value.to_string(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn path_assignment_remains_writable_without_explicit_protect() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("Attributes[$Path]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    let outcome = session
        .eval_text("$Path = {\"/tmp/rum-a\", \"/tmp/rum-a\", \"/tmp/rum-b\"}")
        .unwrap();
    assert!(outcome.messages.is_empty());
    assert_eq!(
        session.eval_text("$Path").unwrap().value.to_string(),
        "List[\"/tmp/rum-a\", \"/tmp/rum-b\"]"
    );
}

#[test]
fn bare_symbols_search_context_path_before_current_context() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("Global`x = 10").unwrap();
    session.set_current_context("Rules");
    session.eval_text("Rules`x = 20").unwrap();
    session.eval_text("local = 30").unwrap();

    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");

    let info = session.inspect("local").unwrap();
    assert_eq!(info.qualified_name, "Rules`local");
    assert_eq!(info.context, "Rules`");
    assert_eq!(
        session.eval_text("Rules`x").unwrap().value.to_string(),
        "20"
    );
}

#[test]
fn context_builtin_reports_session_and_symbol_contexts_without_evaluating_symbols() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("x = 1").unwrap();

    assert_eq!(
        session.eval_text("Context[]").unwrap().value.to_string(),
        "\"Global`\""
    );
    assert_eq!(
        session.eval_text("Context[x]").unwrap().value.to_string(),
        "\"Global`\""
    );
    assert_eq!(
        session
            .eval_text("Context[Identity]")
            .unwrap()
            .value
            .to_string(),
        "\"System`\""
    );
    assert_eq!(
        session
            .eval_text("Context[\"Identity\"]")
            .unwrap()
            .value
            .to_string(),
        "\"System`\""
    );
    assert_eq!(
        session
            .eval_text("Context[\"MissingSymbol\"]")
            .unwrap()
            .value
            .to_string(),
        "Context[\"MissingSymbol\"]"
    );
}

#[test]
fn contexts_lists_known_session_contexts() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("BeginPackage[\"Demo`\"]").unwrap();
    session.eval_text("public = 1").unwrap();
    session.eval_text("Begin[\"`Private`\"]").unwrap();
    session.eval_text("hidden = 2").unwrap();
    session.eval_text("End[]").unwrap();
    session.eval_text("EndPackage[]").unwrap();

    let contexts = session.eval_text("Contexts[]").unwrap();
    assert_eq!(
        contexts.value.to_string(),
        "List[\"Demo`\", \"Demo`Private`\", \"Global`\", \"System`\"]"
    );
    assert!(contexts.messages.is_empty());

    let filtered = session.eval_text("Contexts[\"Demo`*\"]").unwrap();
    assert_eq!(
        filtered.value.to_string(),
        "List[\"Demo`\", \"Demo`Private`\"]"
    );
    assert!(filtered.messages.is_empty());

    let no_match = session.eval_text("Contexts[\"Missing`*\"]").unwrap();
    assert_eq!(no_match.value.to_string(), "List[]");
    assert!(no_match.messages.is_empty());
}

#[test]
fn begin_and_end_balance_context_and_context_path() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("Begin[\"Rules`\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Rules`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Rules`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Rules`\", \"System`\", \"Global`\"]"
    );

    assert_eq!(
        session
            .eval_text("Begin[\"`Private`\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Rules`Private`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Rules`Private`\""
    );
    assert_eq!(
        session.eval_text("End[]").unwrap().value.to_string(),
        "\"Rules`Private`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Rules`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Rules`\", \"System`\", \"Global`\"]"
    );
}

#[test]
fn begin_package_and_end_package_restore_outer_context_and_export_package_context() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("BeginPackage[\"Demo`\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Demo`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Demo`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Demo`\", \"System`\"]"
    );

    session.eval_text("Begin[\"`Private`\"]").unwrap();
    session.eval_text("hidden = 1").unwrap();
    assert_eq!(
        session.eval_text("End[]").unwrap().value.to_string(),
        "\"Demo`Private`\""
    );

    session.eval_text("public = 2").unwrap();

    assert_eq!(
        session.eval_text("EndPackage[]").unwrap().value.to_string(),
        "Null"
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Global`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Demo`\", \"System`\", \"Global`\"]"
    );
    assert_eq!(session.eval_text("public").unwrap().value.to_string(), "2");
    assert_eq!(
        session.eval_text("hidden").unwrap().value.to_string(),
        "hidden"
    );
}

#[test]
fn begin_package_usage_messages_make_public_symbols_visible_from_private_context() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("BeginPackage[\"Demo`\"]").unwrap();
    session.eval_text("foo::usage = \"foo[x]\"").unwrap();
    session.eval_text("Begin[\"`Private`\"]").unwrap();
    session.eval_text("foo[x_] := x").unwrap();
    session.eval_text("End[]").unwrap();
    session.eval_text("EndPackage[]").unwrap();

    assert_eq!(
        session.eval_text("Demo`foo[7]").unwrap().value.to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("Demo`Private`foo[7]")
            .unwrap()
            .value
            .to_string(),
        "Demo`Private`foo[7]"
    );
}

#[test]
fn begin_package_dependencies_stay_on_private_context_path_when_already_loaded() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("BeginPackage[\"DepA`\"]").unwrap();
    session
        .eval_text("depHead::usage = \"depHead[x]\"")
        .unwrap();
    session.eval_text("EndPackage[]").unwrap();
    session.eval_text("BeginPackage[\"DepB`\"]").unwrap();
    session.eval_text("depValue::usage = \"depValue\"").unwrap();
    session.eval_text("EndPackage[]").unwrap();

    session
        .eval_text("BeginPackage[\"Main`\", {\"DepA`\", \"DepB`\"}]")
        .unwrap();
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Main`\", \"DepA`\", \"DepB`\", \"System`\"]"
    );
    session.eval_text("Begin[\"`Private`\"]").unwrap();
    session.eval_text("depHead[x_] := x").unwrap();
    session.eval_text("local = depHead[depValue]").unwrap();
    session.eval_text("End[]").unwrap();
    session.eval_text("EndPackage[]").unwrap();

    assert_eq!(
        session
            .eval_text("DepA`depHead[7]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("Main`Private`local")
            .unwrap()
            .value
            .to_string(),
        "DepB`depValue"
    );
    assert_eq!(
        session
            .eval_text("Main`Private`depHead[7]")
            .unwrap()
            .value
            .to_string(),
        "Main`Private`depHead[7]"
    );
}

#[test]
fn begin_package_loads_present_dependency_and_enters_package() {
    let dir = TempDirGuard::new("begin-package-dependency");
    dir.write(
        "GoodDep.rum",
        "BeginPackage[\"GoodDep`\"]\ngoodVal = 77\nEndPackage[]",
    );
    let mut session = session_with_test_control_builtins();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    let outcome = session
        .eval_text("BeginPackage[\"MainPkg`\", {\"GoodDep`\"}]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "\"MainPkg`\"");
    assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"MainPkg`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"MainPkg`\", \"GoodDep`\", \"System`\"]"
    );
    assert_eq!(
        session.eval_text("$Packages").unwrap().value.to_string(),
        "List[\"MainPkg`\", \"GoodDep`\"]"
    );

    session.eval_text("mainVal = GoodDep`goodVal + 1").unwrap();
    assert_eq!(
        session
            .eval_text("MainPkg`mainVal")
            .unwrap()
            .value
            .to_string(),
        "78"
    );
    assert_eq!(
        session.eval_text("EndPackage[]").unwrap().value.to_string(),
        "Null"
    );
}

#[test]
fn begin_package_missing_dependency_fails_closed_and_restores_state() {
    let mut session = session_with_test_control_builtins();
    session.set_current_context("Outer`");
    session
        .eval_text("$ContextPath = {\"Existing`\", \"System`\", \"Global`\"}")
        .unwrap();
    session
        .eval_text("$Packages = {\"AlreadyLoaded`\"}")
        .unwrap();
    let previous_context = session.current_context.clone();
    let previous_context_path = session.context_path.clone();
    let previous_loaded_packages = session.loaded_packages.clone();
    let previous_context_stack_len = session.context_stack.len();
    let previous_contexts = session.eval_text("Contexts[]").unwrap().value.to_string();

    let outcome = session
        .eval_text("BeginPackage[\"FailMain`\", {\"NoSuchDep`\"}]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Get" && message.tag == "notfound"),
        "{:?}",
        outcome.messages
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Needs" && message.tag == "load"),
        "{:?}",
        outcome.messages
    );
    assert_eq!(session.current_context, previous_context);
    assert_eq!(session.context_path, previous_context_path);
    assert_eq!(session.loaded_packages, previous_loaded_packages);
    assert_eq!(session.context_stack.len(), previous_context_stack_len);

    let contexts = session.eval_text("Contexts[]").unwrap().value.to_string();
    assert_eq!(contexts, previous_contexts);
    assert!(!contexts.contains("FailMain`"), "{contexts}");
}

#[test]
fn begin_package_second_dependency_failure_restores_state_and_package_list() {
    let dir = TempDirGuard::new("begin-package-second-dependency");
    dir.write(
        "GoodDep.rum",
        "BeginPackage[\"GoodDep`\"]\ngoodVal = 77\nEndPackage[]",
    );
    let mut session = session_with_test_control_builtins();
    session.set_package_search_paths([dir.path().to_path_buf()]);
    session.set_current_context("Outer`");
    session
        .eval_text("$ContextPath = {\"Existing`\", \"System`\", \"Global`\"}")
        .unwrap();
    session
        .eval_text("$Packages = {\"AlreadyLoaded`\"}")
        .unwrap();
    let previous_context = session.current_context.clone();
    let previous_context_path = session.context_path.clone();
    let previous_loaded_packages = session.loaded_packages.clone();
    let previous_context_stack_len = session.context_stack.len();

    let outcome = session
        .eval_text("BeginPackage[\"PartialMain`\", {\"GoodDep`\", \"NoSuchDep`\"}]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome.messages.iter().any(|message| {
            message.symbol == "Needs"
                && message.tag == "load"
                && message.text.contains("NoSuchDep`")
        }),
        "{:?}",
        outcome.messages
    );
    assert_eq!(session.current_context, previous_context);
    assert_eq!(session.context_path, previous_context_path);
    assert_eq!(session.loaded_packages, previous_loaded_packages);
    assert_eq!(session.context_stack.len(), previous_context_stack_len);

    let contexts = session.eval_text("Contexts[]").unwrap().value.to_string();
    assert!(contexts.contains("GoodDep`"), "{contexts}");
    assert!(!contexts.contains("PartialMain`"), "{contexts}");
}

#[test]
fn begin_package_expands_relative_context_specs_against_the_current_context() {
    let mut session = session_with_test_control_builtins();
    session.set_current_context("Outer`");

    assert_eq!(
        session
            .eval_text("BeginPackage[\"`Inner`\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Outer`Inner`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Outer`Inner`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Outer`Inner`\", \"System`\"]"
    );

    session.eval_text("value = 7").unwrap();
    session.eval_text("EndPackage[]").unwrap();

    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Outer`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Outer`Inner`\", \"System`\", \"Global`\"]"
    );
    assert_eq!(session.eval_text("value").unwrap().value.to_string(), "7");
}
