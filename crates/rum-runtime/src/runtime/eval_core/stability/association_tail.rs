use super::*;
use crate::runtime::evaluation_cache::stability::{
    AssociationCallCacheMiss, AssociationCallCacheStart,
};

#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static TAIL_ENTRY_COUNT: Cell<usize> = const { Cell::new(0) };
}

enum AssociationStabilityTask {
    Visit(Expr),
    VisitUncachedCall {
        head: SharedExpr,
        args: SharedArgs,
    },
    FinishCallHead {
        head: SharedExpr,
        args: SharedArgs,
    },
    FinishCall {
        miss: AssociationCallCacheMiss,
        head: SharedExpr,
        args: SharedArgs,
    },
    VisitArgs(ArgsState),
    FinishArg(ArgsState),
    VisitAssociationEntry(AssociationState),
    FinishAssociationEntry(AssociationState),
    FinishRuleLhs {
        args: SharedArgs,
        bloom: StabilityBloom,
    },
    FinishRuleRhs {
        bloom: StabilityBloom,
    },
    FinishRuleDelayedLhs {
        bloom: StabilityBloom,
    },
}

struct ArgsState {
    args: SharedArgs,
    next: usize,
    bloom: StabilityBloom,
}

struct AssociationState {
    args: SharedArgs,
    next: usize,
    bloom: StabilityBloom,
    linear_keys: Vec<Expr>,
    indexed_keys: Option<HashSet<Expr>>,
}

impl Session {
    #[cfg(test)]
    pub(in crate::runtime) fn reset_association_stability_tail_entry_count(&self) {
        TAIL_ENTRY_COUNT.with(|count| count.set(0));
    }

