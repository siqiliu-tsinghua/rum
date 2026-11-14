//! Syntax collection is independent of the accepted inventory and of factories.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    parse::Parser,
    spanned::Spanned,
    visit::{self, Visit},
};

use super::{identity::*, resolve};

#[derive(Clone)]
pub struct SourceFile {
    pub path: String,
    pub module: String,
    pub text: String,
}

#[derive(Clone, Default)]
pub struct SourceTree {
    pub files: Vec<SourceFile>,
}

impl SourceTree {
    pub fn workspace(root: &Path) -> Self {
        fn walk(root: &Path, paths: &mut Vec<std::path::PathBuf>) {
            let mut children: Vec<_> = fs::read_dir(root)
                .unwrap()
                .map(|e| e.unwrap().path())
                .collect();
            children.sort();
            for path in children {
                let name = path.file_name().unwrap().to_string_lossy();
                if name == "tests" || name == "tests.rs" || name.starts_with("tests_") {
                    continue;
                }
                if path.is_dir() {
                    walk(&path, paths);
                } else if path.extension().is_some_and(|s| s == "rs") {
                    paths.push(path);
                }
            }
        }
        let mut paths = Vec::new();
        let mut crates: Vec<_> = fs::read_dir(root.join("crates"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        crates.sort();
        for krate in crates {
            if krate.join("src").is_dir() {
                walk(&krate.join("src"), &mut paths);
            }
        }
        Self {
            files: paths
                .into_iter()
                .map(|path| {
                    let rel = path
                        .strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();
                    let components: Vec<_> = rel.split('/').collect();
                    let mut module = components[1].replace('-', "_");
                    for part in &components[3..] {
                        let name = part.trim_end_matches(".rs");
                        if !matches!(name, "lib" | "mod" | "main") {
                            module.push_str("::");
                            module.push_str(name);
                        }
                    }
                    SourceFile {
                        path: rel,
                        module,
                        text: fs::read_to_string(path).unwrap(),
                    }
                })
                .collect(),
        }
    }
}

#[derive(Clone)]
pub(super) struct Function {
    pub module: String,
    pub owner: String,
    pub trait_name: String,
    pub sig: syn::Signature,
    pub body: Option<syn::Block>,
    pub span: SourceSpan,
    pub id: ProducerId,
    pub legacy: String,
}

#[derive(Default)]
pub(super) struct Index {
    pub functions: Vec<Function>,
    pub symbols: BTreeMap<String, BTreeSet<String>>,
    pub uses: Vec<(String, String, String)>,
    pub visibility: BTreeMap<String, syn::Visibility>,
    pub import_visibility: BTreeMap<(String, String), syn::Visibility>,
    pub modules: BTreeSet<String>,
    pub path_cache: std::cell::RefCell<BTreeMap<(String, String), BTreeSet<String>>>,
    pub fields: BTreeMap<(String, String), (String, syn::Type)>,
    pub diagnostics: Diagnostics,
    pub destruction: Vec<String>,
    pub owners: Vec<BTreeSet<String>>,
    pub methods: BTreeMap<String, Vec<usize>>,
    pub functions_by_name: BTreeMap<String, Vec<usize>>,
}

impl Index {
    fn symbol(&mut self, module: &str, name: &str) {
        let key = format!("{module}::{name}");
        self.symbols.entry(key.clone()).or_default().insert(key);
    }
    fn imports(&mut self, module: &str, prefix: &str, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(p) => {
                self.imports(module, &format!("{prefix}{}::", p.ident), &p.tree)
            }
            syn::UseTree::Name(n) => self.uses.push((
                module.into(),
                n.ident.to_string(),
                format!("{prefix}{}", n.ident),
            )),
            syn::UseTree::Rename(n) => self.uses.push((
                module.into(),
                n.rename.to_string(),
                format!("{prefix}{}", n.ident),
            )),
            syn::UseTree::Glob(_) => self.uses.push((
                module.into(),
                "*".into(),
                prefix.trim_end_matches("::").into(),
            )),
            syn::UseTree::Group(g) => {
                for tree in &g.items {
                    self.imports(module, prefix, tree);
                }
            }
        }
    }
    fn function(
        &mut self,
        module: &str,
        file: &str,
        owner: &str,
        trait_name: &str,
        sig: &syn::Signature,
        body: Option<&syn::Block>,
        attrs: &[syn::Attribute],
    ) {
        let name = sig.ident.to_string();
        let qualified = if owner.is_empty() {
            format!("{module}::{name}")
        } else {
            format!("{module}::<{owner} as {trait_name}>::{name}")
        };
        let arm = attrs
            .iter()
            .filter(|a| a.path().is_ident("cfg"))
            .map(normalized)
            .collect::<Vec<_>>()
            .join(";");
        self.functions.push(Function {
            module: module.into(),
            owner: owner.into(),
            trait_name: trait_name.into(),
            sig: sig.clone(),
            body: body.cloned(),
            span: location(file, sig.span()),
            id: ProducerId {
                qualified,
                signature: normalized(sig),
                arm,
            },
            legacy: format!("{file}:{owner}::{name}"),
        });
        if owner.is_empty() {
            self.symbol(module, &name);
        }
    }
    fn items(&mut self, module: &str, file: &str, items: &[syn::Item]) {
        self.modules.insert(module.into());
        for item in items {
            match item {
                syn::Item::Use(u) => {
                    let start = self.uses.len();
                    self.imports(module, "", &u.tree);
                    for (_, alias, target) in &self.uses[start..] {
                        self.import_visibility
                            .insert((module.into(), format!("{alias}:{target}")), u.vis.clone());
                    }
                }
                syn::Item::Mod(m) => {
                    self.symbol(module, &m.ident.to_string());
                    self.visibility
                        .insert(format!("{module}::{}", m.ident), m.vis.clone());
                    if let Some((_, items)) = &m.content {
                        if !test_only(&m.attrs) {
                            self.items(&format!("{module}::{}", m.ident), file, items);
                        }
                    }
                    // Path overrides need an explicit module-tree port; never
                    // manufacture a qualified key from their physical filename.
                    if m.attrs.iter().any(|a| a.path().is_ident("path")) {
                        self.diagnostics.push(Diagnostic::new(
                            Kind::UnknownConstruct,
                            location(file, m.span()).display(),
                            "module-path-override",
                        ));
                    }
                }
                syn::Item::Fn(f) if !test_only(&f.attrs) => {
                    self.function(module, file, "", "", &f.sig, Some(&f.block), &f.attrs);
                    self.visibility
                        .insert(format!("{module}::{}", f.sig.ident), f.vis.clone());
                }
                syn::Item::Impl(i) if !test_only(&i.attrs) => {
                    let owner = normalized(&i.self_ty);
                    let tr = i
                        .trait_
                        .as_ref()
                        .map(|(_, p, _)| normalized(p))
                        .unwrap_or_else(|| "inherent".into());
                    if tr == "Drop" {
                        self.destruction.push(format!("{module}:{}", normalized(i)));
                    }
                    for item in &i.items {
                        if let syn::ImplItem::Fn(f) = item {
                            if !test_only(&f.attrs) {
                                self.function(
                                    module,
                                    file,
                                    &owner,
                                    &tr,
                                    &f.sig,
                                    Some(&f.block),
                                    &f.attrs,
                                );
                            }
                        }
                    }
                }
                syn::Item::Trait(t) => {
                    self.symbol(module, &t.ident.to_string());
                    self.visibility
                        .insert(format!("{module}::{}", t.ident), t.vis.clone());
                    for item in &t.items {
                        if let syn::TraitItem::Fn(f) = item {
                            self.function(
                                module,
                                file,
                                "Self",
                                &t.ident.to_string(),
                                &f.sig,
                                f.default.as_ref(),
                                &f.attrs,
                            );
                        }
                    }
                }
                syn::Item::Struct(s) => {
                    self.symbol(module, &s.ident.to_string());
                    self.visibility
                        .insert(format!("{module}::{}", s.ident), s.vis.clone());
                    self.destruction.push(format!("{module}:{}", normalized(s)));
                    for (i, f) in s.fields.iter().enumerate() {
                        self.fields.insert(
                            (
                                format!("{module}::{}", s.ident),
                                f.ident
                                    .as_ref()
                                    .map(ToString::to_string)
                                    .unwrap_or_else(|| i.to_string()),
                            ),
                            (module.into(), f.ty.clone()),
                        );
                    }
                }
                syn::Item::Enum(e) => {
                    self.symbol(module, &e.ident.to_string());
                    self.visibility
                        .insert(format!("{module}::{}", e.ident), e.vis.clone());
                    self.destruction.push(format!("{module}:{}", normalized(e)));
                }
                syn::Item::Type(t) => {
                    self.symbol(module, &t.ident.to_string());
                    self.visibility
                        .insert(format!("{module}::{}", t.ident), t.vis.clone());
                    self.destruction.push(format!("{module}:{}", normalized(t)));
                }
                syn::Item::Macro(m) if m.ident.is_none() => self.diagnostics.push(Diagnostic::new(
                    Kind::UnknownConstruct,
                    location(file, m.span()).display(),
                    format!("item-macro:{}", normalized(&m.mac.path)),
                )),
                syn::Item::Macro(m) => self.diagnostics.push(Diagnostic::new(
                    Kind::UnknownConstruct,
                    location(file, m.span()).display(),
                    format!(
                        "macro-definition:{}:{}",
                        m.ident.as_ref().unwrap(),
                        digest(normalized(m).as_bytes())
                    ),
                )),
                _ => {}
            }
        }
    }
}

fn test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("test")
            || (a.path().is_ident("cfg") && normalized(&a.meta) == "cfg (test)")
    })
}

