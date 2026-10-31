//! Source census and earned typing receipts. G15-C1 deliberately refuses preflip.
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs, process::Command};

/// Owns one directory for its lifetime. There is no constructor from a path,
/// and dropping it removes the tree.
struct PrivateScratch {
    path: PathBuf,
}

impl PrivateScratch {
    fn create() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = root()
            .join(".temp/typed-slot-census/private")
            .join(format!("{}-{id}", std::process::id()));
        // A leftover tree from a killed run with a recycled pid is removed,
        // never reused.
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for PrivateScratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

const SCRATCH_ENV: &str = "RUM_TYPED_SLOT_FORCED_RED_TREE";

// Reuse the real lexical resolver; the census does not invent a second trait
// namespace or infer identity from a spelling count.
#[allow(dead_code)]
#[path = "../src/tests/ownership_graph/effects.rs"]
mod effects;
#[allow(dead_code)]
#[path = "../src/tests/ownership_graph/evidence.rs"]
mod evidence;
#[allow(dead_code)]
#[path = "../src/tests/ownership_graph/graph.rs"]
mod graph;
#[allow(dead_code)]
#[path = "../src/tests/ownership_graph/model.rs"]
pub(crate) mod model;
#[allow(dead_code)]
#[path = "../src/tests/ownership_graph/resolve.rs"]
mod resolve;
#[allow(dead_code)]
#[path = "../src/tests/ownership_graph/source.rs"]
mod source;
mod tests {
    pub(crate) mod ownership_graph {
        pub(crate) use crate::model;
    }
}

mod typed_slot_census {
    pub mod classify;
    pub mod scan;
}
use typed_slot_census::{classify, scan};
#[path = "typed_slot_census/discovery.rs"]
mod discovery;
#[path = "typed_slot_census/g15a.rs"]
mod g15a;
#[path = "typed_slot_census/g15c1.rs"]
mod g15c1;
#[path = "typed_slot_census/receipts.rs"]
mod receipts;
#[path = "typed_slot_census/reporting.rs"]
mod reporting;
#[path = "typed_slot_census/routes.rs"]
mod routes;
#[allow(dead_code)]
#[path = "typed_slot_census/s3.rs"]
mod s3;
#[path = "typed_slot_census/s4.rs"]
mod s4;
#[path = "typed_slot_census/s5.rs"]
mod s5;
#[path = "typed_slot_census/s6.rs"]
mod s6;
#[path = "typed_slot_census/s7.rs"]
mod s7;
#[path = "typed_slot_census/s8.rs"]
mod s8;
#[path = "typed_slot_census/s9.rs"]
mod s9;
#[path = "typed_slot_census/witnessed.rs"]
mod witnessed;
use discovery::{discover, feature, production, root};
use receipts::{baseline, preflip};
use reporting::emit;
use scan::Population;
#[path = "suspension_oracle/fixtures/typed_slots.rs"]
mod stage;

// Only the census selects a flip gate; the oracle reads the typing receipt.
const FLIP_SLOT: Option<&str> = None;

#[test]
fn source_baseline_matches_earned_receipt() {
    let root = root();
    let (_, facts, report) = production();
    emit(&root, &facts, &report);
    baseline(&facts, &report).unwrap_or_else(|error| panic!("{error}"));
    routes::compatible_driver(&facts).unwrap();
}
#[test]
fn preflip_no_legacy_transport_uses() {
    let (_, facts, report) = production();
    if let Some(slot) = FLIP_SLOT {
        preflip(&facts, &report, slot).unwrap();
    } else {
        assert_eq!(stage::TYPING_RECEIPT, "G15-C1");
        let error = preflip(&facts, &report, "Option<Expr>").unwrap_err();
        assert_eq!(
            error,
            "preflip refused: S54 completed receipt is required first"
        );
        println!("G15-C1 negative control: {error}; no zero-legacy claim");
    }
}

#[test]
fn removing_a_frame_in_a_scratch_copy_is_red() {
    forced_red(false);
}
#[test]
fn removing_a_test_guard_in_a_scratch_copy_is_red() {
    forced_red(true);
}

fn forced_red(remove_guard: bool) {
    let root = root();
    let (files, facts, report) = production();
    baseline(&facts, &report).unwrap();
    let scratch = PrivateScratch::create();
    for (file, source) in files {
        let p = scratch.path().join(file);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, source).unwrap();
    }
    let site = if remove_guard {
        facts
            .guards
            .iter()
            .find(|site| {
                let module = &facts.guard_modules[&site.key()];
                report.frames.iter().any(|frame| {
                    frame.population == Population::TestOnly
                        && frame.identity.starts_with(&format!("{module}::"))
                })
            })
            .expect("cfg guard on the discovered test-only frame cohort")
    } else {
        &report
            .frames
            .iter()
            .find(|frame| {
                frame.population == Population::Production && frame.kind == "SuspensionFrame"
            })
            .unwrap()
            .site
    };
    let source = &files[&site.file];
    let offset = |line: usize, column: usize| {
        source
            .split_inclusive('\n')
            .take(line - 1)
            .map(str::len)
            .sum::<usize>()
            + column
            - 1
    };
    let (start, end) = (
        offset(site.line, site.column),
        offset(site.end_line, site.end_column),
    );
    let mut changed = source.clone();
    changed.replace_range(
        start..end,
        &source[start..end]
            .chars()
            .map(|c| if c == '\n' { '\n' } else { ' ' })
            .collect::<String>(),
    );
    fs::write(scratch.path().join(&site.file), changed).unwrap();
    let output = Command::new("python3")
        .arg(root.join("scripts/run-bounded.py"))
        .args(["--timeout", "900", "--"])
        .arg(std::env::current_exe().unwrap())
        .env(SCRATCH_ENV, scratch.path())
        .args(["--exact", "forced_red_probe", "--ignored", "--nocapture"])
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let kind = if remove_guard { "guard" } else { "frame" };
    let log_path = root.join(format!(
        ".temp/typed-slot-census/forced-red/{}-{kind}.log",
        feature()
    ));
    fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    fs::write(&log_path, &log).unwrap();
    let marker = if remove_guard {
        "production frames=472 finalizers=55 implementations=626"
    } else {
        "G15-C1 PARTITION implementations=625"
    };
    assert!(
        !output.status.success()
            && log.contains(marker)
            && log.contains("G15-C1 baseline requires"),
        "{log}"
    );
    println!("FORCED RED removed {kind} at {}:\n{log}", site.fields());
}
#[test]
#[ignore = "invoked explicitly by both bounded scratch-copy falsifiers"]
fn forced_red_probe() {
    let scratch = std::env::var(SCRATCH_ENV)
        .unwrap_or_else(|_| panic!("forced_red_probe needs {SCRATCH_ENV}"));
    let path = Path::new(&scratch);
    let private = root().join(".temp/typed-slot-census/private");
    assert!(
        path.starts_with(&private) && path != private,
        "forced_red_probe requires a private scratch path: {scratch}"
    );
    let (_, facts, report) = discover(path);
    baseline(&facts, &report).unwrap();
}

