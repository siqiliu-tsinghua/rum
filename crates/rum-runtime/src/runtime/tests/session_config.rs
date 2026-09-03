use super::*;

#[test]
fn default_session_configuration_matches_default_session_state() {
    let session = Session::new();

    assert_eq!(
        session.configuration(),
        SessionConfig::default().normalized()
    );
}

#[test]
fn default_native_stack_budget_is_derived_from_minimum_supported_stack() {
    assert_eq!(
        NATIVE_STACK_POLL_HEADROOM_BYTES,
        NATIVE_STACK_DEPTH_POLL_STRIDE * NATIVE_STACK_UNPOLLED_LEVEL_RESERVE_BYTES,
    );
    assert_eq!(
        DEFAULT_NATIVE_STACK_LIMIT
            + NATIVE_STACK_POLL_HEADROOM_BYTES
            + NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES,
        MIN_SUPPORTED_NATIVE_STACK_BYTES,
    );
    assert_eq!(
        native_stack_limit_for_declared_stack(MIN_SUPPORTED_NATIVE_STACK_BYTES),
        Some(DEFAULT_NATIVE_STACK_LIMIT),
    );
    assert_eq!(
        native_stack_limit_for_declared_stack(MIN_SUPPORTED_NATIVE_STACK_BYTES - 1),
        None,
    );
    assert_eq!(
        ResourceLimits::default().native_stack_limit,
        DEFAULT_NATIVE_STACK_LIMIT,
    );
}

#[test]
fn session_with_config_normalizes_contexts_paths_and_limits() {
    let config = SessionConfig::default()
        .with_resource_limits(ResourceLimits {
            recursion_limit: 17,
            native_stack_limit: 19,
            replace_repeated_limit: 23,
            evaluation_pass_limit: 31,
            eval_memory_limit: Some(29),
            generated_element_limit: 37,
            integer_work_limit: 41,
            iteration_work_limit: 43,
        })
        .with_current_context("Rules")
        .with_context_path(["System", "Rules", "Global", "Rules"])
        .with_package_search_paths(["./pkg-a", "./pkg-a", "./pkg-b"])
        .with_tracing(true)
        .with_asserts_enabled(true);
    let session = Session::with_config(config);

    assert_eq!(session.current_context(), "Rules`");
    assert_eq!(
        session.context_path(),
        &[
            "System`".to_string(),
            "Rules`".to_string(),
            "Global`".to_string(),
        ]
    );
    assert_eq!(
        session.package_search_paths(),
        &[PathBuf::from("./pkg-a"), PathBuf::from("./pkg-b")]
    );
    assert_eq!(
        session.resource_limits(),
        ResourceLimits {
            recursion_limit: 17,
            native_stack_limit: 19,
            replace_repeated_limit: 23,
            evaluation_pass_limit: 31,
            eval_memory_limit: Some(29),
            generated_element_limit: 37,
            integer_work_limit: 41,
            iteration_work_limit: 43,
        }
    );
    assert!(session.configuration().tracing_enabled);
    assert!(session.configuration().asserts_enabled);
}

