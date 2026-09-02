use crate::{arithmetic, associations, install_core, strings, structure_ops};
use rum_core::{BigInt, Expr, SharedArgs};
use rum_runtime::kernel_api::{
    BuiltinSpec, EvalError, MemoryGrowthContract, Session, expr_as_string, system_expr,
};

const PREFLIGHT_PROBE_CASE_ENV: &str = "RUM_STDLIB_PREFLIGHT_PROBE_CASE";
const LOW_CEILING_ALLOWANCE: usize = 1024 * 1024;
const MAX_REFUSAL_HWM_GROWTH: usize = 4 * 1024 * 1024;

#[test]
fn projected_preflight_builtins_refuse_before_allocation_and_admit_realistic_inputs() {
    let Some(case) = std::env::var_os(PREFLIGHT_PROBE_CASE_ENV) else {
        for case in [
            "append",
            "commonest",
            "delete-duplicates",
            "delete-missing",
            "diff",
            "parse-args",
            "range",
            "union",
            "url-parse",
        ] {
            run_probe_subprocess(case);
        }
        return;
    };
    let case = case.to_string_lossy();

    run_probe_evaluation(&case, "low", LOW_CEILING_ALLOWANCE);
    run_probe_evaluation(&case, "realistic", realistic_allowance(&case));
}

fn run_probe_subprocess(case: &str) {
    let current_exe = std::env::current_exe().unwrap();
    let test_name = format!(
        "{}::projected_preflight_builtins_refuse_before_allocation_and_admit_realistic_inputs",
        module_path!()
            .strip_prefix("rum_stdlib::")
            .unwrap_or(module_path!())
    );
    let output = std::process::Command::new(current_exe)
        .arg("--exact")
        .arg(test_name)
        .arg("--test-threads=1")
        .env(PREFLIGHT_PROBE_CASE_ENV, case)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "preflight probe {case} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_probe_evaluation(case: &str, phase: &str, allowance: usize) {
    let mut session = Session::new();
    install_core(&mut session);
    session.register_builtin(BuiltinSpec::ordinary(
        "Brief420PreflightProbe",
        "Exercises stdlib Preflight projection admission in an isolated process.",
        eval_preflight_probe,
        MemoryGrowthContract::Delegated,
    ));
    session.set_generated_element_limit(1_000_000);

    let outcome = session
        .eval_text(&format!(
            "MemoryConstrained[Brief420PreflightProbe[\"{phase}\"], MemoryInUse[] + {allowance}, blocked]"
        ))
        .unwrap();
    assert_eq!(
        outcome.value,
        system_expr("True"),
        "{case} {phase} probe should complete inside its ceiling"
    );
}

