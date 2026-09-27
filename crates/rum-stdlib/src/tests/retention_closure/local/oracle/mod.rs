//! Compiler observations of the actual probe, copied into its original lexical
//! module. Value claims use invariant carriers; resolution uses named MIR
//! temporaries and a different same-named live definition as negative control.
use super::super::source;
use super::*;
use std::{fs, process::Command};

pub(super) fn offset(text: &str, pos: (usize, usize)) -> usize {
    text.lines()
        .take(pos.0 - 1)
        .map(|l| l.len() + 1)
        .sum::<usize>()
        + pos.1
        - 1
}

pub(super) fn json(text: &str) -> String {
    let mut result = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c < ' ' => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

#[test]
fn local_probe_claims_match_stable_rustc() {
    let source = source::test_module(
        source::production(),
        "crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs",
        "rum_runtime::runtime::accounting_mode::tests",
    );
    let index = Interfaces::new(&source);
    let marks = Marks::contracts(
        contracts::read(include_str!("../../inventory/local-contracts.tsv")).unwrap(),
    )
    .unwrap()
    .with_covered(&index.earned);
    let mut functions = vec![];
    for name in ["probe", "start", "drive", "resume"] {
        let original = resolve::function(
            &source,
            &format!("rum_runtime::runtime::accounting_mode::tests::{name}"),
        )
        .unwrap();
        let modes = if name == "probe" {
            vec![None]
        } else {
            types::MODES
                .iter()
                .map(|id| Some(protocol::nominal(id)))
                .collect()
        };
        for (i, mode) in modes.into_iter().enumerate() {
            let f = types::with_mode(&source, original, mode.as_ref()).unwrap();
            let report = check_instance(&index, &f, &marks);
            assert!(report.rejected.is_empty(), "{}", report.text());
            let text = &source.files[&f.site.file];
            let body_site = Site::new(&f.site.file, f.body.span());
            let start = offset(text, body_site.start);
            let end = offset(text, body_site.end);
            struct Sites(Vec<(Site, bool, bool)>);
            impl<'a> Visit<'a> for Sites {
                fn visit_expr(&mut self, e: &'a syn::Expr) {
                    if matches!(
                        e,
                        syn::Expr::Call(_)
                            | syn::Expr::MethodCall(_)
                            | syn::Expr::Try(_)
                            | syn::Expr::Index(_)
                            | syn::Expr::Binary(_)
                            | syn::Expr::Lit(_)
                            | syn::Expr::Reference(_)
                            | syn::Expr::Field(_)
                            | syn::Expr::Path(_)
                    ) {
                        self.0.push((
                            Site::new("", e.span()),
                            matches!(e, syn::Expr::Index(_) | syn::Expr::Field(_)),
                            !matches!(e, syn::Expr::Path(_)),
                        ));
                    }
                    visit::visit_expr(self, e);
                }
            }
            let mut sites = Sites(vec![]);
            sites.visit_block(&f.body);
            let mut claims = vec![];
            for (mut site, place, value) in sites.0 {
                site.file = f.site.file.clone();
                let targets = report.targets.get(&site).cloned().unwrap_or_default();
                let ty = report.facts.get(&site);
                // Path function items are resolution-only. Other places retain
                // their original move/autoref behavior rather than coercing it.
                let exact = ty
                    .filter(|t| !matches!(t, ExactTy::Function(_)))
                    .filter(|_| value)
                    .map(ExactTy::spelling);
                if exact.is_none() && targets.is_empty() {
                    continue;
                }
                claims.push(format!("{{\"start\":{},\"end\":{},\"site\":{},\"exact\":{},\"place\":{},\"targets\":[{}]}}", offset(text,site.start)-start,offset(text,site.end)-start,json(&site.display()),exact.as_ref().map(|s|json(s)).unwrap_or_else(||"null".into()),place,targets.iter().map(|s|json(s)).collect::<Vec<_>>().join(",")));
            }
            let mut sig = original.sig.clone();
            sig.ident = syn::Ident::new(&format!("__b540_{name}_{i}"), sig.ident.span());
            sig.generics.params = sig
                .generics
                .params
                .into_iter()
                .filter(|p| matches!(p, syn::GenericParam::Lifetime(_)))
                .collect();
            let signature = tokens(&sig)
                .split_whitespace()
                .map(|s| {
                    if s == "M" {
                        mode.as_ref()
                            .unwrap()
                            .spelling()
                            .replace("rum_runtime::", "crate::")
                    } else {
                        s.into()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            functions.push(format!(
                "{{\"name\":{},\"signature\":{},\"body\":{},\"claims\":[{}]}}",
                json(&format!("{name}-{i}")),
                json(&signature),
                json(&text[start..end]),
                claims.join(",")
            ));
        }
    }
    let dir = source::workspace().join("target/brief-540/oracle");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("claims.json"),
        format!("[{}]\n", functions.join(",\n")),
    )
    .unwrap();
    let features = if cfg!(feature = "hash") {
        "hash"
    } else {
        "no-default-features"
    };
    let fingerprint =
        super::super::compiler_oracle::cache::fingerprint(&dir, &["claims.json"], features);
    if let Some(output) = super::super::compiler_oracle::cache::reuse(&dir, &fingerprint) {
        print!("{output}");
        return;
    }
    let result = Command::new("python3")
        .arg(source::workspace().join("scripts/run-bounded.py"))
        .args(["--timeout", "900", "--", "python3"])
        .arg(
            source::workspace()
                .join("crates/rum-stdlib/src/tests/retention_closure/local/oracle/probe.py"),
        )
        .arg(source::workspace())
        .arg(&dir)
        .arg(features)
        .output()
        .unwrap();
    fs::write(
        dir.join("harness.log"),
        [result.stdout.as_slice(), result.stderr.as_slice()].concat(),
    )
    .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let output = String::from_utf8_lossy(&result.stdout).into_owned();
    super::super::compiler_oracle::cache::record(&dir, &fingerprint, &output);
    print!("{output}");
}
