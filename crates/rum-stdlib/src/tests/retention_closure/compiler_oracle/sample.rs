//! A small explicit sample, selected from live census claims (never invented
//! edges). Duplicate names exercise lexical/import/inherent identity handling.

use std::collections::BTreeSet;
use std::fmt::Write;
use std::fs;
use std::path::Path;

use super::super::model::*;
use super::super::{census, regions::ResolvedEdge, resolve, types};

fn site_fields(site: &Site) -> Vec<String> {
    vec![
        site.file.clone(),
        site.start.0.to_string(),
        site.start.1.to_string(),
        site.end.0.to_string(),
        site.end.1.to_string(),
    ]
}

fn signature_fields(source: &Source, id: &str) -> Vec<String> {
    let f = resolve::function(source, id).unwrap();
    let close = f.sig.paren_token.span.close().start();
    vec![
        id.into(),
        f.site.file.clone(),
        close.line.to_string(),
        (close.column + 1).to_string(),
        if f.sig.inputs.is_empty() || f.sig.inputs.trailing_punct() {
            ""
        } else {
            ", "
        }
        .into(),
    ]
}

pub(super) fn write(source: &Source, claims: &BTreeSet<ResolvedEdge>, directory: &Path) {
    let pairs = [
        (
            "n_dispatch",
            "control_flow::dispatch",
            "arithmetic::dispatch",
        ),
        (
            "imported_constructor",
            "number_theory::IntegerWorkBudget::new",
            "csv::CsvParseError::new",
        ),
        (
            "same_file_constructor",
            "csv::CsvParseError::new",
            "csv::CsvEncodeError::new",
        ),
        (
            "module_function",
            "bytes::codec::base64_decode",
            "strings::encoding::base64_decode",
        ),
        (
            "reexport",
            "associations::data::predicate_bool_expr",
            "predicates::predicate_bool_expr",
        ),
        (
            "sibling_import",
            "yaml::decode::scalars::count_leading_spaces",
            "yaml::decode::quoted::count_leading_spaces",
        ),
    ];
    let mut out = "name\tcaller\tfile\tstart_line\tstart_column\tend_line\tend_column\ttarget\ttarget_file\ttarget_line\ttarget_column\ttarget_prefix\tnegative\tnegative_file\tnegative_line\tnegative_column\tnegative_prefix\tcontrol_file\tcontrol_start_line\tcontrol_start_column\tcontrol_end_line\tcontrol_end_column\n".to_string();
    for (name, target, negative) in pairs {
        let target = format!("rum_stdlib::{target}");
        let negative = format!("rum_stdlib::{negative}");
        assert_ne!(target, negative);
        assert_eq!(target.rsplit("::").next(), negative.rsplit("::").next());
        let edge = claims
            .iter()
            .find(|e| {
                e.target == target
                    && (name != "n_dispatch"
                        || e.definition == "rum_stdlib::control_flow::eval_assert")
            })
            .expect("sample must remain a census claim");
        if name == "n_dispatch" {
            assert!(
                census::n_roots(source)
                    .iter()
                    .any(|root| root.target.as_ref().ok() == Some(&edge.definition)),
                "N sample must remain a live N attachment"
            );
        }
        let control = claims
            .iter()
            .find(|e| e.target == negative)
            .expect("negative definition must have a live call to witness its mutation");
        let mut fields = vec![name.into(), edge.definition.clone()];
        fields.extend(site_fields(&edge.site));
        fields.extend(signature_fields(source, &target));
        fields.extend(signature_fields(source, &negative));
        fields.extend(site_fields(&control.site));
        writeln!(out, "{}", fields.join("\t")).unwrap();
    }
    fs::write(directory.join("resolution-claims.tsv"), out).unwrap();

    let mut out =
        "name\tcaller\tfile\tstart_line\tstart_column\tend_line\tend_column\ttarget\texact_type\n"
            .to_string();
    for (name, target) in [
        ("bool", "rum_stdlib::datetime::calendar::is_leap_year"),
        (
            "usize",
            "rum_stdlib::arithmetic::stats::projected_expr_memory_proxy_bytes",
        ),
    ] {
        let edge = claims.iter().find(|e| e.target == target).unwrap();
        let ty = types::output(source, resolve::function(source, target).unwrap()).unwrap();
        let Ty::Scalar(exact) = ty else {
            panic!("type probe requires a concrete Copy scalar claim, got {ty:?}");
        };
        let mut fields = vec![name.into(), edge.definition.clone()];
        fields.extend(site_fields(&edge.site));
        fields.extend([edge.target.clone(), exact]);
        writeln!(out, "{}", fields.join("\t")).unwrap();
    }
    fs::write(directory.join("type-claims.tsv"), out).unwrap();
}
