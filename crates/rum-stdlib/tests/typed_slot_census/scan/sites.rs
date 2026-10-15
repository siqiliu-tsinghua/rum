//! AST function, implementation, expression and macro sites.
use super::{
    Function, Implementation, Population, Site, Use,
    discovery::{Walker, test_only},
    text,
};
use proc_macro2::Span;
use syn::{
    parse::Parser,
    spanned::Spanned,
    visit::{self, Visit},
};

impl Walker<'_> {
    fn record(&mut self, kind: &str, name: String, target: String, receiver: String, span: Span) {
        let ordinal = self
            .ordinals
            .entry((self.caller.clone(), kind.into()))
            .or_default();
        *ordinal += 1;
        let shadowed = self.bindings.contains(&name);
        self.facts.uses.push(Use {
            caller: self.caller.clone(),
            scope: self.module.clone(),
            kind: kind.into(),
            name,
            target,
            receiver,
            shadowed,
            site: Site::new(&self.file, span),
            test: self.test,
            population: self.population,
            ordinal: *ordinal,
        });
    }
    fn function(&mut self, sig: &syn::Signature, block: &syn::Block, attrs: &[syn::Attribute]) {
        let previous_test = self.test;
        let previous_population = self.population;
        self.test |= test_only(attrs);
        if test_only(attrs) {
            self.population = Population::TestSupport;
        }
        let identity = format!("{}::{}", self.caller, sig.ident);
        self.facts.functions.push(Function {
            identity: identity.clone(),
            scope: self.module.clone(),
            owner: self.owner.clone(),
            trait_name: self.trait_name.clone(),
            name: sig.ident.to_string(),
            signature: text(sig),
            signature_ast: sig.clone(),
            body: block.clone(),
            cold: attrs.iter().any(|a| a.path().is_ident("cold")),
            site: Site::new(&self.file, sig.span()),
            test: self.test,
            population: self.population,
            boundary: Some(self.module.clone()),
        });
        let old = std::mem::replace(&mut self.caller, identity);
        let bindings = std::mem::take(&mut self.bindings);
        for input in &sig.inputs {
            if let syn::FnArg::Typed(input) = input {
                self.visit_pat(&input.pat);
            }
        }
        self.visit_block(block);
        self.bindings = bindings;
        self.caller = old;
        self.test = previous_test;
        self.population = previous_population;
    }
}
impl<'ast> Visit<'ast> for Walker<'_> {
    fn visit_pat_struct(&mut self, pat: &'ast syn::PatStruct) {
        self.record(
            "pattern",
            pat.path.segments.last().unwrap().ident.to_string(),
            text(&pat.path),
            String::new(),
            pat.span(),
        );
        visit::visit_pat_struct(self, pat);
    }
    fn visit_pat_ident(&mut self, pat: &'ast syn::PatIdent) {
        self.bindings.insert(pat.ident.to_string());
        visit::visit_pat_ident(self, pat);
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let saved = self.bindings.clone();
        visit::visit_block(self, block);
        self.bindings = saved;
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        Walker::visit_item_mod(self, item);
    }
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let index = self.facts.functions.len();
        self.function(&item.sig, &item.block, &item.attrs);
        self.facts.functions[index].boundary = match &item.vis {
            syn::Visibility::Public(_) => None,
            syn::Visibility::Inherited => Some(self.module.clone()),
            syn::Visibility::Restricted(vis) => {
                let mut module: Vec<_> = self.module.split("::").map(str::to_owned).collect();
                let mut path = vis.path.segments.iter().map(|s| s.ident.to_string());
                match path.next().as_deref() {
                    Some("crate") => module.truncate(1),
                    Some("super") => {
                        module.pop();
                    }
                    Some("self") => {}
                    _ => unreachable!("restricted Rust visibility"),
                }
                module.extend(path);
                Some(module.join("::"))
            }
        };
    }
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.function(&item.sig, &item.block, &item.attrs);
    }
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        let old = (
            self.owner.clone(),
            self.trait_name.clone(),
            self.caller.clone(),
        );
        self.owner = "Self".into();
        self.trait_name = item.ident.to_string();
        self.caller = format!("{}::<Self as {}>", self.module, item.ident);
        for member in &item.items {
            if let syn::TraitItem::Fn(f) = member {
                if let Some(body) = &f.default {
                    self.function(&f.sig, body, &f.attrs);
                }
            }
        }
        (self.owner, self.trait_name, self.caller) = old;
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let old = (
            self.owner.clone(),
            self.trait_name.clone(),
            self.test,
            self.caller.clone(),
            self.population,
        );
        self.owner = text(&item.self_ty);
        self.trait_name = item
            .trait_
            .as_ref()
            .map(|(_, p, _)| text(p))
            .unwrap_or_else(|| "inherent".into());
        self.test |= test_only(&item.attrs);
        if test_only(&item.attrs) {
            self.population = Population::TestSupport;
        }
        self.caller = format!("{}::<{} as {}>", self.caller, self.owner, self.trait_name);
        {
            let associated = |name: &str| {
                item.items
                    .iter()
                    .find_map(|i| {
                        if let syn::ImplItem::Type(t) = i {
                            (t.ident == name).then(|| text(&t.ty))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default()
            };
            let site = Site::new(&self.file, item.span());
            self.facts.implementations.insert(
                site.key(),
                Implementation {
                    site,
                    input: associated("Input"),
                    output: associated("Output"),
                    mode: associated("Mode"),
                    population: self.population,
                    scope: self.module.clone(),
                    trait_path: self.trait_name.clone(),
                },
            );
        }
        visit::visit_item_impl(self, item);
        (
            self.owner,
            self.trait_name,
            self.test,
            self.caller,
            self.population,
        ) = old;
    }
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let (name, target) = if let syn::Expr::Path(path) = call.func.as_ref() {
            (
                path.path.segments.last().unwrap().ident.to_string(),
                text(path),
            )
        } else {
            ("<indirect>".into(), text(&call.func))
        };
        self.record("call", name, target, String::new(), call.func.span());
        // A direct path was just recorded as the call target. A closure or
        // indirect callee can contain nested calls and function references;
        // suppressing its entire subtree would hide live decoder callbacks.
        if !matches!(call.func.as_ref(), syn::Expr::Path(_)) {
            self.visit_expr(&call.func);
        }
        for argument in &call.args {
            self.visit_expr(argument);
        }
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.record(
            "method",
            call.method.to_string(),
            call.method.to_string(),
            text(&call.receiver),
            call.method.span(),
        );
        visit::visit_expr_method_call(self, call);
    }
    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if !self.in_call {
            self.record(
                "reference",
                path.path.segments.last().unwrap().ident.to_string(),
                text(path),
                String::new(),
                path.span(),
            );
        }
        visit::visit_expr_path(self, path);
    }
    fn visit_expr_struct(&mut self, expr: &'ast syn::ExprStruct) {
        let name = expr.path.segments.last().unwrap().ident.to_string();
        if matches!(name.as_str(), "Evaluate" | "Finalize") {
            self.record(
                "construction",
                name,
                text(&expr.path),
                String::new(),
                expr.span(),
            );
        }
        visit::visit_expr_struct(self, expr);
    }
    fn visit_lit_str(&mut self, lit: &'ast syn::LitStr) {
        if lit.value().starts_with("InternalContinuation")
            || lit.value().starts_with("InternalPreparedInterpreted")
        {
            self.record(
                "head",
                lit.value(),
                String::new(),
                String::new(),
                lit.span(),
            );
        }
    }
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let name = mac.path.segments.last().unwrap().ident.to_string();
        if name == "resume_or_suspend" || name == "resume_typed_or_suspend" {
            let parser = |input: syn::parse::ParseStream| -> syn::Result<(String, Vec<syn::Expr>)> {
                let mut expressions = Vec::new();
                let arm =
                    if input.peek(syn::Ident) && input.fork().parse::<syn::Ident>()? == "insert" {
                        input.parse::<syn::Ident>()?;
                        expressions.push(input.parse()?);
                        input.parse::<syn::Token![,]>()?;
                        expressions.push(input.parse()?);
                        "insert"
                    } else {
                        expressions.push(input.parse()?);
                        input.parse::<syn::Token![,]>()?;
                        expressions.push(input.parse()?);
                        input.parse::<syn::Token![,]>()?;
                        input.parse::<syn::Ident>()?;
                        let content;
                        syn::parenthesized!(content in input);
                        let _: proc_macro2::TokenStream = content.parse()?;
                        input.parse::<syn::Token![=>]>()?;
                        expressions.push(input.parse()?);
                        input.parse::<syn::Token![,]>()?;
                        input.parse::<syn::Ident>()?;
                        input.parse::<syn::Token![=>]>()?;
                        expressions.push(input.parse()?);
                        "result"
                    };
                if input.peek(syn::Token![,]) {
                    input.parse::<syn::Token![,]>()?;
                }
                Ok((arm.into(), expressions))
            };
            match parser.parse2(mac.tokens.clone()) {
                Ok((arm, expressions)) => {
                    self.record("macro", name, text(&mac.path), arm, mac.span());
                    for expression in &expressions {
                        self.visit_expr(expression);
                    }
                }
                Err(error) => self.facts.unresolved.push(format!(
                    "macro-parse\t{}\t{}\t{error}",
                    self.file,
                    mac.span().start().line
                )),
            }
        } else if name != "macro_rules" {
            // Expression-list macros include decoder calls and function values
            // (notably vec![decode(value)]). This visits their source arguments,
            // without claiming to expand arbitrary macros or prove execution.
            let parser = |input: syn::parse::ParseStream| -> syn::Result<Vec<syn::Expr>> {
                let mut expressions = Vec::new();
                while !input.is_empty() {
                    expressions.push(input.parse()?);
                    if input.is_empty() {
                        break;
                    }
                    if name == "vec" && input.peek(syn::Token![;]) {
                        input.parse::<syn::Token![;]>()?;
                    } else {
                        input.parse::<syn::Token![,]>()?;
                    }
                }
                Ok(expressions)
            };
            match parser.parse2(mac.tokens.clone()) {
                Ok(expressions) => {
                    for expression in &expressions {
                        self.visit_expr(expression);
                    }
                }
                Err(_) => self.facts.unresolved.push(format!(
                    "macro-expansion\t{}\t{}\t{name}: non-expression tokens",
                    self.file,
                    mac.span().start().line
                )),
            }
        }
    }
}
