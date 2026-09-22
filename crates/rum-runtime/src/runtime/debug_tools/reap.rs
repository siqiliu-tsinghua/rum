use super::reap_state::{
    ReapRenderAccumulator, SuspendedReapBucketRender, SuspendedReapScopeRender,
};
use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
pub(super) struct ReapScopeRenderState {
    pub(super) handler: Option<Expr>,
    pub(super) pattern_list_form: bool,
    pub(super) groups: Vec<ReapBucketGroup>,
}

#[derive(FrameRootTrace)]
pub(super) struct ReapBucketRenderState {
    pub(super) handler: Option<Expr>,
    pub(super) buckets: Vec<ReapBucket>,
}

impl Session {
    pub(in crate::runtime) fn eval_sow(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let (value, tag) = match args.as_ref() {
            [value] => (value.clone(), system_expr("None")),
            [value, tag] => (value.clone(), tag.clone()),
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        let mut matched = Vec::new();
        for scope_index in 0..self.reap_scopes.len() {
            let pattern_count = self.reap_scopes[scope_index].patterns.len();
            for pattern_index in 0..pattern_count {
                let pattern = self.reap_scopes[scope_index].patterns[pattern_index].clone();
                let outcome =
                    self.match_expr_owned(pattern, tag.clone(), SymbolExprMap::default())?;
                if outcome.matched {
                    matched.push((scope_index, pattern_index));
                }
            }
        }

        for (scope_index, pattern_index) in matched {
            let added_tag = record_sown_value(
                &mut self.reap_scopes[scope_index].buckets_by_pattern[pattern_index],
                tag.clone(),
                value.clone(),
            );
            if self.current_memory_limit.is_some() {
                self.charge_reap_occurrence(&value);
                if added_tag {
                    self.charge_reap_occurrence(&tag);
                }
            }
        }

        Ok(value)
    }

    pub(in crate::runtime) fn eval_reap(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Reap", args, 1, 3));
        }

        let (patterns, pattern_list_form) = normalize_reap_patterns(args.get(1));
        let handler = args.get(2).cloned();
        self.push_accounted_reap_scope(ReapScope {
            buckets_by_pattern: vec![ReapBucketGroup::default(); patterns.len()],
            patterns,
            pattern_list_form,
            handler,
        });

        let body_result = self.evaluate(args[0].clone());
        let outcome = crate::resume_or_suspend!(
            self,
            body_result,
            ok(value) => Ok(value),
            suspend => EvalReapSuspensionFrame,
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_reap_after_body(outcome),
        }
    }

    fn resume_reap_after_body(
        &mut self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let scope = self
            .pop_accounted_reap_scope()
            .expect("reap scope should exist while finishing Reap");

        match outcome {
            Ok(value) => {
                let rendered_result = self.render_reap_scope(scope);
                let rendered = crate::resume_or_suspend!(
                    self,
                    rendered_result,
                    ok(value) => Ok(value),
                    suspend => {
                let suspended_value = value.clone();
                ResumeReapAfterBodySuspensionFrame { suspended_value }
                },
                )?;
                Ok(finish_reap_result(value, rendered))
            }
            Err(error) => Err(error),
        }
    }

    fn render_reap_scope(&mut self, scope: ReapScope) -> Result<Expr, EvalError> {
        self.resume_reap_scope_groups_from_index(
            ReapScopeRenderState {
                handler: scope.handler,
                pattern_list_form: scope.pattern_list_form,
                groups: scope.buckets_by_pattern,
            },
            0,
            Vec::new(),
        )
    }