fn eval_preflight_probe(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let [phase] = args.as_ref() else {
        panic!("preflight probe expects one phase argument")
    };
    let phase = expr_as_string(phase).expect("preflight probe phase should be a string");
    let low = match phase {
        "low" => true,
        "realistic" => false,
        other => panic!("unknown preflight probe phase {other}"),
    };
    let case = std::env::var(PREFLIGHT_PROBE_CASE_ENV).unwrap();

    match case.as_str() {
        "append" => {
            let input = one_string_list(payload_size(low));
            let call_args = SharedArgs::new(vec![input, Expr::small_integer(1)]);
            let head = system_expr("Append");
            check_projection_result(low, || {
                structure_ops::eval_append(session, &head, &call_args)
            })?;
        }
        "commonest" => {
            let input = one_string_list(payload_size(low));
            let call_args = SharedArgs::new(vec![input]);
            let head = system_expr("Commonest");
            check_projection_result(low, || {
                arithmetic::eval_commonest(session, &head, &call_args)
            })?;
            if !low {
                let wide_input = Expr::call(
                    system_expr("List"),
                    (0..5_000).map(Expr::small_integer).collect(),
                );
                let wide_args = SharedArgs::new(vec![wide_input]);
                check_projection_result(false, || {
                    arithmetic::eval_commonest(session, &head, &wide_args)
                })?;
            }
        }
        "delete-duplicates" => {
            let input = one_string_list(payload_size(low));
            let call_args = SharedArgs::new(vec![input]);
            let head = system_expr("DeleteDuplicates");
            check_projection_result(low, || {
                structure_ops::eval_delete_duplicates(session, &head, &call_args)
            })?;
        }
        "delete-missing" => {
            let input = one_string_list(payload_size(low));
            let call_args = SharedArgs::new(vec![input]);
            let head = system_expr("DeleteMissing");
            check_projection_result(low, || {
                associations::eval_delete_missing(session, &head, &call_args)
            })?;
        }
        "diff" => {
            let len = if low { 2_200 } else { 128 };
            let call_args = SharedArgs::new(vec![
                Expr::string("a".repeat(len)),
                Expr::string("b".repeat(len)),
            ]);
            let head = system_expr("Diff");
            check_projection_result(low, || strings::eval_diff(session, &head, &call_args))?;
        }
        "parse-args" => {
            let flag_count = if low { 400_000 } else { 4_096 };
            let item = Expr::string(format!("-{}", "a".repeat(flag_count)));
            let items = Expr::call(system_expr("List"), vec![item]);
            let option = Expr::call(
                system_expr("Rule"),
                vec![Expr::string("ShortOptions"), system_expr("True")],
            );
            let call_args = SharedArgs::new(vec![items, option]);
            let head = system_expr("ParseArgs");
            check_projection_result(low, || strings::eval_parse_args(session, &head, &call_args))?;
        }
        "range" => {
            let count = if low { 500_000 } else { 8_192 };
            let call_args = SharedArgs::new(vec![Expr::integer(BigInt::from(count))]);
            let head = system_expr("Range");
            check_projection_result(low, || {
                structure_ops::eval_range(session, &head, &call_args)
            })?;
        }
        "union" => {
            let input = one_string_list(payload_size(low));
            let call_args = SharedArgs::new(vec![input]);
            let head = system_expr("Union");
            check_projection_result(low, || {
                structure_ops::eval_union(session, &head, &call_args)
            })?;
        }
        "url-parse" => {
            let head = system_expr("UrlParse");
            if low {
                let default_scheme = Expr::call(
                    system_expr("Rule"),
                    vec![
                        Expr::string("DefaultScheme"),
                        Expr::string("s".repeat(payload_size(true))),
                    ],
                );
                let call_args = SharedArgs::new(vec![Expr::string("example.test"), default_scheme]);
                check_projection_result(true, || {
                    strings::eval_url_parse(session, &head, &call_args)
                })?;

                let url = format!("https://example.test/{}", "x".repeat(payload_size(true)));
                let unused_default_scheme = Expr::call(
                    system_expr("Rule"),
                    vec![
                        Expr::string("DefaultScheme"),
                        Expr::string("s".repeat(payload_size(true))),
                    ],
                );
                let call_args = SharedArgs::new(vec![Expr::string(url), unused_default_scheme]);
                check_projection_result(true, || {
                    strings::eval_url_parse(session, &head, &call_args)
                })?;
            } else {
                let url = format!("https://example.test/{}", "x".repeat(payload_size(false)));
                let call_args = SharedArgs::new(vec![Expr::string(url)]);
                check_projection_result(false, || {
                    strings::eval_url_parse(session, &head, &call_args)
                })?;

                let default_scheme = Expr::call(
                    system_expr("Rule"),
                    vec![Expr::string("DefaultScheme"), Expr::string("https")],
                );
                let schemeless = format!("example.test/{}", "x".repeat(128 * 1024));
                let call_args = SharedArgs::new(vec![Expr::string(schemeless), default_scheme]);
                check_projection_result(false, || {
                    strings::eval_url_parse(session, &head, &call_args)
                })?;
            }
        }
        other => panic!("unknown stdlib preflight probe case {other}"),
    }

    Ok(system_expr("True"))
}

fn one_string_list(bytes: usize) -> Expr {
    Expr::call(system_expr("List"), vec![Expr::string("x".repeat(bytes))])
}

fn payload_size(low: bool) -> usize {
    if low { 8 * 1024 * 1024 } else { 256 * 1024 }
}

fn realistic_allowance(case: &str) -> usize {
    match case {
        "diff" => 380_000,
        "parse-args" => 190_000,
        _ => 340_000,
    }
}

fn check_projection_result(
    low: bool,
    evaluate: impl FnOnce() -> Result<Expr, EvalError>,
) -> Result<(), EvalError> {
    let before = linux_process_high_water_bytes();
    let result = evaluate();
    let after = linux_process_high_water_bytes();

    if low {
        assert!(
            matches!(result, Err(EvalError::MemoryConstraintExceeded { .. })),
            "low ceiling should reject at the projection choke point"
        );
        if let Some((before, after)) = before.zip(after) {
            assert!(
                after.saturating_sub(before) <= MAX_REFUSAL_HWM_GROWTH,
                "large allocation happened before refusal: before={before}, after={after}"
            );
        }
    } else {
        result.expect("realistic input should fit a tightly sized ceiling");
    }

    Ok(())
}

fn linux_process_high_water_bytes() -> Option<usize> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let kib = status.lines().find_map(|line| {
        line.strip_prefix("VmHWM:")?
            .split_whitespace()
            .next()?
            .parse::<usize>()
            .ok()
    })?;
    Some(kib.saturating_mul(1024))
}
