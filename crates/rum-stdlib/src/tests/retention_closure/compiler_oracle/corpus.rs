//! Exhaustive compiler observations of the census's new constructor claims.
//! Conditional source alternatives remain visible, including unobserved ones.

use super::super::{model::*, regions, source};
use super::cache;
use std::{collections::BTreeMap, fmt::Write, fs, process::Command};
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

#[test]
fn census_constructor_resolutions_are_compiler_observed() {
    let source = source::production();
    let directory = source::workspace().join("target/brief-535/oracle-corpus");
    fs::create_dir_all(&directory).unwrap();
    let mut claims = BTreeMap::new();
    for f in source.functions.values().flatten() {
        for edge in regions::definition_edges(&source, f).constructors {
            claims.insert((edge.site, edge.definition), edge.target);
        }
    }
    let mut out="name\tcaller\tfile\tstart_line\tstart_column\tend_line\tend_column\ttarget\tcallee\tcallee_start_line\tcallee_start_column\tcallee_end_line\tcallee_end_column\n".to_string();
    for (i, ((site, caller), target)) in claims.iter().enumerate() {
        let functions = &source.functions[caller];
        let mut found = None;
        for f in functions {
            let mut find = CallAt {
                source: &source,
                function: f,
                site,
                callee: None,
            };
            find.visit_block(&f.body);
            if find.callee.is_some() {
                found = find.callee;
            }
        }
        let (callee, callee_site) = found.expect("every claim is an actual indexed call");
        writeln!(
            out,
            "corpus_{i}\t{caller}\t{}\t{}\t{}\t{}\t{}\t{target}\t{callee}\t{}\t{}\t{}\t{}",
            site.file,
            site.start.0,
            site.start.1,
            site.end.0,
            site.end.1,
            callee_site.start.0,
            callee_site.start.1,
            callee_site.end.0,
            callee_site.end.1
        )
        .unwrap();
    }
    assert!(!claims.is_empty());
    fs::write(directory.join("claims.tsv"), out).unwrap();
    let features = if cfg!(feature = "hash") {
        "hash"
    } else {
        "no-default-features"
    };
    let fingerprint = cache::fingerprint(&directory, &["claims.tsv"], features);
    let stdout = match cache::reuse(&directory, &fingerprint) {
        Some(recorded) => recorded,
        None => {
            let output = Command::new("python3")
                .arg(source::workspace().join("scripts/run-bounded.py"))
                .args(["--timeout", "700", "--", "python3"])
                .arg(source::workspace().join(
                    "crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.py",
                ))
                .arg(source::workspace())
                .arg(&directory)
                .arg(features)
                .output()
                .unwrap();
            fs::write(
                directory.join("harness.log"),
                [&output.stdout[..], &output.stderr[..]].concat(),
            )
            .unwrap();
            assert!(
                output.status.success(),
                "corpus oracle instrument failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            cache::record(&directory, &fingerprint, &stdout);
            stdout
        }
    };
    print!("{stdout}");
}

struct CallAt<'a> {
    source: &'a Source,
    function: &'a Function,
    site: &'a Site,
    callee: Option<(String, Site)>,
}
impl<'ast> Visit<'ast> for CallAt<'_> {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let Some(arm) = super::super::typed_resume_macro::checked(self.source, self.function, mac)
        else {
            return;
        };
        self.visit_expr(&arm.result);
        self.visit_pat(&arm.value);
        self.visit_expr(&arm.success);
        self.visit_expr(&arm.session);
        self.visit_expr(&arm.frame);
    }
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if Site::new(&self.function.site.file, call.span()) == *self.site {
            self.callee = Some((
                tokens(&*call.func),
                Site::new(&self.function.site.file, call.func.span()),
            ));
        }
        visit::visit_expr_call(self, call);
    }
}

