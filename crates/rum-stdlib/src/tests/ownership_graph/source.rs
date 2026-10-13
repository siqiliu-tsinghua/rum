//! Follow the module tree, including local items, and reconcile every source file.
//!
//! Test-only modules are followed in coverage-only mode, not guessed from paths.
//! Other cfg alternatives are retained and explicitly reported as unresolved.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use syn::spanned::Spanned;
use syn::visit::Visit;

use super::model::*;

#[path = "source_items.rs"]
mod items;
#[path = "root_set.rs"]
mod root_set;

// Start with the kernel/stdlib and include workspace crates whose values they
// own, transitively. rum-runtime owns rum-syntax ASTs and diagnostics; syntax
// owns no other workspace types. Host/adapters depend in the opposite direction;
// rum-derive supplies macros, not stored runtime values.
pub(super) const ROOTS: &[&str] = &["rum-core", "rum-runtime", "rum-stdlib", "rum-syntax"];

pub(super) fn production() -> Source {
    scan(production_files(ROOTS), ROOTS)
}

fn production_files(roots: &[&str]) -> BTreeMap<String, String> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let mut files = BTreeMap::new();
    for krate in roots {
        collect_files(
            workspace,
            &workspace.join(format!("crates/{krate}/src")),
            &mut files,
        );
    }
    files
}

fn collect_files(workspace: &Path, dir: &Path, files: &mut BTreeMap<String, String>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_files(workspace, &path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let key = path
                .strip_prefix(workspace)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            files.insert(key, fs::read_to_string(&path).unwrap());
        }
    }
}

pub(super) fn fixture(files: &[(&str, &str)]) -> Source {
    fixture_with_roots(files, &["fixture"])
}

pub(super) fn fixture_with_roots(files: &[(&str, &str)], roots: &[&str]) -> Source {
    scan(
        files
            .iter()
            .map(|(p, s)| (p.to_string(), s.to_string()))
            .collect(),
        roots,
    )
}

fn scan(files: BTreeMap<String, String>, roots: &[&str]) -> Source {
    let mut loader = Loader {
        files,
        source: Source::default(),
    };
    for root in roots {
        let file = format!("crates/{root}/src/lib.rs");
        loader.file(&file, &root.replace('-', "_"), false, false);
        let bin_prefix = format!("crates/{root}/src/bin/");
        let bins: Vec<_> = loader
            .files
            .keys()
            .filter_map(|file| {
                let relative = file.strip_prefix(&bin_prefix)?;
                let name = if !relative.contains('/') {
                    relative.strip_suffix(".rs")?
                } else {
                    relative
                        .strip_suffix("/main.rs")
                        .filter(|name| !name.contains('/'))?
                };
                Some((
                    file.clone(),
                    format!("{}_bin_{}", root.replace('-', "_"), name.replace('-', "_")),
                ))
            })
            .collect();
        for (file, module) in bins {
            loader.file(&file, &module, false, false);
        }
    }
    for file in loader.files.keys() {
        if !loader.source.files.contains(file) && !loader.source.test_files.contains(file) {
            loader.source.unknown(
                Location {
                    file: file.clone(),
                    line: 1,
                    column: 1,
                },
                "source file is not reachable from a production or test module declaration".into(),
            );
        }
    }
    loader.source
}

struct Loader {
    files: BTreeMap<String, String>,
    source: Source,
}

impl Loader {
    fn file(&mut self, file: &str, module: &str, test_only: bool, conditional: bool) {
        let covered = if test_only {
            &mut self.source.test_files
        } else {
            &mut self.source.files
        };
        if !covered.insert(file.into()) {
            return;
        }
        let Some(text) = self.files.get(file) else {
            self.source.unknown(
                Location {
                    file: file.into(),
                    line: 1,
                    column: 1,
                },
                "missing module source".into(),
            );
            return;
        };
        let syntax = match syn::parse_file(text) {
            Ok(syntax) => syntax,
            Err(error) => {
                self.source.unknown(
                    Location::new(file, error.span()),
                    format!("unparsed source: {error}"),
                );
                return;
            }
        };
        let path = Path::new(file);
        let dir = if matches!(
            path.file_stem().and_then(|s| s.to_str()),
            Some("lib" | "mod")
        ) {
            path.parent().unwrap().to_path_buf()
        } else {
            path.with_extension("")
        };
        self.module(file, module, &dir, &syntax.items, test_only, conditional);
    }

