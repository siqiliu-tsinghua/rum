//! Re-authenticate retained build artifacts before accepting their raw records.
use super::*;

pub(crate) struct ArtifactIdentity {
    binary: String,
    tree: String,
    configuration: String,
}

impl ArtifactIdentity {
    pub(crate) fn binary(&self) -> &str {
        &self.binary
    }
    pub(crate) fn tree(&self) -> &str {
        &self.tree
    }
    pub(crate) fn configuration(&self) -> &str {
        &self.configuration
    }
}

/// The same exact overlay verifier used inside the running copy also checks
/// the retained copy and binary. This is build evidence, never execution credit.
pub(crate) fn artifact(
    baseline: &Path,
    copied: &Path,
    binary: &Path,
    receipt: &Path,
    arm: &str,
) -> AuthResult<ArtifactIdentity> {
    let before = Snapshot::read(baseline)?;
    let after = Snapshot::read(copied)?;
    let binary = digest(&std::fs::read(binary).map_err(|_| AuthError::Io)?);
    let receipt = Receipt::read(receipt)?;
    if arm == "real" {
        if receipt.fields["arm"] != "real"
            || !receipt.overlays.is_empty()
            || receipt.fields["binary"] != binary
            || receipt.fields["tree"] != after.tree
            || receipt.fields["baseline_tree"] != before.tree
            || receipt.fields["profile"] != "release"
            || receipt.fields["codegen_units"] != "1"
        {
            return Err(AuthError::Receipt);
        }
        let lane = match receipt.fields["configuration"].as_str() {
            "p0543-unchecked" => true,
            "default" | "no-default" => false,
            _ => return Err(AuthError::Receipt),
        };
        if before.files.keys().ne(after.files.keys()) {
            return Err(AuthError::UnlistedFile);
        }
        for (file, original) in &before.files {
            if lane && manifests::PINS.iter().any(|(p, _, _)| *p == file) {
                let (_, pre, post) = manifests::PINS.iter().find(|(p, _, _)| *p == file).unwrap();
                if digest(original) != *pre || digest(&after.files[file]) != *post {
                    return Err(AuthError::Manifest);
                }
            } else if after.files[file] != *original {
                return Err(AuthError::UnlistedFile);
            }
        }
    } else {
        match arm {
            "disabled" => verify::<Disabled>(&before, &after, &binary, &receipt)?,
            "armed" => verify::<Armed>(&before, &after, &binary, &receipt)?,
            _ => return Err(AuthError::WrongArm),
        }
    }
    Ok(ArtifactIdentity {
        binary,
        tree: after.tree,
        configuration: receipt.fields["configuration"].clone(),
    })
}
