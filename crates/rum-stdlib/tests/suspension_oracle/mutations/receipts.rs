//! Imports a complete artifact index, re-authenticates builds, then checks logs.
use super::{
    records::{self, Records, Rejection},
    schema, source_auth,
};
use crate::oracle::obligations::{bindings, content_digest};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

fn read_pinned(path: &Path, hash: &str) -> Result<Vec<u8>, Rejection> {
    let bytes = std::fs::read(path).map_err(|_| Rejection::RawArtifact)?;
    if content_digest(&bytes) != hash {
        return Err(Rejection::RawArtifact);
    }
    Ok(bytes)
}

fn artifact_path(root: &Path, text: &str) -> Result<PathBuf, Rejection> {
    let path = PathBuf::from(text);
    let full = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    let resolved = full.canonicalize().map_err(|_| Rejection::RawArtifact)?;
    if !super::source_auth::admitted_scratch(root, &resolved) {
        return Err(Rejection::RawArtifact);
    }
    Ok(resolved)
}

pub(super) struct ArtifactIndex {
    pub builds: BTreeMap<(String, String), source_auth::artifacts::ArtifactIdentity>,
    pub runs: BTreeMap<(String, String, String), (PathBuf, String)>,
}

/// Only the authenticated artifact-import path below constructs this receipt.
/// The pure predicate falsifiers have no constructor for it.
#[derive(Debug)]
struct MutationReceipt {
    row: String,
    tuple: String,
    source_key: String,
    tail_digest: String,
    carrier: String,
    phases: Vec<String>,
    selector: String,
    observer: String,
    builds: Vec<(String, String, String)>,
    raw_artifacts: Vec<(String, String)>,
}

impl MutationReceipt {
    fn assert_binding(&self, spec: &schema::MutationSpec, tuple: &str) {
        assert_eq!(self.row, spec.row);
        assert_eq!(self.tuple, tuple);
        assert_eq!(self.source_key, spec.source_key);
        assert_eq!(self.tail_digest, spec.tail_digest);
        assert_eq!(self.carrier, spec.carrier);
        assert_eq!(self.selector, spec.selector);
        assert_eq!(self.observer, spec.observer);
        assert!(
            !self.phases.is_empty()
                && self
                    .phases
                    .iter()
                    .all(|p| spec.phases.contains(&p.as_str()))
        );
        assert_eq!(self.builds.len(), 3);
        assert_eq!(self.raw_artifacts.len(), 3);
    }
}

pub(super) fn read_index(index: &str) -> ArtifactIndex {
    let root = crate::oracle::coverage::source::workspace();
    let baseline = std::env::var_os("RUM_CM740_BASELINE")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.clone());
    let root = baseline.canonicalize().unwrap();
    let index = artifact_path(&root, &index).unwrap();
    let text = std::fs::read_to_string(index).unwrap();
    let mut lines = text.lines();
    assert_eq!(
        lines.next(),
        Some(
            "kind\tconfiguration\tarm\trow\tsource\tbinary\tauthentication\tauthentication_sha256\tbinary_sha256\tlog\tlog_sha256\tspot_binary"
        )
    );
    let mut builds = BTreeMap::new();
    let mut runs = BTreeMap::new();
    for line in lines {
        let c: Vec<_> = line.split('\t').collect();
        assert_eq!(c.len(), 12);
        match c[0] {
            "build" => {
                assert_eq!(c[3], "-");
                let source = artifact_path(&root, c[4]).unwrap();
                let binary = artifact_path(&root, c[5]).unwrap();
                let auth = artifact_path(&root, c[6]).unwrap();
                read_pinned(&auth, c[7]).unwrap();
                read_pinned(&binary, c[8]).unwrap();
                let identity =
                    source_auth::artifacts::artifact(&baseline, &source, &binary, &auth, c[2])
                        .unwrap();
                assert_eq!(identity.configuration(), c[1]);
                if c[2] == "real" {
                    read_pinned(&artifact_path(&root, c[11]).unwrap(), identity.binary()).unwrap();
                } else {
                    assert_eq!(c[11], "-");
                }
                assert!(
                    builds
                        .insert((c[1].to_string(), c[2].to_string()), identity)
                        .is_none()
                );
            }
            "run" => {
                let path = artifact_path(&root, c[9]).unwrap();
                let bytes = read_pinned(&path, c[10]).unwrap();
                let raw = String::from_utf8(bytes).unwrap();
                assert!(
                    runs.insert(
                        (c[1].to_string(), c[2].to_string(), c[3].to_string()),
                        (path, raw)
                    )
                    .is_none()
                );
            }
            _ => panic!("unknown packet record"),
        }
    }
    ArtifactIndex { builds, runs }
}

