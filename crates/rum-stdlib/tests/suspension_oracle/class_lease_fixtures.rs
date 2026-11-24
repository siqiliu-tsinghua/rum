//! Generated non-admission class-lease fixtures. Not an importable receipt.
use super::oracle::fixtures::obligations::guest::{self, RunObservation};
use crate::HostEvalOutcome;

fn output_text(observed: &RunObservation) -> String {
    match &observed.output {
        HostEvalOutcome::Success(success) => success.value.text.clone(),
        other => format!("{other:?}"),
    }
}

fn legacy_positive_receipt(line: &str) -> bool {
    if !(line.starts_with("CLASS_RECEIPT ") && line.contains(" commit=true ")) {
        return false;
    }
    let fields = line.contains(" kind=")
        && line.contains(" issuer=")
        && line.contains(" owner=")
        && line.contains(" occurrence=")
        && line.contains(" slot=");
    if !fields {
        return false;
    }
    if line.contains(" issuer=Return ") {
        return line.contains(" kind=EvalError::InputRequired ");
    }
    !line.contains(" generation=0 ")
}

fn kind_matches(kind: &str, frame: &str) -> bool {
    !frame.is_empty() && (kind == frame || kind.ends_with(&format!("::{frame}")))
}

fn kind_of(line: &str) -> &str {
    match line.find(" kind=") {
        Some(at) => {
            let rest = &line[at + " kind=".len()..];
            match rest.find(' ') {
                Some(end) => &rest[..end],
                None => rest,
            }
        }
        None => "",
    }
}

fn field<'a>(line: &'a str, key: &str) -> &'a str {
    let marker = format!(" {key}=");
    match line.find(&marker) {
        Some(at) => {
            let rest = &line[at + marker.len()..];
            match rest.find(' ') {
                Some(end) => &rest[..end],
                None => rest,
            }
        }
        None => "",
    }
}

fn contract_hit(group: &str, frame: &str, lines: &[&str]) -> bool {
    fn receipt(line: &str) -> bool {
        line.starts_with("CLASS_RECEIPT ")
            && line.contains(" commit=true ")
            && line.contains(" kind=")
            && line.contains(" issuer=")
            && line.contains(" owner=")
            && line.contains(" occurrence=")
            && line.contains(" slot=")
            && (!line.contains(" issuer=Return ")
                || line.contains(" kind=EvalError::InputRequired "))
            && (line.contains(" issuer=Return ") || !line.contains(" generation=0 "))
    }
    let consumed = lines.iter().any(|line| line.starts_with("CLASS_CONSUME "));
    if !consumed {
        return false;
    }
    match group {
        "synchronous" => lines.iter().any(|line| {
            receipt(line)
                && line.contains(" issuer=Return ")
                && line.contains(" kind=EvalError::InputRequired ")
        }),
        "cursor" => {
            lines
                .iter()
                .any(|line| line.starts_with("CLASS_CURSOR ") && line.contains(" accept=true"))
                && lines
                    .iter()
                    .any(|line| receipt(line) && kind_matches(kind_of(line), frame))
        }
        "queued-success" => {
            lines.iter().any(|line| {
                line.starts_with("CLASS_QUEUED_PERMIT ")
                    && line.contains(" paired=true")
                    && !line.contains(" withheld=true")
            }) && lines
                .iter()
                .any(|line| receipt(line) && kind_matches(kind_of(line), frame))
        }
        "typed" | "raw" => lines.iter().any(|line| {
            receipt(line) && !line.contains(" issuer=Return ") && kind_matches(kind_of(line), frame)
        }),
        _ => false,
    }
}

fn install_line<'a>(lines: &'a [String], commit: bool) -> &'a str {
    let marker = format!(" commit={commit} ");
    lines
        .iter()
        .find(|line| line.starts_with("CLASS_INSTALL ") && line.contains(&marker))
        .map(String::as_str)
        .unwrap_or_else(|| panic!("FIXTURE INSTALL_LINE commit={commit}"))
}

