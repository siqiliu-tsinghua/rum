//! Complete module discovery, independently of obligations and promotions.
//! Non-test cfg alternatives are indexed, never selected using the host cfg.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use syn::parse::Parser;
use syn::spanned::Spanned;

use super::model::*;
use super::resolve;

pub(super) fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .into()
}

fn normalized(path: &Path) -> String {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            _ => result.push(part),
        }
    }
    result.to_str().unwrap().replace('\\', "/")
}

fn files(dir: &Path, source: &mut Source) {
    for entry in fs::read_dir(workspace().join(dir)).unwrap() {
        let path = entry.unwrap().path();
        let relative = path.strip_prefix(workspace()).unwrap();
        if path.is_dir() {
            files(relative, source);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            source
                .files
                .insert(normalized(relative), fs::read_to_string(path).unwrap());
        }
    }
}

pub(super) fn production() -> Source {
    let mut source = Source::default();
    // The manifest, rather than a stored inventory, selects workspace roots.
    let manifest = fs::read_to_string(workspace().join("Cargo.toml")).unwrap();
    let mut members = false;
    let mut roots = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if line == "members = [" {
            members = true;
            continue;
        }
        if members && line == "]" {
            break;
        }
        if !members {
            continue;
        }
        let directory = line.trim_end_matches(',').trim_matches('"');
        assert!(directory.starts_with("crates/") && !directory.contains('*'));
        let text = fs::read_to_string(workspace().join(directory).join("Cargo.toml")).unwrap();
        let name = text
            .lines()
            .find_map(|line| line.strip_prefix("name = "))
            .unwrap()
            .trim_matches('"')
            .replace('-', "_");
        let mut dependencies = BTreeSet::from(["std".into(), "core".into(), "alloc".into()]);
        let mut in_dependencies = false;
        for line in text.lines().map(str::trim) {
            if line.starts_with('[') {
                in_dependencies = line.contains("dependencies");
            }
            if in_dependencies {
                if let Some((dep, _)) = line.split_once('=') {
                    dependencies.insert(dep.trim().replace('-', "_"));
                }
            }
        }
        source.dependencies.insert(name.clone(), dependencies);
        files(&Path::new(directory).join("src"), &mut source);
        for (file, id) in [("lib.rs", name.clone()), ("main.rs", format!("{name}_bin"))] {
            let file = format!("{directory}/src/{file}");
            if source.files.contains_key(&file) {
                roots.push((file, id));
            }
        }
        let prefix = format!("{directory}/src/bin/");
        for file in source.files.keys().filter(|p| p.starts_with(&prefix)) {
            let rest = file.strip_prefix(&prefix).unwrap();
            if !rest.contains('/') && rest.ends_with(".rs") {
                roots.push((
                    file.clone(),
                    format!(
                        "{name}_bin_{}",
                        rest.trim_end_matches(".rs").replace('-', "_")
                    ),
                ));
            }
        }
    }
    let mut loader = Loader {
        source,
        visits: BTreeSet::new(),
        active_files: BTreeSet::new(),
    };
    for (file, id) in roots {
        loader.file(&file, &id, false, false);
    }
    let missing: Vec<_> = loader
        .source
        .files
        .keys()
        .filter(|p| !loader.source.reached.contains(*p))
        .cloned()
        .collect();
    for path in missing {
        loader
            .source
            .errors
            .push(format!("unreconciled source file: {path}"));
    }
    finish(loader.source)
}

pub(super) fn fixture(files: &[(&str, &str, &str)]) -> Source {
    let mut loader = Loader {
        source: Source::default(),
        visits: BTreeSet::new(),
        active_files: BTreeSet::new(),
    };
    loader
        .source
        .modules
        .insert("fixture".into(), Module::default());
    loader.source.dependencies.insert(
        "fixture".into(),
        BTreeSet::from(["std".into(), "core".into(), "alloc".into()]),
    );
    for (module, file, text) in files {
        loader.source.files.insert((*file).into(), (*text).into());
        let id = format!("fixture::{module}");
        bind(&mut loader.source, "fixture", module, &id, None, false);
        loader.file(file, &id, false, false);
    }
    finish(loader.source)
}

fn finish(mut source: Source) -> Source {
    resolve::imports(&mut source);
    resolve::impls(&mut source);
    source
}

pub(super) fn cfg(meta: &syn::Meta) -> Option<bool> {
    match meta {
        syn::Meta::Path(path) if path.is_ident("test") => Some(false),
        syn::Meta::List(list) => {
            let entries =
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
                    .parse2(list.tokens.clone())
                    .ok()?;
            let values: Vec<_> = entries.iter().map(cfg).collect();
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

pub(super) fn test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("test")
            || (a.path().is_ident("cfg")
                && a.parse_args::<syn::Meta>().ok().as_ref().and_then(cfg) == Some(false))
    })
}

