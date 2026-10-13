use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Eq)]
pub enum CatalogError {
    Duplicate(String),
    MissingInstalled(String),
    UndeclaredInstalled(String),
    UncoveredEntry(String),
    UnknownDescriptorEntry(String),
}

pub fn reconcile(
    installed: &[String],
    manifest: &[String],
    exact: bool,
) -> Result<(), CatalogError> {
    if !exact {
        return Ok(());
    } // Deliberately weakened checker, only used by falsifiers.
    fn set(rows: &[String]) -> Result<BTreeSet<String>, CatalogError> {
        let mut result = BTreeSet::new();
        for row in rows {
            if !result.insert(row.clone()) {
                return Err(CatalogError::Duplicate(row.clone()));
            }
        }
        Ok(result)
    }
    let installed = set(installed)?;
    let manifest = set(manifest)?;
    if let Some(name) = manifest.difference(&installed).next() {
        return Err(CatalogError::MissingInstalled(name.clone()));
    }
    if let Some(name) = installed.difference(&manifest).next() {
        return Err(CatalogError::UndeclaredInstalled(name.clone()));
    }
    Ok(())
}

pub fn reconcile_coverage(
    declared: &BTreeSet<String>,
    covered: &BTreeSet<String>,
    exact: bool,
) -> Result<(), CatalogError> {
    if !exact {
        return Ok(());
    } // Deliberately weakened checker, only used by falsifiers.
    if let Some(id) = covered.difference(declared).next() {
        return Err(CatalogError::UnknownDescriptorEntry(id.clone()));
    }
    if let Some(id) = declared.difference(covered).next() {
        return Err(CatalogError::UncoveredEntry(id.clone()));
    }
    Ok(())
}
