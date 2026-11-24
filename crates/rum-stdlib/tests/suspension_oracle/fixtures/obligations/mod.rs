//! Immutable programs imported from authenticated CM evidence packets.
pub(crate) mod boundaries;
pub(crate) mod guest;
pub(crate) mod native;
pub(crate) mod proof_controls;
pub(crate) mod semantic_controls;

// Compile the canonical loader at the same module depth so its fixture
// references retain their original meaning. There is no second resolver.
#[allow(dead_code)]
#[path = "../logical_profile/plans.rs"]
pub(crate) mod canonical_plans;

#[derive(Clone)]
pub(crate) struct CanonicalFixture(pub super::super::descriptors::Fixture);

impl std::fmt::Debug for CanonicalFixture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanonicalFixture")
            .field("id", &self.0.id)
            .field(
                "fingerprint",
                &canonical_plans::fixture_fingerprint(&self.0),
            )
            .finish()
    }
}

#[test]
fn cm_canonical_plans_preserve_frozen_796() {
    use crate::oracle::support::logical_profile;
    let plans = canonical_plans::plans();
    let fingerprints =
        logical_profile::rows(include_str!("../logical_profile/fingerprints.tsv"), 2)
            .unwrap()
            .into_iter()
            .map(|row| (row[0], row[1]))
            .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(plans.len(), 796);
    assert_eq!(fingerprints.len(), plans.len());
    for plan in plans {
        assert_eq!(
            canonical_plans::fixture_fingerprint(&plan.fixture),
            fingerprints[plan.id.as_str()],
            "canonical fixture changed: {}",
            plan.id
        );
    }
    println!("CM CANONICAL LEGACY PASS plans=796 frozen_fingerprints=796 executions=0");
}

#[test]
#[ignore = "exports canonical derivations for scratch authentication; credits no execution"]
fn cm_export_canonical_plans() {
    use crate::oracle::support::logical_profile::json;
    let input =
        std::env::var_os("RUM_CM739_CANONICAL_INPUT").expect("authenticated canonical input");
    let text = std::fs::read_to_string(input).unwrap();
    let plans = canonical_plans::from_rows(&text);
    let mut seen = std::collections::BTreeSet::new();
    for plan in &plans {
        assert!(seen.insert(&plan.id), "duplicate canonical plan");
        let f = &plan.fixture;
        let pauses = f
            .pauses
            .iter()
            .map(|p| {
                format!(
                    "[{}, {}, {}]",
                    json(&format!("{:?}", p.kind)),
                    json(&p.prompt),
                    json(&p.answer)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let projections = f
            .projections
            .iter()
            .map(|s| json(s))
            .collect::<Vec<_>>()
            .join(",");
        let files = f.files.as_ref().map_or_else(
            || "null".into(),
            |files| {
                format!(
                    "[{}]",
                    files
                        .iter()
                        .map(|(path, text)| format!("[{},{}]", json(path), json(text)))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            },
        );
        println!(
            "CM_CANONICAL {{\"id\":{},\"source\":{},\"setup\":{},\"inputs\":[{pauses}],\"trace\":{},\"projections\":[{projections}],\"files\":{files},\"duration_paths\":{:?},\"fingerprint\":{}}}",
            json(&plan.id),
            json(&f.source),
            json(&f.setup),
            f.profile_trace,
            f.duration_paths,
            json(&canonical_plans::fixture_fingerprint(f))
        );
    }
    println!(
        "CM CANONICAL DERIVATION PASS plans={} executions=0",
        plans.len()
    );
}
