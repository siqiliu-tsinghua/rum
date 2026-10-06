use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Eq)]
pub enum CatalogError {
    Duplicate(String),
    MissingInstalled(String),
    UndeclaredInstalled(String),
    StaleOwed(String),
    MissingOwed(String),
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

pub fn reconcile_owed(
    declared: &BTreeSet<String>,
    covered: &BTreeSet<String>,
    owed: &BTreeSet<String>,
    exact: bool,
) -> Result<(), CatalogError> {
    if !exact {
        return Ok(());
    }
    if let Some(id) = covered.difference(declared).next() {
        return Err(CatalogError::UnknownDescriptorEntry(id.clone()));
    }
    let missing: BTreeSet<_> = declared.difference(covered).cloned().collect();
    if let Some(id) = owed.difference(&missing).next() {
        return Err(CatalogError::StaleOwed(id.clone()));
    }
    if let Some(id) = missing.difference(owed).next() {
        return Err(CatalogError::MissingOwed(id.clone()));
    }
    Ok(())
}
