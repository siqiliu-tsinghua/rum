mod argument_traversal_resolution;
mod associations_roles;
mod boundaries;
mod capture_output;
mod control_cleanup;
mod control_roles;
mod debug_roles;
mod definition_application;
mod definition_lhs_cycle;
mod definition_storage;
mod functional_roles;
mod guards_binding_completion;
mod part_selectors_held_assignment;
mod predicate_roles;
pub mod preparation;
pub mod primary;
mod replaceall_traversal_collectors;
mod restore_boundary;
use super::descriptors::{Fixture, Pause};
use rum_runtime::RuntimeInputKind;
mod debug;
mod definitions;
mod evaluation_dispatch;
mod evaluation_stack;
mod lazy;
mod loops;
mod options;
mod patterns;
mod rewrite;
mod rewrite_roles;
mod rule_construction;
mod rules;
mod schema;
mod scoping;
mod shared_completion;
mod status_function;
mod strings;
mod strings_roles;
mod typed_slots;

fn make(
    id: &str,
    source: &str,
    pauses: &[(&str, &str, &str)],
    value: Option<&str>,
    log: &str,
) -> Fixture {
    let mut f = Fixture::new(id, source);
    f.configure = super::support::files::configure;
    f.pauses = pauses
        .iter()
        .map(|(kind, prompt, answer)| Pause {
            kind: match *kind {
                "Input" => RuntimeInputKind::Input,
                "InputString" => RuntimeInputKind::InputString,
                _ => panic!("invalid kind"),
            },
            prompt: (*prompt).into(),
            answer: (*answer).into(),
        })
        .collect();
    let label = id.split_once('.').unwrap().1;
    if label.starts_with("destructure-") || label == "Set-destructure" {
        f.projections.push("{Global`a,Global`b,Global`r}".into());
    }
    if label.starts_with("default-") {
        f.projections
            .push("{Global`n,DefaultValues[Global`d],DownValues[Global`d]}".into());
    }
    if label.starts_with("OptionValue-") || label == "Option-scope" {
        f.projections.push("Options[Global`f]".into());
    }
    f.expected_value = value.map(str::to_owned);
    f.expected_log = Some(log.into());
    f
}
pub fn historical() -> Vec<Fixture> {
    let mut rows = Vec::new();
    rows.extend(loops::fixtures());
    rows.extend(lazy::fixtures());
    rows.extend(schema::fixtures());
    rows.extend(patterns::fixtures());
    rows.extend(rule_construction::fixtures());
    rows.extend(debug::fixtures());
    rows.extend(options::fixtures());
    rows.extend(scoping::fixtures());
    rows.extend(rewrite::fixtures());
    rows.extend(strings::fixtures());
    rows.extend(definitions::fixtures());
    rows
}

#[test]
fn historical_label_ledger_and_equality() {
    let rows = historical();
    assert_eq!(rows.len(), 209, "exact historical fixture population");
    let ledger = include_str!("../inventory/historical-labels.tsv");
    let ids: std::collections::BTreeSet<_> = ledger
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| l.split('\t').nth(2).unwrap())
        .collect();
    assert_eq!(
        ledger.lines().filter(|l| l.starts_with("548\t")).count(),
        195
    );
    assert_eq!(
        ledger.lines().filter(|l| l.starts_with("583\t")).count(),
        14
    );
    assert_eq!(ids.len(), 209);
    assert_eq!(ids, rows.iter().map(|f| f.id.as_str()).collect());
    let mut rejected = Vec::new();
    for f in &rows {
        if let Err(error) = super::descriptors::accept(f, super::descriptors::Checks::ALL) {
            rejected.push(format!("{}: {error:?}", f.id));
        } else {
            println!("HISTORICAL {} PASS", f.id);
        }
    }
    assert!(rejected.is_empty(), "{}", rejected.join("\n"));
    println!("HISTORICAL labels=209 (195 + 14) equality=209");
}
