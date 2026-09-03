use super::super::*;
use super::common::*;
use std::fs;

#[test]
fn every_host_profile_inherits_the_derived_native_stack_budget() {
    assert_eq!(
        HostResourceLimits::default().native_stack_limit,
        DEFAULT_NATIVE_STACK_LIMIT,
    );

    for config in [
        HostConfig::default(),
        HostConfig::sandboxed(),
        HostConfig::hermetic(),
        HostConfig::scripting(),
    ] {
        assert_eq!(
            config.resource_limits.native_stack_limit,
            DEFAULT_NATIVE_STACK_LIMIT,
        );
    }
}

#[test]
fn sandboxed_profile_pins_all_capabilities_and_providers() {
    let config = HostConfig::sandboxed();

    assert_profile_capability_surface(config.clone(), ExpectedProfileSurface::PureOnly);
    assert_profile_providers(config, false);
}

#[test]
fn hermetic_profile_pins_all_capabilities_and_providers() {
    let config = HostConfig::hermetic();

    assert_profile_capability_surface(config.clone(), ExpectedProfileSurface::PureOnly);
    assert_profile_providers(config, false);
}

#[test]
fn scripting_profile_pins_all_capabilities_and_providers() {
    let config = HostConfig::scripting();

    assert_profile_capability_surface(config.clone(), ExpectedProfileSurface::Unrestricted);
    assert_profile_providers(config, true);
}

#[test]
fn hermetic_profile_replays_rendered_results_across_sessions_and_reset() {
    const SOURCE: &str = "SeedRandom[242]; {RandomInteger[1000000], RandomInteger[1000000], RandomInteger[1000000], RandomInteger[1000000]}";

    let mut first_session = HostSession::builder()
        .with_config(HostConfig::hermetic())
        .build();
    let mut second_session = HostSession::builder()
        .with_config(HostConfig::hermetic())
        .build();

    let first = successful_eval(first_session.eval(SOURCE));
    let second = successful_eval(second_session.eval(SOURCE));
    assert_eq!(first.value.text.as_bytes(), second.value.text.as_bytes());
    assert!(first.messages.is_empty(), "{:?}", first.messages);
    assert!(second.messages.is_empty(), "{:?}", second.messages);

    first_session.reset();
    let after_reset = successful_eval(first_session.eval(SOURCE));
    assert_eq!(
        first.value.text.as_bytes(),
        after_reset.value.text.as_bytes()
    );
    assert!(
        after_reset.messages.is_empty(),
        "{:?}",
        after_reset.messages
    );
}

#[test]
fn sandboxed_host_config_blocks_filesystem_builtins_at_dispatch() {
    let dir = TempDirGuard::new("sandbox-blocks-fs");
    let package = dir.write("Pkg.rum", "BeginPackage[\"Pkg`\"]\nvalue = 7\nEndPackage[]");
    let input = dir.write("input.wl", "77");
    let save_path = dir.path().join("save.wl");
    let dump_path = dir.path().join("dump.mx");

    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed().with_package_search_paths([dir.path().to_path_buf()]))
        .build();

    assert_sandbox_denied(
        &mut session,
        &format!("Get[{}]", wl_string(&rum_path(&input))),
    );
    assert_sandbox_denied(
        &mut session,
        &format!("Needs[\"Pkg`\", {}]", wl_string(&rum_path(&package))),
    );
    assert_sandbox_denied(
        &mut session,
        &format!("FindFile[{}]", wl_string(&rum_path(&input))),
    );
    assert_sandbox_denied(
        &mut session,
        &format!("FileNames[\"*\", {{{}}}]", wl_string(&rum_path(dir.path()))),
    );
    assert_sandbox_denied(
        &mut session,
        &format!("x = 1; Save[{}, x]", wl_string(&rum_path(&save_path))),
    );
    assert_sandbox_denied(
        &mut session,
        &format!("x = 1; DumpSave[{}, x]", wl_string(&rum_path(&dump_path))),
    );
}

