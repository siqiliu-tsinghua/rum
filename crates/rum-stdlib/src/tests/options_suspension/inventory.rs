use std::collections::BTreeSet;

use syn::{Expr, ImplItem, Item, Pat, Stmt};

fn descriptor_inventory(source: &str) -> Result<BTreeSet<String>, String> {
    let file = syn::parse_file(source).map_err(|error| error.to_string())?;
    let variants = file
        .items
        .iter()
        .find_map(|item| match item {
            Item::Enum(item) if item.ident == "OptionConsumer" => Some(
                item.variants
                    .iter()
                    .map(|variant| variant.ident.to_string())
                    .collect::<BTreeSet<_>>(),
            ),
            _ => None,
        })
        .ok_or("missing descriptor enum")?;
    let implementation = file
        .items
        .iter()
        .find_map(|item| match item {
            Item::Impl(item)
                if item
                    .trait_
                    .as_ref()
                    .is_some_and(|(_, path, _)| path.is_ident("OptionReentry")) =>
            {
                Some(item)
            }
            _ => None,
        })
        .ok_or("missing re-entry implementation")?;
    let resume = implementation
        .items
        .iter()
        .find_map(|item| match item {
            ImplItem::Fn(item) if item.sig.ident == "resume" => Some(item),
            _ => None,
        })
        .ok_or("missing resume")?;
    let [Stmt::Expr(Expr::Match(dispatch), None)] = resume.block.stmts.as_slice() else {
        return Err("resume must be an exhaustive descriptor match".into());
    };
    let mut arms = BTreeSet::new();
    for arm in &dispatch.arms {
        let Pat::Path(pattern) = &arm.pat else {
            return Err("each descriptor needs its own live arm; no wildcard".into());
        };
        if arm.guard.is_some() {
            return Err("descriptor arms cannot be conditional".into());
        }
        let variant = pattern.path.segments.last().unwrap().ident.to_string();
        let body = match arm.body.as_ref() {
            Expr::Block(block) => match block.block.stmts.as_slice() {
                [Stmt::Expr(expr, None)] => expr,
                _ => return Err(format!("{variant}: arm must directly invoke its consumer")),
            },
            expr => expr,
        };
        let Expr::Call(call) = body else {
            return Err(format!("{variant}: no live consumer call"));
        };
        let Expr::Path(path) = call.func.as_ref() else {
            return Err(format!("{variant}: consumer must be a named handler"));
        };
        let mut handler = String::from("eval_");
        for (index, ch) in variant.chars().enumerate() {
            if index > 0 && ch.is_uppercase() {
                handler.push('_');
            }
            handler.push(ch.to_ascii_lowercase());
        }
        handler.push_str("_with_options");
        if path.path.segments.last().unwrap().ident != handler {
            return Err(format!("{variant}: no live consumer handler"));
        }
        use quote::ToTokens;
        if call
            .args
            .iter()
            .map(|arg| arg.to_token_stream().to_string())
            .collect::<Vec<_>>()
            != ["session", "head", "args", "Some (options)"]
        {
            return Err(format!(
                "{variant}: consumer must receive frozen call and resolved options"
            ));
        }
        if !arms.insert(variant) {
            return Err("duplicate consumer arm".into());
        }
    }
    if arms != variants {
        return Err("descriptor and live arm sets differ".into());
    }
    Ok(variants)
}