#[test]
fn removing_a_witnessed_insertion_in_a_scratch_copy_is_red() {
    forced_red_witness("no-insertion");
}

#[test]
fn witnessing_a_different_binding_in_a_scratch_copy_is_red() {
    forced_red_witness("wrong-binding");
}

#[test]
fn shadowing_a_witness_in_a_cold_helper_is_red() {
    forced_red_witness("shadowed-witness");
}

fn forced_red_witness(kind: &str) {
    let root = root();
    let (files, facts, report) = production();
    baseline(&facts, &report).unwrap();
    let scratch = root.join(format!(".temp/typed-slot-census/forced-red/{kind}"));
    for (file, source) in files {
        let path = scratch.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    let path = scratch.join("crates/rum-runtime/src/runtime/definitions/application/rhs.rs");
    let source = fs::read_to_string(&path).unwrap();
    let (before, after) = if kind == "wrong-binding" {
        (
            "let witness = slot_witness(&rhs_result);",
            "let unrelated = Ok(Expr::null()); let witness = slot_witness(&unrelated);",
        )
    } else if kind == "shadowed-witness" {
        (
            "witness.suspend(self, request, EvaluateDefinitionRhsSuspensionFrame)",
            "let unrelated = Ok(Expr::null()); let witness = slot_witness(&unrelated); witness.suspend(self, request, EvaluateDefinitionRhsSuspensionFrame)",
        )
    } else {
        assert_eq!(kind, "no-insertion");
        (
            "witness.suspend(self, request, EvaluateDefinitionRhsSuspensionFrame)",
            "Err(EvalError::InputRequired { request })",
        )
    };
    assert!(source.contains(before));
    fs::write(&path, source.replacen(before, after, 1)).unwrap();
    let changed = scan::scan(&scan::files(&scratch));
    let error = s5::transitions(&changed, &report).unwrap_err();
    assert_eq!(error, "M0245: witnessed typed insertion absent");
    let log = format!("FORCED RED {kind}: {error}\n");
    fs::write(scratch.join(format!("{}.log", feature())), &log).unwrap();
    print!("{log}");
}