// The fixture is compiled against the actual typed API. No frame or row is
// invented: the caller supplies any frame with the stated input/output types.
#[allow(dead_code)]
mod typed_argument_fixture {
    pub struct Local {
        value: usize,
    }
    impl Local {
        pub fn choose(self) -> usize {
            self.value
        }
    }
    pub fn invocation<
        F: rum_runtime::kernel_api::TypedFrame<
                Input = rum_core::Expr,
                Output = Option<rum_core::Expr>,
            >,
    >(
        session: &mut rum_runtime::Session,
        produced: Result<rum_core::Expr, rum_runtime::EvalError>,
        local: Local,
        frame: F,
    ) -> Result<Option<rum_core::Expr>, rum_runtime::EvalError> {
        rum_runtime::resume_typed_or_suspend!(
            session, produced,
            ok(value) => {
                let _ = Some(1usize);
                let _ = local.choose();
                Ok(Some(value))
            },
            suspend => frame,
        )
    }
    pub fn invocation_expression<
        F: rum_runtime::kernel_api::TypedFrame<
                Input = rum_core::Expr,
                Output = Option<rum_core::Expr>,
            >,
    >(
        session: &mut rum_runtime::Session,
        produced: Result<rum_core::Expr, rum_runtime::EvalError>,
        local: Local,
        frame: F,
    ) -> Result<Option<rum_core::Expr>, rum_runtime::EvalError> {
        rum_runtime::resume_typed_or_suspend!(
            session, { produced },
            ok(value) => {
                let _ = Some(1usize);
                let _ = local.choose();
                Ok(Some(value))
            },
            suspend => frame,
        )
    }
}

fn typed_argument_source() -> (Source, Function) {
    let file = "crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs";
    let module = "rum_stdlib::tests::retention_closure::compiler_oracle::corpus";
    let mut source = source::production();
    source
        .files
        .insert(file.into(), include_str!("corpus.rs").into());
    let source = source::test_module(source, file, module);
    let f = super::super::resolve::function(
        &source,
        &format!("{module}::typed_argument_fixture::invocation"),
    )
    .unwrap()
    .clone();
    (source, f)
}

#[test]
fn typed_macro_constructor_locator_reads_arguments() {
    let (source, f) = typed_argument_source();
    let edges = regions::definition_edges(&source, &f);
    assert!(edges.rejected.iter().any(|row| {
        row.kind == Kind::RET002
            && row.reason.contains("cold helper closure capture/move")
            && row
                .reason
                .contains("frame inputs; FnOnce call and closure drop")
    }));
    assert!(
        edges.rejected.iter().any(|row| {
            row.kind == Kind::RET002 && row.reason.contains("caller producer place")
        })
    );
    let claims = edges.constructors;
    assert_eq!(claims.len(), 3, "all three calls are visible to regions");
    for claim in claims {
        let mut find = CallAt {
            source: &source,
            function: &f,
            site: &claim.site,
            callee: None,
        };
        find.visit_block(&f.body);
        assert!(
            find.callee.is_some(),
            "constructor locator hid {}",
            claim.site.display()
        );
        println!(
            "TYPED_MACRO_LOCATED {} {}",
            claim.site.display(),
            claim.target
        );
    }
}

#[test]
fn typed_macro_method_selection_reads_arguments() {
    let (source, f) = typed_argument_source();
    let claims = super::super::methods::claims(&source, &f);
    assert_eq!(claims.len(), 1, "typed local method must be visited");
    let (site, selected) = claims.into_iter().next().unwrap();
    let selected = selected.unwrap_or_else(|e| panic!("{e}"));
    assert!(selected.function.id.ends_with("::Local::choose"));
    println!(
        "TYPED_MACRO_METHOD {} {}",
        site.display(),
        selected.function.id
    );
}

#[test]
fn typed_macro_expected_constructor_type_reads_arguments() {
    let (source, f) = typed_argument_source();
    let claims = super::super::types::constructor_claims(&source, &f);
    assert_eq!(
        claims.len(),
        3,
        "explicit Some and both contextual constructors"
    );
    let expected = super::super::types::exact_output(&source, &f).unwrap();
    let ok = claims
        .iter()
        .find(|c| c.target == "std::result::Result::Ok")
        .unwrap();
    assert_eq!(
        ok.exact, expected,
        "Ok obtains its type from the enclosing signature"
    );
    for claim in claims {
        println!(
            "TYPED_MACRO_TYPE {} {} {}",
            claim.site.display(),
            claim.target,
            claim.exact.spelling()
        );
    }
}