#[test]
fn option_reentry_descriptor_inventory_is_exhaustive() {
    let source = include_str!("../../option_consumers.rs");
    let variants = descriptor_inventory(source).expect("every descriptor has a live consumer arm");
    let mut certificates = super::remaining::REMAINING_CONSUMERS
        .iter()
        .map(|name| name.to_string())
        .collect::<BTreeSet<_>>();
    let remaining = syn::parse_file(include_str!("remaining.rs")).unwrap();
    let tests = remaining
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(item) if item.attrs.iter().any(|attr| attr.path().is_ident("test")) => {
                Some(item.sig.ident.to_string())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let expected_tests = certificates
        .iter()
        .map(|name| format!("{}_resumes_into_consumer", name.to_lowercase()))
        .collect();
    assert_eq!(
        tests, expected_tests,
        "each remaining certificate is a required test"
    );
    // Read the unchanged brief-530 certificate table, rather than maintain a
    // second list which could claim coverage without running a certificate.
    let tests = syn::parse_file(include_str!("../options_suspension.rs")).unwrap();
    let test = tests
        .items
        .iter()
        .find_map(|item| match item {
            Item::Fn(item)
                if item.sig.ident == "string_option_consumers_finish_after_actual_suspension" =>
            {
                Some(item)
            }
            _ => None,
        })
        .unwrap();
    let Stmt::Expr(Expr::ForLoop(loop_), _) = &test.block.stmts[0] else {
        panic!("certificate loop");
    };
    let Expr::Array(cases) = loop_.expr.as_ref() else {
        panic!("certificate table");
    };
    for case in &cases.elems {
        let Expr::Tuple(tuple) = case else {
            panic!("certificate row");
        };
        let Expr::Lit(lit) = &tuple.elems[0] else {
            panic!("consumer name");
        };
        let syn::Lit::Str(name) = &lit.lit else {
            panic!("consumer name string");
        };
        assert!(certificates.insert(name.value()));
    }
    assert_eq!(
        variants, certificates,
        "every descriptor needs a suspension certificate"
    );

    // The permanent falsifier checks a compiling but inert arm, not merely a
    // non-exhaustive Rust match. The live implementation was also mutated and
    // this required test watched red before restoration.
    let missing = source.replace(
        "crate::strings::eval_string_wrap_with_options(session, head, args, Some(options))",
        "Ok(head.clone())",
    );
    assert_ne!(missing, source, "mutation must reach the live arm");
    assert!(
        descriptor_inventory(&missing)
            .unwrap_err()
            .contains("StringWrap: no live consumer handler")
    );
}

#[test]
fn production_option_reentry_implementations_are_exact() {
    use quote::ToTokens;
    use std::{fs, path::Path};
    use syn::visit::{self, Visit};

    #[derive(Default)]
    struct Implementations(Vec<String>);
    impl<'ast> Visit<'ast> for Implementations {
        fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
            if item
                .attrs
                .iter()
                .any(|attr| attr.meta.to_token_stream().to_string() == "cfg (test)")
            {
                return;
            }
            visit::visit_item_mod(self, item);
        }
        fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
            if item
                .trait_
                .as_ref()
                .is_some_and(|(_, path, _)| path.segments.last().unwrap().ident == "OptionReentry")
            {
                self.0.push(item.self_ty.to_token_stream().to_string());
            }
            visit::visit_item_impl(self, item);
        }
    }
    fn scan(root: &Path, dir: &Path, found: &mut BTreeSet<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() && path.file_name().is_some_and(|name| name == "tests") {
                continue;
            }
            if path.is_dir() {
                scan(root, &path, found);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let mut implementations = Implementations::default();
                implementations
                    .visit_file(&syn::parse_file(&fs::read_to_string(&path).unwrap()).unwrap());
                for name in implementations.0 {
                    assert!(found.insert(format!(
                        "{}:{name}",
                        path.strip_prefix(root).unwrap().display()
                    )));
                }
            }
        }
    }
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let mut implementations = BTreeSet::new();
    scan(
        workspace,
        &workspace.join("crates/rum-runtime/src"),
        &mut implementations,
    );
    scan(
        workspace,
        &workspace.join("crates/rum-stdlib/src"),
        &mut implementations,
    );
    assert_eq!(
        implementations,
        BTreeSet::from(["crates/rum-stdlib/src/option_consumers.rs:OptionConsumer".to_string()]),
        "new production re-entries must join the exhaustive descriptor and certificate set"
    );
}