#[test]
fn apply_config_updates_host_settings_without_clearing_existing_definitions() {
    let mut session = Session::new();
    session.eval_text("x = 1").unwrap();

    session.apply_config(
        SessionConfig::default()
            .with_current_context("Rules")
            .with_context_path(["System", "Global"])
            .with_package_search_paths(["./packages"])
            .with_tracing(true)
            .with_asserts_enabled(true)
            .with_resource_limits(ResourceLimits {
                recursion_limit: 9,
                native_stack_limit: 15,
                replace_repeated_limit: 10,
                evaluation_pass_limit: 11,
                eval_memory_limit: None,
                generated_element_limit: 12,
                integer_work_limit: 13,
                iteration_work_limit: 14,
            }),
    );

    assert_eq!(session.current_context(), "Rules`");
    assert_eq!(
        session.context_path(),
        &["System`".to_string(), "Global`".to_string()]
    );
    assert_eq!(
        session.package_search_paths(),
        &[PathBuf::from("./packages")]
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
    assert_eq!(
        session.resource_limits(),
        ResourceLimits {
            recursion_limit: 9,
            native_stack_limit: 15,
            replace_repeated_limit: 10,
            evaluation_pass_limit: 11,
            eval_memory_limit: None,
            generated_element_limit: 12,
            integer_work_limit: 13,
            iteration_work_limit: 14,
        }
    );
    assert!(session.configuration().tracing_enabled);
    assert!(session.configuration().asserts_enabled);
}

#[test]
fn session_setters_update_limits_capabilities_asserts_and_context_lookup() {
    let mut session = Session::new();

    session.set_current_context("Pkg");
    session.eval_text("value = 7").unwrap();
    session.set_current_context("Global");
    session.set_context_path(["Pkg", "System", "Pkg`"]);

    assert_eq!(
        session.context_path(),
        &[
            "Pkg`".to_string(),
            "System`".to_string(),
            "Pkg`".to_string(),
        ]
    );
    assert_eq!(session.eval_text("value").unwrap().value.to_string(), "7");

    let limits = ResourceLimits {
        recursion_limit: 101,
        native_stack_limit: 102,
        replace_repeated_limit: 103,
        evaluation_pass_limit: 104,
        eval_memory_limit: None,
        generated_element_limit: 105,
        integer_work_limit: 106,
        iteration_work_limit: 107,
    };
    session.set_resource_limits(limits);
    assert_eq!(session.resource_limits(), limits);

    let policy = CapabilityPolicy::allow([Capability::Pure, Capability::Random]);
    session.set_capability_policy(policy);
    assert_eq!(session.capability_policy(), policy);

    session.enable_asserts(true);
    assert!(session.asserts_enabled());
    session.enable_asserts(false);
    assert!(!session.asserts_enabled());
}

#[test]
fn random_u64_sequences_are_pinned_for_byte_and_integer_seeds() {
    let mut session = Session::new();

    session.seed_random_with_bytes(&[]);
    assert_eq!(
        [
            session.random_u64(),
            session.random_u64(),
            session.random_u64()
        ],
        [
            11091344671253066420,
            13793997310169335082,
            1900383378846508768,
        ]
    );

    session.seed_random_with_bytes(&[0, 42]);
    assert_eq!(
        [
            session.random_u64(),
            session.random_u64(),
            session.random_u64()
        ],
        [
            12315904777143787841,
            9616277548420421527,
            7930860611823695873,
        ]
    );

    let long_seed = (0_u8..32).collect::<Vec<_>>();
    session.seed_random_with_bytes(&long_seed);
    assert_eq!(
        [
            session.random_u64(),
            session.random_u64(),
            session.random_u64()
        ],
        [
            6374328123670283815,
            7769092342154173696,
            13795338270539439721,
        ]
    );

    session.seed_random_with_bytes(&[0x80]);
    assert_eq!(
        [
            session.random_u64(),
            session.random_u64(),
            session.random_u64()
        ],
        [
            16058641776494096580,
            3549933512096225125,
            4196352782529325668,
        ]
    );

    session.seed_random_with_integer(&BigInt::from(42));
    let integer_seeded = [
        session.random_u64(),
        session.random_u64(),
        session.random_u64(),
    ];
    assert_eq!(
        integer_seeded,
        [
            1865750160070900731,
            6791145067590612263,
            14118064813682728970,
        ]
    );

    session.seed_random_with_bytes(&[0, 42]);
    let byte_seeded = [
        session.random_u64(),
        session.random_u64(),
        session.random_u64(),
    ];
    assert_ne!(byte_seeded, integer_seeded);
}

#[test]
fn trace_message_and_print_accessors_expose_and_take_buffers() {
    let mut session = Session::new();

    session.push_message("probe", "stop", "stopped");
    session.push_print("printed");
    assert_eq!(session.messages().len(), 1);
    assert_eq!(session.prints(), &["printed".to_string()]);
    assert_eq!(session.take_messages()[0].tag, "stop");
    assert_eq!(session.take_prints(), vec!["printed".to_string()]);
    assert!(session.messages().is_empty());
    assert!(session.prints().is_empty());

    session.enable_tracing(true);
    session.eval_text("f[x_Integer] := x").unwrap();
    session.eval_text("f[3]").unwrap();
    assert!(
        session
            .trace()
            .iter()
            .any(|event| matches!(event, TraceEvent::RuleApplied { .. }))
    );
    assert!(!session.take_trace().is_empty());
    assert!(session.trace().is_empty());
}
