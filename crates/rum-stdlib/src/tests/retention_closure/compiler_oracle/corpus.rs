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
    let directory = source::workspace().join("target/brief-534/oracle-corpus");
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
        let mut find = CallAt {
            file: &site.file,
            site,
            callee: None,
        };
        for f in functions {
            find.visit_block(&f.body);
        }
        let (callee, callee_site) = find.callee.expect("every claim is an actual indexed call");
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
    file: &'a str,
    site: &'a Site,
    callee: Option<(String, Site)>,
}
impl<'ast> Visit<'ast> for CallAt<'_> {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if Site::new(self.file, call.span()) == *self.site {
            self.callee = Some((tokens(&*call.func), Site::new(self.file, call.func.span())));
        }
        visit::visit_expr_call(self, call);
    }
}