pub(super) fn location(file: &str, span: Span) -> SourceSpan {
    SourceSpan {
        file: file.into(),
        line: span.start().line,
        column: span.start().column + 1,
    }
}

#[derive(Clone)]
pub(super) struct Call {
    pub function: usize,
    pub expression: syn::Expr,
    pub locals: BTreeMap<String, syn::Type>,
    pub tokens: String,
    pub span: SourceSpan,
    pub occurrence: usize,
    pub strict: bool,
    pub tail: String,
    pub pass: String,
}

struct Calls<'a> {
    index: &'a Index,
    function: usize,
    locals: BTreeMap<String, syn::Type>,
    seen: BTreeMap<String, usize>,
    calls: Vec<Call>,
    diagnostics: Diagnostics,
    strict: BTreeSet<(usize, usize)>,
    pass: String,
}

fn tails(expr: &syn::Expr, out: &mut BTreeSet<(usize, usize)>) {
    match expr {
        syn::Expr::Call(_) | syn::Expr::MethodCall(_) => {
            out.insert((expr.span().start().line, expr.span().start().column));
        }
        syn::Expr::Return(r) => {
            if let Some(e) = &r.expr {
                tails(e, out);
            }
        }
        syn::Expr::Block(b) => block_tail(&b.block, out),
        syn::Expr::If(i) => {
            block_tail(&i.then_branch, out);
            if let Some((_, e)) = &i.else_branch {
                tails(e, out);
            }
        }
        syn::Expr::Match(m) => {
            for a in &m.arms {
                tails(&a.body, out);
            }
        }
        syn::Expr::Paren(p) => tails(&p.expr, out),
        syn::Expr::Group(g) => tails(&g.expr, out),
        _ => {}
    }
}
fn block_tail(block: &syn::Block, out: &mut BTreeSet<(usize, usize)>) {
    if let Some(syn::Stmt::Expr(expr, None)) = block.stmts.last() {
        tails(expr, out);
    }
}

