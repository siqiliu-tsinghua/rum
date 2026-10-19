//! LP01 counter context. The frozen semantic oracle remains fully active.
mod plans;
use super::super::{
    descriptors::{self, Checks},
    support::{faults, logical_profile as lp, observe},
};
use std::{collections::BTreeMap, fs, process::Command};

const REQUIRED: &str = include_str!("../../inventory/logical-profile-required.tsv");
const BINDINGS: &str = include_str!("../../inventory/logical-profile-bindings.tsv");
fn scratch() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.temp/brief-625")
}

#[test]
fn logical_counter_port() {
    if let Ok(mode) = std::env::var("RUM_LP01_CHILD") {
        return child(&mode);
    }
    let executable = std::env::current_exe().unwrap();
    for mode in ["disabled", "lexical", "timing", "cache", "all"] {
        let path = scratch().join(format!("logical-{}-{mode}.log", cfg!(feature = "hash")));
        let mut command = Command::new(&executable);
        command.args([
            "--exact",
            "oracle::fixtures::logical_profile::logical_counter_port",
            "--nocapture",
            "--test-threads=1",
        ]);
        for (name, _) in std::env::vars().filter(|(k, _)| k.starts_with("RUM_PROFILE_")) {
            command.env_remove(name);
        }
        command
            .env("RUM_LP01_CHILD", mode)
            .env("RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL", "0");
        if mode != "disabled" {
            command.env("RUM_PROFILE_LEXICAL_SUBSTITUTION", "1");
        }
        if matches!(mode, "timing" | "all") {
            for name in [
                "BUILTIN_TIMING",
                "DEFINITION_RULE_TIMING",
                "RULE_RHS_TIMING",
            ] {
                command.env(format!("RUM_PROFILE_{name}"), "1");
            }
        }
        if matches!(mode, "cache" | "all") {
            command.env("RUM_PROFILE_EVALUATION_CACHE", "1");
        }
        if mode == "all" {
            for name in [
                "DETAIL",
                "SHAPES",
                "BINDINGS",
                "SOURCES",
                "SOURCE_FINGERPRINTS",
                "PROVENANCE",
            ] {
                command.env(format!("RUM_PROFILE_LEXICAL_{name}"), "1");
            }
        }
        let log = fs::File::create(&path).unwrap();
        let status = command
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .status()
            .unwrap();
        let text = fs::read_to_string(&path).unwrap();
        for line in text.lines().filter(|l| {
            l.starts_with("LP01 ")
                || l.starts_with("test result:")
                || l.starts_with("continuation-slot fault-occurrence")
        }) {
            println!("{mode}: {line}");
        }
        assert!(
            status.success(),
            "LP01 {mode}: {}\n{}",
            path.display(),
            text.lines().rev().take(12).collect::<Vec<_>>().join("\n")
        );
    }
}

