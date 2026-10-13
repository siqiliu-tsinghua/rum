//! AST collection, including lexical scopes and anonymous/generated constructs.

use std::path::PathBuf;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};

use super::{Loader, cfg_value, conditional, test_only};
use crate::tests::ownership_graph::model::*;

pub(super) struct Collector<'a> {
    pub loader: &'a mut Loader,
    pub file: String,
    pub scope: Id,
    pub module: Id,
    pub dir: PathBuf,
    pub test_only: bool,
    pub conditional: bool,
    pub impl_parameters: std::collections::BTreeMap<String, String>,
}

impl Collector<'_> {
    fn location(&self, span: proc_macro2::Span) -> Location {
        Location::new(&self.file, span)
    }

    fn boundary(&self, vis: &syn::Visibility) -> Option<Id> {
        match vis {
            syn::Visibility::Public(_) => None,
            syn::Visibility::Inherited => Some(self.module.clone()),
            syn::Visibility::Restricted(vis) => {
                let mut module: Vec<_> = self.module.split("::").map(str::to_string).collect();
                let mut path = vis.path.segments.iter().map(|s| s.ident.to_string());
                match path.next().as_deref() {
                    Some("crate") => module.truncate(1),
                    Some("super") => {
                        module.pop();
                    }
                    Some("self") => {}
                    _ => unreachable!("Rust restricted visibility starts with crate/self/super"),
                }
                module.extend(path);
                Some(module.join("::"))
            }
        }
    }

    fn bind(&mut self, name: String, target: Id, vis: &syn::Visibility) {
        let binding = Binding {
            target,
            boundary: self.boundary(vis),
            priority: 0,
        };
        self.loader
            .source
            .scopes
            .get_mut(&self.scope)
            .unwrap()
            .bindings
            .entry(name)
            .or_default()
            .insert(binding);
    }

    fn definition(
        &mut self,
        name: &syn::Ident,
        vis: &syn::Visibility,
        generics: &syn::Generics,
        fields: Vec<Field>,
        alias: Option<syn::Type>,
    ) {
        let id = format!("{}::{name}", self.scope);
        self.bind(name.to_string(), id.clone(), vis);
        let definition = Definition {
            scope: self.scope.clone(),
            location: self.location(name.span()),
            generics: generics.clone(),
            fields,
            alias,
        };
        if self
            .loader
            .source
            .definitions
            .insert(id.clone(), definition)
            .is_some()
        {
            self.loader.source.unknown(
                self.location(name.span()),
                format!("ambiguous conditional definition {id}"),
            );
        }
    }

    fn fields(&self, fields: &syn::Fields, prefix: &str) -> Vec<Field> {
        fields
            .iter()
            .enumerate()
            .filter(|(_, f)| !test_only(&f.attrs))
            .map(|(index, field)| Field {
                path: format!(
                    "{prefix}{}",
                    field
                        .ident
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or(index.to_string())
                ),
                ty: field.ty.clone(),
                location: self.location(field.ty.span()),
                conditional: self.conditional || conditional(&field.attrs),
            })
            .collect()
    }

    fn enter(&mut self, name: String, run: impl FnOnce(&mut Self)) {
        let next = format!("{}::{name}", self.scope);
        let previous = std::mem::replace(&mut self.scope, next.clone());
        if !self.test_only {
            self.loader
                .source
                .scopes
                .entry(next)
                .or_insert_with(|| Scope {
                    module: self.module.clone(),
                    lexical_parent: Some(previous.clone()),
                    ..Scope::default()
                });
        }
        run(self);
        self.scope = previous;
    }

    fn signature(&mut self, sig: &syn::Signature, vis: &syn::Visibility) {
        if sig.asyncness.is_some() {
            self.loader.source.unknown(
                self.location(sig.span()),
                format!("async function future captures: {}", sig.ident),
            );
        }
        if !matches!(vis, syn::Visibility::Public(_)) || self.test_only {
            return;
        }
        if self.conditional {
            self.loader.source.unknown(
                self.location(sig.span()),
                format!("conditional public input signature {}", sig.ident),
            );
            return;
        }
        let mut parameters = self.impl_parameters.clone();
        for parameter in sig.generics.type_params() {
            parameters.insert(
                parameter.ident.to_string(),
                format!(
                    "{} {}",
                    spelling(parameter),
                    spelling(&sig.generics.where_clause)
                ),
            );
        }
        for input in &sig.inputs {
            if let syn::FnArg::Typed(arg) = input {
                self.loader.source.boundaries.push(Boundary {
                    scope: self.scope.clone(),
                    location: self.location(arg.ty.span()),
                    name: format!("{}::{}({})", self.scope, sig.ident, spelling(&arg.pat)),
                    ty: (*arg.ty).clone(),
                    parameters: parameters.clone(),
                });
            }
        }
    }

    fn imports(&mut self, tree: &syn::UseTree, prefix: Vec<String>, vis: &syn::Visibility) {
        match tree {
            syn::UseTree::Path(path) => {
                let mut prefix = prefix;
                prefix.push(path.ident.to_string());
                self.imports(&path.tree, prefix, vis);
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.imports(item, prefix.clone(), vis);
                }
            }
            _ => {
                let (path, name) = match tree {
                    syn::UseTree::Name(name) if name.ident == "self" => {
                        let name = prefix.last().cloned();
                        (prefix, name)
                    }
                    syn::UseTree::Name(name) => {
                        let mut path = prefix;
                        path.push(name.ident.to_string());
                        (path, Some(name.ident.to_string()))
                    }
                    syn::UseTree::Rename(rename) => {
                        let mut path = prefix;
                        if rename.ident != "self" {
                            path.push(rename.ident.to_string());
                        }
                        (path, Some(rename.rename.to_string()))
                    }
                    syn::UseTree::Glob(_) => (prefix, None),
                    _ => unreachable!(),
                };
                let import = Import {
                    path,
                    name,
                    boundary: self.boundary(vis),
                };
                self.loader
                    .source
                    .scopes
                    .get_mut(&self.scope)
                    .unwrap()
                    .imports
                    .push(import);
            }
        }
    }
}