fn suffix(stream: TokenStream, end: proc_macro2::LineColumn, out: &mut Vec<String>) {
    for token in stream {
        if let TokenTree::Group(g) = token {
            if (g.span_open().start().line, g.span_open().start().column) >= (end.line, end.column)
            {
                out.push(format!("{:?}:open", g.delimiter()));
            }
            suffix(g.stream(), end, out);
            if (g.span_close().start().line, g.span_close().start().column)
                >= (end.line, end.column)
            {
                out.push(format!("{:?}:close", g.delimiter()));
            }
        } else if (token.span().start().line, token.span().start().column) >= (end.line, end.column)
        {
            out.push(token.to_string());
        }
    }
}

impl Calls<'_> {
    fn shadow(&mut self, pattern: &syn::Pat) {
        struct Bindings(Vec<String>);
        impl<'a> Visit<'a> for Bindings {
            fn visit_pat_ident(&mut self, pat: &'a syn::PatIdent) {
                self.0.push(pat.ident.to_string());
                visit::visit_pat_ident(self, pat);
            }
        }
        let mut names = Bindings(Vec::new());
        names.visit_pat(pattern);
        for name in names.0 {
            self.locals.insert(name, syn::parse_str("_").unwrap());
        }
        if let syn::Pat::Type(typed) = pattern {
            if let syn::Pat::Ident(id) = &*typed.pat {
                self.locals
                    .insert(id.ident.to_string(), (*typed.ty).clone());
            }
        }
    }
    fn record(&mut self, expr: syn::Expr) {
        let f = &self.index.functions[self.function];
        let tokens = normalized(&expr);
        let occurrence = self.seen.entry(tokens.clone()).or_default();
        let mut after = Vec::new();
        suffix(
            f.body.as_ref().unwrap().to_token_stream(),
            expr.span().end(),
            &mut after,
        );
        self.calls.push(Call {
            function: self.function,
            span: location(&f.span.file, expr.span()),
            strict: self
                .strict
                .contains(&(expr.span().start().line, expr.span().start().column)),
            tail: digest(after.join(" ").as_bytes()),
            tokens,
            occurrence: *occurrence,
            expression: expr,
            locals: self.locals.clone(),
            pass: self.pass.clone(),
        });
        *occurrence += 1;
    }
}