/// Import is deliberately all-or-nothing for each row. Missing, invalid or
/// surviving receipts produce PortBlocked; a declaration has no conversion.
#[test]
fn cm_own_mutant_receipts_exact() {
    let specs = schema::specs();
    let cases = schema::cases();
    let Ok(index) = std::env::var("RUM_CM740_PACKETS") else {
        println!(
            "CM4 OWN RECEIPT INVENTORY PortBlocked=151 reason=final_schema_copied_packets_not_supplied caught=0 preliminary_receipts_reused=false"
        );
        return;
    };
    let ArtifactIndex { builds, runs } = read_index(&index);
    let programs = crate::oracle::fixtures::obligations::guest::programs().unwrap();
    let program_ids: BTreeMap<_, _> = programs
        .values()
        .map(|p| {
            let mut ids = Vec::new();
            let mut current = Some(p);
            while let Some(p) = current {
                ids.push(p.id.clone());
                current = p.paired.as_deref();
            }
            (p.key.line(), ids)
        })
        .collect();
    let required = bindings::ActiveRequiredBag::committed().unwrap();
    let configuration = if cfg!(feature = "hash") {
        "default"
    } else {
        "no-default"
    };
    let mut caught = 0;
    let mut blocked = Vec::new();
    for spec in specs.values() {
        let config = if spec.row == "P0543" {
            "p0543-unchecked"
        } else {
            configuration
        };
        let mut records = Vec::new();
        let mut rejection = None;
        for arm in ["real", "disabled", "armed"] {
            let Some(identity) = builds.get(&(config.into(), arm.into())) else {
                rejection = Some(Rejection::BuildIdentity);
                break;
            };
            let Some((_, raw)) = runs.get(&(config.into(), arm.into(), spec.row.into())) else {
                rejection = Some(Rejection::RawArtifact);
                break;
            };
            match Records::parse(raw) {
                Ok(record) if record.binary == identity.binary() => {
                    if let Err(e) = records::validate_constituents(&record, &program_ids) {
                        rejection = Some(e);
                        break;
                    }
                    records.push(record)
                }
                Ok(_) => {
                    rejection = Some(Rejection::BuildIdentity);
                    break;
                }
                Err(e) => {
                    rejection = Some(e);
                    break;
                }
            }
        }
        if let Some(reason) = rejection {
            blocked.push((spec.row, reason));
            continue;
        }
        let real_keys: BTreeSet<_> = required
            .keys
            .iter()
            .filter(|k| k.row() == spec.row)
            .map(|k| k.line())
            .collect();
        let selected = cases
            .iter()
            .find(|c| c.kind == "own" && c.row == spec.row)
            .unwrap();
        match records::verify_own_omission(
            spec,
            &real_keys,
            &selected.tuple,
            &records[0],
            &records[1],
            &records[2],
        ) {
            Ok(receipt) => {
                let actual = MutationReceipt {
                    row: spec.row.into(),
                    tuple: selected.tuple.clone(),
                    source_key: spec.source_key.into(),
                    tail_digest: spec.tail_digest.into(),
                    carrier: spec.carrier.into(),
                    phases: records[2].hits.iter().map(|h| h.1.clone()).collect(),
                    selector: spec.selector.into(),
                    observer: spec.observer.into(),
                    builds: ["real", "disabled", "armed"]
                        .into_iter()
                        .map(|arm| {
                            let id = &builds[&(config.into(), arm.into())];
                            (arm.into(), id.binary().into(), id.tree().into())
                        })
                        .collect(),
                    raw_artifacts: ["real", "disabled", "armed"]
                        .into_iter()
                        .map(|arm| {
                            let (path, raw) = &runs[&(config.into(), arm.into(), spec.row.into())];
                            (path.display().to_string(), content_digest(raw.as_bytes()))
                        })
                        .collect(),
                };
                actual.assert_binding(spec, &selected.tuple);
                println!("CM740_AUTHENTICATED_MUTATION_RECEIPT {actual:?}");
                caught += 1;
                println!(
                    "CM4 OWN OMISSION RECEIPT PASS row={} real_keys={} hits={} configuration={config} disabled_record_exact=true raw_builds_authenticated=true",
                    receipt.row(),
                    receipt.real_keys(),
                    receipt.hits()
                );
            }
            Err(reason) => blocked.push((spec.row, reason)),
        }
    }
    for (row, reason) in &blocked {
        println!("CM4 OWN OMISSION PortBlocked row={row} missing_evidence={reason:?} caught=false");
    }
    println!(
        "CM4 OWN RECEIPT RECONCILIATION configuration={configuration} required=151 caught={caught} PortBlocked={} settlements=0",
        blocked.len()
    );
    assert_eq!(caught + blocked.len(), 151);
    assert!(blocked.is_empty(), "final own-omission gate remains red");
}
