mod resume_frames;
use resume_frames::*;

use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::collections::HashMap;

use super::*;

const INDEXED_PATH_COMPATIBILITY_THRESHOLD: usize = 32;

enum MapThreadDispatch {
    Threaded(Expr),
    NotApplicable,
    Mismatch,
}

#[derive(FrameRootTrace)]
struct MapAtEvalState {
    operator: Expr,
    paths: Vec<Vec<isize>>,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedMapAtEvalState {
    operator: Expr,
    paths: FrozenFunctionalSequence<Vec<isize>>,
}

impl SuspendedMapAtEvalState {
    fn freeze(state: MapAtEvalState) -> Self {
        Self {
            operator: state.operator,
            paths: FrozenFunctionalSequence::freeze(state.paths),
        }
    }
}

#[derive(Clone, FrameRootTrace)]
struct SubsetMapEvalState {
    expr: Expr,
    paths: Vec<Vec<isize>>,
    fallback: Expr,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedSubsetMapEvalState {
    expr: Expr,
    paths: FrozenFunctionalSequence<Vec<isize>>,
    fallback: Expr,
}

impl SuspendedSubsetMapEvalState {
    fn freeze(state: SubsetMapEvalState) -> Self {
        Self {
            expr: state.expr,
            paths: FrozenFunctionalSequence::freeze(state.paths),
            fallback: state.fallback,
        }
    }
}

struct MissingExactPath {
    path: Vec<isize>,
    part: isize,
    target: Expr,
}

enum SubsetMapPathFailure {
    Duplicate(Vec<isize>),
    Overlap(Vec<isize>, Vec<isize>),
    Missing(MissingExactPath),
}

#[derive(Default)]
struct PathPrefixNode {
    children: HashMap<isize, usize>,
    terminal_min_index: Option<usize>,
    subtree_min_index: Option<usize>,
}

struct PathPrefixIndex {
    nodes: Vec<PathPrefixNode>,
}

impl PathPrefixIndex {
    fn new() -> Self {
        Self {
            nodes: vec![PathPrefixNode::default()],
        }
    }

    fn earliest_conflict(&self, path: &[isize]) -> Option<usize> {
        let mut node_index = 0;
        let mut earliest = None;

        for component in path {
            earliest = earlier_index(earliest, self.nodes[node_index].terminal_min_index);
            let Some(child_index) = self.nodes[node_index].children.get(component).copied() else {
                return earliest;
            };
            node_index = child_index;
        }

        let node = &self.nodes[node_index];
        earliest = earlier_index(earliest, node.terminal_min_index);
        earlier_index(earliest, node.subtree_min_index)
    }

    fn insert(&mut self, path: &[isize], index: usize) {
        let mut node_index = 0;
        if self.nodes[node_index].subtree_min_index.is_none() {
            self.nodes[node_index].subtree_min_index = Some(index);
        }

        for component in path {
            let child_index = if let Some(child_index) =
                self.nodes[node_index].children.get(component).copied()
            {
                child_index
            } else {
                let child_index = self.nodes.len();
                self.nodes.push(PathPrefixNode::default());
                self.nodes[node_index]
                    .children
                    .insert(*component, child_index);
                child_index
            };
            node_index = child_index;
            if self.nodes[node_index].subtree_min_index.is_none() {
                self.nodes[node_index].subtree_min_index = Some(index);
            }
        }

        if self.nodes[node_index].terminal_min_index.is_none() {
            self.nodes[node_index].terminal_min_index = Some(index);
        }
    }
}

fn earlier_index(lhs: Option<usize>, rhs: Option<usize>) -> Option<usize> {
    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => Some(lhs.min(rhs)),
        (Some(index), None) | (None, Some(index)) => Some(index),
        (None, None) => None,
    }
}

pub(crate) fn eval_subset_map(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [operator, positions] => Ok(unary_operator_form_with_tail(
            "SubsetMap",
            operator.clone(),
            positions.clone(),
        )),
        [operator, expr, positions] => {
            let Some(paths) = subset_map_paths_from_expr(expr, positions) else {
                return Ok(session.invalid_call_with_arg_form(
                    "SubsetMap",
                    args,
                    "expected an exact part specification or list of exact part specifications as the third argument",
                ));
            };

            let explicit_paths = subset_map_explicit_integer_paths_from_expr(positions);
            if let Some(failure) = subset_map_path_failure(expr, &paths, explicit_paths.as_deref())
            {
                session.push_message("SubsetMap", "path", subset_map_path_message(&failure));
                return Ok(expr.clone());
            }

            apply_subset_map(session, head, operator, expr, &paths, args)
        }
        _ => Ok(session.invalid_call_with_arg_range("SubsetMap", args, 2, 3)),
    }
}

