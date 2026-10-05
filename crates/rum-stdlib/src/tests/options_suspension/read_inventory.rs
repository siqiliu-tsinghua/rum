//! A closed source inventory, not a whole-program semantic proof.
use quote::ToTokens;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
use syn::{
    Block, Expr, Stmt,
    spanned::Spanned,
    visit::{self, Visit},
};

const READERS: &[&str] = &[
    "system_option_bool",
    "runtime_option_expr_for_symbol",
    "option_rule_expr_for_symbol",
    "eval_option_syntax",
];
const RAW: &[&str] = &[
    "option_value_for_symbol",
    "resolve_option_entry_value",
    "raw_option_value_for_symbol",
];

#[derive(Default)]
struct Calls {
    function: String,
    tails: BTreeSet<(usize, usize)>,
    calls: Vec<(String, String, bool, usize)>,
    closure: bool,
    opaque_reader: Option<String>,
}
fn position(expr: &Expr) -> (usize, usize) {
    let p = expr.span().start();
    (p.line, p.column)
}
fn tail_block(block: &Block, tails: &mut BTreeSet<(usize, usize)>) {
    if let Some(Stmt::Expr(expr, None)) = block.stmts.last() {
        tail(expr, tails);
    }
}
fn tail(expr: &Expr, tails: &mut BTreeSet<(usize, usize)>) {
    match expr {
        Expr::MethodCall(_) | Expr::Call(_) => {
            tails.insert(position(expr));
        }
        Expr::Block(expr) => tail_block(&expr.block, tails),
        Expr::Paren(expr) => tail(&expr.expr, tails),
        Expr::Group(expr) => tail(&expr.expr, tails),
        Expr::If(expr) => {
            tail_block(&expr.then_branch, tails);
            if let Some((_, expr)) = &expr.else_branch {
                tail(expr, tails);
            }
        }
        Expr::Match(expr) => {
            for arm in &expr.arms {
                tail(&arm.body, tails);
            }
        }
        _ => {}
    }
}
fn test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("test") || attr.meta.to_token_stream().to_string() == "cfg (test)"
    })
}
impl Calls {
    fn function(&mut self, name: String, block: &Block) {
        let old_name = std::mem::replace(&mut self.function, name);
        let old_tails = std::mem::take(&mut self.tails);
        tail_block(block, &mut self.tails);
        self.visit_block(block);
        self.function = old_name;
        self.tails = old_tails;
    }
    fn record(&mut self, name: String, expr: &Expr) {
        if READERS.contains(&name.as_str()) || RAW.contains(&name.as_str()) {
            self.calls.push((
                self.function.clone(),
                name,
                !self.closure && self.tails.contains(&position(expr)),
                expr.span().start().line,
            ));
        }
    }
}
impl<'ast> Visit<'ast> for Calls {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        fn contains_reader(tokens: proc_macro2::TokenStream) -> Option<String> {
            for token in tokens {
                match token {
                    proc_macro2::TokenTree::Ident(ident)
                        if READERS.contains(&ident.to_string().as_str())
                            || RAW.contains(&ident.to_string().as_str()) =>
                    {
                        return Some(ident.to_string());
                    }
                    proc_macro2::TokenTree::Group(group) => {
                        if let Some(name) = contains_reader(group.stream()) {
                            return Some(name);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        if let Some(name) = contains_reader(mac.tokens.clone()) {
            self.opaque_reader = Some(format!("opaque reader in macro: {name}"));
        }
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if !test_only(&item.attrs) {
            visit::visit_item_mod(self, item);
        }
    }
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if !test_only(&item.attrs) {
            self.function(item.sig.ident.to_string(), &item.block);
        }
    }
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        if !test_only(&item.attrs) {
            self.function(item.sig.ident.to_string(), &item.block);
        }
    }
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if let Expr::Return(ret) = expr {
            if !self.closure {
                if let Some(expr) = &ret.expr {
                    tail(expr, &mut self.tails);
                }
            }
        }
        match expr {
            Expr::MethodCall(call) => self.record(call.method.to_string(), expr),
            Expr::Call(call) => {
                if let Expr::Path(path) = call.func.as_ref() {
                    let name = path.path.segments.last().unwrap().ident.to_string();
                    if READERS.contains(&name.as_str()) || RAW.contains(&name.as_str()) {
                        self.record(name, expr);
                        // Direct UFCS calls are inventoried once. A bare method
                        // reference elsewhere would hide a subsequent alias call.
                        for arg in &call.args {
                            self.visit_expr(arg);
                        }
                        return;
                    }
                }
            }
            Expr::Path(path) => {
                let name = path.path.segments.last().unwrap().ident.to_string();
                if READERS.contains(&name.as_str()) || RAW.contains(&name.as_str()) {
                    self.opaque_reader =
                        Some(format!("reader reference without direct call: {name}"));
                }
            }
            _ => {}
        }
        let old = self.closure;
        if matches!(expr, Expr::Closure(_) | Expr::Async(_)) {
            self.closure = true;
        }
        visit::visit_expr(self, expr);
        self.closure = old;
    }
}

// Exactly the step-B/C readers remain legacy. Counts distinguish multiple reads
// in a single phase. Extending this list requires a reviewed migration decision.
const LEGACY: &[(&str, &str, &str, usize)] = &[
    (
        "rum-runtime/src/runtime/debug_tools/monitor.rs",
        "monitor_option_expr",
        "runtime_option_expr_for_symbol",
        1,
    ),
    (
        "rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs",
        "parse_test_report_options",
        "runtime_option_expr_for_symbol",
        1,
    ),
    (
        "rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs",
        "test_report_constraint_expr",
        "runtime_option_expr_for_symbol",
        1,
    ),
    (
        "rum-runtime/src/runtime/namespace_tools/names.rs",
        "eval_names",
        "runtime_option_expr_for_symbol",
        2,
    ),
    (
        "rum-runtime/src/runtime/namespace_tools/names.rs",
        "names_ignore_case",
        "runtime_option_expr_for_symbol",
        1,
    ),
    (
        "rum-runtime/src/runtime/package_tools.rs",
        "eval_persist_package_state",
        "runtime_option_expr_for_symbol",
        1,
    ),
    (
        "rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs",
        "eval_check_abort",
        "option_rule_expr_for_symbol",
        1,
    ),
    (
        "rum-stdlib/src/functional_ops/arrays/mod.rs",
        "eval_array_filter",
        "option_rule_expr_for_symbol",
        1,
    ),
];
const BOOLEAN: &[(&str, &str)] = &[
    (
        "rum-runtime/src/runtime/debug_tools/trace.rs",
        "eval_trace_named",
    ),
    (
        "rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs",
        "resume_cases_after_resolved_spec",
    ),
    (
        "rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs",
        "resume_cases_after_resolved_spec_suspended",
    ),
    (
        "rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs",
        "resume_delete_cases_after_pattern",
    ),
    (
        "rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs",
        "resume_delete_cases_after_pattern_suspended",
    ),
    (
        "rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs",
        "resume_replace_after_rule_exprs",
    ),
    (
        "rum-runtime/src/runtime/tree_walkers/count_like.rs",
        "resume_count_after_pattern",
    ),
    (
        "rum-runtime/src/runtime/tree_walkers/count_like.rs",
        "resume_member_q_after_pattern",
    ),
    (
        "rum-runtime/src/runtime/tree_walkers/count_like.rs",
        "resume_free_q_after_pattern",
    ),
    (
        "rum-runtime/src/runtime/tree_walkers/position.rs",
        "resume_position_after_pattern",
    ),
    (
        "rum-runtime/src/runtime/tree_walkers/level.rs",
        "eval_level",
    ),
    ("rum-stdlib/src/structure_ops/surface.rs", "eval_depth"),
];
const RAW_SITES: &[(&str, &str, &str, usize)] = &[
    (
        "rum-runtime/src/runtime/options/core.rs",
        "system_option_bool",
        "option_value_for_symbol",
        1,
    ),
    (
        "rum-runtime/src/runtime/options/core.rs",
        "option_value_for_symbol",
        "resolve_option_entry_value",
        1,
    ),
    (
        "rum-runtime/src/runtime/options/core.rs",
        "option_rule_expr_for_symbol",
        "resolve_option_entry_value",
        1,
    ),
    (
        "rum-runtime/src/runtime/options/core.rs",
        "runtime_option_expr_for_symbol",
        "raw_option_value_for_symbol",
        1,
    ),
    (
        "rum-runtime/src/runtime/options/builtins.rs",
        "resolve_option_value_scalar",
        "resolve_option_entry_value",
        2,
    ),
    (
        "rum-runtime/src/runtime/builtins_input.rs",
        "compile_input_request",
        "raw_option_value_for_symbol",
        1,
    ),
    (
        "rum-runtime/src/runtime/builtins_input.rs",
        "dialog_option_raw_expr",
        "raw_option_value_for_symbol",
        1,
    ),
    (
        "rum-runtime/src/runtime/package_tools.rs",
        "once_should_cache",
        "raw_option_value_for_symbol",
        2,
    ),
];
fn expected() -> BTreeMap<String, usize> {
    LEGACY
        .iter()
        .chain(RAW_SITES)
        .map(|(path, function, reader, count)| (format!("{path}:{function}:{reader}"), *count))
        .chain(
            BOOLEAN
                .iter()
                .map(|(path, function)| (format!("{path}:{function}:system_option_bool"), 1)),
        )
        .collect()
}
fn discover(path: &str, source: &str, found: &mut BTreeMap<String, usize>) -> Result<(), String> {
    let mut calls = Calls::default();
    calls.visit_file(&syn::parse_file(source).map_err(|error| error.to_string())?);
    if let Some(name) = calls.opaque_reader {
        return Err(name);
    }
    for (function, reader, tail, line) in calls.calls {
        if READERS.contains(&reader.as_str())
            && reader != "runtime_option_expr_for_symbol"
            && reader != "option_rule_expr_for_symbol"
            && !tail
        {
            return Err(format!("non-tail reader {path}:{line}:{function}:{reader}"));
        }
        println!("READ {path}:{line}:{function}:{reader} tail={tail}");
        *found
            .entry(format!("{path}:{function}:{reader}"))
            .or_default() += 1;
    }
    Ok(())
}
fn scan(root: &Path, directory: &Path, found: &mut BTreeMap<String, usize>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().unwrap() != "tests" {
                scan(root, &path, found);
            }
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            discover(
                path.strip_prefix(root).unwrap().to_str().unwrap(),
                &fs::read_to_string(&path).unwrap(),
                found,
            )
            .unwrap();
        }
    }
}
#[test]
fn read_inventory_is_closed_and_boolean_calls_are_tails() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut found = BTreeMap::new();
    for entry in fs::read_dir(root).unwrap() {
        let src = entry.unwrap().path().join("src");
        if src.is_dir() {
            scan(root, &src, &mut found);
        }
    }
    assert_eq!(
        found,
        expected(),
        "new readers/raw bypasses cannot enter the legacy list silently"
    );
    let source = fs::read_to_string(root.join("rum-runtime/src/runtime/options/core.rs")).unwrap();
    let file = syn::parse_file(&source).unwrap();
    for item in file.items {
        if let syn::Item::Impl(item) = item {
            for item in item.items {
                if let syn::ImplItem::Fn(item) = item {
                    if item.sig.ident == "option_value_for_symbol"
                        || item.sig.ident == "resolve_option_entry_value"
                    {
                        assert_eq!(
                            item.vis.to_token_stream().to_string(),
                            "pub (in crate :: runtime :: options)"
                        );
                    }
                    if item.sig.ident == "system_option_bool" {
                        assert_eq!(item.sig.inputs.len(), 5, "continuation required");
                        assert_eq!(
                            item.sig.output.to_token_stream().to_string(),
                            "-> Result < Expr , EvalError >"
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn read_inventory_falsifies_new_callers_and_non_tail_use() {
    let good =
        r#"fn finish(s: &mut Session, c: C) { s.system_option_bool("Depth", "Heads", false, c) }"#;
    let mut found = BTreeMap::new();
    discover("new.rs", good, &mut found).unwrap();
    assert!(
        !expected().contains_key(found.keys().next().unwrap()),
        "new caller rejected by exact inventory"
    );
    for bad in [
        r#"fn finish(s: &mut Session, c: C) { let value = s.system_option_bool("Depth", "Heads", false, c)?; Ok(value) }"#,
        r#"fn finish(s: &mut Session, c: C) { s.system_option_bool("Depth", "Heads", false, c)?; Ok(()) }"#,
        r#"fn finish(s: &mut Session, c: C) { (|| { return s.system_option_bool("Depth", "Heads", false, c); })() }"#,
    ] {
        assert!(
            discover("new.rs", bad, &mut BTreeMap::new())
                .unwrap_err()
                .contains("non-tail reader")
        );
    }
    assert!(
        discover(
            "new.rs",
            r#"fn finish() { opaque!(s.system_option_bool("Depth", "Heads", false, c)) }"#,
            &mut BTreeMap::new()
        )
        .unwrap_err()
        .contains("opaque reader")
    );
    assert!(discover("new.rs", r#"fn finish(s: &mut Session, c: C) { let read = Session::system_option_bool; read(s, "Depth", "Heads", false, c) }"#, &mut BTreeMap::new()).unwrap_err().contains("reader reference without direct call"));
    discover("new.rs", r#"fn finish(s: &mut Session, c: C) { Session::system_option_bool(s, "Depth", "Heads", false, c) }"#, &mut BTreeMap::new()).unwrap();
    for good in [
        good,
        r#"fn finish(s: &mut Session, c: C) { return s.system_option_bool("Depth", "Heads", false, c); }"#,
        r#"fn finish(s: &mut Session, c: C) { if yes { s.system_option_bool("Depth", "Heads", false, c) } else { Ok(()) } }"#,
    ] {
        discover("new.rs", good, &mut BTreeMap::new()).unwrap();
    }
}
