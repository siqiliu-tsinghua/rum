//! Source census and earned typing receipts. S1 deliberately refuses preflip.
use std::{fs, process::Command};

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
#[path = "typed_slot_census/receipts.rs"]
mod receipts;
#[path = "typed_slot_census/reporting.rs"]
mod reporting;
use discovery::{discover, feature, root};
use receipts::{baseline, preflip};
use reporting::emit;
use scan::Population;
#[path = "suspension_oracle/fixtures/typed_slots.rs"]
mod stage;

#[test]
fn source_baseline_matches_earned_receipt() {
    let root = root();
    let (_, facts, report) = discover(&root);
    emit(&root, &facts, &report);
    baseline(&facts, &report).unwrap_or_else(|error| panic!("{error}"));
}
#[test]
fn preflip_no_legacy_transport_uses() {
    let (_, facts, report) = discover(&root());
    if let Some(slot) = stage::FLIP_SLOT {
        preflip(&facts, &report, slot).unwrap();
    } else {
        assert_eq!(stage::TYPING_RECEIPT, "S1");
        let error = preflip(&facts, &report, "Option<Expr>").unwrap_err();
        assert_eq!(
            error,
            "preflip refused: S54 completed receipt is required first"
        );
        println!("S1 negative control: {error}; no zero-legacy claim");
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
    let (files, facts, report) = discover(&root);
    baseline(&facts, &report).unwrap();
    let scratch = root.join(".temp/brief-607/census/forced-red/tree");
    for (file, source) in &files {
        let p = scratch.join(file);
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
    fs::write(scratch.join(&site.file), changed).unwrap();
    let output = Command::new("python3")
        .arg(root.join("scripts/run-bounded.py"))
        .args(["--timeout", "900", "--"])
        .arg(std::env::current_exe().unwrap())
        .args(["--exact", "forced_red_probe", "--ignored", "--nocapture"])
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let kind = if remove_guard { "guard" } else { "frame" };
    fs::write(
        root.join(format!(
            ".temp/brief-607/census/forced-red/{}-{kind}.log",
            feature()
        )),
        &log,
    )
    .unwrap();
    let marker = if remove_guard {
        "production frames=564 finalizers=59 implementations=623"
    } else {
        "S1 PARTITION implementations=622"
    };
    assert!(
        !output.status.success() && log.contains(marker) && log.contains("S1 baseline requires"),
        "{log}"
    );
    println!("FORCED RED removed {kind} at {}:\n{log}", site.fields());
}
#[test]
#[ignore = "invoked explicitly by both bounded scratch-copy falsifiers"]
fn forced_red_probe() {
    let (_, facts, report) = discover(&root().join(".temp/brief-607/census/forced-red/tree"));
    baseline(&facts, &report).unwrap();
}