pub(crate) fn eval_map_at(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [operator, position_spec] => Ok(unary_operator_form_with_tail(
            "MapAt",
            operator.clone(),
            position_spec.clone(),
        )),
        [operator, expr, position_spec] => {
            let Some(paths) = exact_position_paths_from_expr(expr, position_spec) else {
                return Ok(session.invalid_call_with_arg_form(
                    "MapAt",
                    args,
                    "expected an exact part specification or list of exact part specifications as the third argument",
                ));
            };

            if let Some(explicit_paths) = map_at_explicit_integer_paths_from_expr(position_spec) {
                if let Some(missing) = first_missing_exact_path(expr, &explicit_paths) {
                    push_missing_part_message(session, "MapAt", "partw", &missing);
                    return Ok(expr.clone());
                }
            }

            apply_map_at_paths(session, operator, expr.clone(), paths)
        }
        _ => Ok(session.invalid_call_with_arg_range("MapAt", args, 2, 3)),
    }
}

pub(crate) fn eval_map_thread(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [operator] => Ok(unary_operator_form("MapThread", operator)),
        [operator, expr] => finish_map_thread(
            session,
            head,
            args,
            operator,
            expr,
            Expr::integer(BigInt::from(1_u8)),
            1,
        ),
        [operator, expr, level_expr] => {
            let Some(level) = expr_as_non_negative_usize(level_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "MapThread",
                    args,
                    "expected a positive integer level as the third argument",
                ));
            };

            if level == 0 {
                return Ok(session.invalid_call_with_arg_form(
                    "MapThread",
                    args,
                    "expected a positive integer level as the third argument",
                ));
            }

            finish_map_thread(
                session,
                head,
                args,
                operator,
                expr,
                level_expr.clone(),
                level,
            )
        }
        _ => Ok(session.invalid_call_with_arg_range("MapThread", args, 1, 3)),
    }
}

fn finish_map_thread(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    operator: &Expr,
    expr: &Expr,
    level_expr: Expr,
    level: usize,
) -> Result<Expr, EvalError> {
    let Some(inputs) = call_items_with_head(expr, &system_expr("List")) else {
        return Ok(session.invalid_call_with_arg_form(
            "MapThread",
            args,
            "expected a list of expressions as the second argument",
        ));
    };

    match map_thread_inputs(session, operator, inputs, level)? {
        MapThreadDispatch::Threaded(threaded) => Ok(threaded),
        MapThreadDispatch::NotApplicable | MapThreadDispatch::Mismatch => Ok(if level == 1 {
            Expr::call(head.clone(), vec![operator.clone(), expr.clone()])
        } else {
            Expr::call(
                head.clone(),
                vec![operator.clone(), expr.clone(), level_expr],
            )
        }),
    }
}

fn apply_map_at_paths(
    session: &mut Session,
    operator: &Expr,
    expr: Expr,
    paths: Vec<Vec<isize>>,
) -> Result<Expr, EvalError> {
    resume_map_at_from_index(
        session,
        MapAtEvalState {
            operator: operator.clone(),
            paths,
        },
        0,
        expr,
    )
}

fn resume_map_at_from_index(
    session: &mut Session,
    state: MapAtEvalState,
    mut index: usize,
    mut current: Expr,
) -> Result<Expr, EvalError> {
    loop {
        let Some(path) = state.paths.get(index) else {
            return Ok(current);
        };

        let Some(part) = exact_expr_at_path(&current, path) else {
            index += 1;
            continue;
        };

        let mapped = match session.evaluate(Expr::call(state.operator.clone(), vec![part])) {
            Ok(mapped) => mapped,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedMapAtEvalState::freeze(state);
                ResumeMapAtFromIndexSuspensionFrame { state, index, current }
                },
                );
            }
            Err(error) => return Err(error),
        };

        current = replace_exact_path(current, path, mapped);
        index += 1;
    }
}

