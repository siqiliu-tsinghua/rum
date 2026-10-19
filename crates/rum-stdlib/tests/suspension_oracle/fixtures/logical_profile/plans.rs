use super::super::super::{
    descriptors::{Fixture, Pause, Variant},
    support::logical_profile::unhex,
};
use rum_runtime::RuntimeInputKind;

pub struct Plan {
    pub id: String,
    pub group: String,
    pub row: String,
    pub fixture: Fixture,
}
fn variant(v: &str) -> Variant {
    match v {
        "Middle" => Variant::Middle,
        "Twice" => Variant::Twice,
        "Decline" => Variant::Decline,
        "Error" => Variant::Error,
        "AlreadyResumed" => Variant::AlreadyResumed,
        "Nested" => Variant::Nested,
        _ => panic!("undeclared variant {v}"),
    }
}
pub fn plans() -> Vec<Plan> {
    let descriptors = super::super::primary::descriptors();
    super::super::super::support::logical_profile::rows(include_str!("plans.tsv"), 12)
        .unwrap()
        .into_iter()
        .map(|r| {
            let r = r
                .into_iter()
                .map(|s| (s != "-").then(|| unhex(s)))
                .collect::<Vec<_>>();
            let get = |i: usize| r[i].as_deref().unwrap();
            let template = if get(1) == "-" {
                Fixture::new(get(0), "")
            } else if let Some(index) = get(1).strip_prefix("historical:") {
                super::super::historical().remove(index.parse().unwrap())
            } else {
                let d = descriptors.iter().find(|d| d.id == get(1)).unwrap();
                (d.factory)(d.entry, variant(get(2)))
            };
            // A separate counter context retains the original session configuration,
            // state projections, files and declared duration leaves. Its value/log
            // expectations come from the frozen queued receipts, not an unrelated
            // descriptor's unwrapped source.
            let mut f = Fixture::new(get(0), r[4].as_deref().unwrap_or(&template.source));
            f.setup = r[5].as_ref().unwrap_or(&template.setup).clone();
            f.configure = template.configure;
            f.projections = template.projections;
            f.files = template.files;
            f.duration_paths = if let Some(s) = &r[7] {
                if s.is_empty() {
                    vec![]
                } else {
                    s.split(';')
                        .map(|p| p.split(',').map(|v| v.parse().unwrap()).collect())
                        .collect()
                }
            } else {
                template.duration_paths
            };
            f.pauses = if let Some(s) = &r[6] {
                if s.is_empty() {
                    vec![]
                } else {
                    let fields = s.split('\0').collect::<Vec<_>>();
                    assert_eq!(fields.len() % 3, 0);
                    fields
                        .chunks_exact(3)
                        .map(|p| Pause {
                            kind: match p[0] {
                                "Input" => RuntimeInputKind::Input,
                                "InputString" => RuntimeInputKind::InputString,
                                _ => panic!("unknown kind"),
                            },
                            prompt: p[1].into(),
                            answer: p[2].into(),
                        })
                        .collect()
                }
            } else {
                template.pauses
            };
            f.expected_value = r[10].clone();
            f.expected_log = r[11].clone();
            f.logical_profile = true;
            f.profile_trace = get(3) == "1";
            Plan {
                id: get(0).into(),
                group: get(8).into(),
                row: get(9).into(),
                fixture: f,
            }
        })
        .collect()
}

/// Attribute the newly bound rows to the actual rule-selection arm, not merely
/// to a shared phase name. The source hash and trace pin the rule; dispatch
/// decision counters distinguish ground, fast-pattern and compiled-matcher arms.
pub fn prove_new(p: &Plan, q: &super::super::super::descriptors::ResultView) {
    if p.group != "new" {
        return;
    }
    let s = q.profile.as_ref().unwrap();
    let arm = match p.row.as_str() {
        "P0256" => Some(("down_value", "f", 0)),
        "P0259" => Some(("down_value", "f", 2)),
        "P0266" => Some(("down_value", "f", 4)),
        "P0314" => Some(("up_value", "a", 4)),
        "P0315" => Some(("up_value", "a", 2)),
        _ => None,
    };
    if let Some((kind, symbol, column)) = arm {
        let prefix = format!("definition-dispatch|{kind}|{symbol}|");
        let counts = s
            .logical
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| v)
            .collect::<Vec<_>>();
        assert!(!counts.is_empty(), "{} designated producer absent", p.id);
        assert!(
            counts.iter().any(|v| v[column] > 0),
            "{} wrong rule-selection arm",
            p.id
        );
        assert!(q.observation.trace.iter().any(|e| matches!(e,
            rum_runtime::TraceEvent::RuleApplied { source, .. }
            if format!("{source:?}") == if kind == "down_value" { "DownValue" } else { "UpValue" })), "{} rule trace absent", p.id);
    }
    if p.row == "P0817" {
        assert!(
            p.fixture
                .source
                .contains("WithCleanup[Throw[1,lpTag],Throw[2,lpTag]]")
        );
        assert!(
            s.logical
                .iter()
                .any(|(k, v)| k.starts_with("runtime|catch.resume_after_body|") && v[6] > 0),
            "{} incoming bypassing Throw unobserved",
            p.id
        );
    }
    println!(
        "LP01 ATTRIBUTED {} row={} planned_requests={} source={}",
        p.id,
        p.row,
        p.fixture.pauses.len(),
        fixture_fingerprint(&p.fixture)
    );
}

pub fn fixture_fingerprint(f: &Fixture) -> String {
    let pauses = f
        .pauses
        .iter()
        .map(|p| format!("{:?}\0{}\0{}", p.kind, p.prompt, p.answer))
        .collect::<Vec<_>>()
        .join("\0");
    super::super::super::support::logical_profile::fingerprint(&[
        &f.setup,
        &f.source,
        &pauses,
        &format!("{:?}", f.projections),
        &format!("{:?}", f.duration_paths),
        &format!("{:?}", f.files),
        &f.profile_trace.to_string(),
    ])
}
