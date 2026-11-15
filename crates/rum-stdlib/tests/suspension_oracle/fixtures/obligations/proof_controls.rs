//! Current-source closure controls, never guest executions or historical proof.
use crate::oracle::obligations::ledger::proofs::ProofError;
use std::{collections::BTreeMap, path::Path};

pub const CALL: &str = "crates/rum-runtime/src/runtime/eval_core/call.rs";
const PLAN: &str = "crates/rum-runtime/src/runtime/eval_dispatch/mod.rs";
const LHS: &str = "crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs";
const STATE: &str = "crates/rum-runtime/src/runtime/session/session_state.rs";
const TARGET: &str = "resume_suspended_call_with_dispatch_plan";
pub const SKIP: &str = "if stage == CallDispatchStage::PrepareArguments && state.is_prepared() {";
pub type Sources = BTreeMap<String, String>;

pub fn current_sources() -> Sources {
    fn walk(root: &Path, path: &Path, sources: &mut Sources) {
        let mut entries: Vec<_> = std::fs::read_dir(path)
            .unwrap()
            .map(Result::unwrap)
            .collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let p = entry.path();
            if p.is_dir() {
                walk(root, &p, sources);
            } else if p.extension().is_some_and(|s| s == "rs") {
                sources.insert(
                    p.strip_prefix(root).unwrap().to_str().unwrap().to_string(),
                    std::fs::read_to_string(p).unwrap(),
                );
            }
        }
    }
    let root = crate::oracle::coverage::source::workspace();
    let mut sources = Sources::new();
    // The archived closure scans crate sources, including in-source tests.
    // Integration-test controls are outside the production caller domain.
    for entry in std::fs::read_dir(root.join("crates")).unwrap() {
        let path = entry.unwrap().path().join("src");
        if path.is_dir() {
            walk(&root, &path, &mut sources);
        }
    }
    sources
}

fn occurrences(sources: &Sources, name: &str) -> Vec<(String, usize)> {
    let mut result = Vec::new();
    for (path, text) in sources {
        for (offset, _) in text.match_indices(name) {
            let boundary = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric() && c != '_');
            if boundary(text[..offset].chars().next_back())
                && boundary(text[offset + name.len()..].chars().next())
            {
                result.push((
                    path.clone(),
                    text[..offset].bytes().filter(|b| *b == b'\n').count() + 1,
                ));
            }
        }
    }
    result
}

fn span<'a>(text: &'a str, first: &str, last: &str) -> Result<&'a str, ProofError> {
    let a = text.find(first).ok_or(ProofError::Closure)?;
    let b = text[a + first.len()..]
        .find(last)
        .ok_or(ProofError::Closure)?
        + a
        + first.len();
    Ok(&text[a..b])
}

pub fn ordinary_reachable(sources: &Sources) -> bool {
    let c = &sources[CALL];
    let ordinary = span(
        c,
        "    pub(super) fn resume_call_after_head(",
        "\n    pub(in crate::runtime) fn resume_call_with_dispatch_plan(",
    )
    .unwrap();
    ordinary.contains("let state = CallDispatchState::new(args);")
        && ordinary
            .contains("self.resume_call_with_dispatch_plan(evaluated_head, plan, state, 0, None)")
        && sources[STATE].contains("prepared_identity: None,")
}

pub fn validate_closure(sources: &Sources) -> Result<(), ProofError> {
    let expected =
        [(LHS, 230), (CALL, 363), (CALL, 524), (CALL, 544)].map(|(p, l)| (p.to_string(), l));
    if occurrences(sources, TARGET) != expected {
        return Err(ProofError::ClosedCallers);
    }
    let (c, p, l, st) = (
        &sources[CALL],
        &sources[PLAN],
        &sources[LHS],
        &sources[STATE],
    );
    let resumed = span(
        c,
        "    pub(in crate::runtime) fn resume_suspended_call_with_dispatch_plan(",
        "\n    fn install_prepared_call_arguments(",
    )?;
    if !resumed.contains(&format!(
        "{SKIP}\n                stage_index += 1;\n                continue;\n            }}"
    )) || resumed.find("state.is_prepared()")
        >= resumed.find("self.prepare_arguments_for_call_with_unevaluated_restore(")
    {
        return Err(ProofError::PreparedSkip);
    }
    let builder = span(
        p,
        "    pub(super) fn build_call_dispatch_plan(",
        "\n    pub(super) fn ",
    )?;
    let prep = span(
        c,
        "    fn resume_prepared_call_dispatch_frame<",
        "\n    fn resume_suspended_call_dispatch_frame(",
    )?;
    let opt = span(c, "    fn resume_suspended_call_dispatch_frame(", "\n}\n")?;
    let lhs = span(
        l,
        "    pub fn evaluate_prepared_definition_lhs(",
        "\n    #[cold]",
    )?;
    let helpers = span(
        c,
        "fn suspend_prepared_stage_with_boundary(",
        "\n#[cold]\n#[inline(never)]\nfn suspend_call_stage_with_boundary(",
    )?;
    let mut builders = vec![
        (LHS, 553),
        (CALL, 227),
        (PLAN, 42),
        ("crates/rum-runtime/src/runtime/eval_call_prepare.rs", 522),
    ];
    for line in [253, 256, 265, 276] {
        builders.push((
            "crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs",
            line,
        ));
    }
    builders.sort();
    let checks = [
        builder.contains("return CallDispatchPlan::special_form(attributes, builtin_spec);"),
        builder.matches("CallDispatchStage::PrepareArguments").count() == 1,
        builder.split_once("plan.push(").is_some_and(|(_,rest)| rest.starts_with("CallDispatchStage::PrepareArguments)")),
        occurrences(sources, "build_call_dispatch_plan") == builders.into_iter().map(|(p,l)|(p.to_string(),l)).collect::<Vec<_>>(),
        st.contains("let mut plan = Self::new(attributes, builtin_spec);\n        plan.push(CallDispatchStage::SpecialForm);\n        plan"),
        st.contains("stages: [CallDispatchStage::SpecialForm; MAX_CALL_DISPATCH_STAGES],\n            stage_count: 0,"),
        st.contains("stages: self.stages,\n            stage_count: self.stage_count,"),
        [prep,opt].into_iter().all(|s| s.contains("plan,\n            state,\n            stage_index + 1,") || s.contains("plan,\n                state,\n                stage_index + 1,")),
        lhs.contains("CallDispatchState::from_prepared(outer_call.prepared_args.clone()),\n            0,"),
        st.contains("prepared_identity: Some(prepared_args),") && st.contains("fn is_prepared(&self) -> bool {\n        self.prepared_identity.is_some()\n    }"),
        resumed.matches("stage_index +=").count() == 3 && !resumed.contains("stage_index -=") && !resumed.contains("stage_index ="),
        helpers.matches("PreparedDispatchFrame::new(evaluated_head, plan, state, stage_index)").count() == 2,
        helpers.matches("return suspend_profiled_prepared_stage(").count() == 2,
        c.matches("prepared: PreparedDispatchFrame::new(evaluated_head, plan, state, stage_index)").count() == 1,
        occurrences(sources, "resume_prepared_call_dispatch_frame") == [(CALL.to_string(),504),(CALL.to_string(),767),(CALL.to_string(),805)],
        c.matches("session.resume_prepared_call_dispatch_frame::<false>(").count() == 1 && c.matches("session.resume_prepared_call_dispatch_frame::<true>(").count() == 1,
        ordinary_reachable(sources),
    ];
    if checks.into_iter().all(|b| b) {
        Ok(())
    } else {
        Err(ProofError::Closure)
    }
}
