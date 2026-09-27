//! Observe claims at the actual C02 sites, including expressions in refused
//! macro tokens. Compiler agreement supplies Rust facts, never macro admission.
use super::super::{
    super::{regions, source},
    oracle::{json, offset},
};
use super::*;
use std::{fs, process::Command};

#[test]
fn local_option_read_claims_match_stable_rustc() {
    let source = source::production();
    let index = Interfaces::new(&source);
    let marks = Marks::contracts(tests::records()).unwrap();
    let f = resolve::function(
        &source,
        &format!("{}::evaluate_runtime_option_expr", regions::SESSION),
    )
    .unwrap();
    let report = check(&index, f, &marks);
    let text = &source.files[&f.site.file];
    let body = Site::new(&f.site.file, f.body.span());
    let start = offset(text, body.start);
    let end = offset(text, body.end);
    let mut claims = vec![];
    let mut targets_seen = BTreeSet::new();
    for (site, ty) in &report.facts {
        if matches!(ty, ExactTy::Function(_)) || site.start < body.start || site.end > body.end {
            continue;
        }
        if matches!(ty, ExactTy::Reference(_, true)) {
            continue;
        }
        let raw = &text[offset(text, site.start)..offset(text, site.end)];
        let Ok(expr) = syn::parse_str::<syn::Expr>(raw) else {
            continue;
        };
        if !matches!(
            expr,
            syn::Expr::Path(_)
                | syn::Expr::Lit(_)
                | syn::Expr::Call(_)
                | syn::Expr::MethodCall(_)
                | syn::Expr::Binary(_)
                | syn::Expr::Index(_)
        ) {
            continue;
        }
        // Constructor/function identifiers and all unknown expressions remain
        // outside this claim set. Index places are borrowed by the carrier.
        let targets = report.targets.get(site).cloned().unwrap_or_default();
        let allowed =
            ["I01", "I02", "I04", "B04", "B05", "B13"].map(super::super::contracts::target);
        let selected = targets
            .intersection(&allowed.into_iter().collect())
            .cloned()
            .collect::<BTreeSet<_>>();
        if targets != selected {
            continue;
        }
        let place = matches!(syn::parse_str::<syn::Expr>(raw), Ok(syn::Expr::Index(_)));
        let adjustment = if selected.contains(&super::super::contracts::target("I02")) {
            json("&rum_core::expr::Expr")
        } else {
            "null".into()
        };
        targets_seen.extend(selected.iter().cloned());
        claims.push(format!("{{\"start\":{},\"end\":{},\"site\":{},\"exact\":{},\"place\":{},\"adjustment\":{},\"targets\":[{}]}}",
            offset(text,site.start)-start, offset(text,site.end)-start, json(&site.display()), json(&ty.spelling()), place, adjustment,
            selected.iter().map(|s| json(s)).collect::<Vec<_>>().join(",")));
    }
    for id in ["I01", "I02", "I04", "B04", "B05", "B13"] {
        assert!(
            targets_seen.contains(&super::super::contracts::target(id)),
            "missing {id}"
        );
    }
    let dir = source::workspace()
        .join(".temp/brief-542/oracle")
        .join(if cfg!(feature = "hash") {
            "hash"
        } else {
            "no-default-features"
        });
    fs::create_dir_all(&dir).unwrap();
    let mut sig = f.sig.clone();
    sig.ident = syn::Ident::new("__b542_option_reads", sig.ident.span());
    fs::write(
        dir.join("claims.json"),
        format!(
            "[{{\"name\":\"option_C02\",\"signature\":{},\"body\":{},\"claims\":[{}]}}]\n",
            json(&tokens(&sig)),
            json(&text[start..end]),
            claims.join(",")
        ),
    )
    .unwrap();
    let features = if cfg!(feature = "hash") {
        "hash"
    } else {
        "no-default-features"
    };
    let fingerprint =
        super::super::super::compiler_oracle::cache::fingerprint(&dir, &["claims.json"], features);
    if let Some(output) = super::super::super::compiler_oracle::cache::reuse(&dir, &fingerprint) {
        print!("{output}");
        return;
    }
    let result = Command::new("python3")
        .arg(source::workspace().join("scripts/run-bounded.py"))
        .args(["--timeout", "900", "--", "python3"])
        .arg(
            source::workspace()
                .join("crates/rum-stdlib/src/tests/retention_closure/local/reads/oracle.py"),
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
    let output = String::from_utf8_lossy(&result.stdout);
    super::super::super::compiler_oracle::cache::record(&dir, &fingerprint, &output);
    print!("{output}");
}
