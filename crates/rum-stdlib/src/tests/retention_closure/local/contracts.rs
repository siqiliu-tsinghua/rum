//! brief-537 records: an exact closed declaration table, not free-form waivers.
//! Discharged guarantees are narrower than retention coverage. In particular,
//! B09 gives no receipt for a returned Expr; inert constructors mint no lease.

use super::super::{regions, source};
use super::interface::hash;
use std::{collections::BTreeSet, fs};

#[derive(Clone, Debug)]
pub(super) struct Contract {
    pub id: String,
    pub target: String,
    pub status: String,
    pub fingerprint: String,
    pub current: String,
    pub invalid: Option<String>,
    pub inert: bool,
}

pub(super) const IDS: &[&str] = &[
    "B01", "B02", "B03", "B04", "B05", "B06", "B07", "B08", "B09", "B10", "B11", "B12", "I01",
    "I02", "I03", "I04", "B13",
];

pub(super) fn target(id: &str) -> String {
    match id {
        "B01" => format!("{}::option_rule_exprs_for_symbol", regions::SESSION),
        "B02" => format!("{}::resume_merged_option_exprs", regions::SESSION),
        "B03" => "rum_runtime::runtime::options::reentry::OptionReentry::resume".into(),
        "B04" => "<rum_core::expr::Expr as std::clone::Clone>::clone".into(),
        "B05" => "std::vec::Vec<rum_core::expr::Expr>::len".into(),
        "B06" => "std::option::Option<&rum_core::expr::Symbol>::map<bool,literal>".into(),
        "B07" => "std::option::Option<bool>::unwrap_or".into(),
        "B08" => "rum_core::expr::Symbol::is_system_name".into(),
        "B09" => format!("{}::evaluate", regions::SESSION),
        "B10" => "rum_core::expr::Expr::call".into(),
        "B11" => "rum_runtime::runtime::expr_helpers::system_expr".into(),
        "B12" => format!("{}::insert_suspension_frame", regions::SESSION),
        "I01" => "<rum_core::expr::SharedArgs as std::ops::Deref>::deref".into(),
        "I02" => "<rum_core::expr::SharedExpr as std::ops::Deref>::deref".into(),
        "I03" => "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Deref>::deref".into(),
        "I04" => "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index".into(),
        "B13" => super::reads::HELPER.into(),
        _ => panic!("unknown boundary"),
    }
}

// Direct substrate dependencies, deliberately a flat list, not a discovered
// call closure. Whole files conservatively cover signature/body, schema,
// Clone/Drop/root adapters, build identity and the executable falsifiers.
pub(super) const DEPENDENCIES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "crates/rum-runtime/Cargo.toml",
    "crates/rum-stdlib/Cargo.toml",
    "crates/rum-core/src/expr.rs",
    "crates/rum-runtime/src/runtime/options/core.rs",
    "crates/rum-runtime/src/runtime/options/reentry.rs",
    "crates/rum-runtime/src/runtime/options/resume_frames.rs",
    "crates/rum-runtime/src/runtime/options/tests/resume.rs",
    "crates/rum-runtime/src/runtime/resolution.rs",
    "crates/rum-runtime/src/runtime/debug_tools/resources.rs",
    "crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs",
    "crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs",
    "crates/rum-runtime/src/runtime/debug_tools/resources/occurrence.rs",
    "crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/expr.rs",
    "crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs",
    "crates/rum-runtime/src/runtime/model/session.rs",
    "crates/rum-runtime/src/runtime/suspension/core.rs",
    "crates/rum-runtime/src/runtime/suspension/trace.rs",
    "crates/rum-runtime/src/runtime/accounting_mode/mod.rs",
    "crates/rum-runtime/src/runtime/accounting_mode/lease.rs",
    "crates/rum-runtime/src/runtime/accounting_mode/retained.rs",
    "crates/rum-runtime/src/runtime/accounting_mode/bridge.rs",
    "crates/rum-runtime/src/runtime/accounting_mode/counter.rs",
    "crates/rum-runtime/src/runtime/session/session_state.rs",
    "crates/rum-runtime/src/runtime/accounting_mode/frozen.rs",
    "crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs",
    "crates/rum-runtime/src/runtime/accounting_mode/tests/exits.rs",
    "crates/rum-runtime/src/runtime/accounting_mode/tests/frozen.rs",
    "crates/rum-stdlib/src/tests/retention_closure/local/contracts.rs",
    "crates/rum-stdlib/src/tests/retention_closure/local/marks.rs",
    "crates/rum-stdlib/src/tests/retention_closure/local/protocol.rs",
    "crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs",
];

