//! Five fixture-only audited operations, not production exemptions.
//! FNV-1a is a drift fingerprint, not a cryptographic proof or safety proof.

use std::collections::BTreeSet;

use super::model::*;
use super::{resolve, source};

pub(super) const PREFIX: &str = "fixture::accepted::substrate";

pub(super) fn nominal(id: &str) -> Option<Ty> {
    Some(match id {
        "fixture::accepted::substrate::Accumulator" | "fixture::accepted::substrate::Retained" => {
            Ty::Retained(id.into())
        }
        "fixture::accepted::substrate::Raw" => Ty::Callback(id.into()),
        "fixture::accepted::substrate::Outgoing" => Ty::Outgoing(id.into()),
        _ => return None,
    })
}

// These records were chosen for the fixture protocol, independently of any
// production rejection. TSV data cannot add a new executable effect rule.
pub(super) const AUDIT: &[(&str, &str, &str, &str, &str, &str, &str, &str, &str, &str)] = &[
    (
        "start",
        "fn start() -> Accumulator",
        "none",
        "none",
        "none",
        "none",
        "0",
        "Retained(Accumulator)",
        "none;Drop=closed-free-only",
        "accepted::root;bare_vec_loop",
    ),
    (
        "callback",
        "fn callback(n: usize) -> Raw",
        "usize",
        "callee-private-excluded",
        "arbitrary-current-W;live-prefix-must-be-retained",
        "none",
        "W=capacity(result.bytes);no-private-bound",
        "Callback(Raw);adopt-before-step",
        "allocation-failure-aborts;Drop=Vec<u8>-free-only",
        "accepted::root;unleased_callback_result",
    ),
    (
        "adopt",
        "fn adopt(raw: Raw) -> Retained",
        "move-current-W",
        "none",
        "none",
        "none",
        "receipt=capacity(raw.bytes)*size_of(u8)",
        "Retained(Retained);consumes-W",
        "none;Drop=Vec<u8>-free-only",
        "accepted::root;unleased_callback_result",
    ),
    (
        "append",
        "fn append(mut acc: Accumulator, value: Retained) -> Accumulator",
        "move-retained,move-retained",
        "under-live-receipt",
        "none",
        "none",
        "delta=size_of(Node);payload-receipts-move",
        "Retained(Accumulator);consumes-both",
        "checked-overflow-before-allocation;Drop=closed-Node-free-only",
        "accepted::root;bare_vec_loop",
    ),
    (
        "send",
        "fn send(owner: Accumulator) -> Outgoing",
        "move-retained",
        "none",
        "none",
        "none",
        "0;all-receipts-transfer",
        "Outgoing(Accumulator);consumes-owner",
        "none;Drop=closed-Node-free-only",
        "accepted::root;unleased_callback_result",
    ),
];

#[derive(Clone)]
pub(super) struct Leaf {
    pub id: Id,
    pub instance: Vec<ExactTy>,
    pub operation: String,
    pub signature: String,
    pub record: Vec<String>,
}

pub(super) fn read(text: &str) -> Result<Vec<Leaf>, String> {
    text.lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let record: Vec<_> = line.split('\t').map(str::to_owned).collect();
            if record.len() != 12 || record.iter().any(|r| r.is_empty()) {
                return Err("leaf requires all twelve contract columns".into());
            }
            if record
                .iter()
                .any(|r| r.contains('*') && r != "receipt=capacity(raw.bytes)*size_of(u8)")
            {
                return Err("wildcard trusted entry".into());
            }
            if let Some(arg) = record[0]
                .strip_prefix("fixture::constructors::keep::<")
                .and_then(|s| s.strip_suffix('>'))
            {
                if !matches!(arg, "u8" | "usize") {
                    return Err("entry has no exact executable effect contract".into());
                }
                if record[1..11].iter().map(String::as_str).ne(INSTANCE_AUDIT) {
                    return Err("unaudited instance contract".into());
                }
                return Ok(Leaf {
                    id: "fixture::constructors::keep".into(),
                    instance: vec![ExactTy::Scalar(arg.into())],
                    operation: "keep".into(),
                    signature: record[1].clone(),
                    record,
                });
            }
            let Some(audit) = AUDIT
                .iter()
                .find(|a| record[0] == format!("{PREFIX}::{}", a.0))
            else {
                return Err("entry has no exact executable effect contract".into());
            };
            let expected = [
                audit.1,
                audit.2,
                audit.3,
                audit.4,
                audit.5,
                audit.6,
                audit.7,
                audit.8,
                audit.9,
                "substrate+Cargo.lock+rust-toolchain.toml",
            ];
            if record[1..11].iter().map(String::as_str).ne(expected) {
                return Err(format!("unaudited contract for {}", record[0]));
            }
            Ok(Leaf {
                id: record[0].clone(),
                instance: Vec::new(),
                operation: audit.0.into(),
                signature: record[1].clone(),
                record,
            })
        })
        .collect()
}