#[test]
fn trusted_host_file_names_uses_the_std_file_io_provider() {
    let dir = TempDirGuard::new("trusted-file-names");
    let b = dir.write("b.wl", "b");
    let a = dir.write("a.wl", "a");
    dir.write("c.txt", "c");
    let mut session = HostSession::new();

    let success = successful_eval(session.eval(format!(
        "FileNames[\"*.wl\", {{{}}}]",
        wl_string(&rum_path(dir.path()))
    )));

    assert_eq!(
        success.value.text,
        format!(
            "List[{}, {}]",
            wl_string(&rum_path(&a)),
            wl_string(&rum_path(&b))
        )
    );
    assert!(success.messages.is_empty(), "{:?}", success.messages);
}

#[test]
fn sandboxed_host_config_blocks_interactive_builtins_at_dispatch() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_sandbox_denied(&mut session, "Input[]");
    assert_sandbox_denied(&mut session, "InputString[]");
    assert_sandbox_denied(&mut session, "Dialog[1]");
}

#[test]
fn sandboxed_host_config_with_file_provider_blocks_internal_loader_calls() {
    let dir = TempDirGuard::new("sandbox-blocks-internal-loads");
    dir.write(
        "DepPkg.rum",
        "BeginPackage[\"DepPkg`\"]\ndependencyValue = 33\nEndPackage[]",
    );
    dir.write(
        "StubPkg.rum",
        "BeginPackage[\"StubPkg`\"]\nautoloadValue = 44\nEndPackage[]",
    );

    let mut session = HostSession::builder()
        .with_config(
            HostConfig::sandboxed()
                .with_file_io(StdFileIo::provider())
                .with_package_search_paths([dir.path().to_path_buf()]),
        )
        .build();

    let begin_package = successful_eval(session.eval("BeginPackage[\"MainPkg`\", {\"DepPkg`\"}]"));
    assert_eq!(begin_package.value.text, "$Failed");
    assert!(begin_package.messages.iter().any(|message| {
        message.symbol == "General"
            && message.tag == "sandbox"
            && message.text == "Needs is not permitted in this sandbox."
    }));
    let context = successful_eval(session.eval("$Context")).value.text;
    let context_path = successful_eval(session.eval("$ContextPath")).value.text;
    let packages = successful_eval(session.eval("$Packages")).value.text;
    eprintln!(
        "restricted+provider BeginPackage value={} messages={:?} context={} context_path={} packages={}",
        begin_package.value.text, begin_package.messages, context, context_path, packages
    );
    assert_eq!(context, "\"Global`\"");
    assert_eq!(context_path, "List[\"System`\", \"Global`\"]");
    assert_eq!(packages, "List[]");

    successful_eval(session.eval("DeclarePackage[\"StubPkg`\", {\"autoloadValue\"}]"));
    let declared_stub = successful_eval(session.eval("StubPkg`autoloadValue"));
    assert_eq!(declared_stub.value.text, "$Failed");
    assert!(declared_stub.messages.iter().any(|message| {
        message.symbol == "General"
            && message.tag == "sandbox"
            && message.text == "Needs is not permitted in this sandbox."
    }));
    let packages = successful_eval(session.eval("$Packages")).value.text;
    eprintln!(
        "restricted+provider declared-stub value={} messages={:?} packages={}",
        declared_stub.value.text, declared_stub.messages, packages
    );
    assert_eq!(packages, "List[]");
}

#[test]
fn sandboxed_host_config_blocks_runtime_constructed_filesystem_calls() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_sandbox_denied(&mut session, "Symbol[\"Get\"][\"/etc/passwd\"]");

    let runtime_constructed = format!(
        "ToExpression[{}]",
        wl_string(&format!("Get[{}]", wl_string("/etc/passwd")))
    );
    assert_sandbox_denied(&mut session, &runtime_constructed);
}