    fn module(
        &mut self,
        file: &str,
        module: &str,
        dir: &Path,
        nodes: &[syn::Item],
        test_only: bool,
        conditional: bool,
    ) {
        if !test_only {
            self.source.modules.insert(module.into());
            self.source
                .scopes
                .entry(module.into())
                .or_insert_with(|| Scope {
                    module: module.into(),
                    ..Scope::default()
                });
        }
        let mut visitor = items::Collector {
            loader: self,
            file: file.into(),
            scope: module.into(),
            module: module.into(),
            dir: dir.to_path_buf(),
            test_only,
            conditional,
            impl_parameters: Default::default(),
        };
        for item in nodes {
            visitor.visit_item(item);
        }
    }

    fn child(
        &mut self,
        file: &str,
        module: &str,
        dir: &Path,
        item: &syn::ItemMod,
        test_only: bool,
        conditional: bool,
    ) {
        let child = format!("{module}::{}", item.ident);
        if let Some((_, nodes)) = &item.content {
            self.module(
                file,
                &child,
                &dir.join(item.ident.to_string()),
                nodes,
                test_only,
                conditional,
            );
            return;
        }
        let explicit = item
            .attrs
            .iter()
            .find(|a| a.path().is_ident("path"))
            .and_then(|a| {
                if let syn::Meta::NameValue(meta) = &a.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(path),
                        ..
                    }) = &meta.value
                    {
                        return Some(path.value());
                    }
                }
                None
            });
        let paths: Vec<PathBuf> = if let Some(path) = explicit {
            // #[path] is relative to the containing file (or inline module directory).
            let parent = Path::new(file).parent().unwrap();
            let base = if matches!(
                Path::new(file).file_stem().and_then(|s| s.to_str()),
                Some("lib" | "mod")
            ) {
                parent.to_path_buf()
            } else {
                Path::new(file).with_extension("")
            };
            vec![if dir == base { parent } else { dir }.join(path)]
        } else {
            vec![
                dir.join(format!("{}.rs", item.ident)),
                dir.join(item.ident.to_string()).join("mod.rs"),
            ]
        };
        let found: Vec<_> = paths
            .iter()
            .filter_map(|p| p.to_str())
            .filter(|p| self.files.contains_key(*p))
            .collect();
        if found.len() == 1 {
            self.file(found[0], &child, test_only, conditional);
        } else {
            self.source.unknown(
                Location::new(file, item.span()),
                format!(
                    "module {} has {} source candidates",
                    item.ident,
                    found.len()
                ),
            );
        }
    }
}

// Three-valued cfg evaluation fixes test=false. Unknown feature/target choices
// remain visible; their presence never silently removes a source subtree.
fn cfg_value(meta: &syn::Meta) -> Option<bool> {
    match meta {
        syn::Meta::Path(path) if path.is_ident("test") => Some(false),
        syn::Meta::List(list) => {
            use syn::parse::Parser;
            let items = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
                .parse2(list.tokens.clone())
                .ok()?;
            let values: Vec<_> = items.iter().map(cfg_value).collect();
            if list.path.is_ident("not") && values.len() == 1 {
                values[0].map(|v| !v)
            } else if list.path.is_ident("all") {
                if values.contains(&Some(false)) {
                    Some(false)
                } else if values.iter().all(|v| *v == Some(true)) {
                    Some(true)
                } else {
                    None
                }
            } else if list.path.is_ident("any") {
                if values.contains(&Some(true)) {
                    Some(true)
                } else if values.iter().all(|v| *v == Some(false)) {
                    Some(false)
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("test")
            || (a.path().is_ident("cfg")
                && a.parse_args::<syn::Meta>()
                    .ok()
                    .as_ref()
                    .and_then(cfg_value)
                    == Some(false))
    })
}

fn conditional(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg_attr")
            || (a.path().is_ident("cfg")
                && a.parse_args::<syn::Meta>()
                    .ok()
                    .as_ref()
                    .and_then(cfg_value)
                    .is_none())
    })
}