pub(super) fn fingerprint(source: &Source) -> String {
    let file = source
        .files
        .iter()
        .find(|(path, _)| path.ends_with("fixtures/accepted.rs"))
        .expect("compiled accepted fixture")
        .1;
    let substrate = file
        .split_once("pub mod substrate {")
        .unwrap()
        .1
        .split_once("\npub fn next")
        .unwrap()
        .0;
    let mut hash = 0xcbf29ce484222325u64;
    let mut bytes = substrate.as_bytes().to_vec();
    for file in ["Cargo.lock", "rust-toolchain.toml"] {
        bytes.extend(std::fs::read(source::workspace().join(file)).unwrap());
    }
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

pub(super) fn validate(
    source: &Source,
    leaves: &[Leaf],
    used: &BTreeSet<LeafKey>,
) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    let fingerprint = fingerprint(source);
    for leaf in leaves {
        if !seen.insert(leaf.key()) {
            return Err("duplicate trusted identity".into());
        }
        let f = resolve::function(source, &leaf.id)?;
        let expected: syn::ItemFn =
            syn::parse_str(&format!("{} {{}}", leaf.signature)).map_err(|e| e.to_string())?;
        if tokens(&f.sig) != tokens(&expected.sig) {
            return Err(format!("stale trusted signature {}", leaf.id));
        }
        let expected_fingerprint = if leaf.operation == "keep" {
            instance_fingerprint(source)
        } else {
            fingerprint.clone()
        };
        let parsed = read(&leaf.record.join("\t"))?;
        if parsed[0].key() != leaf.key() {
            return Err("trusted instance differs from audited record".into());
        }
        if leaf.record[11] != expected_fingerprint {
            return Err(format!("stale dependency fingerprint {}", leaf.id));
        }
        if !used.contains(&leaf.key()) {
            return Err(format!(
                "unexercised trusted entry {}",
                leaf.key().display()
            ));
        }
    }
    if &seen != used {
        return Err("missing exact leaf contract".into());
    }
    Ok(())
}

impl Leaf {
    pub fn key(&self) -> LeafKey {
        LeafKey {
            id: self.id.clone(),
            instance: self.instance.clone(),
        }
    }
}

// This is a separate compiled control, never an open generic family and never
// an inventory promotion. Both concrete records must actually be exercised.
const INSTANCE_AUDIT: [&str; 10] = [
    "fn keep<T>(value: T) -> T",
    "move-exact-instance",
    "none",
    "none",
    "none",
    "0",
    "same-value-and-retention",
    "none;Drop=primitive",
    "constructors::instances;missing-instance",
    "constructors+Cargo.lock+rust-toolchain.toml",
];

fn instance_fingerprint(source: &Source) -> String {
    let file = source
        .files
        .iter()
        .find(|(p, _)| p.ends_with("fixtures/constructors.rs"))
        .expect("compiled instance fixture")
        .1;
    let mut hash = 0xcbf29ce484222325u64;
    let mut bytes = file.as_bytes().to_vec();
    for file in ["Cargo.lock", "rust-toolchain.toml"] {
        bytes.extend(std::fs::read(source::workspace().join(file)).unwrap());
    }
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

pub(super) fn instance_contracts(source: &Source) -> Vec<Leaf> {
    let text = ["u8", "usize"]
        .map(|t| {
            format!(
                "fixture::constructors::keep::<{t}>\t{}\t{}",
                INSTANCE_AUDIT.join("\t"),
                instance_fingerprint(source)
            )
        })
        .join("\n");
    read(&text).unwrap()
}