#[test]
fn sandboxed_host_config_blocks_diagnostic_builtins_at_dispatch() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_sandbox_denied(&mut session, "RumProfileSnapshot[]");
}

#[test]
fn sandboxed_host_config_blocks_clock_builtins_at_dispatch() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    let success = successful_eval(session.eval("AbsoluteTime[]"));
    eprintln!(
        "sandboxed AbsoluteTime[] value={} messages={:?}",
        success.value.text, success.messages
    );
    assert_eq!(success.value.text, "$Failed");
    assert!(
        success
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "sandbox")
    );
}

#[test]
fn sandboxed_host_config_blocks_exit_at_dispatch_but_trusted_allows_it() {
    let mut sandboxed = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_sandbox_denied(&mut sandboxed, "Exit[0]");

    let mut trusted = HostSession::new();
    match trusted.eval("Exit[7]") {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Exit);
            assert_eq!(failure.exit_status, Some(7));
            assert_eq!(failure.message, "Exit[7]");
        }
        other => panic!("expected trusted Exit failure, got {other:?}"),
    }
}

#[test]
fn sandboxed_host_config_still_allows_pure_work_and_path_string_helpers() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_eq!(successful_eval(session.eval("2 + 3")).value.text, "5");
    assert_eq!(
        successful_eval(session.eval("Reverse[{1, 2, 3}]"))
            .value
            .text,
        "List[3, 2, 1]"
    );
    assert_eq!(
        successful_eval(session.eval("FileNameJoin[{\"alpha\", \"beta.wl\"}]"))
            .value
            .text,
        "\"alpha/beta.wl\""
    );
    assert_eq!(
        successful_eval(session.eval("DirectoryName[\"/tmp/a/b.txt\"]"))
            .value
            .text,
        "\"/tmp/a/\""
    );
    assert_eq!(
        successful_eval(session.eval("ParentDirectory[\"/tmp/a/b/\"]"))
            .value
            .text,
        "\"/tmp/a/\""
    );
    assert_eq!(
        successful_eval(session.eval("FileNameTake[\"/tmp/a/b.txt\"]"))
            .value
            .text,
        "\"b.txt\""
    );
}

#[test]
fn capability_policy_can_allow_reads_without_writes() {
    let dir = TempDirGuard::new("sandbox-read-only");
    let input = dir.write("input.wl", "loaded = 42\nloaded");
    let save_path = dir.path().join("save.wl");
    let policy = CapabilityPolicy::allow([Capability::Pure, Capability::FileRead]);
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_capability_policy(policy))
        .build();

    assert_eq!(
        successful_eval(session.eval(format!("Get[{}]", wl_string(&rum_path(&input)))))
            .value
            .text,
        "42"
    );
    assert_sandbox_denied(
        &mut session,
        &format!("Save[{}, loaded]", wl_string(&rum_path(&save_path))),
    );
    assert_sandbox_denied(&mut session, "RumProfileSnapshot[]");
}

#[test]
fn capability_policy_can_allow_clock_without_filesystem() {
    let policy = CapabilityPolicy::allow([Capability::Pure, Capability::Clock]);
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_capability_policy(policy))
        .build();

    let absolute_time = successful_eval(session.eval("AbsoluteTime[]"));
    eprintln!(
        "clock policy AbsoluteTime[] value={}",
        absolute_time.value.text
    );
    assert_ne!(absolute_time.value.text, "$Failed");

    let get = successful_eval(session.eval("Get[\"/etc/passwd\"]"));
    eprintln!(
        "clock policy Get[] value={} messages={:?}",
        get.value.text, get.messages
    );
    assert_eq!(get.value.text, "$Failed");
    assert!(
        get.messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "sandbox")
    );
}