fn resume_suspended_map_at_from_index(
    session: &mut Session,
    state: SuspendedMapAtEvalState,
    mut index: usize,
    mut current: Expr,
) -> Result<Expr, EvalError> {
    loop {
        let Some(path) = state.paths.get(index) else {
            return Ok(current);
        };

        let Some(part) = exact_expr_at_path(&current, path) else {
            index += 1;
            continue;
        };

        let mapped = match session.evaluate(Expr::call(state.operator.clone(), vec![part])) {
            Ok(mapped) => mapped,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedMapAtFromIndexSuspensionFrame { state, index, current },
                );
            }
            Err(error) => return Err(error),
        };

        current = replace_exact_path(current, path, mapped);
        index += 1;
    }
}

fn resume_suspended_map_at_with_value(
    session: &mut Session,
    state: SuspendedMapAtEvalState,
    index: usize,
    current: Expr,
    mapped: Expr,
) -> Result<Expr, EvalError> {
    let Some(path) = state.paths.get(index) else {
        return Ok(current);
    };

    let current = replace_exact_path(current, path, mapped);
    resume_suspended_map_at_from_index(session, state, index + 1, current)
}

fn finish_subset_map_from_mapped(
    session: &mut Session,
    state: SubsetMapEvalState,
    mapped: Expr,
) -> Result<Expr, EvalError> {
    let Expr::Call {
        args: replacements, ..
    } = mapped
    else {
        return Ok(state.fallback);
    };

    if replacements.len() != state.paths.len() {
        return Ok(state.fallback);
    }

    let mut current = state.expr;
    for (path, replacement) in state.paths.iter().zip(replacements.into_iter()) {
        current = replace_exact_path(current, path, replacement);
    }

    let evaluated_result = session.evaluate(current);
    rum_runtime::resume_or_suspend!(
        session,
        evaluated_result,
        ok(value) => Ok(value),
        suspend => FinishSubsetMapFromMappedSuspensionFrame,
    )
}

fn finish_suspended_subset_map_from_mapped(
    session: &mut Session,
    state: SuspendedSubsetMapEvalState,
    mapped: Expr,
) -> Result<Expr, EvalError> {
    let Expr::Call {
        args: replacements, ..
    } = mapped
    else {
        return Ok(state.fallback);
    };

    if replacements.len() != state.paths.len() {
        return Ok(state.fallback);
    }

    let mut current = state.expr;
    for (path, replacement) in state.paths.as_slice().iter().zip(replacements.iter()) {
        current = replace_exact_path(current, path, replacement.clone());
    }

    let evaluated_result = session.evaluate(current);
    rum_runtime::resume_or_suspend!(
        session,
        evaluated_result,
        ok(value) => Ok(value),
        suspend => FinishSuspendedSubsetMapFromMappedSuspensionFrame,
    )
}

fn map_thread_inputs(
    session: &mut Session,
    operator: &Expr,
    inputs: &[Expr],
    level: usize,
) -> Result<MapThreadDispatch, EvalError> {
    if level == 1 {
        return Ok(
            match session.thread_call_args(
                operator,
                inputs,
                &system_expr("List"),
                &system_expr("List"),
                &ThreadArgSpec::All,
            )? {
                ThreadingDispatch::Threaded(expr) => MapThreadDispatch::Threaded(expr),
                ThreadingDispatch::NotApplicable => MapThreadDispatch::NotApplicable,
                ThreadingDispatch::Mismatch => MapThreadDispatch::Mismatch,
            },
        );
    }

    let Some(first) = inputs.first() else {
        return Ok(MapThreadDispatch::Threaded(Expr::call(
            system_expr("List"),
            Vec::new(),
        )));
    };

    let Expr::Call {
        head: common_head,
        args: first_args,
        ..
    } = first
    else {
        return Ok(MapThreadDispatch::NotApplicable);
    };

    if is_system_symbol_name(common_head, "Association") {
        return Ok(MapThreadDispatch::NotApplicable);
    }

    let mut all_args = vec![first_args.clone()];
    let mut lengths = vec![first_args.len()];

    for input in inputs.iter().skip(1) {
        let Expr::Call { head, args, .. } = input else {
            return Ok(MapThreadDispatch::NotApplicable);
        };

        if head.as_ref() != common_head.as_ref() {
            return Ok(MapThreadDispatch::NotApplicable);
        }

        lengths.push(args.len());
        if args.len() != first_args.len() {
            session.emit_thread_length_message(&system_expr("MapThread"), common_head, &lengths);
            return Ok(MapThreadDispatch::Mismatch);
        }

        all_args.push(args.clone());
    }

    let mut mapped = Vec::with_capacity(first_args.len());
    for index in 0..first_args.len() {
        let child_inputs = all_args
            .iter()
            .map(|args| args[index].clone())
            .collect::<Vec<_>>();

        match map_thread_inputs(session, operator, &child_inputs, level - 1)? {
            MapThreadDispatch::Threaded(expr) => mapped.push(expr),
            MapThreadDispatch::NotApplicable => return Ok(MapThreadDispatch::NotApplicable),
            MapThreadDispatch::Mismatch => return Ok(MapThreadDispatch::Mismatch),
        }
    }

    Ok(MapThreadDispatch::Threaded(Expr::call(
        common_head.as_ref().clone(),
        mapped,
    )))
}