#[test]
fn typed_macro_expression_form_reads_all_four_views() {
    let (source, original) = typed_argument_source();
    let f = super::super::resolve::function(
        &source,
        &original
            .id
            .replace("::invocation", "::invocation_expression"),
    )
    .unwrap();
    let edges = regions::definition_edges(&source, f);
    assert!(
        edges.rejected.iter().any(|row| {
            row.kind == Kind::RET002 && row.reason.contains("owned producer binding")
        })
    );
    assert_eq!(edges.constructors.len(), 3);
    for claim in edges.constructors {
        let mut find = CallAt {
            source: &source,
            function: f,
            site: &claim.site,
            callee: None,
        };
        find.visit_block(&f.body);
        assert!(
            find.callee.is_some(),
            "expression-form constructor is visible"
        );
    }
    let methods = super::super::methods::claims(&source, f);
    assert_eq!(methods.len(), 1);
    assert!(
        methods
            .into_iter()
            .next()
            .unwrap()
            .1
            .unwrap()
            .function
            .id
            .ends_with("::Local::choose")
    );
    let types = super::super::types::constructor_claims(&source, f);
    assert_eq!(types.len(), 3);
    let expected = super::super::types::exact_output(&source, f).unwrap();
    assert_eq!(
        types
            .iter()
            .find(|c| c.target == "std::result::Result::Ok")
            .unwrap()
            .exact,
        expected,
    );
    println!("TYPED_MACRO_EXPRESSION regions/locator/methods/types");
}

#[test]
fn legacy_macro_stays_opaque_in_all_four_views() {
    let (source, mut f) = typed_argument_source();
    let typed_claims = regions::definition_edges(&source, &f).constructors;
    let mac = match f.body.stmts.last_mut().unwrap() {
        syn::Stmt::Expr(syn::Expr::Macro(m), _) => &mut m.mac,
        syn::Stmt::Macro(m) => &mut m.mac,
        _ => panic!("fixture tail is a macro"),
    };
    // Preserve every argument span while changing only the macro path.
    mac.path = syn::parse_quote!(rum_runtime::resume_or_suspend);
    let edges = regions::definition_edges(&source, &f);
    assert!(edges.constructors.is_empty() && edges.resolved.is_empty());
    assert_eq!(edges.rejected.len(), 1);
    assert_eq!(edges.rejected.iter().next().unwrap().kind, Kind::RET003);
    assert!(super::super::methods::claims(&source, &f).is_empty());
    assert!(super::super::types::constructor_claims(&source, &f).is_empty());
    for claim in typed_claims {
        let mut find = CallAt {
            source: &source,
            function: &f,
            site: &claim.site,
            callee: None,
        };
        find.visit_block(&f.body);
        assert!(find.callee.is_none(), "legacy arguments must remain opaque");
    }
    println!("LEGACY_MACRO_OPAQUE regions/locator/methods/types");
}

