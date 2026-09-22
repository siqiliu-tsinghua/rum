//! In-memory adversarial maps. No environment flag or permissive checker mode.

use super::model::*;
use super::{fixtures, regions, source};

// Watched-red runs temporarily replace this function's return expression in
// source, then restore it byte for byte. The ordinary gate always gets the
// valid promoted map; the separate negative fixtures remain permanently bad.
pub(super) fn promoted_map() -> Source {
    fixtures::program()
}

pub(super) fn inject(name: &str) -> Source {
    let mut source = fixtures::program();
    let bad = source.functions[&format!("fixture::{name}::root")][0].clone();
    let root = &mut source.functions.get_mut(regions::DEMO).unwrap()[0];
    root.module = bad.module;
    root.site = bad.site;
    root.sig = bad.sig;
    root.body = bad.body;
    source
}

pub(super) fn probe(text: &str) -> Source {
    let mut files = fixtures::FILES.to_vec();
    files.push(("probe", "probe.rs", text));
    source::fixture(&files)
}

pub(super) fn extracted() -> Source {
    let mut source = fixtures::program();
    let root = &mut source.functions.get_mut(regions::DEMO).unwrap()[0];
    root.body.stmts.insert(
        0,
        syn::parse_str("super::extracted_allocator::root();").unwrap(),
    );
    source
}
