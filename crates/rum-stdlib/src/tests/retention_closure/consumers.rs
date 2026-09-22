//! Escaping copies are separate obligations, never option-region coverage.

use std::collections::BTreeSet;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};

use super::model::{Site, Source, tokens};
use super::resolve;

const EXPECTED: &[(&str, &str, &str)] = &[
    (
        "D3-predicates-same-test",
        "rum_stdlib::predicates::collections::effective_same_test",
        "value . clone ()",
    ),
    (
        "D3-sets-same-test",
        "rum_stdlib::structure_ops::ordering_sets::effective_same_test",
        "value . clone ()",
    ),
    (
        "D3-align-columns-separator",
        "rum_stdlib::strings::scripting::text_shape::parse_align_columns_options",
        "value . to_string ()",
    ),
    (
        "D3-url-parse-scheme",
        "rum_stdlib::strings::scripting::url::parse_url_parse_options",
        "default_scheme . map (str :: to_owned)",
    ),
];

struct Copies<'a> {
    spelling: &'a str,
    file: &'a str,
    sites: Vec<Site>,
}

impl<'ast> Visit<'ast> for Copies<'_> {
    fn visit_expr_method_call(&mut self, expr: &'ast syn::ExprMethodCall) {
        if tokens(expr) == self.spelling {
            self.sites.push(Site::new(self.file, expr.span()));
        }
        visit::visit_expr_method_call(self, expr);
    }
}

fn reconcile(source: &Source, inventory: &str) -> Result<Vec<String>, String> {
    let mut seen = BTreeSet::new();
    let mut records = Vec::new();
    for line in inventory
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let row: Vec<_> = line.split('\t').collect();
        if row.len() != 7 || row.iter().any(|field| field.is_empty()) {
            return Err("consumer obligation needs seven nonempty fields".into());
        }
        let expected = EXPECTED
            .iter()
            .find(|expected| expected.0 == row[0])
            .ok_or("unknown escaping consumer obligation")?;
        if !seen.insert(row[0]) {
            return Err("duplicate escaping consumer obligation".into());
        }
        if row[1] != "UnresolvedConsumer" || row[2] != expected.1 || row[4] != expected.2 {
            return Err(format!(
                "consumer obligation changed or claimed covered: {}",
                row[0]
            ));
        }
        let function = resolve::function(source, row[2])?;
        let mut copies = Copies {
            spelling: expected.2,
            file: &function.site.file,
            sites: Vec::new(),
        };
        copies.visit_block(&function.body);
        let [site] = copies.sites.as_slice() else {
            return Err(format!("escaping copy missing or ambiguous: {}", row[0]));
        };
        if row[3] != format!("{}:{}", site.file, site.start.0) {
            return Err(format!("escaping copy source anchor drifted: {}", row[0]));
        }
        records.push(format!("CONSUMER_OPEN\t{line}\t{}", site.display()));
    }
    if seen != EXPECTED.iter().map(|expected| expected.0).collect() {
        return Err("escaping consumer obligation removed".into());
    }
    Ok(records)
}

pub(super) fn verify(source: &Source) {
    let inventory = include_str!("inventory/consumer-obligations.tsv");
    for record in reconcile(source, inventory).expect("all four D3 copies remain open") {
        println!("{record}");
    }
    assert_eq!(
        reconcile(source, "").unwrap_err(),
        "escaping consumer obligation removed"
    );
    assert!(
        reconcile(
            source,
            &inventory.replacen("UnresolvedConsumer", "Covered", 1)
        )
        .unwrap_err()
        .contains("claimed covered")
    );
}