fn child(mode: &str) {
    let plans = plans::plans();
    let required = lp::required(REQUIRED).unwrap();
    let bindings = lp::rows(BINDINGS, 6).unwrap();
    assert_eq!(required.len(), 1115);
    assert_eq!(bindings.len(), 875);
    let active = required.iter().filter(|r| r[20] == "-").collect::<Vec<_>>();
    assert_eq!(required.len() - active.len(), 240);
    lp::exact(
        &active.iter().map(|r| r[16].to_string()).collect::<Vec<_>>(),
        &bindings
            .iter()
            .map(|b| b[0].to_string())
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for b in &bindings {
        let r = active.iter().find(|r| r[16] == b[0]).unwrap();
        assert_eq!((r[17], r[18], r[21]), (b[3], b[4], b[2]));
        let file = r[17].rsplit_once(':').unwrap().0;
        let source = fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(file),
        )
        .unwrap();
        assert_eq!(
            lp::fingerprint(&[file, &source]),
            r[19],
            "stale producer {file}"
        );
    }
    let frozen_hashes = lp::hashes_by_program(&bindings);
    let all_hashes = lp::rows(include_str!("fingerprints.tsv"), 2)
        .unwrap()
        .into_iter()
        .map(|r| (r[0], r[1]))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(all_hashes.len(), plans.len());
    let mut actual = Vec::new();
    let mut completed = BTreeMap::new();
    let mut groups = BTreeMap::new();
    for p in &plans {
        let hash = plans::fixture_fingerprint(&p.fixture);
        assert_eq!(hash, all_hashes[p.id.as_str()], "unfrozen program {}", p.id);
        if let Some(expected) = frozen_hashes.get(&p.id) {
            assert_eq!(&hash, expected, "source drift {}", p.id);
        }
        let q = descriptors::run(&p.fixture, true, Checks::ALL)
            .unwrap_or_else(|e| panic!("{} queued {e:?}", p.id));
        let r = descriptors::run(&p.fixture, false, Checks::ALL)
            .unwrap_or_else(|e| panic!("{} resumed {e:?}", p.id));
        observe::compare(&q.observation, &r.observation).unwrap_or_else(|e| {
            panic!(
                "STOP semantic difference {} {e:?}: queued={:?}; resumed={:?}",
                p.id, q.observation, r.observation
            )
        });
        assert_eq!(q.log, r.log, "{} effects", p.id);
        assert_eq!(q.projections, r.projections, "{} projections", p.id);
        if let Some(value) = &p.fixture.expected_value {
            assert_eq!(
                r.observation
                    .value
                    .as_ref()
                    .map(ToString::to_string)
                    .as_ref(),
                Some(value),
                "{} frozen value",
                p.id
            );
        }
        if matches!(mode, "timing" | "all") {
            plans::prove_new(p, &q);
        }
        let (q, r) = (q.profile.unwrap(), r.profile.unwrap());
        if mode == "disabled" {
            assert!(q.maps.values().all(|n| *n == 0));
            assert!(r.maps.values().all(|n| *n == 0));
        }
        for (delivery, s) in [("queued", &q), ("resumed", &r)] {
            actual.push(lp::key_record(mode, &p.id, delivery, &hash, s));
        }
        if completed.is_empty() {
            let original = lp::key_record(mode, &p.id, "queued", &hash, &q);
            let mut dropped = q.clone();
            dropped.logical.pop_first().unwrap();
            assert!(
                lp::exact(
                    &[original],
                    &[lp::key_record(mode, &p.id, "queued", &hash, &dropped)]
                )
                .is_err()
            );
            println!("LP01 CONTROL dropped-key RED");
        }
        let attributed = bindings
            .iter()
            .filter(|b| b[1].split(',').any(|id| id == p.id))
            .collect::<Vec<_>>();
        for b in &attributed {
            if matches!(mode, "timing" | "all") {
                assert!(
                    q.logical
                        .keys()
                        .any(|k| k.starts_with(&format!("{}|", b[4]))),
                    "{} missing designated phase {}",
                    p.id,
                    b[4]
                );
            }
        }
        let diffs = lp::differences(&r, &q);
        for (key, old, new) in diffs {
            if attributed.is_empty() {
                let producer = required
                    .iter()
                    .find(|r| r[0] == p.row)
                    .map(|r| r[17])
                    .unwrap_or("crates/rum-runtime/src/runtime/resolution.rs:399");
                actual.push(
                    lp::loss_record(mode, &p.id, &key, producer, &p.group, &old, &new).unwrap(),
                );
            } else {
                for b in &attributed {
                    let requirement = active.iter().find(|r| r[16] == b[0]).unwrap();
                    let route = format!(
                        "{}|{}|{}|{}|{}",
                        requirement[16],
                        requirement[0],
                        requirement[4],
                        requirement[5],
                        requirement[2]
                    );
                    actual.push(
                        lp::loss_record(mode, &p.id, &key, b[3], &route, &old, &new).unwrap(),
                    );
                }
            }
        }
        completed.insert(p.id.clone(), hash);
        *groups.entry(p.group.as_str()).or_insert(0usize) += 1;
        println!("LP01 EXECUTED {} {mode}", p.id);
    }
    let mut receipts = Vec::new();
    for b in &bindings {
        if b[1].split(',').all(|id| completed.contains_key(id)) {
            let r = active.iter().find(|r| r[16] == b[0]).unwrap();
            receipts.push(lp::record(
                "receipt",
                &[
                    mode,
                    r[0],
                    r[2],
                    r[4],
                    r[5],
                    &b[1]
                        .split(',')
                        .map(|id| completed[id].as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                ],
            ));
        }
    }
    let tag = format!("{}-{mode}", cfg!(feature = "hash"));
    actual.sort();
    fs::write(
        scratch().join(format!("actual-{tag}.json")),
        format!("[\n{}\n]\n", actual.join(",\n")),
    )
    .unwrap();
    fs::write(
        scratch().join(format!("executed-{tag}.json")),
        format!(
            "{{\"build_hash\":{},\"flag_mode\":{},\"receipts\":[\n{}\n]}}\n",
            cfg!(feature = "hash"),
            lp::json(mode),
            receipts.join(",\n")
        ),
    )
    .unwrap();
    let mut absent = receipts.clone();
    absent.pop();
    assert!(lp::exact(&receipts, &absent).is_err());
    println!("LP01 CONTROL absent-execution RED");
    faults::assert_all();
    println!(
        "LP01 required=1115 bound={} executed={} encoding-owned=240 programs={groups:?} counter-losses={}",
        bindings.len(),
        receipts.len(),
        actual.iter().filter(|r| r.starts_with("[\"loss\"")).count()
    );
    assert_eq!(receipts.len(), 875);
    lp::exact(&lp::expected_receipts(mode), &receipts).unwrap();
    lp::exact(&lp::expected_for(mode), &actual).unwrap();
}

#[test]
fn collection_negative_controls() {
    let r = lp::required(REQUIRED).unwrap();
    let mut malformed = r[0].clone();
    malformed.pop();
    assert!(lp::required(&malformed.join("\t")).is_err());
    println!("LP01 CONTROL malformed-requirement RED");
    let duplicated = format!("{}\n{}", REQUIRED.trim_end(), r[0].join("\t"));
    assert!(lp::required(&duplicated).is_err());
    println!("LP01 CONTROL duplicate-tuple RED");
    let mut f = descriptors::Fixture::new(
        "LP01.missing-second",
        "InputString[\"one\"];Throw[0];InputString[\"two\"]",
    );
    f.pauses = ["one", "two"]
        .into_iter()
        .map(|prompt| descriptors::Pause {
            prompt: prompt.into(),
            kind: rum_runtime::RuntimeInputKind::InputString,
            answer: "x".into(),
        })
        .collect();
    assert!(matches!(
        descriptors::run(&f, false, Checks::ALL),
        Err(observe::Rejection::PauseCount {
            expected: 2,
            actual: 1
        })
    ));
    println!("LP01 CONTROL missing-second-pause RED");
}
