use super::*;

const RECURSIVE_REPLACE_PLAN_DEPTH: usize = 16;

enum ReplacePlanWork {
    Visit {
        expr: Expr,
        plan: ReplacePlanNode,
    },
    FinishAtom,
    FinishCallHead {
        args: Vec<Expr>,
        plan: ReplacePlanNode,
    },
    FinishCallArg {
        head: Expr,
        args: Vec<Expr>,
        plan: ReplacePlanNode,
        index: usize,
    },
}

impl Session {
    pub(super) fn apply_replace_plan_owned(
        &mut self,
        expr: Expr,
        plan: ReplacePlanNode,
    ) -> Result<Expr, EvalError> {
        self.apply_replace_plan_owned_recursive_bounded(expr, plan, RECURSIVE_REPLACE_PLAN_DEPTH)
    }

    fn apply_replace_plan_owned_recursive_bounded(
        &mut self,
        mut expr: Expr,
        mut plan: ReplacePlanNode,
        remaining_plan_depth: usize,
    ) -> Result<Expr, EvalError> {
        if remaining_plan_depth == 0 {
            return self.apply_replace_plan_owned_iterative(expr, plan);
        }

        if let Some(action) = plan.action.take() {
            expr = match action {
                ReplacePlanAction::Immediate(rhs) => rhs,
            };
        }

        if plan.children.is_empty() {
            return Ok(expr);
        }

        let child_plan_depth = remaining_plan_depth - 1;
        match expr {
            Expr::Atom(_) => {
                if let Some(head_plan) = plan.children.remove(&0) {
                    let replaced_head = self.apply_replace_plan_owned_recursive_bounded(
                        structural_head_expr(&expr),
                        head_plan,
                        child_plan_depth,
                    )?;
                    Ok(Expr::call(replaced_head, Vec::new()))
                } else {
                    Ok(expr)
                }
            }
            Expr::Call { head, args } => {
                let mut replaced_head = head.as_ref().clone();
                if let Some(head_plan) = plan.children.remove(&0) {
                    replaced_head = self.apply_replace_plan_owned_recursive_bounded(
                        replaced_head,
                        head_plan,
                        child_plan_depth,
                    )?;
                }

                let mut args = Expr::unwrap_shared_args(args);
                for (index, arg) in args.iter_mut().enumerate() {
                    if let Some(child_plan) = plan.children.remove(&(index as isize + 1)) {
                        *arg = self.apply_replace_plan_owned_recursive_bounded(
                            arg.clone(),
                            child_plan,
                            child_plan_depth,
                        )?;
                    }
                }

                Ok(Expr::call_with_shared_args(
                    replaced_head,
                    SharedArgs::new(args),
                ))
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn apply_replace_plan_owned_iterative(
        &mut self,
        expr: Expr,
        plan: ReplacePlanNode,
    ) -> Result<Expr, EvalError> {
        let mut work = vec![ReplacePlanWork::Visit { expr, plan }];
        let mut completed = None;

        while let Some(task) = work.pop() {
            match task {
                ReplacePlanWork::Visit { mut expr, mut plan } => {
                    debug_assert!(completed.is_none());
                    if let Some(action) = plan.action.take() {
                        expr = match action {
                            ReplacePlanAction::Immediate(rhs) => rhs,
                        };
                    }

                    if plan.children.is_empty() {
                        completed = Some(expr);
                        continue;
                    }

                    match expr {
                        Expr::Atom(_) => {
                            if let Some(head_plan) = plan.children.remove(&0) {
                                let head = structural_head_expr(&expr);
                                work.push(ReplacePlanWork::FinishAtom);
                                work.push(ReplacePlanWork::Visit {
                                    expr: head,
                                    plan: head_plan,
                                });
                            } else {
                                completed = Some(expr);
                            }
                        }
                        Expr::Call { head, args } => {
                            let head = head.as_ref().clone();
                            let args = Expr::unwrap_shared_args(args);
                            if let Some(head_plan) = plan.children.remove(&0) {
                                work.push(ReplacePlanWork::FinishCallHead { args, plan });
                                work.push(ReplacePlanWork::Visit {
                                    expr: head,
                                    plan: head_plan,
                                });
                            } else {
                                schedule_replace_plan_call_args(
                                    &mut work,
                                    &mut completed,
                                    head,
                                    args,
                                    plan,
                                    0,
                                );
                            }
                        }
                    }
                }
                ReplacePlanWork::FinishAtom => {
                    let head = completed
                        .take()
                        .expect("ReplacePart atom plan completes its head");
                    completed = Some(Expr::call(head, Vec::new()));
                }
                ReplacePlanWork::FinishCallHead { args, plan } => {
                    let head = completed
                        .take()
                        .expect("ReplacePart call plan completes its head");
                    schedule_replace_plan_call_args(&mut work, &mut completed, head, args, plan, 0);
                }
                ReplacePlanWork::FinishCallArg {
                    head,
                    mut args,
                    plan,
                    index,
                } => {
                    args[index] = completed
                        .take()
                        .expect("ReplacePart call plan completes its argument");
                    schedule_replace_plan_call_args(
                        &mut work,
                        &mut completed,
                        head,
                        args,
                        plan,
                        index + 1,
                    );
                }
            }
        }

        Ok(completed.expect("ReplacePart iterative plan completes its root"))
    }
}

fn schedule_replace_plan_call_args(
    work: &mut Vec<ReplacePlanWork>,
    completed: &mut Option<Expr>,
    head: Expr,
    args: Vec<Expr>,
    mut plan: ReplacePlanNode,
    mut index: usize,
) {
    while index < args.len() {
        if let Some(child_plan) = plan.children.remove(&(index as isize + 1)) {
            let child = args[index].clone();
            work.push(ReplacePlanWork::FinishCallArg {
                head,
                args,
                plan,
                index,
            });
            work.push(ReplacePlanWork::Visit {
                expr: child,
                plan: child_plan,
            });
            return;
        }
        index += 1;
    }

    *completed = Some(Expr::call_with_shared_args(head, SharedArgs::new(args)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterative_replace_plan_rebuild_preserves_unchanged_sibling_storage() {
        let unchanged = Expr::call(system_expr("unchanged"), vec![Expr::small_integer(2)]);
        let unchanged_before = unchanged.clone();
        let mut expr = Expr::call(system_expr("List"), vec![Expr::small_integer(1), unchanged]);
        let mut plan = ReplacePlanNode::default();
        plan.children.insert(
            1,
            ReplacePlanNode {
                action: Some(ReplacePlanAction::Immediate(Expr::small_integer(9))),
                children: BTreeMap::new(),
            },
        );
        for _ in 0..RECURSIVE_REPLACE_PLAN_DEPTH {
            expr = Expr::call(system_expr("List"), vec![expr]);
            let mut parent_plan = ReplacePlanNode::default();
            parent_plan.children.insert(1, plan);
            plan = parent_plan;
        }

        let replaced = Session::new()
            .apply_replace_plan_owned(expr, plan)
            .expect("immediate ReplacePart plan should complete");
        let mut cursor = &replaced;
        for _ in 0..RECURSIVE_REPLACE_PLAN_DEPTH {
            let Expr::Call { args, .. } = cursor else {
                panic!("rebuilt wrapper should remain a call")
            };
            cursor = &args[0];
        }
        let Expr::Call { args, .. } = cursor else {
            panic!("iteratively rebuilt expression should remain a call")
        };
        assert_eq!(args[0], Expr::small_integer(9));
        let Expr::Call {
            head: before_head,
            args: before_args,
        } = &unchanged_before
        else {
            panic!("unchanged fixture should be a call")
        };
        let Expr::Call {
            head: after_head,
            args: after_args,
        } = &args[1]
        else {
            panic!("unchanged sibling should remain a call")
        };

        assert!(SharedExpr::ptr_eq(before_head, after_head));
        assert!(SharedArgs::ptr_eq(before_args, after_args));
    }
}