    #[cfg(test)]
    pub(in crate::runtime) fn association_stability_tail_entry_count(&self) -> usize {
        TAIL_ENTRY_COUNT.with(Cell::get)
    }

    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn association_stability_tail_with_bloom(
        &mut self,
        root: Expr,
    ) -> (bool, StabilityBloom) {
        #[cfg(test)]
        TAIL_ENTRY_COUNT.with(|count| count.set(count.get() + 1));
        let mut tasks = vec![AssociationStabilityTask::Visit(root)];
        let mut result = None;

        while let Some(task) = tasks.pop() {
            match task {
                AssociationStabilityTask::Visit(expr) => {
                    debug_assert!(result.is_none());
                    match self.begin_association_call_cache_visit(&expr) {
                        AssociationCallCacheStart::Complete(complete) => {
                            result = Some(complete);
                        }
                        AssociationCallCacheStart::Miss(miss) => {
                            let Expr::Call { head, args } = expr else {
                                unreachable!("only calls can miss the call-stability cache")
                            };
                            tasks.push(AssociationStabilityTask::FinishCall {
                                miss,
                                head: head.clone(),
                                args: args.clone(),
                            });
                            tasks.push(AssociationStabilityTask::VisitUncachedCall { head, args });
                        }
                    }
                }
                AssociationStabilityTask::VisitUncachedCall { head, args } => {
                    debug_assert!(result.is_none());
                    let head_expr = head.as_ref().clone();
                    tasks.push(AssociationStabilityTask::FinishCallHead { head, args });
                    tasks.push(AssociationStabilityTask::Visit(head_expr));
                }
                AssociationStabilityTask::FinishCallHead { head, args } => {
                    let (head_stable, bloom) = result
                        .take()
                        .expect("a call head visit must produce a stability result");
                    if !head_stable {
                        result = Some((false, bloom));
                        continue;
                    }

                    let data_symbol = expr_as_symbol(&head)
                        .filter(|symbol| symbol.context() == SYSTEM_CONTEXT)
                        .map(Symbol::name);
                    match data_symbol {
                        Some("Association") => {
                            tasks.push(AssociationStabilityTask::VisitAssociationEntry(
                                AssociationState {
                                    linear_keys: Vec::with_capacity(
                                        args.len().min(ASSOCIATION_INDEX_THRESHOLD + 1),
                                    ),
                                    indexed_keys: None,
                                    args,
                                    next: 0,
                                    bloom,
                                },
                            ));
                        }
                        Some("Failure") => {
                            let details_would_normalize = match args.as_ref() {
                                [_, details] => failure_details_would_normalize(details),
                                _ => false,
                            };
                            if details_would_normalize {
                                result = Some((false, bloom));
                            } else {
                                tasks.push(AssociationStabilityTask::VisitArgs(ArgsState {
                                    args,
                                    next: 0,
                                    bloom,
                                }));
                            }
                        }
                        Some("Missing" | "Success" | "TerminatedEvaluation") => {
                            tasks.push(AssociationStabilityTask::VisitArgs(ArgsState {
                                args,
                                next: 0,
                                bloom,
                            }));
                        }
                        Some("HoldAllComplete" | "HoldComplete") => {
                            result = Some((true, bloom));
                        }
                        Some("Function" | "Hold" | "HoldAll" | "HoldPattern") => {
                            result = Some((hold_all_literal_args_are_stable(&args), bloom));
                        }
                        Some("Rule") => {
                            if args.len() != 2 {
                                result = Some((false, bloom));
                            } else {
                                tasks.push(AssociationStabilityTask::FinishRuleLhs {
                                    args: args.clone(),
                                    bloom,
                                });
                                tasks.push(AssociationStabilityTask::Visit(args[0].clone()));
                            }
                        }
                        Some("RuleDelayed") => {
                            if args.len() != 2 {
                                result = Some((false, bloom));
                            } else {
                                tasks
                                    .push(AssociationStabilityTask::FinishRuleDelayedLhs { bloom });
                                tasks.push(AssociationStabilityTask::Visit(args[0].clone()));
                            }
                        }
                        _ if self.call_head_has_evaluation_effect(&head, &args) => {
                            result = Some((false, bloom));
                        }
                        _ => {
                            tasks.push(AssociationStabilityTask::VisitArgs(ArgsState {
                                args,
                                next: 0,
                                bloom,
                            }));
                        }
                    }
                }
                AssociationStabilityTask::FinishCall { miss, head, args } => {
                    let uncached = result
                        .take()
                        .expect("an uncached call visit must produce a stability result");
                    result =
                        Some(self.finish_association_call_cache_miss(miss, &head, &args, uncached));
                }
                AssociationStabilityTask::VisitArgs(state) => {
                    debug_assert!(result.is_none());
                    if state.next == state.args.len() {
                        result = Some((true, state.bloom));
                    } else {
                        let arg = state.args[state.next].clone();
                        tasks.push(AssociationStabilityTask::FinishArg(ArgsState {
                            next: state.next + 1,
                            ..state
                        }));
                        tasks.push(AssociationStabilityTask::Visit(arg));
                    }
                }
                AssociationStabilityTask::FinishArg(mut state) => {
                    let (arg_stable, arg_bloom) = result
                        .take()
                        .expect("an argument visit must produce a stability result");
                    state.bloom.union_assign(arg_bloom);
                    if arg_stable {
                        tasks.push(AssociationStabilityTask::VisitArgs(state));
                    } else {
                        result = Some((false, state.bloom));
                    }
                }
                AssociationStabilityTask::VisitAssociationEntry(mut state) => {
                    debug_assert!(result.is_none());
                    if state.next == state.args.len() {
                        result = Some((true, state.bloom));
                        continue;
                    }

                    let arg = state.args[state.next].clone();
                    let Some(entry) = parse_association_entry_form(&arg) else {
                        result = Some((false, state.bloom));
                        continue;
                    };
                    if let Some(keys) = state.indexed_keys.as_mut() {
                        if !keys.insert(entry.key) {
                            result = Some((false, state.bloom));
                            continue;
                        }
                    } else {
                        if state.linear_keys.iter().any(|key| key == &entry.key) {
                            result = Some((false, state.bloom));
                            continue;
                        }
                        state.linear_keys.push(entry.key);
                        if state.linear_keys.len() > ASSOCIATION_INDEX_THRESHOLD {
                            let mut keys = HashSet::with_capacity(state.args.len());
                            keys.extend(state.linear_keys.drain(..));
                            state.indexed_keys = Some(keys);
                        }
                    }

                    state.next += 1;
                    tasks.push(AssociationStabilityTask::FinishAssociationEntry(state));
                    tasks.push(AssociationStabilityTask::Visit(arg));
                }
                AssociationStabilityTask::FinishAssociationEntry(mut state) => {
                    let (arg_stable, arg_bloom) = result
                        .take()
                        .expect("an association entry visit must produce a stability result");
                    state.bloom.union_assign(arg_bloom);
                    if arg_stable {
                        tasks.push(AssociationStabilityTask::VisitAssociationEntry(state));
                    } else {
                        result = Some((false, state.bloom));
                    }
                }
                AssociationStabilityTask::FinishRuleLhs { args, mut bloom } => {
                    let (lhs_stable, lhs_bloom) = result
                        .take()
                        .expect("a Rule lhs visit must produce a stability result");
                    bloom.union_assign(lhs_bloom);
                    if lhs_stable {
                        tasks.push(AssociationStabilityTask::FinishRuleRhs { bloom });
                        tasks.push(AssociationStabilityTask::Visit(args[1].clone()));
                    } else {
                        result = Some((false, bloom));
                    }
                }
                AssociationStabilityTask::FinishRuleRhs { mut bloom } => {
                    let (rhs_stable, rhs_bloom) = result
                        .take()
                        .expect("a Rule rhs visit must produce a stability result");
                    bloom.union_assign(rhs_bloom);
                    result = Some((rhs_stable, bloom));
                }
                AssociationStabilityTask::FinishRuleDelayedLhs { mut bloom } => {
                    let (lhs_stable, lhs_bloom) = result
                        .take()
                        .expect("a RuleDelayed lhs visit must produce a stability result");
                    bloom.union_assign(lhs_bloom);
                    result = Some((lhs_stable, bloom));
                }
            }
        }

        result.expect("the association stability tail must finish its root")
    }
}