pub(super) fn conditional(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg_attr")
            || (a.path().is_ident("cfg")
                && a.parse_args::<syn::Meta>()
                    .ok()
                    .as_ref()
                    .and_then(cfg)
                    .is_none())
    })
}

pub(super) fn attrs(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Fn(i) => &i.attrs,
        syn::Item::Mod(i) => &i.attrs,
        syn::Item::Struct(i) => &i.attrs,
        syn::Item::Enum(i) => &i.attrs,
        syn::Item::Type(i) => &i.attrs,
        syn::Item::Impl(i) => &i.attrs,
        syn::Item::Use(i) => &i.attrs,
        syn::Item::Const(i) => &i.attrs,
        syn::Item::Static(i) => &i.attrs,
        syn::Item::Trait(i) => &i.attrs,
        syn::Item::Macro(i) => &i.attrs,
        syn::Item::ForeignMod(i) => &i.attrs,
        syn::Item::ExternCrate(i) => &i.attrs,
        syn::Item::Union(i) => &i.attrs,
        syn::Item::TraitAlias(i) => &i.attrs,
        _ => &[],
    }
}

pub(super) fn boundary(vis: &syn::Visibility, module: &str) -> Option<Id> {
    match vis {
        syn::Visibility::Public(_) => None,
        syn::Visibility::Inherited => Some(module.into()),
        syn::Visibility::Restricted(v) => Some(resolve::relative(
            module,
            &v.path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>(),
        )),
    }
}

fn bind(
    source: &mut Source,
    module: &str,
    name: &str,
    id: &str,
    boundary: Option<Id>,
    conditional: bool,
) {
    source
        .modules
        .entry(module.into())
        .or_default()
        .bindings
        .entry(name.into())
        .or_default()
        .insert(Binding {
            target: id.into(),
            boundary,
            glob: false,
            conditional,
        });
}

struct Loader {
    source: Source,
    visits: BTreeSet<(String, String, bool)>,
    active_files: BTreeSet<String>,
}

impl Loader {
    fn file(&mut self, file: &str, module: &str, test: bool, conditional: bool) {
        let canonical = normalized(Path::new(file));
        let file = canonical.as_str();
        if self.active_files.contains(file) {
            self.source
                .errors
                .push(format!("recursive source path: {file} in {module}"));
            return;
        }
        if !self.visits.insert((file.into(), module.into(), test)) {
            return;
        }
        self.source.reached.insert(file.into());
        if !test {
            self.source.production_files.insert(file.into());
        }
        let Some(text) = self.source.files.get(file) else {
            self.source.errors.push(format!("missing source {file}"));
            return;
        };
        let parsed = match syn::parse_file(text) {
            Ok(parsed) => parsed,
            Err(e) => {
                self.source.errors.push(format!("{file}: {e}"));
                return;
            }
        };
        self.active_files.insert(file.into());
        let path = Path::new(file);
        let dir = if matches!(
            path.file_stem().and_then(|s| s.to_str()),
            Some("lib" | "main" | "mod")
        ) {
            path.parent().unwrap().to_path_buf()
        } else {
            path.with_extension("")
        };
        self.items(
            file,
            module,
            &dir,
            &parsed.items,
            test,
            conditional || conditional_attrs(&parsed.attrs),
        );
        self.active_files.remove(file);
    }