fn subset_map_path_failure(
    expr: &Expr,
    paths: &[Vec<isize>],
    explicit_paths: Option<&[Vec<isize>]>,
) -> Option<SubsetMapPathFailure> {
    if let Some(failure) = subset_map_path_compatibility_failure(paths) {
        return Some(failure);
    }

    if let Some(explicit_paths) = explicit_paths {
        if let Some(failure) = subset_map_path_compatibility_failure(explicit_paths) {
            return Some(failure);
        }

        if let Some(missing) = first_missing_exact_path(expr, explicit_paths) {
            return Some(SubsetMapPathFailure::Missing(missing));
        }
    }

    None
}

fn subset_map_path_compatibility_failure(paths: &[Vec<isize>]) -> Option<SubsetMapPathFailure> {
    if paths.len() > INDEXED_PATH_COMPATIBILITY_THRESHOLD {
        return indexed_subset_map_path_compatibility_failure(paths);
    }

    for (index, path) in paths.iter().enumerate() {
        for other in paths.iter().skip(index + 1) {
            if path == other {
                return Some(SubsetMapPathFailure::Duplicate(path.clone()));
            }

            if path.starts_with(other.as_slice()) || other.starts_with(path.as_slice()) {
                return Some(SubsetMapPathFailure::Overlap(path.clone(), other.clone()));
            }
        }
    }

    None
}

fn indexed_subset_map_path_compatibility_failure(
    paths: &[Vec<isize>],
) -> Option<SubsetMapPathFailure> {
    let mut prefix_index = PathPrefixIndex::new();
    let mut first_conflict = None;

    for (right_index, path) in paths.iter().enumerate() {
        if let Some(left_index) = prefix_index.earliest_conflict(path) {
            let conflict = (left_index, right_index);
            if first_conflict.is_none_or(|first| conflict < first) {
                first_conflict = Some(conflict);
            }
        }
        prefix_index.insert(path, right_index);
    }

    let (left_index, right_index) = first_conflict?;
    let left = &paths[left_index];
    let right = &paths[right_index];
    if left == right {
        Some(SubsetMapPathFailure::Duplicate(left.clone()))
    } else {
        Some(SubsetMapPathFailure::Overlap(left.clone(), right.clone()))
    }
}

fn subset_map_path_message(failure: &SubsetMapPathFailure) -> String {
    match failure {
        SubsetMapPathFailure::Duplicate(path) => {
            format!(
                "Duplicate subset path {} is not allowed.",
                position_expr_from_path(path)
            )
        }
        SubsetMapPathFailure::Overlap(left, right) => {
            format!(
                "Overlapping subset paths {} and {} are not allowed.",
                position_expr_from_path(left),
                position_expr_from_path(right)
            )
        }
        SubsetMapPathFailure::Missing(missing) => {
            format!(
                "Path {} cannot be used because part {} of {} does not exist.",
                position_expr_from_path(&missing.path),
                missing.part,
                missing.target
            )
        }
    }
}