    /// Renders one group per loop iteration instead of recursing per group.
    /// The ordinary path keeps owned vectors. The first suspension freezes
    /// the remaining groups and accumulator into immutable continuation state.
    fn resume_reap_scope_groups_from_index(
        &mut self,
        mut state: ReapScopeRenderState,
        group_index: usize,
        mut rendered_groups: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let mut group_index = group_index;
        loop {
            let Some(group) = state.groups.get_mut(group_index) else {
                return Ok(if state.pattern_list_form {
                    Expr::call(system_expr("List"), rendered_groups)
                } else {
                    rendered_groups
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| Expr::call(system_expr("List"), Vec::new()))
                });
            };

            let bucket_state = ReapBucketRenderState {
                handler: state.handler.clone(),
                buckets: std::mem::take(&mut group.buckets),
            };
            let group_result = self.resume_reap_buckets_from_index(bucket_state, 0, Vec::new());
            match group_result {
                Ok(group) => {
                    rendered_groups.push(group);
                    group_index += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    let suspended =
                        SuspendedReapScopeRender::freeze(state, group_index, rendered_groups);
                    crate::resume_or_suspend!(insert self, ResumeReapScopeGroupsFromIndexSuspensionFrame { suspended });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_reap_scope_group_with_value(
        &mut self,
        mut state: SuspendedReapScopeRender,
        group: Expr,
    ) -> Result<SuspendedReapScopeCompletion, EvalError> {
        state.rendered = state.rendered.appended(group);
        state.group_index += 1;
        self.resume_suspended_reap_scope_groups(state)
    }

    fn resume_suspended_reap_scope_groups(
        &mut self,
        mut state: SuspendedReapScopeRender,
    ) -> Result<SuspendedReapScopeCompletion, EvalError> {
        loop {
            let Some(group) = state.groups.get(state.group_index) else {
                return Ok(SuspendedReapScopeCompletion {
                    rendered: state.rendered,
                    pattern_list_form: state.pattern_list_form,
                });
            };

            let group_result = self.collect_suspended_reap_buckets(SuspendedReapBucketRender {
                handler: state.handler.clone(),
                buckets: group.buckets.clone(),
                bucket_index: 0,
                rendered: ReapRenderAccumulator::freeze(Vec::new()),
            });
            match group_result {
                Ok(rendered) => {
                    state.rendered = state
                        .rendered
                        .appended(Expr::call(system_expr("List"), rendered.into_vec()));
                    state.group_index += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ResumeSuspendedReapScopeGroupsSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    /// Renders one bucket per loop iteration instead of recursing per bucket.
    /// The ordinary path keeps owned vectors. The first suspension freezes
    /// the remaining buckets and accumulator into immutable continuation state.
    fn resume_reap_buckets_from_index(
        &mut self,
        state: ReapBucketRenderState,
        bucket_index: usize,
        mut rendered: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let mut bucket_index = bucket_index;
        loop {
            let Some(bucket) = state.buckets.get(bucket_index) else {
                return Ok(Expr::call(system_expr("List"), rendered));
            };

            let values_expr = Expr::call(system_expr("List"), bucket.values.clone());
            let Some(handler) = &state.handler else {
                rendered.push(values_expr);
                bucket_index += 1;
                continue;
            };

            let applied = Expr::call(handler.clone(), vec![bucket.tag.clone(), values_expr]);
            let group_result = self.evaluate(applied);
            match group_result {
                Ok(group) => {
                    rendered.push(group);
                    bucket_index += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    let suspended =
                        SuspendedReapBucketRender::freeze(state, bucket_index, rendered);
                    crate::resume_or_suspend!(insert self, ResumeReapBucketsFromIndexSuspensionFrame { suspended });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_reap_bucket_with_value(
        &mut self,
        mut state: SuspendedReapBucketRender,
        group: Expr,
    ) -> Result<ReapRenderAccumulator, EvalError> {
        state.rendered = state.rendered.appended(group);
        state.bucket_index += 1;
        self.collect_suspended_reap_buckets(state)
    }

    fn collect_suspended_reap_buckets(
        &mut self,
        mut state: SuspendedReapBucketRender,
    ) -> Result<ReapRenderAccumulator, EvalError> {
        loop {
            let Some(bucket) = state.buckets.get(state.bucket_index) else {
                return Ok(state.rendered);
            };

            let values_expr = Expr::call(system_expr("List"), bucket.values.clone());
            let Some(handler) = &state.handler else {
                state.rendered = state.rendered.appended(values_expr);
                state.bucket_index += 1;
                continue;
            };

            let applied = Expr::call(handler.clone(), vec![bucket.tag.clone(), values_expr]);
            match self.evaluate(applied) {
                Ok(group) => {
                    state.rendered = state.rendered.appended(group);
                    state.bucket_index += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ResumeSuspendedReapBucketsSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }
}

struct SuspendedReapScopeCompletion {
    rendered: ReapRenderAccumulator,
    pattern_list_form: bool,
}

struct FinishReapScopeFinalizer {
    completion: SuspendedReapScopeCompletion,
}

impl crate::runtime::suspension::PureFrameFinalizer for FinishReapScopeFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let mut rendered = self.completion.rendered.into_vec();
        Ok(if self.completion.pattern_list_form {
            Expr::call(system_expr("List"), rendered)
        } else {
            rendered
                .pop()
                .unwrap_or_else(|| Expr::call(system_expr("List"), Vec::new()))
        })
    }
}

struct FinishReapBucketsFinalizer {
    rendered: ReapRenderAccumulator,
}

impl crate::runtime::suspension::PureFrameFinalizer for FinishReapBucketsFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(system_expr("List"), self.rendered.into_vec()))
    }
}

fn normalize_reap_patterns(expr: Option<&Expr>) -> (Vec<Expr>, bool) {
    match expr {
        None => (vec![Expr::call(system_expr("Blank"), vec![])], false),
        Some(Expr::Call { head, args }) if is_system_symbol_name(head, "List") => {
            (args.to_vec(), true)
        }
        Some(pattern) => (vec![pattern.clone()], false),
    }
}

const REAP_BUCKET_INDEX_THRESHOLD: usize = 16;

fn record_sown_value(group: &mut ReapBucketGroup, tag: Expr, value: Expr) -> bool {
    if group.tag_positions.is_none() && group.buckets.len() > REAP_BUCKET_INDEX_THRESHOLD {
        group.tag_positions = Some(reap_tag_positions(&group.buckets));
    }

    if let Some(tag_positions) = &group.tag_positions {
        if let Some(position) = tag_positions.get(&tag).copied() {
            group.buckets[position].values.push(value);
            return false;
        }
    } else if let Some(bucket) = group.buckets.iter_mut().find(|bucket| bucket.tag == tag) {
        bucket.values.push(value);
        return false;
    }

    let position = group.buckets.len();
    if let Some(tag_positions) = &mut group.tag_positions {
        tag_positions.insert(tag.clone(), position);
    }
    group.buckets.push(ReapBucket {
        tag,
        values: vec![value],
    });

    if group.tag_positions.is_none() && group.buckets.len() > REAP_BUCKET_INDEX_THRESHOLD {
        group.tag_positions = Some(reap_tag_positions(&group.buckets));
    }
    true
}

#[inline(never)]
fn reap_tag_positions(buckets: &[ReapBucket]) -> std::collections::HashMap<Expr, usize> {
    buckets
        .iter()
        .enumerate()
        .map(|(position, bucket)| (bucket.tag.clone(), position))
        .collect()
}

fn finish_reap_result(value: Expr, harvested: Expr) -> Expr {
    Expr::call(system_expr("List"), vec![value, harvested])
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<ReapBucketRenderState>();
    assert_traces::<ReapScopeRenderState>();
};

#[derive(rum_derive::FrameRootTrace)]
struct EvalReapSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for EvalReapSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = (|| session.resume_reap_after_body(outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for EvalReapSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.pop_reap_scope();
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeReapAfterBodySuspensionFrame {
    suspended_value: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeReapAfterBodySuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut suspended_value = self.suspended_value.clone();
        let resumed = match outcome {
            Ok(rendered) => (|| Ok(finish_reap_result(suspended_value, rendered)))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeReapScopeGroupsFromIndexSuspensionFrame {
    suspended: crate::runtime::debug_tools::reap_state::SuspendedReapScopeRender,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeReapScopeGroupsFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut suspended = self.suspended.clone();
        let group = match outcome {
            Ok(group) => group,
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_reap_scope_action(
            session.resume_suspended_reap_scope_group_with_value(suspended, group),
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeSuspendedReapScopeGroupsSuspensionFrame {
    state: crate::runtime::debug_tools::reap_state::SuspendedReapScopeRender,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeSuspendedReapScopeGroupsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let group = match outcome {
            Ok(group) => group,
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_reap_scope_action(session.resume_suspended_reap_scope_group_with_value(state, group))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeReapBucketsFromIndexSuspensionFrame {
    suspended: crate::runtime::debug_tools::reap_state::SuspendedReapBucketRender,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeReapBucketsFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut suspended = self.suspended.clone();
        let group = match outcome {
            Ok(group) => group,
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_reap_bucket_action(session.resume_suspended_reap_bucket_with_value(suspended, group))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeSuspendedReapBucketsSuspensionFrame {
    state: crate::runtime::debug_tools::reap_state::SuspendedReapBucketRender,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeSuspendedReapBucketsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let group = match outcome {
            Ok(group) => group,
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_reap_bucket_action(session.resume_suspended_reap_bucket_with_value(state, group))
    }
}

fn finish_reap_scope_action(
    result: Result<SuspendedReapScopeCompletion, EvalError>,
) -> crate::runtime::suspension::FrameAction {
    match result {
        Ok(completion) => {
            crate::runtime::suspension::FrameAction::finalize(FinishReapScopeFinalizer {
                completion,
            })
        }
        Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
    }
}

fn finish_reap_bucket_action(
    result: Result<ReapRenderAccumulator, EvalError>,
) -> crate::runtime::suspension::FrameAction {
    match result {
        Ok(rendered) => {
            crate::runtime::suspension::FrameAction::finalize(FinishReapBucketsFinalizer {
                rendered,
            })
        }
        Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
    }
}
