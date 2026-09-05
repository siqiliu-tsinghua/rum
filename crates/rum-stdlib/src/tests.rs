use super::*;

#[path = "tests/file_io_support.rs"]
mod file_io_support;
use file_io_support::install_test_file_io;

#[path = "tests/association_depth_contract/mod.rs"]
mod association_depth_contract;
#[path = "tests/association_key_reshape_depth.rs"]
mod association_key_reshape_depth;
#[path = "tests/association_key_take_drop_depth.rs"]
mod association_key_take_drop_depth;
#[path = "tests/associations/mod.rs"]
mod associations;
#[path = "tests/associations_missing_surface.rs"]
mod associations_missing_surface;
#[path = "tests/builtin_reference_docgen.rs"]
mod builtin_reference_docgen;
#[path = "tests/bytes.rs"]
mod bytes;
#[path = "tests/calculus.rs"]
mod calculus;
#[path = "tests/catalog_capabilities.rs"]
mod catalog_capabilities;
#[path = "tests/codecs_matrix.rs"]
mod codecs_matrix;
#[path = "tests/codecs_text.rs"]
mod codecs_text;
#[path = "tests/codecs_toml.rs"]
mod codecs_toml;
#[path = "tests/codecs_xml.rs"]
mod codecs_xml;
#[path = "tests/codecs_yaml.rs"]
mod codecs_yaml;
#[path = "tests/combinatorics.rs"]
mod combinatorics;
#[path = "tests/datetime.rs"]
mod datetime;
#[path = "tests/debug_tools_evaluation.rs"]
mod debug_tools_evaluation;
#[path = "tests/debug_tools_reports.rs"]
mod debug_tools_reports;
#[path = "tests/debug_tools_resources.rs"]
mod debug_tools_resources;
#[path = "tests/definition_lhs_assignment_resume.rs"]
mod definition_lhs_assignment_resume;
#[path = "tests/delete_missing_depth.rs"]
mod delete_missing_depth;
#[path = "tests/delete_missing_target_certificates.rs"]
mod delete_missing_target_certificates;
#[path = "tests/delete_missing_target_semantic_locks.rs"]
mod delete_missing_target_semantic_locks;
#[path = "tests/exact_reduction_shapes.rs"]
mod exact_reduction_shapes;
#[path = "tests/functional_ops/mod.rs"]
mod functional_ops;
#[path = "tests/held_association_exact_path_depth.rs"]
mod held_association_exact_path_depth;
#[path = "tests/holding_and_control.rs"]
mod holding_and_control;
#[path = "tests/holding_and_control_abort.rs"]
mod holding_and_control_abort;
#[path = "tests/holding_and_control_cleanup.rs"]
mod holding_and_control_cleanup;
#[path = "tests/holding_and_control_failure_edges.rs"]
mod holding_and_control_failure_edges;
#[path = "tests/holding_and_control_failure_properties.rs"]
mod holding_and_control_failure_properties;
#[path = "tests/holding_and_control_part3.rs"]
mod holding_and_control_part3;
#[path = "tests/holding_and_control_part4.rs"]
mod holding_and_control_part4;
#[path = "tests/holding_and_control_part5.rs"]
mod holding_and_control_part5;
#[path = "tests/holding_and_control_part6.rs"]
mod holding_and_control_part6;
#[path = "tests/holding_and_control_part8.rs"]
mod holding_and_control_part8;
#[path = "tests/holding_and_control_success.rs"]
mod holding_and_control_success;
#[path = "tests/incidental_arithmetic.rs"]
mod incidental_arithmetic;
#[path = "tests/incidental_associations_part2.rs"]
mod incidental_associations_part2;
#[path = "tests/incidental_context_and_loader.rs"]
mod incidental_context_and_loader;
#[path = "tests/incidental_debug_tools_dialogs.rs"]
mod incidental_debug_tools_dialogs;
#[path = "tests/incidental_debug_tools_resources.rs"]
mod incidental_debug_tools_resources;
#[path = "tests/incidental_definitions_and_rules_cases.rs"]
mod incidental_definitions_and_rules_cases;
#[path = "tests/incidental_definitions_and_rules_lifecycle.rs"]
mod incidental_definitions_and_rules_lifecycle;
#[path = "tests/incidental_definitions_and_rules_part2.rs"]
mod incidental_definitions_and_rules_part2;
#[path = "tests/incidental_definitions_and_rules_patterns.rs"]
mod incidental_definitions_and_rules_patterns;
#[path = "tests/incidental_definitions_updates.rs"]
mod incidental_definitions_updates;
#[path = "tests/incidental_holding_and_control_part2.rs"]
mod incidental_holding_and_control_part2;
#[path = "tests/incidental_holding_and_control_part3.rs"]
mod incidental_holding_and_control_part3;
#[path = "tests/incidental_holding_and_control_part5.rs"]
mod incidental_holding_and_control_part5;
#[path = "tests/incidental_holding_and_control_part6.rs"]
mod incidental_holding_and_control_part6;
#[path = "tests/incidental_holding_and_control_part8.rs"]
mod incidental_holding_and_control_part8;
#[path = "tests/incidental_lowering_and_context.rs"]
mod incidental_lowering_and_context;
#[path = "tests/incidental_nothing.rs"]
mod incidental_nothing;
#[path = "tests/incidental_options_and_protection.rs"]
mod incidental_options_and_protection;
#[path = "tests/incidental_package_tools.rs"]
mod incidental_package_tools;
#[path = "tests/incidental_predicates.rs"]
mod incidental_predicates;
#[path = "tests/incidental_print_sink.rs"]
mod incidental_print_sink;
#[path = "tests/incidental_scoping_unique.rs"]
mod incidental_scoping_unique;
#[path = "tests/incidental_session_requests_debug_tooling_resume.rs"]
mod incidental_session_requests_debug_tooling_resume;
#[path = "tests/incidental_session_requests_functional_sequence_resume.rs"]
mod incidental_session_requests_functional_sequence_resume;
#[path = "tests/incidental_session_requests_input_resume.rs"]
mod incidental_session_requests_input_resume;
#[path = "tests/incidental_session_requests_matcher_resume.rs"]
mod incidental_session_requests_matcher_resume;
#[path = "tests/incidental_session_requests_transforms_resume.rs"]
mod incidental_session_requests_transforms_resume;
#[path = "tests/incidental_session_requests_transforms_resume_associations.rs"]
mod incidental_session_requests_transforms_resume_associations;
#[path = "tests/incidental_structure_ops_ordering_sets.rs"]
mod incidental_structure_ops_ordering_sets;
#[path = "tests/incidental_structure_ops_replace.rs"]
mod incidental_structure_ops_replace;
#[path = "tests/incidental_structure_ops_selectors_part2.rs"]
mod incidental_structure_ops_selectors_part2;
#[path = "tests/incidental_structure_ops_surface.rs"]
mod incidental_structure_ops_surface;
#[path = "tests/incidental_thread_and_dispatch.rs"]
mod incidental_thread_and_dispatch;
#[path = "tests/integer_digits.rs"]
mod integer_digits;
#[path = "tests/iterative_traversal_depth.rs"]
mod iterative_traversal_depth;
#[path = "tests/json.rs"]
mod json;
#[path = "tests/lazy.rs"]
mod lazy;
#[path = "tests/logic/mod.rs"]
mod logic;
#[path = "tests/map_indexed_association_depth.rs"]
mod map_indexed_association_depth;
#[path = "tests/merge_native_stack_guard.rs"]
mod merge_native_stack_guard;
#[path = "tests/number_theory.rs"]
mod number_theory;
#[path = "tests/polynomials.rs"]
mod polynomials;
#[path = "tests/predicates.rs"]
mod predicates;
#[path = "tests/predicates_part2.rs"]
mod predicates_part2;
#[path = "tests/preflight_memory.rs"]
mod preflight_memory;
#[path = "tests/random.rs"]
mod random;
#[path = "tests/resumable_traversal_regression.rs"]
mod resumable_traversal_regression;
#[path = "tests/scan_and_keypath_set_native_stack_guard.rs"]
mod scan_and_keypath_set_native_stack_guard;
#[path = "tests/schema.rs"]
mod schema;
#[path = "tests/schema_operator_forms.rs"]
mod schema_operator_forms;
#[path = "tests/session_requests/functional_iteration_resume.rs"]
mod session_requests_functional_iteration_resume;
#[path = "tests/session_requests/functional_sequence_resume.rs"]
mod session_requests_functional_sequence_resume;
#[path = "tests/session_requests/functional_targeted_resume.rs"]
mod session_requests_functional_targeted_resume;
#[path = "tests/session_requests/functional_tensor_string_resume.rs"]
mod session_requests_functional_tensor_string_resume;
#[path = "tests/session_requests/predicate_collections_resume.rs"]
mod session_requests_predicate_collections_resume;
#[path = "tests/session_requests/set_same_test_resume.rs"]
mod session_requests_set_same_test_resume;
#[path = "tests/session_requests/transforms_resume_associations.rs"]
mod session_requests_transforms_resume_associations;
#[path = "tests/stress_corpus.rs"]
mod stress_corpus;
#[path = "tests/strings/mod.rs"]
mod strings;
#[path = "tests/structure_ops/mod.rs"]
mod structure_ops;
#[path = "tests/structured_exceptions.rs"]
mod structured_exceptions;
#[path = "tests/traversal.rs"]
mod traversal;
#[path = "tests/valueq_effect_suppression.rs"]
mod valueq_effect_suppression;