#[test]
fn typed_macro_definition_drift_fails_every_view() {
    let (mut source, f) = typed_argument_source();
    let file = "crates/rum-runtime/src/runtime/suspension/typed/witness.rs";
    let old = source.files[file].clone();
    let mut missed = Vec::new();
    for (label, before, after, last) in [
        (
            "legacy identifier selector",
            "suspend => $frame:expr",
            "suspend => $frame:path",
            false,
        ),
        (
            "evaluation return helper",
            "witness.suspend_evaluation_return($session, request, $frame)",
            "witness.suspend($session, request, $frame)",
            false,
        ),
        ("identifier matcher", "$result:ident", "$result:path", false),
        (
            "identifier witness",
            "let witness = $crate::kernel_api::slot_witness(&($result));",
            "let witness = $crate::kernel_api::slot_witness(&($result)); let drift = ();",
            false,
        ),
        (
            "identifier cold helper",
            "#[inline(never)]",
            "#[inline(always)]",
            false,
        ),
        (
            "identifier owned capture",
            "suspend(move ||",
            "suspend(||",
            false,
        ),
        (
            "expression producer",
            "let produced = $result;",
            "let produced = $result; let drift = ();",
            false,
        ),
        (
            "expression cold helper",
            "#[inline(never)]",
            "#[inline(always)]",
            true,
        ),
        (
            "expression owned capture",
            "suspend(move ||",
            "suspend(||",
            true,
        ),
    ] {
        let offset = if last {
            old.rfind(before)
        } else {
            old.find(before)
        }
        .unwrap();
        let mut changed = old.clone();
        changed.replace_range(offset..offset + before.len(), after);
        assert_ne!(old, changed);
        source.files.insert(file.into(), changed);
        for view in ["regions", "locator", "methods", "types"] {
            let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match view {
                "regions" => {
                    regions::definition_edges(&source, &f);
                }
                "locator" => {
                    let mut find = CallAt {
                        source: &source,
                        function: &f,
                        site: &f.site,
                        callee: None,
                    };
                    find.visit_block(&f.body);
                }
                "methods" => {
                    super::super::methods::claims(&source, &f);
                }
                "types" => {
                    super::super::types::constructor_claims(&source, &f);
                }
                _ => unreachable!(),
            }))
            .err();
            let Some(failed) = failed else {
                println!("TYPED_MACRO_DRIFT_NOT_REJECTED {label} {view}");
                missed.push(format!("{label}: {view}"));
                continue;
            };
            let text = failed
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| failed.downcast_ref::<&str>().copied())
                .unwrap_or("");
            assert!(
                text.contains("typed resume macro definition changed"),
                "{view}: {text}"
            );
            println!("TYPED_MACRO_DRIFT {label} {view} {text}");
        }
    }
    assert!(
        missed.is_empty(),
        "every view must fail on drift: {missed:?}"
    );
}

#[test]
fn typed_macro_unknown_pattern_does_not_leak_or_select() {
    let (source, mut f) = typed_argument_source();
    f.body = syn::parse_str(
        r#"{
        rum_runtime::resume_typed_or_suspend!(
            { let _ = local.choose(); session },
            { let _ = local.choose(); unresolved_producer() },
            ok(local) => { let _ = local.choose(); Ok(None) },
            suspend => { let _ = local.choose(); frame },
        )
    }"#,
    )
    .unwrap();
    let claims = super::super::methods::claims(&source, &f);
    assert_eq!(claims.len(), 4);
    let mut selected = 0;
    let mut unknown = 0;
    for (site, claim) in claims {
        match claim {
            Ok(s) => {
                assert!(s.function.id.ends_with("::Local::choose"));
                selected += 1;
            }
            Err(reason) => {
                assert_eq!(site.start.0, 5, "only the success binding shadows local");
                assert!(
                    reason.contains("lacks checked nominal/place provenance"),
                    "{reason}"
                );
                unknown += 1;
            }
        }
    }
    assert_eq!((selected, unknown), (3, 1));
}

#[test]
fn typed_macro_evaluation_return_reads_all_four_views() {
    let (source, mut f) = typed_argument_source();
    let mac = match f.body.stmts.last_mut().unwrap() {
        syn::Stmt::Expr(syn::Expr::Macro(m), _) => &mut m.mac,
        syn::Stmt::Macro(m) => &mut m.mac,
        _ => panic!("fixture tail is a macro"),
    };
    let arguments = mac
        .tokens
        .to_string()
        .replace("suspend =>", "evaluation_return =>");
    mac.tokens = arguments.parse().unwrap();
    let edges = regions::definition_edges(&source, &f);
    assert!(edges.rejected.iter().any(|row| {
        row.reason
            .contains("inline evaluation-return event insertion before witnessed consumer")
    }));
    assert_eq!(edges.constructors.len(), 3);
    for claim in edges.constructors {
        let mut find = CallAt {
            source: &source,
            function: &f,
            site: &claim.site,
            callee: None,
        };
        find.visit_block(&f.body);
        assert!(find.callee.is_some());
    }
    assert_eq!(super::super::methods::claims(&source, &f).len(), 1);
    assert_eq!(
        super::super::types::constructor_claims(&source, &f).len(),
        3
    );
    println!("TYPED_MACRO_EVALUATION_RETURN regions/locator/methods/types");
}
