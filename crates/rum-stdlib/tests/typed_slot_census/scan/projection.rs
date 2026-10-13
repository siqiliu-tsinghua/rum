//! Private lexical resolver projection; source populations remain authoritative.
use super::{Facts, Implementation, text};
use proc_macro2::Span;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

// The shared ownership resolver indexes production only. Open test declarations
// in an in-memory projection so it can supply lexical identities for all three
// populations. Actual populations, spans and hashes always come from the original
// module walk. Neither the workspace nor its cfg guards are changed here.
pub fn resolution_files(
    files: &BTreeMap<String, String>,
    facts: &Facts,
) -> BTreeMap<String, String> {
    let required: Vec<_> = facts
        .implementations
        .values()
        .filter(|i| {
            [
                "SuspensionFrame",
                "PureFrameFinalizer",
                "TypedFrame",
                "SlotFinalizer",
            ]
            .iter()
            .any(|name| i.trait_path.split(" :: ").last() == Some(name))
        })
        .map(|i| i.scope.as_str())
        .collect();
    let selected: BTreeSet<_> = facts
        .visited
        .iter()
        .filter(|(_, module, _)| {
            required
                .iter()
                .any(|scope| *scope == module || scope.starts_with(&format!("{module}::")))
        })
        .map(|(file, _, _)| file.as_str())
        .collect();
    struct Guards<'a> {
        spans: Vec<Span>,
        module: String,
        required: &'a [&'a str],
        implementations: Vec<&'a Implementation>,
    }
    impl Guards<'_> {
        fn attrs(&mut self, attrs: &[syn::Attribute]) {
            for a in attrs {
                if a.path().is_ident("test")
                    || a.path().is_ident("cfg") && text(&a.meta) == "cfg (test)"
                {
                    self.spans.push(a.span());
                }
            }
        }
    }
    impl<'ast> Visit<'ast> for Guards<'_> {
        fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
            let old = self.module.clone();
            self.module = format!("{old}::{}", item.ident);
            if self.required.iter().any(|scope| {
                *scope == self.module || scope.starts_with(&format!("{}::", self.module))
            }) {
                self.attrs(&item.attrs);
            }
            visit::visit_item_mod(self, item);
            self.module = old;
        }
        fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
            if self.implementations.iter().any(|i| {
                i.site.line >= item.span().start().line && i.site.end_line <= item.span().end().line
            }) {
                self.attrs(&item.attrs);
            }
            visit::visit_item_fn(self, item);
        }
    }
    let mut projected: BTreeMap<_, _> = files
        .iter()
        .filter(|(p, _)| p.contains("/src/"))
        .map(|(path, source)| {
            let module = facts
                .visited
                .iter()
                .find(|(file, _, _)| file == path)
                .map(|(_, module, _)| module.clone())
                .unwrap_or_default();
            let mut guards = Guards {
                spans: Vec::new(),
                module,
                required: &required,
                implementations: facts
                    .implementations
                    .values()
                    .filter(|i| i.site.file == *path)
                    .collect(),
            };
            // Opening only files on a boundary implementation's module ancestry
            // avoids indexing thousands of unrelated fixture declarations.
            if selected.contains(path.as_str()) {
                guards.visit_file(&syn::parse_file(source).unwrap());
            }
            let offset = |p: proc_macro2::LineColumn| {
                source
                    .split_inclusive('\n')
                    .take(p.line - 1)
                    .map(str::len)
                    .sum::<usize>()
                    + p.column
            };
            let mut bytes = source.as_bytes().to_vec();
            for span in guards.spans {
                for byte in &mut bytes[offset(span.start())..offset(span.end())] {
                    if *byte != b'\n' {
                        *byte = b' ';
                    }
                }
            }
            (path.clone(), String::from_utf8(bytes).unwrap())
        })
        .collect();
    // The shared loader compares literal paths. Supply aliases for Rust's
    // normalized #[path] parents in this private index, retaining canonical
    // paths in all emitted facts and hashes.
    for (raw, canonical) in &facts.aliases {
        let raw_parent = Path::new(raw).parent().unwrap();
        let parent = Path::new(canonical).parent().unwrap();
        let additions: Vec<_> = projected
            .iter()
            .filter_map(|(path, source)| {
                let tail = Path::new(path).strip_prefix(parent).ok()?;
                Some((
                    raw_parent.join(tail).to_str().unwrap().to_owned(),
                    source.clone(),
                ))
            })
            .collect();
        projected.extend(additions);
    }
    projected
}