#[test]
fn installs_identity_builtin() {
    let mut session = Session::new();
    install_core(&mut session);

    let info = session.inspect("Identity").unwrap();
    assert!(info.is_builtin);
}

#[test]
fn catalog_attributes_match_kernel_vocabulary() {
    use std::collections::BTreeMap;

    use rum_runtime::kernel_api::stdlib_builtin_attribute_specs;

    let mut catalog_by_name = BTreeMap::new();
    for spec in stdlib_catalog_specs() {
        let previous = catalog_by_name.insert(spec.name, spec.attributes);
        assert!(
            previous.is_none(),
            "duplicate stdlib catalog spec: {}",
            spec.name
        );
    }

    let mut kernel_by_name = BTreeMap::new();
    for spec in stdlib_builtin_attribute_specs() {
        let previous = kernel_by_name.insert(spec.name, spec.attributes);
        assert!(
            previous.is_none(),
            "duplicate kernel attribute vocabulary spec: {}",
            spec.name
        );
    }

    for (name, attributes) in catalog_by_name.iter() {
        assert_eq!(
            kernel_by_name.get(name).copied(),
            Some(*attributes),
            "kernel attribute vocabulary drift for {name}"
        );
    }
}

fn stdlib_catalog_specs() -> impl Iterator<Item = &'static rum_runtime::kernel_api::BuiltinSpec> {
    [
        catalog::ARITHMETIC_BUILTINS,
        catalog::LOGIC_BUILTINS,
        catalog::PREDICATE_BUILTINS,
        catalog::ASSOCIATION_CORE_BUILTINS,
        catalog::ASSOCIATION_TRANSFORM_BUILTINS,
        #[cfg(feature = "hash")]
        catalog::DIGEST_BUILTINS,
        catalog::BYTE_BUILTINS,
        catalog::CHARACTER_CLASS_BUILTINS,
        catalog::COMBINATORICS_BUILTINS,
        catalog::CONTROL_BUILTINS,
        catalog::CSV_BUILTINS,
        catalog::DATETIME_BUILTINS,
        catalog::FUNCTIONAL_BUILTINS,
        catalog::INI_BUILTINS,
        catalog::INTEGER_DIGITS_BUILTINS,
        catalog::JSON_BUILTINS,
        catalog::LAZY_BUILTINS,
        catalog::NUMBER_THEORY_BUILTINS,
        catalog::CALCULUS_BUILTINS,
        catalog::POLYNOMIAL_BUILTINS,
        catalog::RANDOM_BUILTINS,
        catalog::REWRITE_BUILTINS,
        catalog::SCHEMA_BUILTINS,
        catalog::STRING_CORE_BUILTINS,
        catalog::STRING_TEXT_BUILTINS,
        catalog::STRING_MATCH_BUILTINS,
        catalog::STRUCTURE_CORE_BUILTINS,
        catalog::STRUCTURE_ORDERING_BUILTINS,
        catalog::TOML_BUILTINS,
        catalog::XML_BUILTINS,
        catalog::YAML_BUILTINS,
    ]
    .into_iter()
    .flatten()
}