    fn items(
        &mut self,
        file: &str,
        module: &str,
        dir: &Path,
        items: &[syn::Item],
        test: bool,
        parent_cfg: bool,
    ) {
        if !test {
            self.source.modules.entry(module.into()).or_default();
        }
        for item in items {
            let a = attrs(item);
            let excluded = test || test_only(a);
            let uncertain = parent_cfg || conditional(a);
            let site = Site::new(file, item.span());
            if let syn::Item::Mod(m) = item {
                let child = format!("{module}::{}", m.ident);
                if !excluded {
                    bind(
                        &mut self.source,
                        module,
                        &m.ident.to_string(),
                        &child,
                        boundary(&m.vis, module),
                        uncertain,
                    );
                }
                if let Some((_, children)) = &m.content {
                    self.items(
                        file,
                        &child,
                        &dir.join(m.ident.to_string()),
                        children,
                        excluded,
                        uncertain,
                    );
                } else {
                    let explicit =
                        m.attrs
                            .iter()
                            .find(|a| a.path().is_ident("path"))
                            .and_then(|a| match &a.meta {
                                syn::Meta::NameValue(n) => match &n.value {
                                    syn::Expr::Lit(syn::ExprLit {
                                        lit: syn::Lit::Str(s),
                                        ..
                                    }) => Some(s.value()),
                                    _ => None,
                                },
                                _ => None,
                            });
                    let paths = if let Some(p) = explicit {
                        let normal = if matches!(
                            Path::new(file).file_stem().and_then(|s| s.to_str()),
                            Some("lib" | "main" | "mod")
                        ) {
                            Path::new(file).parent().unwrap().to_path_buf()
                        } else {
                            Path::new(file).with_extension("")
                        };
                        vec![normalized(&if dir == normal {
                            Path::new(file).parent().unwrap().join(p)
                        } else {
                            dir.join(p)
                        })]
                    } else {
                        vec![
                            normalized(&dir.join(format!("{}.rs", m.ident))),
                            normalized(&dir.join(m.ident.to_string()).join("mod.rs")),
                        ]
                    };
                    let found: Vec<_> = paths
                        .into_iter()
                        .filter(|p| self.source.files.contains_key(p))
                        .collect();
                    if found.len() != 1 {
                        self.source.errors.push(format!(
                            "{}: {} module candidates",
                            site.display(),
                            found.len()
                        ));
                    } else {
                        self.file(&found[0], &child, excluded, uncertain);
                    }
                }
                continue;
            }
            // include! is followed solely for test-file reconciliation. It is
            // never expanded to prove a production function or binding.
            if excluded {
                if let syn::Item::Macro(m) = item {
                    if m.mac.path.is_ident("include") {
                        if let Ok(lit) = syn::parse2::<syn::LitStr>(m.mac.tokens.clone()) {
                            let path =
                                normalized(&Path::new(file).parent().unwrap().join(lit.value()));
                            self.file(&path, module, true, uncertain);
                        }
                    }
                }
                continue;
            }
            let (ident, vis) = match item {
                syn::Item::Fn(f) => {
                    let id = format!("{module}::{}", f.sig.ident);
                    self.source
                        .functions
                        .entry(id.clone())
                        .or_default()
                        .push(Function {
                            id,
                            module: module.into(),
                            site: site.clone(),
                            sig: f.sig.clone(),
                            body: (*f.block).clone(),
                            attrs: f.attrs.clone(),
                            owner: None,
                            conditional: uncertain,
                        });
                    (&f.sig.ident, &f.vis)
                }
                syn::Item::Struct(s) => {
                    self.nominal(
                        module,
                        site,
                        &s.ident,
                        &s.generics,
                        named_fields(&s.fields),
                        BTreeMap::new(),
                        None,
                        &s.attrs,
                        uncertain,
                    );
                    (&s.ident, &s.vis)
                }
                syn::Item::Enum(e) => {
                    self.nominal(
                        module,
                        site,
                        &e.ident,
                        &e.generics,
                        Vec::new(),
                        e.variants
                            .iter()
                            .map(|v| (v.ident.to_string(), named_fields(&v.fields)))
                            .collect(),
                        None,
                        &e.attrs,
                        uncertain,
                    );
                    (&e.ident, &e.vis)
                }
                syn::Item::Type(t) => {
                    self.nominal(
                        module,
                        site,
                        &t.ident,
                        &t.generics,
                        Vec::new(),
                        BTreeMap::new(),
                        Some((*t.ty).clone()),
                        &t.attrs,
                        uncertain,
                    );
                    (&t.ident, &t.vis)
                }
                syn::Item::Impl(i) => {
                    self.source
                        .impls
                        .push((module.into(), i.clone(), site, uncertain));
                    continue;
                }
                syn::Item::Use(u) => {
                    resolve::collect_imports(
                        &mut self.source,
                        module,
                        &u.tree,
                        Vec::new(),
                        boundary(&u.vis, module),
                        uncertain,
                    );
                    continue;
                }
                syn::Item::Const(c) => {
                    self.source
                        .constants
                        .push((module.into(), file.into(), (*c.expr).clone()));
                    (&c.ident, &c.vis)
                }
                syn::Item::Static(s) => (&s.ident, &s.vis),
                syn::Item::Trait(t) => (&t.ident, &t.vis),
                _ => {
                    self.source
                        .modules
                        .get_mut(module)
                        .unwrap()
                        .hazards
                        .push(site);
                    continue;
                }
            };
            bind(
                &mut self.source,
                module,
                &ident.to_string(),
                &format!("{module}::{ident}"),
                boundary(vis, module),
                uncertain,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn nominal(
        &mut self,
        module: &str,
        site: Site,
        ident: &syn::Ident,
        generics: &syn::Generics,
        fields: Vec<Field>,
        variants: BTreeMap<String, Vec<Field>>,
        alias: Option<syn::Type>,
        attrs: &[syn::Attribute],
        conditional: bool,
    ) {
        self.source
            .types
            .entry(format!("{module}::{ident}"))
            .or_default()
            .push(Nominal {
                module: module.into(),
                site,
                generics: generics.clone(),
                fields,
                variants,
                alias,
                attrs: attrs.to_vec(),
                conditional,
            });
    }
}

fn conditional_attrs(attrs: &[syn::Attribute]) -> bool {
    conditional(attrs)
}

fn named_fields(fields: &syn::Fields) -> Vec<Field> {
    fields
        .iter()
        .enumerate()
        .map(|(i, field)| Field {
            name: field
                .ident
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| i.to_string()),
            ty: field.ty.clone(),
            conditional: conditional_attrs(&field.attrs),
        })
        .collect()
}
