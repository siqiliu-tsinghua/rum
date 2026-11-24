//! Admission of row-specific native differences, separate from full receipts.
use super::source_auth::observer::ROW;
use crate::oracle::fixtures::obligations::guest::{self, Program, RunObservation};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ObserverRejection {
    WrongRow,
    Unarmed,
    SourceHit,
    ProducerPause,
    NativeDifference,
    PublicOutput,
    OwnPredicate,
}

pub(crate) fn validate(
    program: &Program,
    selector: &str,
    hit: Option<(&str, &str)>,
    baseline: &RunObservation,
    resumed: &RunObservation,
    queued: &RunObservation,
) -> Result<(), ObserverRejection> {
    let specs = super::schema::specs();
    let Some(spec) = specs
        .get(program.key.row())
        .filter(|s| s.observer == "NativeEqual")
    else {
        return Err(ObserverRejection::WrongRow);
    };
    if selector != spec.selector {
        return Err(ObserverRejection::Unarmed);
    }
    if !hit.is_some_and(|(row, phase)| row == spec.row && spec.phases.contains(&phase)) {
        return Err(ObserverRejection::SourceHit);
    }
    if !baseline.pauses.iter().any(|p| {
        guest::credited_phase(&p.phase)
            && program.phases.contains(&p.phase)
            && p.producer_route == program.producer_route
            && matches!(&p.output, crate::HostEvalOutcome::InputRequired(out)
                if spec.row != ROW || (p.phase == format!("{ROW}:consumer:operand-1") && out.request.prompt == "finish"))
    }) {
        return Err(ObserverRejection::ProducerPause);
    }
    // An epoch fault or the quiescence panic is deliberately not an input.
    if resumed.counters == queued.counters && resumed.native == queued.native {
        return Err(ObserverRejection::NativeDifference);
    }
    if resumed.output != queued.output {
        return Err(ObserverRejection::PublicOutput);
    }
    if guest::validate(program, resumed, queued) != Err(guest::RunFailure::Native) {
        return Err(ObserverRejection::OwnPredicate);
    }
    Ok(())
}

#[test]
fn cm_mutation_row_observer_predicate_controls() {
    let programs = guest::programs().unwrap();
    let cases = super::schema::cases();
    for spec in super::schema::specs()
        .values()
        .filter(|s| s.observer == "NativeEqual")
    {
        let case = cases
            .iter()
            .find(|c| c.kind == "own" && c.row == spec.row)
            .unwrap();
        let p = &programs[case.program];
        let baseline = guest::run(p, false).unwrap();
        let queued = guest::run(p, true).unwrap();
        guest::validate(p, &baseline, &queued).unwrap();
        // Synthetic predicate falsifiers only; no mutation receipt is constructed.
        let mut different = baseline.clone();
        different
            .counters
            .push(("validator-control-only".into(), [1; 10]));
        let mut absent = baseline.clone();
        absent.pauses.clear();
        let hit = Some((spec.row, spec.phases[0]));
        assert_eq!(
            validate(p, spec.row, hit, &baseline, &different, &queued),
            Ok(())
        );
        for (name, result, error) in [
            (
                "unarmed",
                validate(p, "", hit, &baseline, &different, &queued),
                ObserverRejection::Unarmed,
            ),
            (
                "no-hit",
                validate(p, spec.row, None, &baseline, &different, &queued),
                ObserverRejection::SourceHit,
            ),
            (
                "wrong-phase",
                validate(
                    p,
                    spec.row,
                    Some((spec.row, "wrong")),
                    &baseline,
                    &different,
                    &queued,
                ),
                ObserverRejection::SourceHit,
            ),
            (
                "another-row",
                validate(
                    p,
                    spec.row,
                    Some(("P0543", spec.phases[0])),
                    &baseline,
                    &different,
                    &queued,
                ),
                ObserverRejection::SourceHit,
            ),
            (
                "no-pause",
                validate(p, spec.row, hit, &absent, &different, &queued),
                ObserverRejection::ProducerPause,
            ),
            (
                "no-difference",
                validate(p, spec.row, hit, &baseline, &baseline, &queued),
                ObserverRejection::NativeDifference,
            ),
            (
                "quiescence-only",
                validate(p, spec.row, hit, &baseline, &baseline, &queued),
                ObserverRejection::NativeDifference,
            ),
        ] {
            assert_eq!(result, Err(error), "{} {name}", spec.row);
            println!(
                "CM4 ROW OBSERVER NEGATIVE row={} control={name} predicate={error:?} semantic_catch=false",
                spec.row
            );
        }
        println!(
            "CM4 ROW OBSERVER PREDICATE PASS row={} negatives=7 native_difference_required=true public_output_exact=true",
            spec.row
        );
    }
}

#[test]
fn cm_mutation_observer_predicate_controls() {
    let programs = guest::programs().unwrap();
    let program = programs
        .values()
        .find(|p| {
            p.key.row() == ROW
                && p.key.variant() == "Middle"
                && p.key.context() == "phase_observation.trace"
        })
        .unwrap();
    let baseline = guest::run(program, false).unwrap();
    let queued = guest::run(program, true).unwrap();
    guest::validate(program, &baseline, &queued).unwrap();
    // A validator fixture only. It is never a source mutation, a copied-build
    // receipt, or a CaughtRow; real admission uses the unchanged observations.
    let mut differing = baseline.clone();
    differing
        .counters
        .push(("validator-control-only".into(), [1; 10]));
    let hit = Some((ROW, "completed-generation-inert-pause"));
    assert_eq!(
        validate(program, ROW, hit, &baseline, &differing, &queued),
        Ok(())
    );
    let mut no_pause = baseline.clone();
    no_pause.pauses.clear();
    let mut wrong_row = program.clone();
    wrong_row.key.0[0] = "P0543".into();
    for (name, result, expected) in [
        (
            "wrong-row",
            validate(&wrong_row, ROW, hit, &baseline, &differing, &queued),
            ObserverRejection::WrongRow,
        ),
        (
            "unarmed-selector",
            validate(program, "", hit, &baseline, &differing, &queued),
            ObserverRejection::Unarmed,
        ),
        (
            "absent-source-hit",
            validate(program, ROW, None, &baseline, &differing, &queued),
            ObserverRejection::SourceHit,
        ),
        (
            "wrong-phase",
            validate(
                program,
                ROW,
                Some((ROW, "other")),
                &baseline,
                &differing,
                &queued,
            ),
            ObserverRejection::SourceHit,
        ),
        (
            "absent-producer-pause",
            validate(program, ROW, hit, &no_pause, &differing, &queued),
            ObserverRejection::ProducerPause,
        ),
        (
            "absent-native-difference",
            validate(program, ROW, hit, &baseline, &baseline, &queued),
            ObserverRejection::NativeDifference,
        ),
        (
            "quiescence-only-equal-native",
            validate(program, ROW, hit, &baseline, &baseline, &queued),
            ObserverRejection::NativeDifference,
        ),
        (
            "fault-from-another-row",
            validate(
                program,
                ROW,
                Some(("P0543", "completed-generation-inert-pause")),
                &baseline,
                &differing,
                &queued,
            ),
            ObserverRejection::SourceHit,
        ),
    ] {
        assert_eq!(result, Err(expected));
        println!(
            "CM4 OBSERVER PREDICATE NEGATIVE control={name} predicate={result:?} semantic_catch=false"
        );
    }
    println!(
        "CM4 OBSERVER PREDICATE PASS negative_controls=8 final_scalar_comparison=false semantic_catches=0"
    );
}
