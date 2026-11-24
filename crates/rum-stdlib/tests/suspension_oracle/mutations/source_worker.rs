//! Runs the existing CM3 observations through the authenticated source entry point.
use super::{
    schema,
    source_auth::{self, Armed, Disabled},
};
use crate::oracle::fixtures::obligations::{boundaries, guest};

fn hex(text: &str) -> String {
    text.as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn cm_copy_observation_worker() {
    let Ok(row) = std::env::var("RUM_CM740_ROW") else {
        return;
    };
    let specs = schema::specs();
    let spec = specs.get(row.as_str()).expect("declared own row");
    let selected = std::env::var("RUM_CM740_PROGRAM").ok();
    let arm = std::env::var("RUM_CM740_ARM").unwrap();
    let profile = std::env::var("RUM_CM740_PROFILE").unwrap_or_else(|_| "on".into());
    assert!(matches!(profile.as_str(), "on" | "off"));
    for name in [
        "RUM_PROFILE_LEXICAL_SUBSTITUTION",
        "RUM_PROFILE_BUILTIN_TIMING",
        "RUM_PROFILE_DEFINITION_RULE_TIMING",
        "RUM_PROFILE_RULE_RHS_TIMING",
        "RUM_PROFILE_EVALUATION_CACHE",
    ] {
        assert_eq!(
            std::env::var(name).as_deref(),
            Ok(if profile == "off" { "0" } else { "1" }),
            "exact CM3 profile group"
        );
    }
    assert_eq!(
        std::env::var("RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL").as_deref(),
        Ok("0")
    );
    println!(
        "CM740_RUN_ID row={row} source_key={} tail_digest={} schema={} arm={arm} binary={} selector={} profile={profile}",
        spec.source_key,
        spec.tail_digest,
        spec.source_schema,
        crate::oracle::obligations::content_digest(
            &std::fs::read(std::env::current_exe().unwrap()).unwrap()
        ),
        std::env::var("RUM_CM740_OMIT").unwrap_or_default()
    );
    let authenticated = (arm == "armed").then(|| {
        source_auth::authenticate::<Armed>(
            std::env::var_os("RUM_CM740_BASELINE").unwrap().as_ref(),
            std::env::var_os("RUM_CM740_RECEIPT").unwrap().as_ref(),
        )
        .unwrap()
    });
    let programs = if arm == "real" {
        guest::programs().unwrap()
    } else {
        let baseline = std::env::var_os("RUM_CM740_BASELINE").unwrap();
        let receipt = std::env::var_os("RUM_CM740_RECEIPT").unwrap();
        match arm.as_str() {
            "disabled" => guest::programs_with_mutation(
                &source_auth::authenticate::<Disabled>(baseline.as_ref(), receipt.as_ref())
                    .unwrap(),
            ),
            "armed" => guest::programs_with_mutation(authenticated.as_ref().unwrap()),
            _ => panic!("unrecognized arm"),
        }
        .unwrap()
    };
    let expected = programs
        .values()
        .filter(|p| {
            p.key.row() == row
                && selected.as_ref().is_none_or(|id| &p.id == id)
                && p.key.0[3].split(';').any(|s| s == "PROFILED=false") == (profile == "off")
        })
        .count();
    assert!(expected > 0);
    let ordinary_forced = std::env::var("RUM_CM740_ORDINARY").as_deref() == Ok("1");
    let (class, transport) = source_auth::observer_transport_for(&row, &arm, ordinary_forced);
    println!(
        "CM743_TRANSPORT row={row} arm={arm} class={class} observer_transport={transport} ordinary_forced={ordinary_forced}"
    );
    let mut count = 0;
    for p in programs.values().filter(|p| {
        p.key.row() == row
            && selected.as_ref().is_none_or(|id| &p.id == id)
            && p.key.0[3].split(';').any(|s| s == "PROFILED=false") == (profile == "off")
    }) {
        assert!(!p.key.context().starts_with("guest_heads."));
        println!("CM740_CASE\t{}\t{}", hex(&p.key.line()), hex(&p.id));
        let observer = if transport {
            Some(authenticated.as_ref().unwrap().observer(&row).unwrap())
        } else {
            None
        };
        let mut constituent = Some(p);
        let mut ordinal = 0;
        while let Some(p) = constituent {
            println!(
                "CM740_CONSTITUENT\t{}\t{}\t{ordinal}",
                hex(&p.key.line()),
                hex(&p.id)
            );
            let run = |queued| match &observer {
                Some(permission) => guest::run_with_mutation_observer(p, queued, permission),
                None => guest::run(p, queued),
            };
            let resumed = run(false).unwrap();
            let queued = run(true).unwrap();
            let verdict = guest::validate(p, &resumed, &queued);
            let pauses = resumed
                .pauses
                .iter()
                .filter(|pause| guest::credited_phase(&pause.phase))
                .count();
            let raw_observation = format!(
                "CM740_OBSERVATION\t{}\t{}\t{}\t{verdict:?}\t{pauses}",
                hex(&p.key.line()),
                hex(&format!("{resumed:?}")),
                hex(&format!("{queued:?}"))
            );
            println!("{raw_observation}");
            if super::duration_parity::applies(&row) && arm != "armed" {
                let comparison = super::duration_parity::observation_record(
                    p, &resumed, &queued, verdict, pauses,
                )
                .unwrap();
                println!(
                    "CM740_DURATION_PARITY\t{}\t{}",
                    crate::oracle::obligations::content_digest(raw_observation.as_bytes()),
                    hex(&comparison)
                );
            }
            if arm == "armed" {
                assert_eq!(std::env::var("RUM_CM740_OMIT").as_deref(), Ok(row.as_str()));
                let public_same = resumed.output == queued.output;
                let native_diff =
                    resumed.counters != queued.counters || resumed.native != queued.native;
                println!(
                    "CM740_OWN_PREDICATE row={row} program={} predicate={verdict:?} public_output_exact={public_same} native_difference={native_diff} observer={}",
                    hex(&p.id),
                    spec.observer
                );
                // A survivor or earlier guard is reported as such, never asserted into a catch.
            } else {
                assert!(pauses > 0, "baseline producer pause");
                verdict.unwrap();
                let lifecycle = boundaries::run(p).unwrap();
                boundaries::validate(p, &lifecycle).unwrap();
                println!(
                    "CM740_LIFECYCLE\t{}\t{}",
                    hex(&p.key.line()),
                    hex(&format!("{lifecycle:?}"))
                );
            }
            constituent = p.paired.as_deref();
            ordinal += 1;
        }
        count += 1;
    }
    assert_eq!(count, expected);
    println!("CM740_OBSERVATION_END row={row} records={count} arm={arm}");
}