fn item_attrs(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Const(i) => &i.attrs,
        syn::Item::Enum(i) => &i.attrs,
        syn::Item::ExternCrate(i) => &i.attrs,
        syn::Item::Fn(i) => &i.attrs,
        syn::Item::ForeignMod(i) => &i.attrs,
        syn::Item::Impl(i) => &i.attrs,
        syn::Item::Macro(i) => &i.attrs,
        syn::Item::Mod(i) => &i.attrs,
        syn::Item::Static(i) => &i.attrs,
        syn::Item::Struct(i) => &i.attrs,
        syn::Item::Trait(i) => &i.attrs,
        syn::Item::TraitAlias(i) => &i.attrs,
        syn::Item::Type(i) => &i.attrs,
        syn::Item::Union(i) => &i.attrs,
        syn::Item::Use(i) => &i.attrs,
        _ => &[],
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let excluded = self.test_only || test_only(item_attrs(item));
        let previous_condition = self.conditional;
        self.conditional |= conditional(item_attrs(item));
        if let syn::Item::Mod(module) = item {
            let parent = if self.scope == self.module {
                self.module.clone()
            } else {
                self.scope.clone()
            };
            if !excluded && self.scope != self.module {
                self.loader.source.unknown(
                    self.location(module.span()),
                    format!(
                        "unresolved function/block-local module scope: {}",
                        module.ident
                    ),
                );
                self.conditional = true;
            }
            if !excluded {
                self.bind(
                    module.ident.to_string(),
                    format!("{parent}::{}", module.ident),
                    &module.vis,
                );
                for attr in &module.attrs {
                    self.visit_attribute(attr);
                }
            }
            self.loader.child(
                &self.file,
                &parent,
                &self.dir,
                module,
                excluded,
                self.conditional,
            );
        } else if !excluded {
            if matches!(item, syn::Item::Verbatim(_)) {
                self.loader.source.unknown(
                    self.location(item.span()),
                    format!("unparsed item {}", spelling(item)),
                );
            }
            visit::visit_item(self, item);
        }
        self.conditional = previous_condition;
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.definition(
            &item.ident,
            &item.vis,
            &item.generics,
            self.fields(&item.fields, ""),
            None,
        );
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        let fields = item
            .variants
            .iter()
            .filter(|v| !test_only(&v.attrs))
            .flat_map(|v| {
                self.fields(&v.fields, &format!("{}.", v.ident))
                    .into_iter()
                    .map(|mut field| {
                        field.conditional |= conditional(&v.attrs);
                        field
                    })
            })
            .collect();
        self.definition(&item.ident, &item.vis, &item.generics, fields, None);
        visit::visit_item_enum(self, item);
    }

    fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
        self.definition(&item.ident, &item.vis, &item.generics, Vec::new(), None);
        self.loader.source.unknown(
            self.location(item.span()),
            format!("union storage {}", item.ident),
        );
        visit::visit_item_union(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        self.definition(
            &item.ident,
            &item.vis,
            &item.generics,
            Vec::new(),
            Some((*item.ty).clone()),
        );
        visit::visit_item_type(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        self.bind(
            item.ident.to_string(),
            format!("{}::{}", self.scope, item.ident),
            &item.vis,
        );
        visit::visit_item_trait(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.imports(&item.tree, Vec::new(), &item.vis);
        for attr in &item.attrs {
            self.visit_attribute(attr);
        }
    }

    fn visit_item_trait_alias(&mut self, item: &'ast syn::ItemTraitAlias) {
        self.loader.source.unknown(
            self.location(item.span()),
            format!("unsupported trait alias {}", spelling(item)),
        );
    }

    fn visit_item_foreign_mod(&mut self, item: &'ast syn::ItemForeignMod) {
        self.loader.source.unknown(
            self.location(item.span()),
            "foreign declarations with unmodeled ownership".into(),
        );
        visit::visit_item_foreign_mod(self, item);
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        self.loader.source.unknown(
            self.location(item.span()),
            format!("unresolved extern crate binding {}", spelling(item)),
        );
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.signature(&item.sig, &item.vis);
        self.enter(item.sig.ident.to_string(), |this| {
            visit::visit_item_fn(this, item)
        });
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let parameters = item
            .generics
            .type_params()
            .map(|parameter| {
                (
                    parameter.ident.to_string(),
                    format!(
                        "{} {}",
                        spelling(parameter),
                        spelling(&item.generics.where_clause)
                    ),
                )
            })
            .collect();
        let previous_parameters = std::mem::replace(&mut self.impl_parameters, parameters);
        if let Some((negative, path, _)) = &item.trait_ {
            self.loader.source.drops.push(DropImpl {
                scope: self.scope.clone(),
                ty: (*item.self_ty).clone(),
                trait_path: path.clone(),
                generics: item.generics.clone(),
                positive: negative.is_none(),
                conditional: self.conditional || conditional(&item.attrs),
                location: self.location(item.span()),
            });
        }
        let owner = format!(
            "impl({})@{}",
            spelling(&item.self_ty),
            item.impl_token.span.start().line
        );
        self.enter(owner, |this| visit::visit_item_impl(this, item));
        self.impl_parameters = previous_parameters;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        if test_only(&item.attrs) {
            return;
        }
        self.signature(&item.sig, &item.vis);
        self.enter(item.sig.ident.to_string(), |this| {
            visit::visit_impl_item_fn(this, item)
        });
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let start = block.brace_token.span.open().start();
        self.enter(
            format!("block@{}:{}", start.line, start.column + 1),
            |this| visit::visit_block(this, block),
        );
    }

    fn visit_expr_closure(&mut self, expr: &'ast syn::ExprClosure) {
        self.loader.source.unknown(
            self.location(expr.span()),
            format!(
                "closure captures: {}|{}| (capture types/modes unresolved)",
                if expr.capture.is_some() { "move " } else { "" },
                spelling(&expr.inputs)
            ),
        );
        visit::visit_expr_closure(self, expr);
    }

    fn visit_expr_async(&mut self, expr: &'ast syn::ExprAsync) {
        self.loader.source.unknown(
            self.location(expr.span()),
            "async block captures (capture types/modes unresolved)".into(),
        );
        visit::visit_expr_async(self, expr);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if !test_only(&local.attrs) {
            visit::visit_local(self, local);
        }
    }

    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        macro_rules! attributes {
            ($($variant:ident),* $(,)?) => { match expr {
                $(syn::Expr::$variant(value) => &value.attrs[..],)*
                _ => &[],
            } };
        }
        let attrs = attributes!(
            Array, Assign, Async, Await, Binary, Block, Break, Call, Cast, Closure, Const,
            Continue, Field, ForLoop, Group, If, Index, Infer, Let, Lit, Loop, Macro, Match,
            MethodCall, Paren, Path, Range, RawAddr, Reference, Repeat, Return, Struct, Try,
            TryBlock, Tuple, Unary, Unsafe, While, Yield
        );
        if test_only(attrs) {
            return;
        }
        if matches!(expr, syn::Expr::Verbatim(_)) {
            self.loader.source.unknown(
                self.location(expr.span()),
                format!("unparsed expression {}", spelling(expr)),
            );
        }
        let previous_condition = self.conditional;
        self.conditional |= conditional(attrs);
        visit::visit_expr(self, expr);
        self.conditional = previous_condition;
    }

    fn visit_stmt_macro(&mut self, stmt: &'ast syn::StmtMacro) {
        if !test_only(&stmt.attrs) {
            visit::visit_stmt_macro(self, stmt);
        }
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        if !test_only(&field.attrs) {
            visit::visit_field(self, field);
        }
    }

    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        if !test_only(&variant.attrs) {
            visit::visit_variant(self, variant);
        }
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        self.loader.source.unknown(
            self.location(mac.span()),
            format!(
                "unexpanded macro {}! (ends at {}:{})",
                spelling(&mac.path),
                mac.span().end().line,
                mac.span().end().column + 1
            ),
        );
    }

    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        let name = spelling(attr.path());
        let unknown = match name.as_str() {
            "cfg" => attr
                .parse_args::<syn::Meta>()
                .ok()
                .as_ref()
                .and_then(cfg_value)
                .is_none(),
            "cfg_attr" | "derive" => true,
            "doc" | "allow" | "warn" | "deny" | "forbid" | "inline" | "cold" | "must_use"
            | "repr" | "non_exhaustive" | "deprecated" | "track_caller" | "path"
            | "macro_export" => false,
            _ => true,
        };
        if unknown {
            self.loader.source.unknown(
                self.location(attr.span()),
                format!("unresolved attribute/expansion {}", spelling(&attr.meta)),
            );
        }
    }
}