#[test]
fn sandboxed_host_config_allows_pure_datetime_forms() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_arity_not_sandbox_denial(&mut session, "DateList[]", "DateList", "argx");
    assert_arity_not_sandbox_denial(&mut session, "DateString[]", "DateString", "argx");

    let date_list = successful_eval(session.eval("DateList[0]"));
    assert_eq!(date_list.value.text, "List[1970, 1, 1, 0, 0, 0]");
    assert!(date_list.messages.is_empty(), "{:?}", date_list.messages);

    let date_string = successful_eval(session.eval("DateString[0]"));
    assert_eq!(date_string.value.text, "\"1970-01-01T00:00:00Z\"");
    assert!(
        date_string.messages.is_empty(),
        "{:?}",
        date_string.messages
    );
}

#[test]
fn clock_policy_allows_composed_datetime_current_readers() {
    let policy = CapabilityPolicy::allow([Capability::Pure, Capability::Clock]);
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_capability_policy(policy))
        .build();

    let date_list = successful_eval(session.eval("DateList[AbsoluteTime[]]"));
    assert_ne!(date_list.value.text, "$Failed");
    assert!(date_list.messages.is_empty(), "{:?}", date_list.messages);

    let date_string = successful_eval(session.eval("DateString[AbsoluteTime[]]"));
    assert_ne!(date_string.value.text, "$Failed");
    assert!(
        date_string.messages.is_empty(),
        "{:?}",
        date_string.messages
    );
}

#[test]
fn permissive_policy_without_file_provider_returns_noio_failed() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().without_file_io())
        .build();

    let success = successful_eval(session.eval("Get[\"/nonexistent\"]"));

    assert_eq!(success.value.text, "$Failed");
    assert_eq!(success.messages.len(), 1);
    assert_eq!(success.messages[0].symbol, "General");
    assert_eq!(success.messages[0].tag, "noio");
    assert_eq!(success.messages[0].text, "no file provider installed");
    eprintln!(
        "no_provider_get value={} messages={:?}",
        success.value.text, success.messages
    );

    let file_names = successful_eval(session.eval("FileNames[]"));
    assert_eq!(file_names.value.text, "$Failed");
    assert_eq!(file_names.messages.len(), 1);
    assert_eq!(file_names.messages[0].symbol, "General");
    assert_eq!(file_names.messages[0].tag, "noio");
    assert_eq!(file_names.messages[0].text, "no file provider installed");
}

#[test]
fn sandboxed_host_config_blocks_entropy_builtins_but_allows_prng_builtins() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_sandbox_denied(&mut session, "ReseedRandom[]");
    assert_sandbox_denied(&mut session, "RandomBytes[8]");
    assert_sandbox_denied(&mut session, "RandomPassword[8]");
    assert_arity_not_sandbox_denial(&mut session, "SeedRandom[]", "SeedRandom", "argx");

    let pure = successful_eval(session.eval(
        "SeedRandom[42]; {RandomInteger[10], RandomChoice[{a,b}], RandomSample[{a,b}], RandomString[8], BlockRandom[RandomInteger[10]], UuidQ[RandomUuid[]]}",
    ));
    assert_ne!(pure.value.text, "$Failed");
    assert!(pure.messages.is_empty(), "messages={:?}", pure.messages);
}

#[test]
fn permissive_policy_without_entropy_provider_returns_noentropy_failed() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().without_entropy_source())
        .build();

    for source in ["RandomBytes[8]", "RandomPassword[8]", "ReseedRandom[]"] {
        let success = successful_eval(session.eval(source));

        assert_eq!(success.value.text, "$Failed", "source {source}");
        assert_eq!(success.messages.len(), 1, "source {source}");
        assert_eq!(success.messages[0].symbol, "General");
        assert_eq!(success.messages[0].tag, "noentropy");
        assert_eq!(success.messages[0].text, "no entropy provider installed");
    }
}

#[test]
fn random_policy_allows_entropy_reseed() {
    let policy = CapabilityPolicy::allow([Capability::Pure, Capability::Random]);
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_capability_policy(policy))
        .build();

    let success = successful_eval(session.eval("ReseedRandom[]"));
    assert_eq!(success.value.text, "Null");
    assert!(
        success.messages.is_empty(),
        "messages={:?}",
        success.messages
    );
}