impl<'ast> Visit<'ast> for Calls<'_> {
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.diagnostics.push(Diagnostic::new(
            Kind::UnknownConstruct,
            location(&self.index.functions[self.function].span.file, f.span()).display(),
            "nested-function-scope",
        ));
    }
    fn visit_block(&mut self, b: &'ast syn::Block) {
        let old = self.locals.clone();
        visit::visit_block(self, b);
        self.locals = old;
    }
    fn visit_local(&mut self, l: &'ast syn::Local) {
        // The initializer sees the old binding; the new binding starts afterwards.
        visit::visit_local(self, l);
        self.shadow(&l.pat);
        if let syn::Pat::Type(p) = &l.pat {
            if let syn::Pat::Ident(id) = &*p.pat {
                self.locals.insert(id.ident.to_string(), (*p.ty).clone());
            }
        } else if let (syn::Pat::Ident(id), Some(init)) = (&l.pat, &l.init) {
            if let Some(ty) = resolve::infer(self.index, self.function, &self.locals, &init.expr) {
                self.locals.insert(id.ident.to_string(), ty);
            }
        }
    }
    fn visit_expr_closure(&mut self, c: &'ast syn::ExprClosure) {
        let old = self.locals.clone();
        for pattern in &c.inputs {
            self.shadow(pattern);
        }
        self.visit_expr(&c.body);
        self.locals = old;
    }
    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let old = self.locals.clone();
        self.shadow(&arm.pat);
        if let Some((_, guard)) = &arm.guard {
            self.visit_expr(guard);
        }
        self.visit_expr(&arm.body);
        self.locals = old;
    }
    fn visit_expr_for_loop(&mut self, loop_: &'ast syn::ExprForLoop) {
        self.visit_expr(&loop_.expr);
        let old = self.locals.clone();
        self.shadow(&loop_.pat);
        self.visit_block(&loop_.body);
        self.locals = old;
    }
    fn visit_expr_if(&mut self, branch: &'ast syn::ExprIf) {
        self.visit_expr(&branch.cond);
        let old = self.locals.clone();
        if let syn::Expr::Let(binding) = &*branch.cond {
            self.shadow(&binding.pat);
        }
        self.visit_block(&branch.then_branch);
        self.locals = old;
        if let Some((_, other)) = &branch.else_branch {
            self.visit_expr(other);
        }
    }
    fn visit_expr_while(&mut self, loop_: &'ast syn::ExprWhile) {
        self.visit_expr(&loop_.cond);
        let old = self.locals.clone();
        if let syn::Expr::Let(binding) = &*loop_.cond {
            self.shadow(&binding.pat);
        }
        self.visit_block(&loop_.body);
        self.locals = old;
    }
    fn visit_expr_return(&mut self, r: &'ast syn::ExprReturn) {
        if let Some(e) = &r.expr {
            tails(e, &mut self.strict);
        }
        visit::visit_expr_return(self, r);
    }
    fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
        self.record(syn::Expr::Call(c.clone()));
        visit::visit_expr_call(self, c);
    }
    fn visit_expr_method_call(&mut self, c: &'ast syn::ExprMethodCall) {
        self.record(syn::Expr::MethodCall(c.clone()));
        visit::visit_expr_method_call(self, c);
    }
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let name = m.path.segments.last().unwrap().ident.to_string();
        let parsed = if matches!(
            name.as_str(),
            "resume_or_suspend" | "resume_typed_or_suspend"
        ) {
            resume_arguments.parse2(m.tokens.clone())
        } else {
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
                .parse2(m.tokens.clone())
                .map(|x| x.into_iter().collect())
        };
        match parsed {
            Ok(expressions) => {
                let old = std::mem::replace(
                    &mut self.pass,
                    if name == "resume_typed_or_suspend" {
                        "typed-macro"
                    } else {
                        "macro"
                    }
                    .into(),
                );
                for expr in &expressions {
                    self.visit_expr(expr);
                }
                self.pass = old;
                if !matches!(
                    name.as_str(),
                    "resume_or_suspend"
                        | "resume_typed_or_suspend"
                        | "vec"
                        | "format"
                        | "format_args"
                        | "println"
                        | "eprintln"
                        | "print"
                        | "write"
                        | "writeln"
                        | "panic"
                        | "assert"
                        | "assert_eq"
                        | "assert_ne"
                        | "debug_assert"
                        | "debug_assert_eq"
                        | "debug_assert_ne"
                ) {
                    self.diagnostics.push(Diagnostic::new(
                        Kind::UnknownConstruct,
                        location(&self.index.functions[self.function].span.file, m.span())
                            .display(),
                        format!("macro-expansion:{name}"),
                    ));
                }
            }
            Err(_) => self.diagnostics.push(Diagnostic::new(
                Kind::UnknownConstruct,
                location(&self.index.functions[self.function].span.file, m.span()).display(),
                format!("macro-grammar:{name}"),
            )),
        }
    }
}