pub(super) fn fingerprint(fields: &[&str]) -> String {
    // Flat rot guard, computed once at indexing. Include all source and oracle
    // inputs so a new imported trait or impl in another file expires contracts.
    // This hashes bytes, never analyzes or follows another body's calls.
    static CONTEXT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let context = CONTEXT.get_or_init(|| {
        let mut parts = Vec::<Vec<u8>>::new();
        for file in DEPENDENCIES {
            parts.push(file.as_bytes().to_vec());
            parts.push(fs::read(source::workspace().join(file)).unwrap());
        }
        fn collect(root: &std::path::Path, paths: &mut Vec<std::path::PathBuf>) {
            for entry in fs::read_dir(root).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    collect(&path, paths);
                } else if path
                    .extension()
                    .is_some_and(|s| s == "rs" || s == "py" || s == "toml")
                {
                    paths.push(path);
                }
            }
        }
        let root = source::workspace();
        let mut paths = vec![];
        collect(&root.join("crates"), &mut paths);
        paths.sort();
        for path in paths {
            parts.push(
                path.strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .as_bytes()
                    .to_vec(),
            );
            parts.push(fs::read(path).unwrap());
        }
        hash(parts)
    });
    let mut parts = fields
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect::<Vec<_>>();
    parts.push(context.as_bytes().to_vec());
    hash(parts)
}

pub(super) fn read(text: &str) -> Result<Vec<Contract>, String> {
    let extra = super::protocol::extra_targets();
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for line in text
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let f = line.split('\t').collect::<Vec<_>>();
        if f.len() != 19 || f.iter().any(|s| s.is_empty()) {
            return Err("contract needs all nineteen brief-537 fields".into());
        }
        let identity = extra
            .get(f[0])
            .cloned()
            .or_else(|| IDS.contains(&f[0]).then(|| target(f[0])));
        if identity.as_deref() != Some(f[1]) || !seen.insert(f[0]) || f[1].contains('*') {
            return Err("unknown, duplicate, wildcard or redirected contract identity".into());
        }
        let legacy = matches!(f[0], "B03" | "B09" | "B12");
        let reads = matches!(f[0], "I02" | "B13");
        let added =
            reads || matches!(f[0], "B04" | "B05" | "I01" | "I04") || extra.contains_key(f[0]);
        let installed = legacy || added;
        if f[17] != if installed { "Discharged" } else { "Pending" } {
            return Err(format!(
                "{}: only B03/B09/B12 and the explicitly installed concrete records may be discharged",
                f[0]
            ));
        }
        if legacy && f[12] != "Runtime-enforced" {
            return Err("wrong discharge category".into());
        }
        if added && (f[12] != "Test-checked" || f[6] != super::protocol::effect(f[0])) {
            return Err("added contract needs its exact effect and Test-checked discharge".into());
        }
        if f[11] != "allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt" {
            return Err("unknown exclusions".into());
        }
        let invalid = if installed {
            let (file, test) = f[15]
                .split_once('#')
                .ok_or("falsifier must name source and test")?;
            let contents =
                fs::read_to_string(source::workspace().join(file)).map_err(|e| e.to_string())?;
            if !contents.contains(&format!("fn {test}(")) {
                return Err(format!("missing executable falsifier {}", f[0]));
            }
            if !(reads && test == "local_option_read_contract_controls")
                && !(added && !reads && test == "local_concrete_contract_controls")
                && !matches!(
                    (f[0], test),
                    ("B03", "option_reentry_refused_until_registered")
                        | (
                            "B09",
                            "retention_r_is_separate_from_s_and_both_admission_entries_see_it"
                        )
                        | (
                            "B12",
                            "retention_suspension_transfers_once_resumes_twice_and_abandons"
                        )
                )
            {
                return Err("wrong falsifier domain".into());
            }
            None
        } else {
            Some(format!("{} has no discharged evidence", f[0]))
        };
        result.push(Contract {
            id: f[0].into(),
            target: f[1].into(),
            status: f[17].into(),
            fingerprint: f[18].into(),
            current: fingerprint(&f[..18]),
            invalid,
            inert: installed && added && super::protocol::effect(f[0]).ends_with(";inert"),
        });
    }
    let expected: BTreeSet<_> = IDS
        .iter()
        .copied()
        .chain(extra.keys().map(String::as_str))
        .collect();
    if seen != expected {
        return Err("missing boundary records".into());
    }
    Ok(result)
}
