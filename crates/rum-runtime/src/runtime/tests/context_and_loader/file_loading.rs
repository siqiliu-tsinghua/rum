use super::*;

#[test]
fn get_loads_a_script_file_and_returns_the_last_value() {
    let dir = TempDirGuard::new("get");
    let file = dir.write(
        "script.rum",
        "Print[\"loading script\"]\nx = 1\npair = {\n  x,\n  2\n}\npair",
    );
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text(&format!("Get[\"{}\"]", rum_path(&file)))
        .unwrap();
    assert_eq!(outcome.value.to_string(), "List[1, 2]");
    assert_eq!(outcome.prints, vec!["loading script"]);
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
}

#[test]
fn get_propagates_exit_without_collapsing_it_to_failed_load() {
    let dir = TempDirGuard::new("get-exit");
    let file = dir.write("script.rum", "Exit[6]");
    let mut session = session_with_test_control_builtins();

    match session
        .eval_text(&format!("Get[\"{}\"]", rum_path(&file)))
        .expect_err("Exit inside Get should escape the loader")
    {
        EvalError::Exit { code } => assert_eq!(code, 6),
        other => panic!("expected Exit control flow, got {other:?}"),
    }
    assert!(session.take_messages().is_empty());
}

#[test]
fn nested_get_prefers_the_current_loader_directory_over_package_search_paths() {
    let root = TempDirGuard::new("nested-get-root");
    let fallback = TempDirGuard::new("nested-get-fallback");
    let main = root.write("main.rum", "Get[\"nested/inner.rum\"]");

    root.write("nested/inner.rum", "source = \"local\"\nsource");
    fallback.write("nested/inner.rum", "source = \"fallback\"\nsource");

    let mut session = session_with_test_control_builtins();
    session.set_package_search_paths([fallback.path().to_path_buf()]);

    assert_eq!(
        session
            .eval_text(&format!("Get[\"{}\"]", rum_path(&main)))
            .unwrap()
            .value
            .to_string(),
        "\"local\""
    );
    assert_eq!(
        session.eval_text("source").unwrap().value.to_string(),
        "\"local\""
    );
}

#[test]
fn get_and_needs_syntax_messages_report_the_loaded_file_location() {
    let dir = TempDirGuard::new("get-syntax-location");
    let file = dir.write("broken.rum", "a = 1\nb = 2\nc = 3\nd = 1 ! 2");
    for request in [
        format!("Get[\"{}\"]", rum_path(&file)),
        format!("Needs[\"Broken`\", \"{}\"]", rum_path(&file)),
    ] {
        let mut session = session_with_test_control_builtins();
        let outcome = session.eval_text(&request).unwrap();

        assert_eq!(outcome.value.to_string(), "$Failed");
        let message = outcome
            .messages
            .iter()
            .find(|message| message.symbol == "Get" && message.tag == "syntax")
            .expect("the loader should emit Get's syntax message");
        assert_eq!(
            message.text,
            "trailing input at line 4, column 7 (bytes 24..25)"
        );
    }
}

#[test]
fn get_rolls_back_loader_side_effects_and_collapses_failed_file_messages() {
    let dir = TempDirGuard::new("get-rollback");
    let file = dir.write(
        "broken.rum",
        "$Path = {\"/tmp/rum-loader-should-roll-back\"}\nBeginPackage[\"Broken`\"]\nFindFile[\"Missing`\"]\nflag = 1\nSetAttributes[f, HoldFirst]\n1 ! 2",
    );
    let mut session = session_with_test_control_builtins();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    let outcome = session
        .eval_text(&format!("Get[\"{}\"]", rum_path(&file)))
        .unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Get" && message.tag == "syntax")
    );
    assert!(
        !outcome
            .messages
            .iter()
            .any(|message| message.symbol == "FindFile" && message.tag == "notfound")
    );
    assert_eq!(
        session.eval_text("$Packages").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("$Path").unwrap().value.to_string(),
        format!("List[\"{}\"]", dir.path().display())
    );
    assert_eq!(
        session.eval_text("Context[]").unwrap().value.to_string(),
        "\"Global`\""
    );
    assert_eq!(
        session.eval_text("Broken`flag").unwrap().value.to_string(),
        "Broken`flag"
    );
    assert_eq!(
        session
            .eval_text("Attributes[Broken`f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn get_cycle_termination_does_not_abort_remaining_file_forms() {
    let dir = TempDirGuard::new("get-cycle-termination");
    let file = dir.write(
        "cycle-termination.rum",
        "loopA := loopB\nloopB := loopA\nloopA\npostCycleFlag = 2\npostCycleFlag",
    );
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text(&format!("Get[\"{}\"]", rum_path(&file)))
        .unwrap();

    assert_eq!(outcome.value.to_string(), "2");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "passlim")
    );
    assert!(
        !outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Get" && message.tag == "load")
    );
    assert_eq!(
        session
            .eval_text("postCycleFlag")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
}