fn run_case(row: &'static str, program_id: &str, commit: bool, mode: &str) {
    rum_runtime::cm740_probe::class_lease::clear();
    let programs = guest::programs().unwrap_or_else(|error| {
        panic!("FIXTURE PROGRAMS row={row} mode={mode} err={error:?}");
    });
    let program = programs.get(program_id).unwrap_or_else(|| {
        panic!("FIXTURE MISSING_PROGRAM row={row} id={program_id}");
    });
    let installed =
        rum_runtime::cm740_probe::class_lease::install(row, commit, program.source.as_bytes());
    assert!(installed, "FIXTURE INSTALL_REJECTED row={row} mode={mode}");
    for (half, queued) in [("resumed", false), ("queued", true)] {
        match guest::run(program, queued) {
            Ok(observed) => {
                eprintln!(
                    "FIXTURE HALF row={row} mode={mode} half={half} status=ok pauses={} resumes={} output={}",
                    observed.pauses.len(),
                    observed.native.resumes,
                    output_text(&observed)
                );
            }
            Err(error) => {
                panic!("FIXTURE HALF_FAIL row={row} mode={mode} half={half} err={error:?}")
            }
        }
    }
    let lines = rum_runtime::cm740_probe::class_lease::snapshot_lines();
    for line in &lines {
        eprintln!("{line}");
    }
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let hits = rum_runtime::cm740_probe::class_lease::authorized_hits();
    let consumed = lines.iter().any(|line| line.starts_with("CLASS_CONSUME "));
    if commit {
        let install = install_line(&lines, commit);
        let group = field(install, "group");
        let frame = field(install, "frame");
        assert!(
            contract_hit(group, frame, &refs),
            "FIXTURE POSITIVE_MISS row={row} receipt"
        );
        assert!(consumed, "FIXTURE POSITIVE_MISS row={row} consume");
        eprintln!("FIXTURE VERDICT row={row} mode=positive PASS hits={hits}");
    } else {
        let committed = lines.iter().any(|line| legacy_positive_receipt(line));
        assert_eq!(hits, 0, "FIXTURE WITHHELD_HIT row={row} hits={hits}");
        assert!(!committed, "FIXTURE WITHHELD_RECEIPT row={row}");
        assert!(!consumed, "FIXTURE WITHHELD_CONSUME row={row}");
        eprintln!(
            "FIXTURE IMPORTER row={row} withheld=NoHit clause=records.rs:344-350 before=352-366"
        );
        eprintln!("FIXTURE VERDICT row={row} mode=withheld PASS hits=0");
    }
    rum_runtime::cm740_probe::class_lease::clear();
}