#[test]
fn default_host_config_installs_entropy_provider_for_random_bytes() {
    let mut session = HostSession::new();

    let first = successful_eval(session.eval("RandomBytes[8]"));
    let second = successful_eval(session.eval("RandomBytes[8]"));

    let first_payload =
        byte_array_payload(&first.value.text).expect("RandomBytes should return ByteArray text");
    let second_payload =
        byte_array_payload(&second.value.text).expect("RandomBytes should return ByteArray text");
    assert_eq!(first_payload.len(), 12);
    assert_eq!(second_payload.len(), 12);
    assert_ne!(first.value.text, second.value.text);
}

#[test]
fn default_host_config_installs_entropy_provider_for_random_password() {
    let mut session = HostSession::new();
    let default_alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let custom_alphabet = "abcdefghjkmnpqrstuvwxyz23456789";

    let first = successful_eval(session.eval("RandomPassword[16]"));
    let second = successful_eval(session.eval("RandomPassword[16]"));
    let custom_source = format!("RandomPassword[{custom_alphabet:?}, 20]");
    let custom = successful_eval(session.eval(custom_source.as_str()));

    let first_text = string_payload(&first.value.text).expect("RandomPassword should return text");
    let second_text =
        string_payload(&second.value.text).expect("RandomPassword should return text");
    let custom_text =
        string_payload(&custom.value.text).expect("RandomPassword should return text");

    assert_eq!(first_text.len(), 16);
    assert_eq!(second_text.len(), 16);
    assert!(first_text.chars().all(|ch| default_alphabet.contains(ch)));
    assert!(second_text.chars().all(|ch| default_alphabet.contains(ch)));
    assert_ne!(first.value.text, second.value.text);

    assert_eq!(custom_text.len(), 20);
    assert!(custom_text.chars().all(|ch| custom_alphabet.contains(ch)));
    assert!(first.messages.is_empty(), "messages={:?}", first.messages);
    assert!(second.messages.is_empty(), "messages={:?}", second.messages);
    assert!(custom.messages.is_empty(), "messages={:?}", custom.messages);
}

#[test]
fn default_host_config_keeps_filesystem_catalog_unrestricted() {
    let dir = TempDirGuard::new("default-unrestricted");
    let input = dir.write("input.wl", "loaded = 42\nloaded");
    let save_path = dir.path().join("save.wl");
    let mut session = HostSession::new();

    assert_eq!(
        successful_eval(session.eval(format!("Get[{}]", wl_string(&rum_path(&input)))))
            .value
            .text,
        "42"
    );
    assert_eq!(
        successful_eval(session.eval(format!(
            "Save[{}, loaded]",
            wl_string(&rum_path(&save_path))
        )))
        .value
        .text,
        "Null"
    );
    assert!(save_path.exists());
    assert!(!fs::read_to_string(save_path).unwrap().is_empty());
}

#[test]
fn default_host_config_keeps_diagnostic_catalog_unrestricted() {
    let mut session = HostSession::new();

    assert_eq!(
        successful_eval(session.eval("RumProfileSnapshot[]"))
            .value
            .text,
        "Null"
    );
    assert_ne!(
        successful_eval(session.eval("MemoryInUse[]")).value.text,
        "$Failed"
    );
}

#[test]
fn capability_policy_can_explicitly_allow_diagnostics() {
    let policy = CapabilityPolicy::allow([Capability::Pure, Capability::Diagnostic]);
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_capability_policy(policy))
        .build();

    assert_eq!(
        successful_eval(session.eval("RumProfileSnapshot[]"))
            .value
            .text,
        "Null"
    );
    assert_ne!(
        successful_eval(session.eval("MaxMemoryUsed[]")).value.text,
        "$Failed"
    );
    assert_sandbox_denied(&mut session, "Get[\"/etc/passwd\"]");
}