#[test]
fn get_treats_resource_termination_as_failed_load_and_rolls_back() {
    let dir = TempDirGuard::new("get-resource-limit");
    let file = dir.write(
        "resource-limit.rum",
        "preLimitFlag = 1\ngrow[x_] := grow[f[x]]\ngrow[1]\npostLimitFlag = 2",
    );
    let mut session = session_with_test_control_builtins();
    session.set_recursion_limit(32);

    let outcome = session
        .eval_text(&format!("Get[\"{}\"]", rum_path(&file)))
        .unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Get" && message.tag == "load")
    );
    assert!(
        !outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reclim")
    );
    assert_eq!(
        session.eval_text("preLimitFlag").unwrap().value.to_string(),
        "preLimitFlag"
    );
    assert_eq!(
        session
            .eval_text("postLimitFlag")
            .unwrap()
            .value
            .to_string(),
        "postLimitFlag"
    );
    assert_eq!(
        session.eval_text("grow[1]").unwrap().value.to_string(),
        "grow[1]"
    );
}

#[test]
fn needs_loads_package_once_and_tracks_loaded_packages() {
    let dir = TempDirGuard::new("needs-once");
    dir.write(
        "Demo.rum",
        "BeginPackage[\"Demo`\"]\ntoken = 1\nEndPackage[]",
    );
    let mut session = session_with_test_control_builtins();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    assert_eq!(
        session
            .eval_text("Needs[\"Demo`\"]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(session.eval_text("token").unwrap().value.to_string(), "1");
    assert_eq!(
        session.eval_text("$Packages").unwrap().value.to_string(),
        "List[\"Demo`\"]"
    );

    session.eval_text("Demo`token = 5").unwrap();
    session.eval_text("Needs[\"Demo`\"]").unwrap();

    assert_eq!(
        session.eval_text("Demo`token").unwrap().value.to_string(),
        "5"
    );
}

#[test]
fn findfile_resolves_package_names_and_file_wrappers() {
    let dir = TempDirGuard::new("findfile");
    let file = dir.write(
        "Demo.rum",
        "BeginPackage[\"Demo`\"]\ntoken = 1\nEndPackage[]",
    );
    let mut session = session_with_test_control_builtins();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    assert_eq!(
        session
            .eval_text("FindFile[\"Demo`\"]")
            .unwrap()
            .value
            .to_string(),
        Expr::string(rum_path(&file)).to_string()
    );
    assert_eq!(
        session
            .eval_text(&format!("FindFile[File[\"{}\"]]", rum_path(&file)))
            .unwrap()
            .value
            .to_string(),
        Expr::string(rum_path(&file)).to_string()
    );
    assert_eq!(
        session
            .eval_text("FindFile[\"Missing`\"]")
            .unwrap()
            .value
            .to_string(),
        "$Failed"
    );
}

#[test]
fn loader_builtins_accept_resolved_load_target_specs() {
    let dir = TempDirGuard::new("resolved-loader-specs");
    let package_file = dir.write(
        "Demo.rum",
        "BeginPackage[\"Demo`\"]\ntoken = 1\nEndPackage[]",
    );
    let script_file = dir.write("load-target.rum", "value = 42\nvalue");
    let mut session = session_with_test_control_builtins();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    assert_eq!(
        session
            .eval_script("spec = \"Demo`\"\nFindFile[spec]")
            .unwrap()
            .value
            .to_string(),
        Expr::string(rum_path(&package_file)).to_string()
    );
    assert_eq!(
        session
            .eval_script(&format!(
                "spec = File[\"{}\"]\nGet[spec]",
                rum_path(&script_file)
            ))
            .unwrap()
            .value
            .to_string(),
        "42"
    );
}

#[test]
fn missing_findfile_emits_runtime_message() {
    let mut session = session_with_test_control_builtins();

    let outcome = session.eval_text("FindFile[\"Missing`\"]").unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "FindFile" && message.tag == "notfound")
    );
}

#[test]
fn needs_supports_explicit_file_targets() {
    let dir = TempDirGuard::new("needs-file");
    let file = dir.write(
        "custom-package.rum",
        "BeginPackage[\"Custom`\"]\nflag = 9\nEndPackage[]",
    );
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text(&format!("Needs[\"Custom`\", \"{}\"]", rum_path(&file)))
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(session.eval_text("flag").unwrap().value.to_string(), "9");

    session.eval_text("Custom`flag = 12").unwrap();
    session
        .eval_text(&format!(
            "Needs[\"Custom`\", File[\"{}\"]]",
            rum_path(&file)
        ))
        .unwrap();

    assert_eq!(
        session.eval_text("Custom`flag").unwrap().value.to_string(),
        "12"
    );
}

#[test]
fn needs_accepts_resolved_context_and_file_specs() {
    let dir = TempDirGuard::new("needs-resolved-specs");
    let file = dir.write(
        "custom-package.rum",
        "BeginPackage[\"Custom`\"]\nflag = 9\nEndPackage[]",
    );
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_script(&format!(
                "ctx = \"Custom`\"\nspec = File[\"{}\"]\nNeeds[ctx, spec]",
                rum_path(&file)
            ))
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(session.eval_text("flag").unwrap().value.to_string(), "9");
}

#[test]
fn missing_needs_returns_failed_and_emits_messages() {
    let mut session = session_with_test_control_builtins();

    let outcome = session.eval_text("Needs[\"Missing`\"]").unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Get" && message.tag == "notfound")
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Needs" && message.tag == "load")
    );
}