fn subset_map_paths_from_expr(expr: &Expr, spec_expr: &Expr) -> Option<Vec<Vec<isize>>> {
    let Some(items) = call_items_with_head(spec_expr, &system_expr("List")) else {
        return exact_position_paths_from_expr(expr, spec_expr);
    };

    if items
        .iter()
        .all(|item| !matches!(item, Expr::Call { head, .. } if is_system_symbol_name(head, "List")))
    {
        let mut paths = Vec::new();
        for item in items {
            paths.extend(exact_position_paths_from_expr(expr, item)?);
        }
        Some(paths)
    } else {
        exact_position_paths_from_expr(expr, spec_expr)
    }
}

fn apply_subset_map(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    expr: &Expr,
    paths: &[Vec<isize>],
    raw_args: &[Expr],
) -> Result<Expr, EvalError> {
    let fallback = Expr::call(head.clone(), raw_args.to_vec());
    let Some(parts) = paths
        .iter()
        .map(|path| exact_expr_at_path(expr, path))
        .collect::<Option<Vec<_>>>()
    else {
        return Ok(fallback);
    };

    let state = SubsetMapEvalState {
        expr: expr.clone(),
        paths: paths.to_vec(),
        fallback,
    };
    let mapped_result = session.evaluate(Expr::call(
        operator.clone(),
        vec![Expr::call(system_expr("List"), parts)],
    ));
    match mapped_result {
        Ok(mapped) => finish_subset_map_from_mapped(session, state, mapped),
        Err(EvalError::InputRequired { request }) => {
            rum_runtime::resume_or_suspend!(
                session,
                Err(EvalError::InputRequired { request }),
                ok(value) => Ok(value),
                suspend => {
            let suspended_state = SuspendedSubsetMapEvalState::freeze(state);
            ApplySubsetMapSuspensionFrame { suspended_state }
            },
            )
        }
        Err(error) => Err(error),
    }
}

fn map_at_explicit_integer_paths_from_expr(spec_expr: &Expr) -> Option<Vec<Vec<isize>>> {
    if let Some(index) = expr_as_signed_isize(spec_expr) {
        return Some(vec![vec![index]]);
    }

    if let Some(path) = exact_integer_path_from_expr(spec_expr) {
        return Some(vec![path]);
    }

    let Expr::Call { head, args, .. } = spec_expr else {
        return None;
    };
    if !is_system_symbol_name(head, "List") || !args.iter().all(is_plain_list_call) {
        return None;
    }

    args.iter().map(exact_integer_path_from_expr).collect()
}

fn subset_map_explicit_integer_paths_from_expr(spec_expr: &Expr) -> Option<Vec<Vec<isize>>> {
    if let Some(index) = expr_as_signed_isize(spec_expr) {
        return Some(vec![vec![index]]);
    }

    let Expr::Call { head, args, .. } = spec_expr else {
        return None;
    };
    if !is_system_symbol_name(head, "List") {
        return None;
    }

    if args.iter().all(|arg| !is_plain_list_call(arg)) {
        return args
            .iter()
            .map(|arg| expr_as_signed_isize(arg).map(|index| vec![index]))
            .collect();
    }

    args.iter().map(exact_integer_path_from_expr).collect()
}

fn exact_integer_path_from_expr(expr: &Expr) -> Option<Vec<isize>> {
    if let Some(index) = expr_as_signed_isize(expr) {
        return Some(vec![index]);
    }

    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter().map(expr_as_signed_isize).collect()
}

fn is_plain_list_call(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, .. } if is_system_symbol_name(head, "List")
    )
}

fn first_missing_exact_path(expr: &Expr, paths: &[Vec<isize>]) -> Option<MissingExactPath> {
    for path in paths {
        let mut target = expr.clone();
        for part in path {
            let Some(next) = exact_expr_at_path(&target, &[*part]) else {
                return Some(MissingExactPath {
                    path: path.clone(),
                    part: *part,
                    target,
                });
            };
            target = next;
        }
    }

    None
}

fn push_missing_part_message(
    session: &mut Session,
    symbol: &str,
    tag: &str,
    missing: &MissingExactPath,
) {
    session.push_message(
        symbol,
        tag,
        format!(
            "Part {} of {} does not exist.",
            missing.part, missing.target
        ),
    );
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<MapAtEvalState>();
    assert_traces::<SubsetMapEvalState>();
    assert_traces::<SuspendedMapAtEvalState>();
    assert_traces::<SuspendedSubsetMapEvalState>();
};
