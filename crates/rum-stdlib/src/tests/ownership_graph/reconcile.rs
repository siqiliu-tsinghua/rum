//! Three-sided ownership reconciliation. Discovery never calls this module.
//! The ignore commands write review copies under the scene `target/brief-775/`
//! and do not install inventory rows. Installation is the adoption command only.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::host_payload_ledger::{
    Disposition, Entry, HEADER, LOCAL, PUBLIC_INPUT, check, check_entry, entries,
};
use super::model::{Discovery, Location, Source, Unknown, UnknownKind};
use super::{evidence, graph, source};

const TRACE_FRAMES: &str = "crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs";
const GUARDED_FRAMES: &str =
    "crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded/resume_frames.rs";
const SEARCH_GUARDED_FRAMES: &str =
    "crates/rum-stdlib/src/strings/patterns/search/guarded/replace/resume_frames.rs";
const RULES_FRAMES: &str = "crates/rum-stdlib/src/strings/patterns/search/rules/resume_frames.rs";
const RULES_CASES: &str = "crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs";

const RUM_DERIVE_OLD: &str = "unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)";
const RUM_DERIVE_NEW: &str = "unresolved attribute/expansion derive (rum_derive :: FrameRootTrace)";
const KERNEL_OLD: &str = "unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)";
const KERNEL_NEW: &str =
    "unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace)";
const DELETED_MACRO: &str = "unexpanded macro rum_runtime :: resume_or_suspend! (ends at 515:18)";

const BOUND_LIVE: &str = "UnknownHostPayload\tcrates/rum-runtime/src/lib.rs\t496\t19\trum_runtime::cm740_probe::control::Bound.live/std::boxed::Box[Own]/arg0: opaque owned payload dyn Fn () -> bool";
const OPERATIONS: &str = "UnknownHostPayload\tcrates/rum-runtime/src/lib.rs\t2743\t23\trum_runtime::cm740_probe::Operations.constructing/std::vec::Vec[Own]/arg0/tuple/.2/alias(rum_runtime::cm740_probe::Projection)/std::boxed::Box[Own]/arg0: opaque owned payload dyn Fn (& dyn std :: any :: Any) -> Option < OperationKey >";
const AROUND_HANDOFF: &str = "UnknownHostPayload\tcrates/rum-runtime/src/cm740_probe/q25_ownership.rs\t381\t51\tpublic input rum_runtime::cm740_probe::q25::around_handoff(body): opaque owned payload impl FnOnce () -> R";

type Rewrite = (String, String, String, usize);
type Deletion = (String, String);

fn emit(line: &str) {
    println!("{line}");
    let _ = io::stdout().flush();
}

fn one_line(text: &str) -> String {
    text.replace('\n', " | ")
}

fn scene_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn review_dir() -> PathBuf {
    scene_root().join("target/brief-775")
}

fn inventory_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/ownership_graph/inventory")
}

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join(".temp/brief-776/witness-updates.tsv").is_file() {
            return dir;
        }
        if !dir.pop() {
            panic!("repo root with .temp/brief-776/witness-updates.tsv not found");
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn atomic_write(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let temporary = PathBuf::from(format!("{}.part", path.display()));
    fs::write(&temporary, bytes).unwrap();
    fs::rename(&temporary, path).unwrap();
}

fn render_rows(rows: &[String]) -> String {
    let mut text = String::new();
    for row in rows {
        text.push_str(row);
        text.push('\n');
    }
    text
}

fn read_fact_rows(path: &Path) -> Vec<String> {
    evidence::lines(
        &fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
    )
}

fn take_digits(text: &str) -> Option<(&str, &str)> {
    let count = text
        .bytes()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if count == 0 {
        None
    } else {
        Some(text.split_at(count))
    }
}

fn normalize(construct: &str) -> String {
    let mut out = String::with_capacity(construct.len());
    let mut rest = construct;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix('@') {
            if let Some((_, tail)) = take_digits(after) {
                out.push_str("@ANCHOR");
                rest = tail;
                continue;
            }
        }
        if let Some(after) = rest.strip_prefix("ends at ") {
            if let Some((_, after_line)) = take_digits(after) {
                if let Some(after_colon) = after_line.strip_prefix(':') {
                    if let Some((_, tail)) = take_digits(after_colon) {
                        out.push_str("ends at POS");
                        rest = tail;
                        continue;
                    }
                }
            }
        }
        let ch = rest.chars().next().unwrap();
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}

struct Parts {
    kind: String,
    file: String,
    construct: String,
    row: String,
}

fn parts_of(row: &str) -> Result<Parts, String> {
    if row.starts_with("Unknown") {
        let columns: Vec<_> = row.splitn(5, '\t').collect();
        if columns.len() != 5 {
            return Err(format!("row does not have five columns: {row}"));
        }
        return Ok(Parts {
            kind: columns[0].into(),
            file: columns[1].into(),
            construct: columns[4].into(),
            row: row.into(),
        });
    }
    let columns: Vec<_> = row.splitn(4, '\t').collect();
    if columns.len() != 4 {
        return Err(format!("cycle row does not have four columns: {row}"));
    }
    Ok(Parts {
        kind: columns[0].into(),
        file: columns[1].into(),
        construct: columns[3].into(),
        row: row.into(),
    })
}

fn identity_of(parts: &Parts) -> (String, String, String) {
    (
        parts.kind.clone(),
        parts.file.clone(),
        normalize(&parts.construct),
    )
}

fn count_rows<'a>(rows: &'a [String]) -> BTreeMap<&'a str, usize> {
    let mut counts = BTreeMap::new();
    for row in rows {
        *counts.entry(row.as_str()).or_insert(0) += 1;
    }
    counts
}

