//! Every recorded A1 fixture fact is checked, not a hand-selected sample.
//! Production context claims check the three newly resolved legacy sites.

use super::super::{
    constructor_tests, contracts, effects, fixtures, leaves, model::*, regions, resolve, source,
    types,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    fs,
    process::Command,
};
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

fn columns(site: &Site) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}",
        site.file, site.start.0, site.start.1, site.end.0, site.end.1
    )
}

#[test]
fn exact_types_and_constructor_claims_match_rustc() {
    let source = fixtures::program();
    let directory = source::workspace().join("target/brief-534/oracle-exact");
    fs::create_dir_all(&directory).unwrap();
    let mut claims = BTreeMap::new();
    for name in constructor_tests::ACCEPTED
        .iter()
        .copied()
        .chain(["instances"])
    {
        let root = format!("fixture::constructors::{name}");
        let mut contracts = contracts();
        contracts.extend(leaves::instance_contracts(&source));
        let proof = effects::check(&source, &root, &contracts).unwrap_or_else(|e| panic!("{e}"));
        for fact in proof.facts {
            let Some(exact) = fact.exact else {
                continue;
            }; // transfer has no value
            if matches!(exact, ExactTy::Function(_)) {
                continue;
            } // checked as a resolution
            let mode = if fact.effect == "borrow-place" {
                "place"
            } else {
                "value"
            };
            claims.insert((fact.site, exact.spelling()), mode);
        }
    }
    for f in source
        .functions
        .values()
        .flatten()
        .filter(|f| f.id.starts_with("fixture::constructors::"))
    {
        for claim in types::constructor_claims(&source, f) {
            claims.insert((claim.site, claim.exact.spelling()), "value");
        }
    }
    let mut out = "name\tfile\tstart_line\tstart_column\tend_line\tend_column\texact_type\tmode\n"
        .to_string();
    for (i, ((site, exact), mode)) in claims.into_iter().enumerate() {
        writeln!(out, "type_{i}\t{}\t{exact}\t{mode}", columns(&site)).unwrap();
    }
    fs::write(directory.join("types.tsv"), out).unwrap();
    let mut out="name\tfile\tstart_line\tstart_column\tend_line\tend_column\ttarget\tkind\ttarget_file\ttarget_line\ttarget_column\n".to_string();
    let mut calls = Calls {
        source: &source,
        module: String::new(),
        rows: BTreeSet::new(),
        file: String::new(),
    };
    for f in source
        .functions
        .values()
        .flatten()
        .filter(|f| f.id.starts_with("fixture::constructors::"))
    {
        calls.module = f.module.clone();
        calls.file = f.site.file.clone();
        calls.visit_block(&f.body);
    }
    for (i, (site, target, kind)) in calls.rows.into_iter().enumerate() {
        let definition = resolve::function(&source, &target).ok();
        let (file, line, column) = definition
            .map(|f| {
                let p = f.sig.paren_token.span.close().start();
                (f.site.file.clone(), p.line, p.column + 1)
            })
            .unwrap_or_default();
        writeln!(
            out,
            "resolution_{i}\t{}\t{target}\t{kind}\t{file}\t{line}\t{column}",
            columns(&site)
        )
        .unwrap();
    }
    fs::write(directory.join("resolutions.tsv"), out).unwrap();

    let production = source::production();
    let frontier = regions::frontier(
        &production,
        &[
            format!("{}::merged_option_exprs_for_symbol", regions::SESSION),
            format!("{}::evaluate_runtime_option_expr", regions::SESSION),
        ],
    );
    let mut out =
        "name\tfile\tstart_line\tstart_column\tend_line\tend_column\ttarget\texact_type\n"
            .to_string();
    let mut total = 0;
    for id in &frontier.bodies {
        let f = resolve::function(&production, id).unwrap();
        for claim in types::constructor_claims(&production, f) {
            // None is a value, included too; all contextual claims in these
            // bodies are checked even though the baseline only counted calls.
            writeln!(
                out,
                "production_{total}\t{}\t{}\t{}",
                columns(&claim.site),
                claim.target,
                match &f.sig.output {
                    syn::ReturnType::Type(_, t)
                        if types::exact_output(&production, f).ok().as_ref()
                            == Some(&claim.exact) =>
                        tokens(&**t),
                    _ => claim.exact.spelling(),
                }
            )
            .unwrap();
            total += 1;
            println!(
                "KNOWN_TYPE_EFFECTS_SEPARATE\t{}\t{}\t{}",
                claim.site.display(),
                claim.target,
                claim.exact.spelling()
            );
        }
    }
    assert!(
        total >= 3,
        "three newly resolved constructor sites must have context claims"
    );
    fs::write(directory.join("production.tsv"), out).unwrap();
    let output = Command::new("python3")
        .arg(source::workspace().join("scripts/run-bounded.py"))
        .args(["--timeout", "700", "--", "python3"])
        .arg(
            source::workspace()
                .join("crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.py"),
        )
        .arg(source::workspace())
        .arg(&directory)
        .arg(if cfg!(feature = "hash") {
            "hash"
        } else {
            "no-default-features"
        })
        .output()
        .unwrap();
    fs::write(
        directory.join("harness.log"),
        [&output.stdout[..], &output.stderr[..]].concat(),
    )
    .unwrap();
    print!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "exact oracle instrument failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

struct Calls<'a> {
    source: &'a Source,
    module: String,
    file: String,
    rows: BTreeSet<(Site, Id, String)>,
}
impl<'ast> Visit<'ast> for Calls<'_> {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(p) = &*call.func {
            if p.qself.is_none() {
                if let Ok(id) = resolve::path(self.source, &self.module, &p.path) {
                    let kind = if resolve::standard_variant(&id).is_some() {
                        "constructor"
                    } else {
                        "function"
                    };
                    self.rows
                        .insert((Site::new(&self.file, p.span()), id, kind.into()));
                }
            }
        }
        visit::visit_expr_call(self, call);
    }
    fn visit_expr_path(&mut self, p: &'ast syn::ExprPath) {
        if let Ok(id) = resolve::path(self.source, &self.module, &p.path) {
            if resolve::standard_variant(&id).is_some_and(|(_, v)| v == "None") {
                self.rows
                    .insert((Site::new(&self.file, p.span()), id, "unit-variant".into()));
            }
        }
    }
}