fn resume_arguments(input: syn::parse::ParseStream) -> syn::Result<Vec<syn::Expr>> {
    let mut expressions = Vec::new();
    if input.peek(syn::Ident) && input.fork().parse::<syn::Ident>()? == "insert" {
        let _: syn::Ident = input.parse()?;
        expressions.push(input.parse()?);
        input.parse::<syn::Token![,]>()?;
        expressions.push(input.parse()?);
    } else {
        expressions.push(input.parse()?);
        input.parse::<syn::Token![,]>()?;
        expressions.push(input.parse()?);
        input.parse::<syn::Token![,]>()?;
        let ok: syn::Ident = input.parse()?;
        if ok != "ok" {
            return Err(input.error("expected ok"));
        }
        let pat;
        syn::parenthesized!(pat in input);
        let pattern: syn::Pat = pat.call(syn::Pat::parse_multi_with_leading_vert)?;
        input.parse::<syn::Token![=>]>()?;
        let continuation: syn::Expr = input.parse()?;
        // The success pattern shadows names outside the macro. This synthetic
        // lexical scope preserves the original calls and their source spans.
        expressions.push(syn::parse2(quote::quote!(|#pattern| #continuation))?);
        input.parse::<syn::Token![,]>()?;
        let arm: syn::Ident = input.parse()?;
        if arm != "suspend" && arm != "evaluation_return" {
            return Err(input.error("unknown suspension arm"));
        }
        input.parse::<syn::Token![=>]>()?;
        expressions.push(input.parse()?);
    }
    if input.peek(syn::Token![,]) {
        input.parse::<syn::Token![,]>()?;
    }
    Ok(expressions)
}

pub fn discover(tree: &SourceTree, _mode: FeatureMode) -> DiscoveredGraph {
    // Both feature arms inventory the same source union. cfg never waives an
    // edge; unresolved alternatives are diagnostics, not an arbitrary choice.
    let mut index = Index::default();
    for file in &tree.files {
        match syn::parse_file(&file.text) {
            Ok(ast) => index.items(&file.module, &file.path, &ast.items),
            Err(e) => index.diagnostics.push(Diagnostic::new(
                Kind::UnknownConstruct,
                &file.path,
                e.to_string(),
            )),
        }
    }
    println!(
        "DISCOVERY PARSED files={} functions={}",
        tree.files.len(),
        index.functions.len()
    );
    resolve::prepare(&mut index);
    println!("DISCOVERY IMPORTS symbols={}", index.symbols.len());
    let mut calls = Vec::new();
    let mut diagnostics = index.diagnostics.clone();
    for (function, f) in index.functions.iter().enumerate() {
        let Some(body) = &f.body else {
            continue;
        };
        let mut visitor = Calls {
            index: &index,
            function,
            locals: BTreeMap::new(),
            seen: BTreeMap::new(),
            calls: Vec::new(),
            diagnostics: Vec::new(),
            strict: BTreeSet::new(),
            pass: "lexical".into(),
        };
        for arg in &f.sig.inputs {
            if let syn::FnArg::Typed(arg) = arg {
                if let syn::Pat::Ident(id) = &*arg.pat {
                    visitor
                        .locals
                        .insert(id.ident.to_string(), (*arg.ty).clone());
                }
            }
        }
        if !f.owner.is_empty() {
            if let Ok(ty) = syn::parse_str(&f.owner) {
                visitor.locals.insert("self".into(), ty);
            }
        }
        block_tail(body, &mut visitor.strict);
        visitor.visit_block(body);
        calls.extend(visitor.calls);
        diagnostics.extend(visitor.diagnostics);
    }
    println!("DISCOVERY CALLS count={}", calls.len());
    resolve::graph(index, calls, diagnostics, tree.files.len())
}