fn is_local_construct(construct: &str) -> bool {
    construct
        .split_once(": opaque owned payload ")
        .is_some_and(|(trail, ty)| {
            trail.starts_with("rum_runtime::cm740_probe::") && ty.starts_with("dyn Fn")
        })
}

fn attribute_rewrites() -> BTreeSet<Rewrite> {
    BTreeSet::from([
        (
            TRACE_FRAMES.into(),
            RUM_DERIVE_OLD.into(),
            RUM_DERIVE_NEW.into(),
            1,
        ),
        (
            GUARDED_FRAMES.into(),
            RUM_DERIVE_OLD.into(),
            RUM_DERIVE_NEW.into(),
            6,
        ),
        (
            SEARCH_GUARDED_FRAMES.into(),
            KERNEL_OLD.into(),
            KERNEL_NEW.into(),
            6,
        ),
        (RULES_FRAMES.into(), KERNEL_OLD.into(), KERNEL_NEW.into(), 8),
    ])
}

fn macro_deletions() -> BTreeSet<Deletion> {
    BTreeSet::from([(RULES_CASES.into(), DELETED_MACRO.into())])
}

fn differ_rows(
    committed: &[String],
    discovered: &[String],
    review: &[String],
    path_set: &BTreeSet<String>,
    admitted_rewrites: &BTreeSet<Rewrite>,
    admitted_deletions: &BTreeSet<Deletion>,
    ledger_ok: impl Fn(&str) -> Result<(), String>,
) -> Result<(), String> {
    let mut errors = Vec::new();
    let review_counts = count_rows(review);
    let discovered_counts = count_rows(discovered);
    let mut review_keys: BTreeSet<&str> = review_counts.keys().copied().collect();
    review_keys.extend(discovered_counts.keys().copied());
    for row in review_keys {
        let reviewed = review_counts.get(row).copied().unwrap_or(0);
        let found = discovered_counts.get(row).copied().unwrap_or(0);
        if reviewed > found {
            for _ in 0..(reviewed - found) {
                errors.push(format!("fabricated row: {row}"));
            }
        } else if found > reviewed {
            for _ in 0..(found - reviewed) {
                errors.push(format!("missing from the review copy: {row}"));
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }

    let mut committed_buckets: BTreeMap<(String, String, String), Vec<Parts>> = BTreeMap::new();
    for row in committed {
        let parts = parts_of(row)?;
        committed_buckets
            .entry(identity_of(&parts))
            .or_default()
            .push(parts);
    }
    let mut discovered_buckets: BTreeMap<(String, String, String), Vec<Parts>> = BTreeMap::new();
    for row in discovered {
        let parts = parts_of(row)?;
        discovered_buckets
            .entry(identity_of(&parts))
            .or_default()
            .push(parts);
    }
    let mut keys: BTreeSet<(String, String, String)> = committed_buckets.keys().cloned().collect();
    keys.extend(discovered_buckets.keys().cloned());
    let mut committed_left = Vec::new();
    let mut discovered_left = Vec::new();
    for key in keys {
        let committed_rows = committed_buckets.remove(&key).unwrap_or_default();
        let discovered_rows = discovered_buckets.remove(&key).unwrap_or_default();
        let paired = committed_rows.len().min(discovered_rows.len());
        committed_left.extend(committed_rows.into_iter().skip(paired));
        discovered_left.extend(discovered_rows.into_iter().skip(paired));
    }

    let mut rewrite_left: BTreeMap<(String, String, String), usize> = BTreeMap::new();
    for (file, old, new, count) in admitted_rewrites {
        *rewrite_left
            .entry((file.clone(), old.clone(), new.clone()))
            .or_insert(0) += count;
    }
    let mut deletion_left = admitted_deletions.clone();
    let mut discovered_by: BTreeMap<(String, String), Vec<Parts>> = BTreeMap::new();
    for row in discovered_left {
        discovered_by
            .entry((row.file.clone(), row.construct.clone()))
            .or_default()
            .push(row);
    }

    for row in committed_left {
        let mut consumed = None;
        for ((file, old, new), count) in &rewrite_left {
            if file == &row.file && old == &row.construct && *count > 0 {
                let discovered_key = (file.clone(), new.clone());
                if discovered_by
                    .get(&discovered_key)
                    .is_some_and(|rows| !rows.is_empty())
                {
                    consumed = Some((file.clone(), old.clone(), new.clone()));
                    break;
                }
            }
        }
        if let Some(key) = consumed {
            *rewrite_left.get_mut(&key).unwrap() -= 1;
            discovered_by
                .get_mut(&(key.0.clone(), key.2.clone()))
                .unwrap()
                .pop();
            continue;
        }
        let deletion_key = (row.file.clone(), row.construct.clone());
        if deletion_left.remove(&deletion_key) {
            continue;
        }
        let rewrite_known = admitted_rewrites
            .iter()
            .any(|(file, old, _, _)| file == &row.file && old == &row.construct);
        let deletion_known = admitted_deletions
            .iter()
            .any(|(file, construct)| file == &row.file && construct == &row.construct);
        if rewrite_known || deletion_known {
            errors.push(format!("occurrence beyond admission: {}", row.row));
        } else {
            errors.push(format!("unexplained disappeared row: {}", row.row));
        }
    }

    for rows in discovered_by.values() {
        for row in rows {
            if !path_set.contains(&row.file) {
                errors.push(format!("unattributable file: {}", row.row));
                continue;
            }
            if row.kind == "UnknownHostPayload" {
                if let Err(error) = ledger_ok(&row.row) {
                    errors.push(format!("ledger_ok rejected {}: {error}", row.row));
                }
            }
        }
    }
    for ((file, old, new), count) in &rewrite_left {
        if *count != 0 {
            errors.push(format!(
                "unconsumed admission rewrite: {file}\t{old}\t{new}\t{count}"
            ));
        }
    }
    for (file, construct) in &deletion_left {
        errors.push(format!(
            "unconsumed admission deletion: {file}\t{construct}"
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn unknown_from_row(row: &str) -> Result<Unknown, String> {
    let columns: Vec<_> = row.splitn(5, '\t').collect();
    if columns.len() != 5 {
        return Err(format!("row does not have five columns: {row}"));
    }
    let kind = match columns[0] {
        "UnknownHostPayload" => UnknownKind::UnknownHostPayload,
        "UnknownConstruct" => UnknownKind::UnknownConstruct,
        other => return Err(format!("unknown kind {other}: {row}")),
    };
    let line = columns[2]
        .parse::<usize>()
        .map_err(|_| format!("bad line: {row}"))?;
    let column = columns[3]
        .parse::<usize>()
        .map_err(|_| format!("bad column: {row}"))?;
    Ok(Unknown {
        kind,
        location: Location {
            file: columns[1].into(),
            line,
            column,
        },
        construct: columns[4].into(),
    })
}

fn discovery_of(rows: &[Unknown]) -> Discovery {
    Discovery {
        source: Source::default(),
        edges: BTreeMap::new(),
        cycles: Vec::new(),
        drops: BTreeMap::new(),
        unknowns: rows.iter().cloned().collect(),
    }
}

fn ledger_ok_row(discovered: &Discovery, ledger: &[Entry], row: &str) -> Result<(), String> {
    let Some(entry) = ledger.iter().find(|entry| entry.row == row) else {
        return Err(format!("check 1: missing ledger entry: {row}"));
    };
    check_entry(discovered, &unknown_from_row(row)?, entry)
}

fn write_sorted_review(name: &str, rows: &mut Vec<String>) -> String {
    evidence::sort_rows(rows);
    evidence::compare(rows, rows)
        .unwrap_or_else(|error| panic!("{name} review is not exact: {error}"));
    let bytes = render_rows(rows);
    let path = review_dir().join(name);
    atomic_write(&path, bytes.as_bytes());
    let digest = sha256_hex(bytes.as_bytes());
    emit(&format!(
        "REVIEW WROTE path={} rows={} sha256={digest}",
        path.display(),
        rows.len()
    ));
    digest
}

fn partition_constructs(rows: Vec<String>) -> [(&'static str, Vec<String>); 4] {
    let mut attributes = Vec::new();
    let mut captures = Vec::new();
    let mut macros = Vec::new();
    let mut storage = Vec::new();
    for row in rows {
        let construct = row.splitn(5, '\t').nth(4).expect("construct column");
        if construct.starts_with("unresolved attribute") {
            attributes.push(row);
        } else if construct.starts_with("closure captures") {
            captures.push(row);
        } else if construct.starts_with("unexpanded macro") {
            macros.push(row);
        } else {
            storage.push(row);
        }
    }
    [
        ("unknown_attributes.tsv", attributes),
        ("unknown_captures.tsv", captures),
        ("unknown_macros.tsv", macros),
        ("unknown_storage.tsv", storage),
    ]
}

fn load_witness_updates() -> BTreeMap<String, String> {
    let path = repo_root().join(".temp/brief-776/witness-updates.tsv");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut updates = BTreeMap::new();
    for line in text.lines().filter(|line| !line.is_empty()) {
        let columns: Vec<_> = line.splitn(6, '\t').collect();
        if columns.len() != 6 {
            panic!("witness update does not have six fields: {line}");
        }
        let fact = columns[..5].join("\t");
        if columns[5].is_empty() {
            panic!("witness update has an empty witness: {fact}");
        }
        updates.insert(fact, columns[5].to_string());
    }
    if updates.len() != 4 {
        panic!("witness updates rows={} expected=4", updates.len());
    }
    updates
}

fn ledger_line(
    fact: &str,
    disposition: &str,
    witness: &str,
    owed_by: &str,
    coverage: &str,
) -> String {
    for field in [fact, disposition, witness, owed_by, coverage] {
        if field.contains('\n') {
            panic!("ledger field contains a newline");
        }
    }
    format!("{fact}\t{disposition}\t{witness}\t{owed_by}\t{coverage}")
}

fn disposition_name(disposition: Disposition) -> &'static str {
    match disposition {
        Disposition::Declared => "Declared",
        Disposition::Unbounded => "Unbounded",
        Disposition::PublicInput => "PublicInput",
        Disposition::Local => "Local",
    }
}

fn build_review_ledger(discovered: &Discovery) -> String {
    let updates = load_witness_updates();
    let committed =
        entries(&fs::read_to_string(inventory_dir().join("host_payload_ledger.tsv")).unwrap())
            .unwrap();
    let committed_by: BTreeMap<String, Entry> = committed
        .into_iter()
        .map(|entry| (entry.row.clone(), entry))
        .collect();
    let mut used = BTreeSet::new();
    let mut lines = Vec::new();
    for row in evidence::unknown_rows(discovered, UnknownKind::UnknownHostPayload) {
        let construct = row.splitn(5, '\t').nth(4).expect("construct column");
        // Public input wins over Local: a probe-rooted `public input … dyn Fn`
        // is a signature site, and the Local predicate's trail prefix does not match it.
        let line = if construct.starts_with("public input ") {
            ledger_line(&row, "PublicInput", PUBLIC_INPUT, "-", "-")
        } else if is_local_construct(construct) {
            ledger_line(&row, "Local", LOCAL, "-", "-")
        } else if let Some(witness) = updates.get(&row) {
            let entry = committed_by.get(&row).unwrap_or_else(|| {
                panic!("witness update has no committed entry: {row}");
            });
            if entry.disposition != Disposition::Unbounded {
                panic!("witness update is not Unbounded: {row}");
            }
            used.insert(row.clone());
            ledger_line(
                &row,
                "Unbounded",
                witness,
                &entry.owed_by,
                &entry.external_implementation_coverage,
            )
        } else if let Some(entry) = committed_by.get(&row) {
            ledger_line(
                &row,
                disposition_name(entry.disposition),
                &entry.witness,
                &entry.owed_by,
                &entry.external_implementation_coverage,
            )
        } else {
            panic!("undispositioned host row: {row}");
        };
        lines.push(line);
    }
    if used.len() != updates.len() {
        panic!(
            "a witness update was not applied: used={} updates={}",
            used.len(),
            updates.len()
        );
    }
    evidence::sort_rows(&mut lines);
    let mut text = String::from(HEADER);
    text.push('\n');
    text.push_str(&render_rows(&lines));
    let parsed =
        entries(&text).unwrap_or_else(|error| panic!("review ledger does not parse: {error}"));
    check(discovered, &parsed).unwrap_or_else(|error| panic!("review ledger rejected:\n{error}"));
    let public = parsed
        .iter()
        .filter(|entry| entry.disposition == Disposition::PublicInput)
        .count();
    let unbounded = parsed
        .iter()
        .filter(|entry| entry.disposition == Disposition::Unbounded)
        .count();
    let declared = parsed
        .iter()
        .filter(|entry| entry.disposition == Disposition::Declared)
        .count();
    let local = parsed
        .iter()
        .filter(|entry| entry.disposition == Disposition::Local)
        .count();
    let summary = format!(
        "LEDGER REVIEW rows={} PublicInput={public} Unbounded={unbounded} Declared={declared} Local={local}",
        parsed.len()
    );
    if parsed.len() != 92 || public != 79 || unbounded != 8 || declared != 3 || local != 2 {
        panic!("ledger review counts diverged: {summary}");
    }
    emit(&summary);
    text
}

fn load_chain_table() -> Vec<(String, String)> {
    let path = repo_root().join(".temp/brief-776/emission_chain.tsv");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut rows = Vec::new();
    let mut previous = String::new();
    for line in text.lines().filter(|line| !line.is_empty()) {
        let Some((path, digest)) = line.split_once('\t') else {
            panic!("chain row is not path and digest: {line}");
        };
        if path.is_empty()
            || digest.len() != 64
            || !digest
                .chars()
                .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
        {
            panic!("chain row is not a lowercase sha256: {line}");
        }
        if path <= previous.as_str() {
            panic!("chain table is not strictly sorted: {path}");
        }
        previous = path.to_string();
        rows.push((path.to_string(), digest.to_string()));
    }
    rows
}

fn chain_scene_match(rows: &[(String, String)]) -> usize {
    rows.iter()
        .filter(|(path, digest)| {
            let bytes = fs::read(scene_root().join(path))
                .unwrap_or_else(|error| panic!("read {path}: {error}"));
            sha256_hex(&bytes) == *digest
        })
        .count()
}

fn install_exact(inventory_name: &str, bytes: &[u8]) {
    let review = sha256_hex(bytes);
    let destination = inventory_dir().join(inventory_name);
    let before = if destination.is_file() {
        sha256_hex(&fs::read(&destination).unwrap())
    } else {
        "absent".to_string()
    };
    atomic_write(&destination, bytes);
    let after = sha256_hex(&fs::read(&destination).unwrap());
    let equal = after == review;
    emit(&format!(
        "INSTALL path=crates/rum-stdlib/src/tests/ownership_graph/inventory/{inventory_name} before={before} review={review} after={after} equal={equal}"
    ));
    if !equal {
        panic!("installed bytes differ from the review copy: {inventory_name}");
    }
}

fn family_rows(discovered: &Discovery, family: &str) -> Vec<String> {
    match family {
        "self-reaching" => evidence::cycle_rows(discovered),
        "UnknownHostPayload" => evidence::unknown_rows(discovered, UnknownKind::UnknownHostPayload),
        "unknown_attributes" | "unknown_captures" | "unknown_macros" | "unknown_storage" => {
            partition_constructs(evidence::unknown_rows(
                discovered,
                UnknownKind::UnknownConstruct,
            ))
            .into_iter()
            .find(|(name, _)| name.strip_suffix(".tsv") == Some(family))
            .map(|(_, rows)| rows)
            .unwrap()
        }
        other => panic!("unknown family {other}"),
    }
}

#[test]
#[ignore = "writes a review copy; does not install inventory rows"]
fn regenerate_self_reaching_inventory() {
    let discovered = graph::discover(source::production());
    let mut rows = evidence::cycle_rows(&discovered);
    write_sorted_review("self_reaching.tsv", &mut rows);
}

#[test]
#[ignore = "writes a review copy; does not install inventory rows"]
fn regenerate_host_payload_inventory() {
    let discovered = graph::discover(source::production());
    let mut rows = evidence::unknown_rows(&discovered, UnknownKind::UnknownHostPayload);
    write_sorted_review("host_payloads.tsv", &mut rows);
}

#[test]
#[ignore = "writes a review copy; does not install inventory rows"]
fn regenerate_unknown_construct_inventory() {
    let discovered = graph::discover(source::production());
    for (name, mut rows) in partition_constructs(evidence::unknown_rows(
        &discovered,
        UnknownKind::UnknownConstruct,
    )) {
        write_sorted_review(name, &mut rows);
    }
}

#[test]
#[ignore = "writes a review copy; does not choose a disposition the ruling does not name or install"]
fn regenerate_host_payload_ledger_review() {
    let discovered = graph::discover(source::production());
    let text = build_review_ledger(&discovered);
    let path = review_dir().join("host_payload_ledger_review.tsv");
    atomic_write(&path, text.as_bytes());
    emit(&format!(
        "REVIEW WROTE path={} rows=92 sha256={}",
        path.display(),
        sha256_hex(text.as_bytes())
    ));
}

#[test]
#[ignore = "adopts review copies only after the differ and the chain table pass"]
fn adopt_reconciled_inventory() {
    let discovered = graph::discover(source::production());
    let chain = load_chain_table();
    let scene_match = chain_scene_match(&chain);
    emit(&format!(
        "CHAIN TABLE rows={} scene_match={scene_match}",
        chain.len()
    ));
    let chain_ok = chain.len() == 152 && scene_match == 152;
    let path_set: BTreeSet<String> = chain.iter().map(|(path, _)| path.clone()).collect();
    let ledger_text = fs::read_to_string(review_dir().join("host_payload_ledger_review.tsv"))
        .unwrap_or_else(|error| panic!("read review ledger: {error}"));
    let ledger = entries(&ledger_text).unwrap_or_else(|error| panic!("review ledger: {error}"));

    let families = [
        (
            "self-reaching",
            "self_reaching.tsv",
            BTreeSet::new(),
            BTreeSet::new(),
        ),
        (
            "UnknownHostPayload",
            "host_payloads.tsv",
            BTreeSet::new(),
            BTreeSet::new(),
        ),
        (
            "unknown_attributes",
            "unknown_attributes.tsv",
            attribute_rewrites(),
            BTreeSet::new(),
        ),
        (
            "unknown_captures",
            "unknown_captures.tsv",
            BTreeSet::new(),
            BTreeSet::new(),
        ),
        (
            "unknown_macros",
            "unknown_macros.tsv",
            BTreeSet::new(),
            macro_deletions(),
        ),
        (
            "unknown_storage",
            "unknown_storage.tsv",
            BTreeSet::new(),
            BTreeSet::new(),
        ),
    ];
    let mut differ_ok = true;
    for (family, file_name, rewrites, deletions) in families {
        let committed = read_fact_rows(&inventory_dir().join(file_name));
        let found = family_rows(&discovered, family);
        let review = read_fact_rows(&review_dir().join(file_name));
        let result = differ_rows(
            &committed,
            &found,
            &review,
            &path_set,
            &rewrites,
            &deletions,
            |row| ledger_ok_row(&discovered, &ledger, row),
        );
        match result {
            Ok(()) => emit(&format!("DIFFER VERDICT family={family} result=ok")),
            Err(error) => {
                emit(&format!(
                    "DIFFER VERDICT family={family} result=err {}",
                    one_line(&error)
                ));
                differ_ok = false;
            }
        }
    }
    if !chain_ok || !differ_ok {
        panic!("adoption stopped before install chain_ok={chain_ok} differ_ok={differ_ok}");
    }
    check(&discovered, &ledger)
        .unwrap_or_else(|error| panic!("candidate ledger rejected:\n{error}"));
    let public = ledger
        .iter()
        .filter(|entry| entry.disposition == Disposition::PublicInput)
        .count();
    let unbounded = ledger
        .iter()
        .filter(|entry| entry.disposition == Disposition::Unbounded)
        .count();
    let declared = ledger
        .iter()
        .filter(|entry| entry.disposition == Disposition::Declared)
        .count();
    let local = ledger
        .iter()
        .filter(|entry| entry.disposition == Disposition::Local)
        .count();
    emit(&format!(
        "LEDGER COUNTS rows={} PublicInput={public} Unbounded={unbounded} Declared={declared} Local={local}",
        ledger.len()
    ));
    if ledger.len() != 92 || public != 79 || unbounded != 8 || declared != 3 || local != 2 {
        panic!("candidate ledger counts diverged");
    }

    for name in [
        "self_reaching.tsv",
        "host_payloads.tsv",
        "unknown_attributes.tsv",
        "unknown_captures.tsv",
        "unknown_macros.tsv",
        "unknown_storage.tsv",
    ] {
        let bytes = fs::read(review_dir().join(name)).unwrap();
        install_exact(name, &bytes);
    }
    install_exact("host_payload_ledger.tsv", ledger_text.as_bytes());
    let chain_bytes = fs::read(repo_root().join(".temp/brief-776/emission_chain.tsv")).unwrap();
    install_exact("emission_chain.tsv", &chain_bytes);
}

fn deny_ledger(row: &str) -> Result<(), String> {
    Err(format!("ledger_ok should not run: {row}"))
}

fn construct_row(file: &str, line: usize, column: usize, construct: &str) -> String {
    format!("UnknownConstruct\t{file}\t{line}\t{column}\t{construct}")
}

fn host_entry(row: &str, disposition: Disposition, witness: &str) -> Entry {
    Entry {
        row: row.splitn(5, '\t').take(5).collect::<Vec<_>>().join("\t"),
        disposition,
        witness: witness.into(),
        owed_by: "-".into(),
        external_implementation_coverage: "-".into(),
    }
}

fn expect_disagrees(row: &str, disposition: Disposition, witness: &str) {
    let unknown = unknown_from_row(row).unwrap();
    let discovered = discovery_of(&[unknown.clone()]);
    let entry = host_entry(row, disposition, witness);
    let error = check_entry(&discovered, &unknown, &entry).unwrap_err();
    assert!(
        error.contains("disposition disagrees with discovered storage/input site"),
        "{disposition:?} on {row}: {error}"
    );
    emit(&format!(
        "FALSIFIER d RED {} {} {}",
        disposition_name(disposition),
        row.rsplit('\t').next().unwrap_or(row),
        one_line(&error)
    ));
}

#[test]
fn reconcile_falsifier_unattributable_file() {
    let row = "UnknownHostPayload\tcrates/rum-core/src/expr.rs\t1\t1\tpublic input synthetic::probe(body): opaque owned payload impl Marker";
    let discovered = vec![row.to_string()];
    let review = discovered.clone();
    let empty_paths = BTreeSet::new();
    let rewrites = BTreeSet::new();
    let deletions = BTreeSet::new();
    let called = std::cell::Cell::new(false);
    let red = differ_rows(
        &[],
        &discovered,
        &review,
        &empty_paths,
        &rewrites,
        &deletions,
        |_| {
            called.set(true);
            Ok(())
        },
    )
    .unwrap_err();
    assert!(red.contains("unattributable file"), "{red}");
    assert!(
        !called.get(),
        "ledger_ok runs only after the path set admits the file"
    );
    emit(&format!("FALSIFIER a RED {}", one_line(&red)));

    let paths = BTreeSet::from(["crates/rum-core/src/expr.rs".to_string()]);
    called.set(false);
    differ_rows(
        &[],
        &discovered,
        &review,
        &paths,
        &rewrites,
        &deletions,
        |_| {
            called.set(true);
            Ok(())
        },
    )
    .unwrap();
    assert!(called.get(), "ledger_ok must accept the host row");
    emit("FALSIFIER a GREEN ok");
}

#[test]
fn reconcile_falsifier_mutated_identity() {
    let committed = vec![construct_row(TRACE_FRAMES, 51, 1, RUM_DERIVE_OLD)];
    let discovered = vec![construct_row(TRACE_FRAMES, 51, 1, RUM_DERIVE_NEW)];
    let paths = BTreeSet::from([TRACE_FRAMES.to_string()]);
    let deletions = BTreeSet::new();
    let red = differ_rows(
        &committed,
        &discovered,
        &discovered,
        &paths,
        &BTreeSet::new(),
        &deletions,
        deny_ledger,
    )
    .unwrap_err();
    assert!(red.contains("unexplained disappeared row"), "{red}");
    assert!(!red.contains("unattributable file"), "{red}");
    emit(&format!("FALSIFIER b RED empty {}", one_line(&red)));

    let admission = BTreeSet::from([(
        TRACE_FRAMES.to_string(),
        RUM_DERIVE_OLD.to_string(),
        RUM_DERIVE_NEW.to_string(),
        1,
    )]);
    differ_rows(
        &committed,
        &discovered,
        &discovered,
        &paths,
        &admission,
        &deletions,
        deny_ledger,
    )
    .unwrap();
    emit("FALSIFIER b GREEN ok");

    let mutated = "unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: Extra)";
    let mutated_rows = vec![construct_row(TRACE_FRAMES, 51, 1, mutated)];
    let red = differ_rows(
        &committed,
        &mutated_rows,
        &mutated_rows,
        &paths,
        &admission,
        &deletions,
        deny_ledger,
    )
    .unwrap_err();
    assert!(red.contains("unconsumed admission"), "{red}");
    emit(&format!("FALSIFIER b RED mutated {}", one_line(&red)));
}

#[test]
fn reconcile_falsifier_fabricated_row() {
    let mut committed = Vec::new();
    let mut discovered = Vec::new();
    committed.push(construct_row(
        "crates/rum-core/src/expr.rs",
        10,
        1,
        "closure captures: || item @4 (ends at 12:3)",
    ));
    discovered.push(construct_row(
        "crates/rum-core/src/expr.rs",
        11,
        2,
        "closure captures: || item @18 (ends at 99:8)",
    ));
    for (file, old, new, count) in attribute_rewrites() {
        for index in 0..count {
            committed.push(construct_row(&file, 100 + index, 1, &old));
            discovered.push(construct_row(&file, 200 + index, 1, &new));
        }
    }
    committed.push(construct_row(RULES_CASES, 506, 24, DELETED_MACRO));
    let rewrites = attribute_rewrites();
    let deletions = macro_deletions();
    let paths = BTreeSet::new();
    let mut fabricated = discovered.clone();
    fabricated.push(construct_row(
        "crates/rum-core/src/expr.rs",
        1,
        1,
        "fabricated construct",
    ));
    let red = differ_rows(
        &committed,
        &discovered,
        &fabricated,
        &paths,
        &rewrites,
        &deletions,
        deny_ledger,
    )
    .unwrap_err();
    assert!(red.contains("fabricated row"), "{red}");
    emit(&format!("FALSIFIER c RED {}", one_line(&red)));
    differ_rows(
        &committed,
        &discovered,
        &discovered,
        &paths,
        &rewrites,
        &deletions,
        deny_ledger,
    )
    .unwrap();
    emit("FALSIFIER c GREEN ok");
}

#[test]
fn reconcile_falsifier_disposition() {
    let handoff = unknown_from_row(AROUND_HANDOFF).unwrap();
    let discovered = discovery_of(&[handoff]);
    let row = AROUND_HANDOFF.to_string();
    let paths = BTreeSet::from(["crates/rum-runtime/src/cm740_probe/q25_ownership.rs".to_string()]);
    let red = differ_rows(
        &[],
        &[row.clone()],
        &[row.clone()],
        &paths,
        &BTreeSet::new(),
        &BTreeSet::new(),
        |candidate| ledger_ok_row(&discovered, &[], candidate),
    )
    .unwrap_err();
    assert!(red.contains("check 1: missing ledger entry:"), "{red}");
    assert!(red.contains("q25_ownership.rs\t381\t51"), "{red}");
    emit(&format!("FALSIFIER d RED missing {}", one_line(&red)));

    for dyn_row in [BOUND_LIVE, OPERATIONS] {
        expect_disagrees(dyn_row, Disposition::PublicInput, PUBLIC_INPUT);
        expect_disagrees(dyn_row, Disposition::Declared, "declared");
        expect_disagrees(dyn_row, Disposition::Unbounded, "unbounded");
        let unknown = unknown_from_row(dyn_row).unwrap();
        let found = discovery_of(&[unknown.clone()]);
        let entry = host_entry(dyn_row, Disposition::Local, LOCAL);
        check_entry(&found, &unknown, &entry).unwrap();
        emit(&format!(
            "FALSIFIER d GREEN Local {}",
            dyn_row.rsplit('\t').next().unwrap()
        ));
        let wrong = host_entry(dyn_row, Disposition::Local, "not the local witness");
        let error = check_entry(&found, &unknown, &wrong).unwrap_err();
        assert!(error.contains("witness mismatch"), "{error}");
        emit(&format!("FALSIFIER d RED witness {}", one_line(&error)));
    }

    let foreign = "UnknownHostPayload\tcrates/rum-core/src/expr.rs\t1\t1\trum_core::expr::Symbol.box: opaque owned payload dyn Fn () -> bool";
    expect_disagrees(foreign, Disposition::Local, LOCAL);

    let public_dyn = "UnknownHostPayload\tcrates/rum-runtime/src/lib.rs\t1\t1\tpublic input rum_runtime::cm740_probe::probe(body): opaque owned payload dyn Fn () -> bool";
    expect_disagrees(public_dyn, Disposition::Local, LOCAL);
    let unknown = unknown_from_row(public_dyn).unwrap();
    let found = discovery_of(&[unknown.clone()]);
    let entry = host_entry(public_dyn, Disposition::PublicInput, PUBLIC_INPUT);
    check_entry(&found, &unknown, &entry).unwrap();
    emit("FALSIFIER d GREEN PublicInput probe-rooted public input dyn Fn");

    let unknown = unknown_from_row(BOUND_LIVE).unwrap();
    let found = discovery_of(&[unknown.clone()]);
    let mut entry = host_entry(BOUND_LIVE, Disposition::Local, LOCAL);
    entry.owed_by = "TODO.md#standing-engineering-debt".into();
    let error = check_entry(&found, &unknown, &entry).unwrap_err();
    assert!(error.contains("only Unbounded carries debt"), "{error}");
    emit(&format!("FALSIFIER d RED debt {}", one_line(&error)));
}
