//! File/module traversal and test population classification.
use super::{Facts, Population, Site, text};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};
use syn::{spanned::Spanned, visit::Visit};

pub fn files(root: &Path) -> BTreeMap<String, String> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else if path.extension().is_some_and(|s| s == "rs") {
                out.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace('\\', "/"),
                    fs::read_to_string(path).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(root.join("crates")).unwrap() {
        let path = entry.unwrap().path();
        for name in ["src", "tests"] {
            if path.join(name).is_dir() {
                walk(root, &path.join(name), &mut out);
            }
        }
    }
    out
}

pub(super) fn test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("test") || a.path().is_ident("cfg") && text(&a.meta) == "cfg (test)"
    })
}
pub fn normalize(path: &Path) -> String {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            other => result.push(other.as_os_str()),
        }
    }
    result.to_str().unwrap().replace('\\', "/")
}
fn directory(file: &str) -> PathBuf {
    let path = Path::new(file);
    if matches!(
        path.file_stem().unwrap().to_str(),
        Some("lib" | "main" | "mod")
    ) {
        path.parent().unwrap().to_owned()
    } else {
        path.with_extension("")
    }
}

pub fn scan(files: &BTreeMap<String, String>) -> Facts {
    let mut facts = Facts::default();
    let roots: Vec<_> = files
        .keys()
        .filter(|p| p.ends_with("/src/lib.rs") || p.ends_with("/src/main.rs"))
        .cloned()
        .collect();
    for file in roots {
        let name = file.split('/').nth(1).unwrap().replace('-', "_");
        let module = if file.ends_with("main.rs") {
            format!("{name}_bin")
        } else {
            name
        };
        load(
            files,
            &mut facts,
            &file,
            &module,
            Population::Production,
            true,
        );
    }
    // Integration legacy users are a separate population; their exact site
    // identities must be in the receipt. Strings containing Rust are not ASTs.
    for file in files
        .keys()
        .filter(|p| p.split('/').count() == 4 && p.contains("/tests/"))
        .cloned()
        .collect::<Vec<_>>()
    {
        let module = file
            .trim_end_matches(".rs")
            .replace('/', "::")
            .replace('-', "_");
        load(
            files,
            &mut facts,
            &file,
            &module,
            Population::TestSupport,
            true,
        );
    }
    for file in files
        .keys()
        .filter(|p| !facts.visited.iter().any(|(f, _, _)| f == *p))
    {
        facts.unresolved.push(format!("unreachable-source\t{file}"));
    }
    facts
}
fn load(
    files: &BTreeMap<String, String>,
    facts: &mut Facts,
    file: &str,
    module: &str,
    population: Population,
    root: bool,
) {
    let test = population != Population::Production;
    if !facts.visited.insert((file.into(), module.into(), test)) {
        return;
    }
    let Some(source) = files.get(file) else {
        facts
            .unresolved
            .push(format!("module-source\t{file}\t{module}"));
        return;
    };
    let ast = syn::parse_file(source).unwrap_or_else(|e| panic!("{file}: {e}"));
    Walker {
        files,
        facts,
        file: file.into(),
        module: module.into(),
        dir: if root {
            Path::new(file).parent().unwrap().into()
        } else {
            directory(file)
        },
        caller: module.into(),
        owner: String::new(),
        trait_name: String::new(),
        test,
        population,
        ordinals: BTreeMap::new(),
        in_call: false,
        bindings: BTreeSet::new(),
    }
    .visit_file(&ast);
}

pub(super) struct Walker<'a> {
    pub(super) files: &'a BTreeMap<String, String>,
    pub(super) facts: &'a mut Facts,
    pub(super) file: String,
    pub(super) module: String,
    pub(super) dir: PathBuf,
    pub(super) caller: String,
    pub(super) owner: String,
    pub(super) trait_name: String,
    pub(super) test: bool,
    pub(super) ordinals: BTreeMap<(String, String), usize>,
    pub(super) in_call: bool,
    pub(super) population: Population,
    pub(super) bindings: BTreeSet<String>,
}
impl<'ast> Walker<'_> {
    pub(super) fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let child = format!("{}::{}", self.module, item.ident);
        let test = self.test || test_only(&item.attrs);
        let mut population = self.population;
        if item.ident == "tests" {
            population = Population::TestSupport;
        } else if test_only(&item.attrs) && population == Population::Production {
            population = Population::TestOnly;
        }
        if population == Population::TestOnly {
            for attr in &item.attrs {
                if attr.path().is_ident("cfg") && text(&attr.meta) == "cfg (test)" {
                    let site = Site::new(&self.file, attr.span());
                    self.facts.guard_modules.insert(site.key(), child.clone());
                    self.facts.guards.push(site);
                }
            }
        }
        if let Some((_, items)) = &item.content {
            let old = (
                self.module.clone(),
                self.dir.clone(),
                self.caller.clone(),
                self.test,
                self.population,
            );
            self.module = child.clone();
            self.dir.push(item.ident.to_string());
            self.caller = child;
            self.test = test;
            self.population = population;
            for item in items {
                self.visit_item(item);
            }
            (
                self.module,
                self.dir,
                self.caller,
                self.test,
                self.population,
            ) = old;
        } else {
            let explicit = item.attrs.iter().find_map(|a| {
                if a.path().is_ident("path") {
                    if let syn::Meta::NameValue(n) = &a.meta {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }) = &n.value
                        {
                            return Some(s.value());
                        }
                    }
                    None
                } else {
                    None
                }
            });
            let paths = if let Some(path) = explicit {
                let base = if self.dir == directory(&self.file) {
                    Path::new(&self.file).parent().unwrap().to_owned()
                } else {
                    self.dir.clone()
                };
                let raw = base.join(path);
                let normalized = normalize(&raw);
                if raw.to_str().unwrap() != normalized {
                    self.facts
                        .aliases
                        .insert(raw.to_str().unwrap().into(), normalized.clone());
                }
                vec![normalized]
            } else {
                vec![
                    normalize(&self.dir.join(format!("{}.rs", item.ident))),
                    normalize(&self.dir.join(item.ident.to_string()).join("mod.rs")),
                ]
            };
            let candidates: Vec<_> = paths
                .iter()
                .filter(|p| self.files.contains_key(*p))
                .collect();
            if candidates.len() == 1 {
                let path = Path::new(candidates[0]);
                if path.file_name().is_some_and(|n| n == "tests.rs")
                    || path.components().any(|c| c.as_os_str() == "tests")
                {
                    population = Population::TestSupport;
                }
                load(
                    self.files,
                    self.facts,
                    candidates[0],
                    &child,
                    population,
                    false,
                );
            } else {
                self.facts.unresolved.push(format!(
                    "module-source\t{}\t{}\t{child}",
                    self.file,
                    item.span().start().line
                ));
            }
        }
    }
}
