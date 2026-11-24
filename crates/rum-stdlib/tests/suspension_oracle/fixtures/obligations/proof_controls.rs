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

fn ident_boundary(c: Option<char>) -> bool {
    c.is_none_or(|c| !c.is_alphanumeric() && c != '_')
}

/// Brace body of the single function item named `name`. A `_cm740_driven` twin
/// does not match the plain name: the next character fails the identifier boundary.
fn function_body<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("fn {name}");
    let mut from = 0;
    let mut found = Vec::new();
    while let Some(offset) = text[from..].find(&needle) {
        let at = from + offset;
        let after = at + needle.len();
        from = after;
        let before = if at == 0 {
            None
        } else {
            text[..at].chars().next_back()
        };
        if !ident_boundary(before) || !ident_boundary(text[after..].chars().next()) {
            continue;
        }
        if let Some(body) = brace_body(text, after) {
            found.push(body);
        }
    }
    if found.len() == 1 {
        Some(found[0])
    } else {
        None
    }
}

fn brace_body(text: &str, after_name: usize) -> Option<&str> {
    let bytes = text.as_bytes();
    let mut index = after_name;
    let mut paren = 0i32;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => paren += 1,
            b')' => paren -= 1,
            b'{' if paren == 0 => break,
            _ => {}
        }
        index += 1;
    }
    if index >= bytes.len() || bytes[index] != b'{' {
        return None;
    }
    let start = index;
    let mut depth = 0i32;
    while index < bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[start..=index]);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn normalize_ws(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn call_argument_lists(text: &str, callee: &str) -> Vec<Vec<String>> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(offset) = text[from..].find(callee) {
        let at = from + offset;
        let after = at + callee.len();
        from = after;
        let before = if at == 0 {
            None
        } else {
            text[..at].chars().next_back()
        };
        if !ident_boundary(before) || !ident_boundary(text[after..].chars().next()) {
            continue;
        }
        let bytes = text.as_bytes();
        let mut index = after;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if text[index..].starts_with("::") {
            index += 2;
            if index < bytes.len() && bytes[index] == b'<' {
                let mut depth = 0i32;
                while index < bytes.len() {
                    match bytes[index] {
                        b'<' => depth += 1,
                        b'>' => {
                            depth -= 1;
                            if depth == 0 {
                                index += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    index += 1;
                }
            }
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }
        }
        if index >= bytes.len() || bytes[index] != b'(' {
            continue;
        }
        index += 1;
        let mut args = Vec::new();
        let mut current = String::new();
        let mut paren = 1i32;
        let mut bracket = 0i32;
        let mut brace = 0i32;
        let mut angle = 0i32;
        let mut quote = 0u8;
        while index < bytes.len() && paren > 0 {
            let char = bytes[index];
            if quote != 0 {
                current.push(char as char);
                if char == b'\\' {
                    index += 1;
                    if index < bytes.len() {
                        current.push(bytes[index] as char);
                    }
                } else if char == quote {
                    quote = 0;
                }
                index += 1;
                continue;
            }
            if char == b'"' || char == b'\'' {
                quote = char;
                current.push(char as char);
                index += 1;
                continue;
            }
            if char == b'(' {
                paren += 1;
                current.push('(');
            } else if char == b')' {
                paren -= 1;
                if paren == 0 {
                    args.push(normalize_ws(&current));
                    break;
                }
                current.push(')');
            } else if char == b',' && paren == 1 && bracket == 0 && brace == 0 && angle == 0 {
                args.push(normalize_ws(&current));
                current.clear();
            } else {
                match char {
                    b'[' => bracket += 1,
                    b']' => bracket -= 1,
                    b'{' => brace += 1,
                    b'}' => brace -= 1,
                    b'<' => angle += 1,
                    b'>' => angle = angle.saturating_sub(1),
                    _ => {}
                }
                current.push(char as char);
            }
            index += 1;
        }
        found.push(args.into_iter().filter(|part| !part.is_empty()).collect());
    }
    found
}

fn lists_equal(text: &str, callee: &str, expected: &[&str]) -> bool {
    let lists = call_argument_lists(text, callee);
    lists.len() == 1
        && lists[0]
            .iter()
            .map(String::as_str)
            .eq(expected.iter().copied())
}

fn stage_increments_ok(body: &str) -> bool {
    body.matches("stage_index +=").count() == 3
        && !body.contains("stage_index -=")
        && !body.contains("stage_index =")
}

pub fn ordinary_reachable(sources: &Sources) -> bool {
    let Some(ordinary) = function_body(&sources[CALL], "resume_call_after_head") else {
        return false;
    };
    ordinary.contains("let state = CallDispatchState::new(args);")
        && ordinary
            .contains("self.resume_call_with_dispatch_plan(evaluated_head, plan, state, 0, None)")
        && sources[STATE].contains("prepared_identity: None,")
}

pub fn validate_closure(sources: &Sources) -> Result<(), ProofError> {
    // Lines are the stage-C sites, not a fresh occurrences() reading of this file's inputs.
    let expected =
        [(LHS, 268), (CALL, 625), (CALL, 923), (CALL, 989)].map(|(p, l)| (p.to_string(), l));
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
    span(
        c,
        "    fn resume_prepared_call_dispatch_frame<",
        "\n    fn resume_suspended_call_dispatch_frame(",
    )?;
    span(c, "    fn resume_suspended_call_dispatch_frame(", "\n}\n")?;
    span(
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
        (LHS, 918),
        (CALL, 314),
        (CALL, 338),
        (PLAN, 42),
        ("crates/rum-runtime/src/runtime/eval_call_prepare.rs", 697),
        ("crates/rum-runtime/src/runtime/eval_call_prepare.rs", 715),
    ];
    for line in [253, 256, 265, 276] {
        builders.push((
            "crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs",
            line,
        ));
    }
    builders.sort();
    let plain_handoff = [
        "evaluated_head",
        "plan",
        "state",
        "stage_index + 1",
        "restore",
    ];
    let opt_handoff = [
        "evaluated_head",
        "plan",
        "state",
        "stage_index + 1",
        "unevaluated_restore",
    ];
    let lhs_args = [
        "outer_call.head.clone()",
        "outer_call.plan.clone()",
        "CallDispatchState::from_prepared(outer_call.prepared_args.clone())",
        "0",
        "unevaluated_restore",
    ];
    let plain_prepared = function_body(c, "resume_prepared_call_dispatch_frame");
    let opt_frame = function_body(c, "resume_suspended_call_dispatch_frame");
    let lhs_body = function_body(l, "evaluate_prepared_definition_lhs");
    let plain_resumed = function_body(c, "resume_suspended_call_with_dispatch_plan");
    let twin_resumed = function_body(c, "resume_suspended_call_with_dispatch_plan_cm740_driven");
    let checks = [
        builder.contains("return CallDispatchPlan::special_form(attributes, builtin_spec);"),
        builder.matches("CallDispatchStage::PrepareArguments").count() == 1,
        builder.split_once("plan.push(").is_some_and(|(_, rest)| rest.starts_with("CallDispatchStage::PrepareArguments)")),
        occurrences(sources, "build_call_dispatch_plan") == builders.into_iter().map(|(p, l)| (p.to_string(), l)).collect::<Vec<_>>(),
        st.contains("let mut plan = Self::new(attributes, builtin_spec);\n        plan.push(CallDispatchStage::SpecialForm);\n        plan"),
        st.contains("stages: [CallDispatchStage::SpecialForm; MAX_CALL_DISPATCH_STAGES],\n            stage_count: 0,"),
        st.contains("stages: self.stages,\n            stage_count: self.stage_count,"),
        plain_prepared.is_some_and(|body| lists_equal(body, TARGET, &plain_handoff))
            && opt_frame.is_some_and(|body| lists_equal(body, TARGET, &opt_handoff)),
        lhs_body.is_some_and(|body| lists_equal(body, TARGET, &lhs_args)),
        st.contains("prepared_identity: Some(prepared_args),") && st.contains("fn is_prepared(&self) -> bool {\n        self.prepared_identity.is_some()\n    }"),
        plain_resumed.is_some_and(stage_increments_ok) && twin_resumed.is_some_and(stage_increments_ok),
        helpers.matches("PreparedDispatchFrame::new(evaluated_head, plan, state, stage_index)").count() == 2,
        helpers.matches("return suspend_profiled_prepared_stage(").count() == 2,
        c.matches("prepared: PreparedDispatchFrame::new(evaluated_head, plan, state, stage_index)").count() == 1,
        occurrences(sources, "resume_prepared_call_dispatch_frame") == [(CALL.to_string(), 903), (CALL.to_string(), 1214), (CALL.to_string(), 1256)],
        c.matches("session.resume_prepared_call_dispatch_frame::<false>(").count() == 1 && c.matches("session.resume_prepared_call_dispatch_frame::<true>(").count() == 1,
        ordinary_reachable(sources),
    ];
    if checks.into_iter().all(|b| b) {
        Ok(())
    } else {
        Err(ProofError::Closure)
    }
}
