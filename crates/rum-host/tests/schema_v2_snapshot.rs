//! Guest channel and byte-preserved legacy diagnostics (profiling.md schema v2).
use rum_host::{HostEvalOutcome, HostSession};
use rum_runtime::kernel_api::Session;
use std::process::{Command, Output};

fn run_child(group: &str, mode: &str, case: &str, deferred: bool, base: bool) -> Output {
    let binary = if base {
        std::env::var_os("RUM_SCHEMA_BASE_BINARY").expect("qualified base binary")
    } else {
        std::env::current_exe().unwrap().into_os_string()
    };
    let mut command = Command::new(binary);
    command.args([
        "schema_v2_child",
        "--exact",
        "--nocapture",
        "--test-threads=1",
    ]);
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("RUM_PROFILE_") {
            command.env_remove(key);
        }
    }
    command
        .env("RUM_SCHEMA_CHILD", case)
        .env("RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL", "0");
    if mode != "disabled" {
        command.env("RUM_PROFILE_LEXICAL_SUBSTITUTION", "1");
    }
    if matches!(mode, "timing" | "all") {
        for flag in [
            "BUILTIN_TIMING",
            "RULE_RHS_TIMING",
            "DEFINITION_RULE_TIMING",
        ] {
            command.env(format!("RUM_PROFILE_{flag}"), "1");
        }
    }
    if matches!(mode, "cache" | "all") {
        command.env("RUM_PROFILE_EVALUATION_CACHE", "1");
    }
    if mode == "all" {
        for flag in [
            "DETAIL",
            "SHAPES",
            "BINDINGS",
            "SOURCES",
            "SOURCE_FINGERPRINTS",
            "PROVENANCE",
        ] {
            command.env(format!("RUM_PROFILE_LEXICAL_{flag}"), "1");
        }
    }
    if deferred {
        command
            .env("RUM_PROFILE_LEXICAL_DEFER", "1")
            .env("RUM_PROFILE_EVALUATION_CACHE_DEFER", "1");
    }
    let output = command.output().unwrap();
    // The acceptance runner supplies a scratch directory. Preserve raw bytes
    // before checking them; ordinary workspace runs need no external artifact.
    if let Some(root) = std::env::var_os("RUM_SCHEMA_RECEIPTS") {
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        let stem = format!("{group}-{mode}-{case}-deferred{deferred}-base{base}");
        std::fs::write(root.join(format!("{stem}.stdout")), &output.stdout).unwrap();
        std::fs::write(root.join(format!("{stem}.stderr")), &output.stderr).unwrap();
    }
    assert!(
        output.status.success(),
        "mode={mode} case={case}\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn schema_lines(output: &Output) -> Vec<&str> {
    std::str::from_utf8(&output.stderr)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with("rum-profile schema-v2 "))
        .collect()
}

#[test]
fn schema_v2_child() {
    let Ok(case) = std::env::var("RUM_SCHEMA_CHILD") else {
        return;
    };
    let _lease = Session::logical_profile_test_acquire();
    let mut session = HostSession::new();
    let source = match case.as_str() {
        "noarg" => "RumProfileSnapshot[]",
        "string" => "RumProfileSnapshot[\"host-label\"]",
        "nonstring" => "RumProfileSnapshot[7]",
        "empty" => "RumProfileSnapshot[\"\"]",
        "escaping" => "RumProfileSnapshot[\"a\\\"b\\\\c\\nd\\re\\tf\"]",
        "arity" => "RumProfileSnapshot[1,2]",
        _ => panic!("unknown case"),
    };
    let HostEvalOutcome::Success(success) = session.eval(source) else {
        panic!("snapshot did not complete")
    };
    if case == "arity" {
        assert_eq!(success.value.text, "RumProfileSnapshot[1, 2]");
        assert!(!success.messages.is_empty());
    } else {
        assert_eq!(success.value.text, "Null");
        assert!(success.messages.is_empty());
    }
    println!("GUEST_RESULT case={case} value={}", success.value.text);
}

#[test]
fn schema_v2_five_modes_return_null_and_one_line() {
    for mode in ["disabled", "lexical", "timing", "cache", "all"] {
        let output = run_child("modes", mode, "noarg", false, false);
        let lines = schema_lines(&output);
        assert_eq!(lines.len(), usize::from(mode != "disabled"));
        if mode == "disabled" {
            assert!(
                !output
                    .stderr
                    .windows(b"rum-profile".len())
                    .any(|part| part == b"rum-profile")
            );
        }
        for line in lines {
            println!("HOST_SCHEMA mode={mode} {line}");
        }
        println!(
            "Host schema channel verdict: PASS mode={mode} return=Null configured_line_count_exact=true"
        );
    }
}

#[test]
fn schema_v2_labels_and_arity() {
    for (case, label) in [
        ("noarg", "snapshot"),
        ("string", "host-label"),
        ("nonstring", "snapshot"),
        ("empty", ""),
        ("escaping", "a\\\"b\\\\c\\nd\\re\\tf"),
        ("arity", ""),
    ] {
        let output = run_child("labels", "lexical", case, false, false);
        let lines = schema_lines(&output);
        if case == "arity" {
            assert!(lines.is_empty());
        } else {
            assert_eq!(lines.len(), 1);
            assert!(
                lines[0].contains(&format!("\"label\",\"{label}\"")),
                "{}",
                lines[0]
            );
        }
    }
    println!(
        "Host schema labels verdict: PASS noarg=true string=true nonstring=true empty=true escaping=true excess_arity_unchanged=true"
    );
}

#[test]
fn schema_v2_deferred_and_legacy_bytes() {
    let compare_base = std::env::var_os("RUM_SCHEMA_BASE_BINARY").is_some();
    for mode in ["disabled", "lexical", "timing", "cache", "all"] {
        for case in ["noarg", "string", "nonstring", "empty", "escaping", "arity"] {
            // Deferred collection fixes every legacy counter/duration at zero.
            // This permits an exact byte comparison, without normalizing time,
            // ids, labels, whitespace or any existing diagnostic field.
            let candidate = run_child("legacy", mode, case, true, false);
            let lines = schema_lines(&candidate);
            assert_eq!(
                lines.len(),
                usize::from(mode != "disabled" && case != "arity")
            );
            for line in lines {
                assert!(line.contains("\"epoch\",Null,\"validity\",\"absent\",\"mode\",\"none\""));
            }
            if compare_base {
                let base = run_child("legacy", mode, case, true, true);
                assert!(schema_lines(&base).is_empty());
                let legacy = candidate
                    .stderr
                    .split_inclusive(|byte| *byte == b'\n')
                    .filter(|line| !line.starts_with(b"rum-profile schema-v2 "))
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>();
                assert_eq!(legacy, base.stderr, "legacy bytes: mode={mode} case={case}");
                println!(
                    "Host legacy byte verdict: PASS mode={mode} case={case} deferred=true byte_identical=true normalization=none"
                );
            }
        }
    }
    println!(
        "Host deferred verdict: PASS validity=absent mode=none base_comparison={compare_base}"
    );
}
