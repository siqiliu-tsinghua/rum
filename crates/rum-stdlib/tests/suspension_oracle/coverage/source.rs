//! E1 checks existence only. This is deliberately not effect discovery or a
//! source-edge certificate; compiler-enforced edge binding belongs to E2.
use std::{collections::BTreeSet, path::PathBuf, sync::Mutex};
use syn::visit::Visit;
pub fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}
pub fn owner_exists(file: &str, name: &str) {
    static CACHE: std::sync::OnceLock<Mutex<std::collections::BTreeMap<String, BTreeSet<String>>>> =
        std::sync::OnceLock::new();
    let mut cache = CACHE.get_or_init(Default::default).lock().unwrap();
    let names = cache.entry(file.into()).or_insert_with(|| {
        struct Names(BTreeSet<String>);
        impl<'a> Visit<'a> for Names {
            fn visit_signature(&mut self, sig: &'a syn::Signature) {
                self.0.insert(sig.ident.to_string());
                syn::visit::visit_signature(self, sig);
            }
        }
        let text = std::fs::read_to_string(workspace().join(file)).unwrap();
        let tree = syn::parse_file(&text).unwrap();
        let mut names = Names(BTreeSet::new());
        names.visit_file(&tree);
        names.0
    });
    assert!(names.contains(name), "missing named owner {file}#{name}");
}
