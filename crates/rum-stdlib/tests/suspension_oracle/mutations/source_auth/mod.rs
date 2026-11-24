//! An arm-specific capability, authenticated before exposing any original byte.
pub(crate) mod artifacts;
mod c10;
mod files;
mod manifests;
pub(crate) mod observer;
mod observer_policy;
mod observer_policy_tests;
mod schema;
mod tests;
pub(crate) mod view;

use crate::oracle::obligations::{bindings, content_digest as digest};
use files::Snapshot;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::ErrorKind,
    marker::PhantomData,
    path::{Component, Path, PathBuf},
};
pub(crate) use view::{Arm, Armed, AuthenticatedSource, Disabled, MutationObserver};

/// Transport decision for one armed or ordinary run. The label is the class name.
pub(super) fn observer_transport_for(
    row: &str,
    arm: &str,
    ordinary_forced: bool,
) -> (&'static str, bool) {
    let specs = super::schema::specs();
    let spec = specs.get(row).expect("declared own row");
    let plan = observer_policy::resolve(spec).expect("resolved carrier");
    let mode = observer_policy::observation_mode(spec, plan).expect("observation mode");
    (
        observer_policy::mode_label(mode),
        observer_policy::observer_transport(mode, arm, ordinary_forced),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuthError {
    Io,
    Symlink,
    Receipt,
    WrongArm,
    Binary,
    Tree,
    Inventory,
    BaselineSeal,
    OverlaySet,
    Preimage,
    Postimage,
    UnlistedFile,
    Manifest,
    ObserverRow,
    ObserverUnarmed,
    ObserverProgram,
    ObserverPolicy,
}
type AuthResult<T> = Result<T, AuthError>;

#[derive(Clone)]
struct Receipt {
    fields: BTreeMap<String, String>,
    overlays: BTreeMap<String, (String, String)>,
}

impl Receipt {
    fn read(path: &Path) -> AuthResult<Self> {
        let text = std::fs::read_to_string(path).map_err(|_| AuthError::Io)?;
        let mut fields = BTreeMap::new();
        let mut overlays = BTreeMap::new();
        for line in text.lines() {
            let cells: Vec<_> = line.split('\t').collect();
            match cells.as_slice() {
                ["field", name, value] => {
                    if fields.insert((*name).into(), (*value).into()).is_some() {
                        return Err(AuthError::Receipt);
                    }
                }
                ["overlay", file, before, after] => {
                    if overlays
                        .insert((*file).into(), ((*before).into(), (*after).into()))
                        .is_some()
                    {
                        return Err(AuthError::Receipt);
                    }
                }
                _ => return Err(AuthError::Receipt),
            }
        }
        let expected = [
            "arm",
            "baseline_tree",
            "binary",
            "codegen_units",
            "configuration",
            "profile",
            "tree",
        ];
        if fields.keys().map(String::as_str).collect::<Vec<_>>() != expected {
            return Err(AuthError::Receipt);
        }
        Ok(Self { fields, overlays })
    }
}

fn verify<A: Arm>(
    baseline: &Snapshot,
    copied: &Snapshot,
    binary: &str,
    receipt: &Receipt,
) -> AuthResult<()> {
    if receipt.fields["arm"] != A::NAME {
        return Err(AuthError::WrongArm);
    }
    if receipt.fields["profile"] != "release" || receipt.fields["codegen_units"] != "1" {
        return Err(AuthError::Receipt);
    }
    if receipt.fields["binary"] != binary {
        return Err(AuthError::Binary);
    }
    if baseline.tree != receipt.fields["baseline_tree"] || copied.tree != receipt.fields["tree"] {
        return Err(AuthError::Tree);
    }
    if receipt
        .overlays
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>()
        != schema::pins().keys().copied().collect::<Vec<_>>()
    {
        return Err(AuthError::OverlaySet);
    }
    let inventory = "crates/rum-stdlib/tests/suspension_oracle/inventory/";
    for (file, bytes) in &baseline.files {
        if file.starts_with(inventory) && copied.files.get(file) != Some(bytes) {
            return Err(AuthError::Inventory);
        }
    }
    let provenance_path = format!("{inventory}obligations/provenance.tsv");
    if baseline.files.get(&provenance_path).map(Vec::as_slice)
        != Some(include_bytes!("../../inventory/obligations/provenance.tsv").as_slice())
    {
        return Err(AuthError::BaselineSeal);
    }
    let seals = bindings::provenance().map_err(|_| AuthError::BaselineSeal)?;
    let source_seals: BTreeSet<_> = seals
        .iter()
        .filter(|r| {
            matches!(
                r[0],
                "canonical-source" | "guest-route" | "guest-dependency"
            )
        })
        .map(|r| (r[2], r[3], r[5]))
        .collect();
    for (path, hash, pin) in source_seals {
        if pin != bindings::LEDGER_PIN
            || baseline.files.get(path).map(|b| digest(b)).as_deref() != Some(hash)
        {
            return Err(AuthError::BaselineSeal);
        }
    }
    let program_table = format!("{inventory}obligations/programs.tsv");
    let table = baseline
        .files
        .get(&program_table)
        .ok_or(AuthError::Inventory)?;
    let table_seals: Vec<_> = seals
        .iter()
        .filter(|r| r[0] == "program-table" && r[1] == "programs")
        .collect();
    if table_seals.len() != 1 || digest(table) != table_seals[0][3] {
        return Err(AuthError::BaselineSeal);
    }
    for (file, pin) in schema::pins() {
        let original = baseline.files.get(file).ok_or(AuthError::Preimage)?;
        let (preimage, postimage) = &receipt.overlays[file];
        if digest(original) != pin || preimage != pin {
            return Err(AuthError::Preimage);
        }
        let actual = copied.files.get(file).ok_or(AuthError::Postimage)?;
        if digest(actual) != *postimage
            || *actual != schema::file_postimage(file, original, A::ENABLED)?
        {
            return Err(AuthError::Postimage);
        }
    }
    let additions = schema::additions()?;
    let mut expected: BTreeSet<String> = baseline.files.keys().cloned().collect();
    for file in additions.keys() {
        if baseline.files.contains_key(*file) {
            return Err(AuthError::UnlistedFile);
        }
        expected.insert((*file).to_string());
    }
    let actual: BTreeSet<String> = copied.files.keys().cloned().collect();
    if actual != expected {
        return Err(AuthError::UnlistedFile);
    }
    for (file, (pin, len)) in &additions {
        let bytes = copied.files.get(*file).ok_or(AuthError::UnlistedFile)?;
        if bytes.len() != *len {
            return Err(AuthError::OverlaySet);
        }
        if digest(bytes) != *pin {
            return Err(AuthError::UnlistedFile);
        }
    }
    let lane = match receipt.fields["configuration"].as_str() {
        "p0543-unchecked" => true,
        "default" | "no-default" => false,
        _ => return Err(AuthError::Receipt),
    };
    let pins = schema::pins();
    for (file, original) in &baseline.files {
        if pins.contains_key(file.as_str()) {
            continue;
        }
        let actual = &copied.files[file];
        if lane && manifests::PINS.iter().any(|(path, _, _)| path == file) {
            let (_, pre, post) = manifests::PINS
                .iter()
                .find(|(path, _, _)| path == file)
                .unwrap();
            if digest(original) != *pre || digest(actual) != *post {
                return Err(AuthError::Manifest);
            }
        } else if actual != original {
            return Err(AuthError::UnlistedFile);
        }
    }
    Ok(())
}

pub(crate) fn authenticate<A: Arm>(
    baseline: &Path,
    receipt: &Path,
) -> AuthResult<AuthenticatedSource<A>> {
    let root = crate::oracle::coverage::source::workspace();
    let before = Snapshot::read(baseline)?;
    let after = Snapshot::read(&root)?;
    let binary = digest(
        &std::fs::read(std::env::current_exe().map_err(|_| AuthError::Io)?)
            .map_err(|_| AuthError::Io)?,
    );
    let receipt = Receipt::read(receipt)?;
    verify::<A>(&before, &after, &binary, &receipt)?;
    Ok(AuthenticatedSource {
        root,
        postimages: after
            .files
            .iter()
            .map(|(k, v)| (k.clone(), digest(v)))
            .collect(),
        originals: before.files,
        arm: PhantomData,
    })
}

impl<A: Arm> AuthenticatedSource<A> {
    pub(crate) fn source(&self, path: &str) -> bindings::BindingResult<Vec<u8>> {
        let expected = self
            .postimages
            .get(path)
            .ok_or(bindings::BindingError::SourceIdentity)?;
        let actual = std::fs::read(self.root.join(path))
            .map_err(|_| bindings::BindingError::SourceIdentity)?;
        if digest(&actual) != *expected {
            return Err(bindings::BindingError::SourceIdentity);
        }
        self.originals
            .get(path)
            .cloned()
            .ok_or(bindings::BindingError::SourceIdentity)
    }
}

/// Finite scratch roots. A private path outside this list is not an import root.
pub(crate) fn admitted_scratch(workspace: &Path, candidate: &Path) -> bool {
    const BRIEFS: &[&str] = &[
        "brief-740",
        "brief-742",
        "brief-743",
        "brief-744",
        "brief-745",
        "brief-746",
    ];
    let Ok(canonical) = workspace.canonicalize() else {
        return false;
    };
    let suffix = if let Ok(suffix) = candidate.strip_prefix(workspace) {
        suffix
    } else if let Ok(suffix) = candidate.strip_prefix(&canonical) {
        suffix
    } else if candidate.is_relative() {
        candidate
    } else {
        return false;
    };
    if suffix
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return false;
    }
    let mut lexical = canonical.clone();
    for component in suffix.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => lexical.push(part),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return false,
        }
    }
    let root: Vec<_> = canonical
        .components()
        .map(|component| component.as_os_str().to_os_string())
        .collect();
    let got: Vec<_> = lexical
        .components()
        .map(|component| component.as_os_str().to_os_string())
        .collect();
    if got.len() < root.len() + 2 || got[..root.len()] != root[..] {
        return false;
    }
    if got[root.len()] != std::ffi::OsStr::new(".temp")
        || !BRIEFS.contains(&got[root.len() + 1].to_str().unwrap_or(""))
    {
        return false;
    }
    let root_path = canonical.clone();
    let mut cursor = canonical;
    for part in got.iter().skip(root.len()) {
        cursor.push(part);
        match std::fs::symlink_metadata(&cursor) {
            Ok(meta) if meta.file_type().is_symlink() => return false,
            Ok(_) => {
                let Ok(canon) = cursor.canonicalize() else {
                    return false;
                };
                if !canon.starts_with(&root_path) {
                    return false;
                }
            }
            Err(err) if err.kind() == ErrorKind::NotFound => return true,
            Err(_) => return false,
        }
    }
    true
}

fn environment() -> Option<(PathBuf, PathBuf)> {
    Some((
        std::env::var_os("RUM_CM740_BASELINE")?.into(),
        std::env::var_os("RUM_CM740_RECEIPT")?.into(),
    ))
}