#[test]
fn class_lease_digest_kat() {
    let got = rum_runtime::cm740_probe::class_lease::digest(b"abc");
    assert_eq!(
        got,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    eprintln!("FIXTURE DIGEST_KAT PASS");
}

#[test]
fn class_lease_importer_clause() {
    let text = include_str!("mutations/records.rs");
    let no_hit = text.find("return Err(Rejection::NoHit);").expect("NoHit");
    let survivor = text
        .find("return Err(Rejection::IdentitySurvivor);")
        .expect("survivor");
    assert!(no_hit < survivor);
    eprintln!("FIXTURE IMPORTER_SOURCE NoHit before IdentitySurvivor");
}

#[test]
fn class_lease_contract_predicate_falsifiers() {
    let prepared = "CLASS_RECEIPT row=P0492 commit=true kind=rum_runtime::runtime::eval_core::call::PreparedDispatchFrame generation=2 addr=1 issuer=Typed owner=P0492 occurrence=1 slot=1";
    let prepared_consume = "CLASS_CONSUME row=P0492 addr=1 issuer=Typed kind=rum_runtime::runtime::eval_core::call::PreparedDispatchFrame owner=P0492";
    let wrong = [prepared, prepared_consume];
    assert!(legacy_positive_receipt(prepared));
    assert!(!contract_hit(
        "typed",
        "ProfiledPreparedDispatchFrame",
        &wrong
    ));

    let profiled = "CLASS_RECEIPT row=P0492 commit=true kind=rum_runtime::runtime::eval_core::call::ProfiledPreparedDispatchFrame generation=2 addr=1 issuer=Typed owner=P0492 occurrence=1 slot=1";
    let missing = [profiled];
    assert!(legacy_positive_receipt(profiled));
    assert!(!contract_hit(
        "typed",
        "ProfiledPreparedDispatchFrame",
        &missing
    ));

    let withheld = "CLASS_RECEIPT row=P0492 commit=false kind=rum_runtime::runtime::eval_core::call::ProfiledPreparedDispatchFrame generation=2 addr=1 issuer=Typed owner=P0492 occurrence=1 slot=1";
    assert!(!legacy_positive_receipt(withheld));
    assert!(!contract_hit(
        "typed",
        "ProfiledPreparedDispatchFrame",
        &[withheld]
    ));

    let neighbour_c5 = "CLASS_RECEIPT row=C5.default.profiled.finish commit=true kind=rum_runtime::runtime::definitions::application::down_values::default_fallback::DirectDefaultFallbackFrame<false> generation=2 addr=1 issuer=Typed owner=C5.default.profiled.finish occurrence=1 slot=1";
    let neighbour_c5_consume = "CLASS_CONSUME row=C5.default.profiled.finish addr=1 issuer=Typed kind=rum_runtime::runtime::definitions::application::down_values::default_fallback::DirectDefaultFallbackFrame<false> owner=C5.default.profiled.finish";
    let c5_lines = [neighbour_c5, neighbour_c5_consume];
    assert!(legacy_positive_receipt(neighbour_c5));
    assert!(!contract_hit(
        "synchronous",
        "DirectDefaultFallbackFrame",
        &c5_lines
    ));

    let neighbour_p0289 = "CLASS_RECEIPT row=P0289 commit=true kind=rum_runtime::runtime::definitions::application::rhs::EvaluateDefinitionRhsSuspensionFrame generation=2 addr=1 issuer=Typed owner=P0289 occurrence=1 slot=1";
    let neighbour_p0289_consume = "CLASS_CONSUME row=P0289 addr=1 issuer=Typed kind=rum_runtime::runtime::definitions::application::rhs::EvaluateDefinitionRhsSuspensionFrame owner=P0289";
    let p0289_lines = [neighbour_p0289, neighbour_p0289_consume];
    assert!(legacy_positive_receipt(neighbour_p0289));
    assert!(!contract_hit(
        "synchronous",
        "EvaluateDefinitionRhsSuspensionFrame",
        &p0289_lines
    ));
    let returned = "CLASS_RECEIPT row=P0289 commit=true kind=EvalError::InputRequired generation=0 addr=0 issuer=Return owner=P0289 occurrence=2 slot=2";
    let returned_consume =
        "CLASS_CONSUME row=P0289 addr=0 issuer=Return owner=P0289 occurrence=2 slot=2";
    let with_return = [
        neighbour_p0289,
        neighbour_p0289_consume,
        returned,
        returned_consume,
    ];
    assert!(contract_hit(
        "synchronous",
        "EvaluateDefinitionRhsSuspensionFrame",
        &with_return
    ));

    let vec_kind =
        "rum_stdlib::schema::continuation::SchemaValidationSuspensionFrame<alloc::vec::Vec<usize>>";
    let vec_frame = "SchemaValidationSuspensionFrame<alloc::vec::Vec<usize>>";
    let p2299 = format!(
        "CLASS_RECEIPT row=P2299 commit=true kind={vec_kind} generation=2 addr=1 issuer=Legacy owner=P2299 occurrence=1 slot=1"
    );
    let p2299_consume =
        format!("CLASS_CONSUME row=P2299 addr=1 issuer=LegacyDriver kind={vec_kind} owner=P2299");
    let identity = [p2299.as_str(), p2299_consume.as_str()];
    assert!(contract_hit("raw", vec_frame, &identity));

    let other = "CLASS_RECEIPT row=P2299 commit=true kind=path::SchemaValidationSuspensionFrame<Other> generation=2 addr=1 issuer=Legacy owner=P2299 occurrence=1 slot=1";
    let other_lines = [other, p2299_consume.as_str()];
    assert!(!contract_hit("raw", vec_frame, &other_lines));
    assert!(!contract_hit(
        "raw",
        "SchemaValidationSuspensionFrame",
        &identity
    ));
}

#[test]
fn class_lease_positive_c5_default_profiled_finish() {
    run_case(
        "C5.default.profiled.finish",
        "C12.default-returned.C5.default.profiled.finish.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_c5_default_profiled_finish() {
    run_case(
        "C5.default.profiled.finish",
        "C12.default-returned.C5.default.profiled.finish.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0145() {
    run_case(
        "P0145",
        "C15.sowmatchrecording.P0145.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0145() {
    run_case(
        "P0145",
        "C15.sowmatchrecording.P0145.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0214() {
    run_case(
        "P0214",
        "C15.tracefilteremission.P0214.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0214() {
    run_case(
        "P0214",
        "C15.tracefilteremission.P0214.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0216() {
    run_case(
        "P0216",
        "C15.tracefilteremission.P0216.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0216() {
    run_case(
        "P0216",
        "C15.tracefilteremission.P0216.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0239() {
    run_case(
        "P0239",
        "C26.exact-default-guard-body-evaluation.P0239.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0239() {
    run_case(
        "P0239",
        "C26.exact-default-guard-body-evaluation.P0239.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0245() {
    run_case(
        "P0245",
        "C26.exact-default-guard-body-evaluation.P0245.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0245() {
    run_case(
        "P0245",
        "C26.exact-default-guard-body-evaluation.P0245.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0248() {
    run_case(
        "P0248",
        "C5.default.off.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0248() {
    run_case(
        "P0248",
        "C5.default.off.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0260() {
    run_case(
        "P0260",
        "C18.compiled-matcher-adopted-accounting.P0260.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0260() {
    run_case(
        "P0260",
        "C18.compiled-matcher-adopted-accounting.P0260.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0278() {
    run_case(
        "P0278",
        "C20.rewrite-value-adoption-and-binding-substitution.P0278.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0278() {
    run_case(
        "P0278",
        "C20.rewrite-value-adoption-and-binding-substitution.P0278.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0281() {
    run_case(
        "P0281",
        "C20.rewrite-value-adoption-and-binding-substitution.P0281.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0281() {
    run_case(
        "P0281",
        "C20.rewrite-value-adoption-and-binding-substitution.P0281.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0284() {
    run_case(
        "P0284",
        "C20.rewrite-value-adoption-and-binding-substitution.P0284.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0284() {
    run_case(
        "P0284",
        "C20.rewrite-value-adoption-and-binding-substitution.P0284.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0287() {
    run_case(
        "P0287",
        "CM739.batch4.P0287.45c8e3bb0c79373b4571ae80",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0287() {
    run_case(
        "P0287",
        "CM739.batch4.P0287.45c8e3bb0c79373b4571ae80",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0289() {
    run_case(
        "P0289",
        "C17.returnedtails.P0289.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0289() {
    run_case(
        "P0289",
        "C17.returnedtails.P0289.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0294() {
    run_case(
        "P0294",
        "C17.returnedtails.P0294.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0294() {
    run_case(
        "P0294",
        "C17.returnedtails.P0294.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0304() {
    run_case(
        "P0304",
        "C18.matched-rhs-accounting.P0304.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0304() {
    run_case(
        "P0304",
        "C18.matched-rhs-accounting.P0304.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0305() {
    run_case(
        "P0305",
        "C18.compiled-matcher-adopted-accounting.P0305.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0305() {
    run_case(
        "P0305",
        "C18.compiled-matcher-adopted-accounting.P0305.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0311() {
    run_case(
        "P0311",
        "C18.matched-rhs-accounting.P0311.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0311() {
    run_case(
        "P0311",
        "C18.matched-rhs-accounting.P0311.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0316() {
    run_case(
        "P0316",
        "C22.upvalue-matcher-binding-adoption.P0316.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0316() {
    run_case(
        "P0316",
        "C22.upvalue-matcher-binding-adoption.P0316.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0320() {
    run_case(
        "P0320",
        "C18.matched-rhs-accounting.P0320.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0320() {
    run_case(
        "P0320",
        "C18.matched-rhs-accounting.P0320.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0321() {
    run_case(
        "P0321",
        "C22.upvalue-matcher-binding-adoption.P0321.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0321() {
    run_case(
        "P0321",
        "C22.upvalue-matcher-binding-adoption.P0321.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0338() {
    run_case(
        "P0338",
        "CM739.rule-guard.P0338.0022a9622c4e4f7c5fbb195f",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0338() {
    run_case(
        "P0338",
        "CM739.rule-guard.P0338.0022a9622c4e4f7c5fbb195f",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0339() {
    run_case(
        "P0339",
        "CM739.batch2.P0339.98c7b14db11050b728de7c50",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0339() {
    run_case(
        "P0339",
        "CM739.batch2.P0339.98c7b14db11050b728de7c50",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0348() {
    run_case(
        "P0348",
        "CM739.batch2.P0348.0018420ef962dfe12d4d27ae",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0348() {
    run_case(
        "P0348",
        "CM739.batch2.P0348.0018420ef962dfe12d4d27ae",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0349() {
    run_case(
        "P0349",
        "CM739.batch2.P0349.015b67cd5fd4f11f79ab5da0",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0349() {
    run_case(
        "P0349",
        "CM739.batch2.P0349.015b67cd5fd4f11f79ab5da0",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0350() {
    run_case(
        "P0350",
        "CM739.batch2.P0350.e71990efbc0b1c293e56d787",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0350() {
    run_case(
        "P0350",
        "CM739.batch2.P0350.e71990efbc0b1c293e56d787",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0351() {
    run_case(
        "P0351",
        "CM739.batch2.P0351.473ae691b1275afa0a2a97f9",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0351() {
    run_case(
        "P0351",
        "CM739.batch2.P0351.473ae691b1275afa0a2a97f9",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0369() {
    run_case(
        "P0369",
        "CM739.batch5.P0369.e93d12db89626cf576f6487f",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0369() {
    run_case(
        "P0369",
        "CM739.batch5.P0369.e93d12db89626cf576f6487f",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0372() {
    run_case(
        "P0372",
        "CM739.batch5.P0372.fbce5df70adde5ac108c02f4",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0372() {
    run_case(
        "P0372",
        "CM739.batch5.P0372.fbce5df70adde5ac108c02f4",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0394() {
    run_case(
        "P0394",
        "C17.storetails.P0394.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0394() {
    run_case(
        "P0394",
        "C17.storetails.P0394.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0396() {
    run_case(
        "P0396",
        "C17.storetails.P0396.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0396() {
    run_case(
        "P0396",
        "C17.storetails.P0396.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0403() {
    run_case(
        "P0403",
        "C17.storetails.P0403.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0403() {
    run_case(
        "P0403",
        "C17.storetails.P0403.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0408() {
    run_case(
        "P0408",
        "CM739.batch5.P0408.14b0735a5b7ef2b4e1be673a",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0408() {
    run_case(
        "P0408",
        "CM739.batch5.P0408.14b0735a5b7ef2b4e1be673a",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0416() {
    run_case(
        "P0416",
        "CM739.c4.P0416.014314dcbe2efcea4295",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0416() {
    run_case(
        "P0416",
        "CM739.c4.P0416.014314dcbe2efcea4295",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0435() {
    run_case(
        "P0435",
        "C17.storetails.P0435.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0435() {
    run_case(
        "P0435",
        "C17.storetails.P0435.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0442() {
    run_case(
        "P0442",
        "C17.definitioninner.P0442.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0442() {
    run_case(
        "P0442",
        "C17.definitioninner.P0442.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0450() {
    run_case(
        "P0450",
        "C19.owned-guard-suffix-and-deferred-registration.P0450.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0450() {
    run_case(
        "P0450",
        "C19.owned-guard-suffix-and-deferred-registration.P0450.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0455() {
    run_case(
        "P0455",
        "C19.owned-guard-suffix-and-deferred-registration.P0455.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0455() {
    run_case(
        "P0455",
        "C19.owned-guard-suffix-and-deferred-registration.P0455.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0459() {
    run_case(
        "P0459",
        "C18.readonly-second-target-suffix.P0459.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0459() {
    run_case(
        "P0459",
        "C18.readonly-second-target-suffix.P0459.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0470() {
    run_case(
        "P0470",
        "CM739.batch3-guest.P0470.0480e471013c32bcc593f842",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0470() {
    run_case(
        "P0470",
        "CM739.batch3-guest.P0470.0480e471013c32bcc593f842",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0492() {
    run_case(
        "P0492",
        "CM739.c10.P0492.4757947c5ca715f7a839",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0492() {
    run_case(
        "P0492",
        "CM739.c10.P0492.4757947c5ca715f7a839",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0508() {
    run_case(
        "P0508",
        "C28.between-request-evaluation-retirement.P0508.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0508() {
    run_case(
        "P0508",
        "C28.between-request-evaluation-retirement.P0508.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0527() {
    run_case(
        "P0527",
        "CM739.ownvalue.P0527.13f157dbd1eb3f20c90f3d47",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0527() {
    run_case(
        "P0527",
        "CM739.ownvalue.P0527.13f157dbd1eb3f20c90f3d47",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0544() {
    run_case(
        "P0544",
        "C27.builtin-checked-pause-accounting.P0544.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0544() {
    run_case(
        "P0544",
        "C27.builtin-checked-pause-accounting.P0544.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0558() {
    run_case(
        "P0558",
        "C14.associationstore.P0558.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0558() {
    run_case(
        "P0558",
        "C14.associationstore.P0558.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0765() {
    run_case(
        "P0765",
        "C17.exceptiondispatch.P0765.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0765() {
    run_case(
        "P0765",
        "C17.exceptiondispatch.P0765.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0767() {
    run_case(
        "P0767",
        "C18.exception-rhs-guard-dispatch.P0767.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0767() {
    run_case(
        "P0767",
        "C18.exception-rhs-guard-dispatch.P0767.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0769() {
    run_case(
        "P0769",
        "C17.exceptiondispatch.P0769.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0769() {
    run_case(
        "P0769",
        "C17.exceptiondispatch.P0769.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0773() {
    run_case(
        "P0773",
        "C18.exception-rhs-guard-dispatch.P0773.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0773() {
    run_case(
        "P0773",
        "C18.exception-rhs-guard-dispatch.P0773.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0812() {
    run_case(
        "P0812",
        "C18.returned-catch-and-default-owners.P0812.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0812() {
    run_case(
        "P0812",
        "C18.returned-catch-and-default-owners.P0812.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0813() {
    run_case(
        "P0813",
        "C49.catch-body-result-dispatch.P0813.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0813() {
    run_case(
        "P0813",
        "C49.catch-body-result-dispatch.P0813.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0820() {
    run_case(
        "P0820",
        "CM739.c4.P0820.f5ddb057b8d478f75457",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0820() {
    run_case(
        "P0820",
        "CM739.c4.P0820.f5ddb057b8d478f75457",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0823() {
    run_case(
        "P0823",
        "CM739.batch2.P0823.35fed0d4bdf9b5d75849630b",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0823() {
    run_case(
        "P0823",
        "CM739.batch2.P0823.35fed0d4bdf9b5d75849630b",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0825() {
    run_case(
        "P0825",
        "CM739.c4.P0825.6cdc83724f87564bc285",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0825() {
    run_case(
        "P0825",
        "CM739.c4.P0825.6cdc83724f87564bc285",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0827() {
    run_case(
        "P0827",
        "CM739.batch2.P0827.4064d936037656956c1ca8c5",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0827() {
    run_case(
        "P0827",
        "CM739.batch2.P0827.4064d936037656956c1ca8c5",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p0936() {
    run_case(
        "P0936",
        "CM739.batch3-guest.P0936.5b2cf39c689fe006d9363210",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p0936() {
    run_case(
        "P0936",
        "CM739.batch3-guest.P0936.5b2cf39c689fe006d9363210",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1037() {
    run_case(
        "P1037",
        "C17.repeatedmatch.P1037.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1037() {
    run_case(
        "P1037",
        "C17.repeatedmatch.P1037.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1038() {
    run_case(
        "P1038",
        "C17.repeatedpass.P1038.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1038() {
    run_case(
        "P1038",
        "C17.repeatedpass.P1038.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1070() {
    run_case(
        "P1070",
        "CM739.batch3-guest.P1070.0317dc3fa5585a939157a97f",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1070() {
    run_case(
        "P1070",
        "CM739.batch3-guest.P1070.0317dc3fa5585a939157a97f",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1132() {
    run_case(
        "P1132",
        "C17.optionlist.P1132.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1132() {
    run_case(
        "P1132",
        "C17.optionlist.P1132.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1193() {
    run_case(
        "P1193",
        "C16.namedbinding.P1193.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1193() {
    run_case(
        "P1193",
        "C16.namedbinding.P1193.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1476() {
    run_case(
        "P1476",
        "CM739.c31.P1476.a724c8eeca8611958655a9fb",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1476() {
    run_case(
        "P1476",
        "CM739.c31.P1476.a724c8eeca8611958655a9fb",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1480() {
    run_case(
        "P1480",
        "C18.status-pure-property-value-tail.P1480.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1480() {
    run_case(
        "P1480",
        "C18.status-pure-property-value-tail.P1480.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1568() {
    run_case(
        "P1568",
        "C9.native.P1568.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1568() {
    run_case(
        "P1568",
        "C9.native.P1568.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1577() {
    run_case(
        "P1577",
        "C29.resuming-driver-saved-state-restoration.P1577.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1577() {
    run_case(
        "P1577",
        "C29.resuming-driver-saved-state-restoration.P1577.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1595() {
    run_case(
        "P1595",
        "C14.counttargets.P1595.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1595() {
    run_case(
        "P1595",
        "C14.counttargets.P1595.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1597() {
    run_case(
        "P1597",
        "C14.counttargets.P1597.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1597() {
    run_case(
        "P1597",
        "C14.counttargets.P1597.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1599() {
    run_case(
        "P1599",
        "C14.counttargets.P1599.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1599() {
    run_case(
        "P1599",
        "C14.counttargets.P1599.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1610() {
    run_case(
        "P1610",
        "CM739.batch3-guest.P1610.4d52d4f120eef6d2addd009f",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1610() {
    run_case(
        "P1610",
        "CM739.batch3-guest.P1610.4d52d4f120eef6d2addd009f",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1613() {
    run_case(
        "P1613",
        "CM739.batch3-guest.P1613.0005f4e277bea07fdc70946f",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1613() {
    run_case(
        "P1613",
        "CM739.batch3-guest.P1613.0005f4e277bea07fdc70946f",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1615() {
    run_case(
        "P1615",
        "C15.traversalaccumulation.P1615.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1615() {
    run_case(
        "P1615",
        "C15.traversalaccumulation.P1615.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1621() {
    run_case(
        "P1621",
        "CM739.batch3-guest.P1621.03de4bc49f07fb13ab4c6b77",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1621() {
    run_case(
        "P1621",
        "CM739.batch3-guest.P1621.03de4bc49f07fb13ab4c6b77",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1636() {
    run_case(
        "P1636",
        "CM739.batch3-guest.P1636.0877e3dfae4b84d9745398d1",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1636() {
    run_case(
        "P1636",
        "CM739.batch3-guest.P1636.0877e3dfae4b84d9745398d1",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1637() {
    run_case(
        "P1637",
        "C15.traversalaccumulation.P1637.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1637() {
    run_case(
        "P1637",
        "C15.traversalaccumulation.P1637.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1640() {
    run_case(
        "P1640",
        "C15.traversalaccumulation.P1640.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1640() {
    run_case(
        "P1640",
        "C15.traversalaccumulation.P1640.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1708() {
    run_case(
        "P1708",
        "CM739.batch5.P1708.c17ef4fdcc2e5577f1eee339",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1708() {
    run_case(
        "P1708",
        "CM739.batch5.P1708.c17ef4fdcc2e5577f1eee339",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1814() {
    run_case(
        "P1814",
        "CM739.batch5.P1814.0240acbb8628d357c67e1a03",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1814() {
    run_case(
        "P1814",
        "CM739.batch5.P1814.0240acbb8628d357c67e1a03",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1817() {
    run_case(
        "P1817",
        "CM739.batch5.P1817.070a6949b59741a8d62c81b5",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1817() {
    run_case(
        "P1817",
        "CM739.batch5.P1817.070a6949b59741a8d62c81b5",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p1818() {
    run_case(
        "P1818",
        "C17.storetails.P1818.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p1818() {
    run_case(
        "P1818",
        "C17.storetails.P1818.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2132() {
    run_case(
        "P2132",
        "C16.lazycallback.P2132.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2132() {
    run_case(
        "P2132",
        "C16.lazycallback.P2132.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2133() {
    run_case(
        "P2133",
        "C16.lazycallback.P2133.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2133() {
    run_case(
        "P2133",
        "C16.lazycallback.P2133.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2134() {
    run_case(
        "P2134",
        "C16.lazycallback.P2134.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2134() {
    run_case(
        "P2134",
        "C16.lazycallback.P2134.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2135() {
    run_case(
        "P2135",
        "C16.lazycallback.P2135.Middle.phase_observation.trace",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2135() {
    run_case(
        "P2135",
        "C16.lazycallback.P2135.Middle.phase_observation.trace",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2299() {
    run_case(
        "P2299",
        "C16.schemafinish.P2299.SchemaMode-Conform.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2299() {
    run_case(
        "P2299",
        "C16.schemafinish.P2299.SchemaMode-Conform.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2348() {
    run_case(
        "P2348",
        "C15.stringresultappend.P2348.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2348() {
    run_case(
        "P2348",
        "C15.stringresultappend.P2348.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2354() {
    run_case(
        "P2354",
        "C15.stringresultappend.P2354.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2354() {
    run_case(
        "P2354",
        "C15.stringresultappend.P2354.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2379() {
    run_case(
        "P2379",
        "C15.stringresultappend.P2379.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2379() {
    run_case(
        "P2379",
        "C15.stringresultappend.P2379.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2381() {
    run_case(
        "P2381",
        "C15.linearliteraloutput.P2381.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2381() {
    run_case(
        "P2381",
        "C15.linearliteraloutput.P2381.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2385() {
    run_case(
        "P2385",
        "C15.linearliteraloutput.P2385.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2385() {
    run_case(
        "P2385",
        "C15.linearliteraloutput.P2385.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2387() {
    run_case(
        "P2387",
        "C15.stringresultappend.P2387.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2387() {
    run_case(
        "P2387",
        "C15.stringresultappend.P2387.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2409() {
    run_case(
        "P2409",
        "C15.stringresultappend.P2409.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2409() {
    run_case(
        "P2409",
        "C15.stringresultappend.P2409.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2411() {
    run_case(
        "P2411",
        "C15.linearliteraloutput.P2411.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2411() {
    run_case(
        "P2411",
        "C15.linearliteraloutput.P2411.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2416() {
    run_case(
        "P2416",
        "C15.linearliteraloutput.P2416.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2416() {
    run_case(
        "P2416",
        "C15.linearliteraloutput.P2416.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2418() {
    run_case(
        "P2418",
        "C15.stringresultappend.P2418.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2418() {
    run_case(
        "P2418",
        "C15.stringresultappend.P2418.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2544() {
    run_case(
        "P2544",
        "C16.differencestails.P2544.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2544() {
    run_case(
        "P2544",
        "C16.differencestails.P2544.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2547() {
    run_case(
        "P2547",
        "C16.differencestails.P2547.AlreadyResumed.consumer_routes.route-original",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2547() {
    run_case(
        "P2547",
        "C16.differencestails.P2547.AlreadyResumed.consumer_routes.route-original",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2591() {
    run_case(
        "P2591",
        "CM739.batch5.P2591.13ba6c39ef5087f040968cd9",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2591() {
    run_case(
        "P2591",
        "CM739.batch5.P2591.13ba6c39ef5087f040968cd9",
        false,
        "withheld",
    );
}

#[test]
fn class_lease_positive_p2593() {
    run_case(
        "P2593",
        "CM739.c4.P2593.5a421e33ca0e973cebc2",
        true,
        "positive",
    );
}

#[test]
fn class_lease_withheld_p2593() {
    run_case(
        "P2593",
        "CM739.c4.P2593.5a421e33ca0e973cebc2",
        false,
        "withheld",
    );
}
