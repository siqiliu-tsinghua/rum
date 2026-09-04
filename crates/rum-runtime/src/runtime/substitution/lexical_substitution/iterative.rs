use super::*;
use std::sync::Arc;

type SharedShadowSet = Arc<BTreeSet<Symbol>>;

enum LexicalOperation {
    Expr {
        expr: Expr,
        shadowed: SharedShadowSet,
    },
    WithBindingList {
        expr: Expr,
        shadowed: SharedShadowSet,
    },
    WithBindingItem {
        expr: Expr,
        shadowed: SharedShadowSet,
    },
}

enum LexicalWork {
    Visit(LexicalOperation),
    FinishPattern {
        head: SharedExpr,
        args: SharedArgs,
    },
    FinishAssignmentPrefix {
        head: SharedExpr,
        args: SharedArgs,
        shadowed: SharedShadowSet,
    },
    FinishAssignment {
        head: SharedExpr,
        args: SharedArgs,
        substituted_head: Option<Expr>,
        substituted_lhs: Option<Expr>,
    },
    FinishConditionPrefix {
        head: SharedExpr,
        args: SharedArgs,
        shadowed: SharedShadowSet,
    },
    FinishCondition {
        head: SharedExpr,
        args: SharedArgs,
        substituted_head: Option<Expr>,
        substituted_lhs: Option<Expr>,
    },
    FinishWith {
        head: SharedExpr,
        args: SharedArgs,
    },
    FinishScoped {
        head: SharedExpr,
        args: SharedArgs,
        substituted_arg_indices: Vec<usize>,
    },
    FinishFunctionSlot {
        head: SharedExpr,
        args: SharedArgs,
        substituted_arg_indices: Vec<usize>,
    },
    FinishFunctionExplicit {
        head: SharedExpr,
        args: SharedArgs,
        substituted_arg_indices: Vec<usize>,
    },
    FinishGeneric {
        head: SharedExpr,
        args: SharedArgs,
        substituted_arg_indices: Vec<usize>,
    },
    FinishWithBindingList {
        head: SharedExpr,
        args: SharedArgs,
    },
    FinishWithBindingItem {
        head: SharedExpr,
        args: SharedArgs,
    },
}

pub(super) fn substitute_lexical_bindings_changed_iterative(
    expr: &Expr,
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    run_lexical_operation(
        LexicalOperation::Expr {
            expr: expr.clone(),
            shadowed: Arc::new(shadowed.clone()),
        },
        bindings,
    )
}

pub(super) fn substitute_with_binding_list_changed_iterative(
    expr: &Expr,
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    run_lexical_operation(
        LexicalOperation::WithBindingList {
            expr: expr.clone(),
            shadowed: Arc::new(shadowed.clone()),
        },
        bindings,
    )
}

pub(super) fn substitute_with_binding_item_changed_iterative(
    expr: &Expr,
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    run_lexical_operation(
        LexicalOperation::WithBindingItem {
            expr: expr.clone(),
            shadowed: Arc::new(shadowed.clone()),
        },
        bindings,
    )
}

#[cold]
#[inline(never)]
fn run_lexical_operation(root: LexicalOperation, bindings: &LexicalBindings<'_>) -> Option<Expr> {
    let mut work = vec![LexicalWork::Visit(root)];
    let mut completed = Vec::<Option<Expr>>::new();

    while let Some(task) = work.pop() {
        match task {
            LexicalWork::Visit(operation) => {
                visit_lexical_operation(operation, bindings, &mut work, &mut completed)
            }
            LexicalWork::FinishPattern { head, args } => {
                let mut children = take_completed(&mut completed, 2).into_iter();
                let substituted_head = children.next().expect("Pattern head completes");
                let substituted_pattern = children.next().expect("Pattern body completes");
                debug_assert!(children.next().is_none());

                let _profile_context = lexical_profile_context("pattern_call");
                let result = if substituted_head.is_none() && substituted_pattern.is_none() {
                    None
                } else {
                    Some(record_lexical_rebuild(
                        "pattern_call",
                        Expr::call(
                            substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                            vec![
                                args[0].clone(),
                                substituted_pattern.unwrap_or_else(|| args[1].clone()),
                            ],
                        ),
                    ))
                };
                completed.push(result);
            }
            LexicalWork::FinishAssignmentPrefix {
                head,
                args,
                shadowed,
            } => {
                let mut children = take_completed(&mut completed, 2).into_iter();
                let substituted_head = children.next().expect("assignment head completes");
                let substituted_lhs = children.next().expect("assignment LHS completes");
                debug_assert!(children.next().is_none());

                let lhs_for_shadow = substituted_lhs.as_ref().unwrap_or(&args[0]);
                let rhs_shadowed = Arc::new(shadowed_with_pattern_locals(
                    shadowed.as_ref(),
                    lhs_for_shadow,
                ));
                work.push(LexicalWork::FinishAssignment {
                    head,
                    args: args.clone(),
                    substituted_head,
                    substituted_lhs,
                });
                work.push(LexicalWork::Visit(LexicalOperation::Expr {
                    expr: args[1].clone(),
                    shadowed: rhs_shadowed,
                }));
            }
            LexicalWork::FinishAssignment {
                head,
                args,
                substituted_head,
                substituted_lhs,
            } => {
                let substituted_rhs = completed.pop().expect("assignment RHS completes");
                let _profile_context = lexical_profile_context("assignment_like_call");
                let result = if substituted_head.is_none()
                    && substituted_lhs.is_none()
                    && substituted_rhs.is_none()
                {
                    None
                } else {
                    Some(record_lexical_rebuild(
                        "assignment_like_call",
                        Expr::call(
                            substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                            vec![
                                substituted_lhs.unwrap_or_else(|| args[0].clone()),
                                substituted_rhs.unwrap_or_else(|| args[1].clone()),
                            ],
                        ),
                    ))
                };
                completed.push(result);
            }
            LexicalWork::FinishConditionPrefix {
                head,
                args,
                shadowed,
            } => {
                let mut children = take_completed(&mut completed, 2).into_iter();
                let substituted_head = children.next().expect("Condition head completes");
                let substituted_lhs = children.next().expect("Condition LHS completes");
                debug_assert!(children.next().is_none());

                let lhs_for_shadow = substituted_lhs.as_ref().unwrap_or(&args[0]);
                let condition_shadowed = Arc::new(shadowed_with_pattern_locals(
                    shadowed.as_ref(),
                    lhs_for_shadow,
                ));
                work.push(LexicalWork::FinishCondition {
                    head,
                    args: args.clone(),
                    substituted_head,
                    substituted_lhs,
                });
                work.push(LexicalWork::Visit(LexicalOperation::Expr {
                    expr: args[1].clone(),
                    shadowed: condition_shadowed,
                }));
            }
            LexicalWork::FinishCondition {
                head,
                args,
                substituted_head,
                substituted_lhs,
            } => {
                let substituted_condition = completed.pop().expect("Condition predicate completes");
                let _profile_context = lexical_profile_context("condition_call");
                let result = if substituted_head.is_none()
                    && substituted_lhs.is_none()
                    && substituted_condition.is_none()
                {
                    None
                } else {
                    Some(record_lexical_rebuild(
                        "condition_call",
                        Expr::call(
                            substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                            vec![
                                substituted_lhs.unwrap_or_else(|| args[0].clone()),
                                substituted_condition.unwrap_or_else(|| args[1].clone()),
                            ],
                        ),
                    ))
                };
                completed.push(result);
            }
            LexicalWork::FinishWith { head, args } => {
                let mut children = take_completed(&mut completed, 3).into_iter();
                let substituted_head = children.next().expect("With head completes");
                let substituted_bindings = children.next().expect("With bindings complete");
                let substituted_body = children.next().expect("With body completes");
                debug_assert!(children.next().is_none());

                let _profile_context = lexical_profile_context("with_call");
                let result = if substituted_head.is_none()
                    && substituted_bindings.is_none()
                    && substituted_body.is_none()
                {
                    None
                } else {
                    Some(record_lexical_rebuild(
                        "with_call",
                        Expr::call(
                            substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                            vec![
                                substituted_bindings.unwrap_or_else(|| args[0].clone()),
                                substituted_body.unwrap_or_else(|| args[1].clone()),
                            ],
                        ),
                    ))
                };
                completed.push(result);
            }
            LexicalWork::FinishScoped {
                head,
                args,
                substituted_arg_indices,
            } => {
                let child_count = 2 + substituted_arg_indices.len();
                let mut children = take_completed(&mut completed, child_count).into_iter();
                let substituted_head = children.next().expect("scope head completes");
                let substituted_bindings = children.next().expect("scope bindings complete");
                let substituted_rest =
                    finish_arg_list(&args[1..], &substituted_arg_indices, children.collect());

                let _profile_context = lexical_profile_context("scoped_call");
                let result = if substituted_head.is_none()
                    && substituted_bindings.is_none()
                    && substituted_rest.is_none()
                {
                    None
                } else {
                    let mut substituted_args = Vec::with_capacity(
                        1 + substituted_rest.as_ref().map_or(args.len() - 1, Vec::len),
                    );
                    substituted_args.push(substituted_bindings.unwrap_or_else(|| args[0].clone()));
                    substituted_args.extend(substituted_rest.unwrap_or_else(|| args[1..].to_vec()));
                    Some(record_lexical_rebuild(
                        "scoped_call",
                        Expr::call(
                            substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                            substituted_args,
                        ),
                    ))
                };
                completed.push(result);
            }
            LexicalWork::FinishFunctionSlot {
                head,
                args,
                substituted_arg_indices,
            } => {
                let child_count = 2 + substituted_arg_indices.len();
                let mut children = take_completed(&mut completed, child_count).into_iter();
                let substituted_head = children.next().expect("Function head completes");
                let substituted_body = children.next().expect("Function body completes");
                let substituted_rest =
                    finish_arg_list(&args[1..], &substituted_arg_indices, children.collect());

                let _profile_context = lexical_profile_context("function_call");
                let result = if substituted_head.is_none()
                    && substituted_body.is_none()
                    && substituted_rest.is_none()
                {
                    None
                } else {
                    let mut substituted_args = Vec::with_capacity(
                        1 + substituted_rest.as_ref().map_or(args.len() - 1, Vec::len),
                    );
                    substituted_args.push(substituted_body.unwrap_or_else(|| args[0].clone()));
                    substituted_args.extend(substituted_rest.unwrap_or_else(|| args[1..].to_vec()));
                    Some(record_lexical_rebuild(
                        "function_call",
                        Expr::call(
                            substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                            substituted_args,
                        ),
                    ))
                };
                completed.push(result);
            }
            LexicalWork::FinishFunctionExplicit {
                head,
                args,
                substituted_arg_indices,
            } => {
                let child_count = 2 + substituted_arg_indices.len();
                let mut children = take_completed(&mut completed, child_count).into_iter();
                let substituted_head = children.next().expect("Function head completes");
                let substituted_body = children.next().expect("Function body completes");
                let substituted_rest =
                    finish_arg_list(&args[2..], &substituted_arg_indices, children.collect());

                let _profile_context = lexical_profile_context("function_call");
                let result = if substituted_head.is_none()
                    && substituted_body.is_none()
                    && substituted_rest.is_none()
                {
                    None
                } else {
                    let mut substituted_args = Vec::with_capacity(
                        2 + substituted_rest.as_ref().map_or(args.len() - 2, Vec::len),
                    );
                    substituted_args.push(args[0].clone());
                    substituted_args.push(substituted_body.unwrap_or_else(|| args[1].clone()));
                    substituted_args.extend(substituted_rest.unwrap_or_else(|| args[2..].to_vec()));
                    Some(record_lexical_rebuild(
                        "function_call",
                        Expr::call(
                            substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                            substituted_args,
                        ),
                    ))
                };
                completed.push(result);
            }
            LexicalWork::FinishGeneric {
                head,
                args,
                substituted_arg_indices,
            } => {
                let child_count = 1 + substituted_arg_indices.len();
                let mut children = take_completed(&mut completed, child_count).into_iter();
                let substituted_head = children.next().expect("generic head completes");
                let substituted_args =
                    finish_arg_list(&args, &substituted_arg_indices, children.collect());
                let _profile_context = lexical_profile_context("generic_call");
                completed.push(finish_generic_call(
                    head.as_ref(),
                    &args,
                    substituted_head,
                    substituted_args,
                ));
            }
            LexicalWork::FinishWithBindingList { head, args } => {
                let child_count = 1 + args.len();
                let mut children = take_completed(&mut completed, child_count).into_iter();
                let substituted_head = children.next().expect("binding-list head completes");
                let mut substituted_items = None;
                for (index, substituted_item) in children.enumerate() {
                    if let Some(substituted_item) = substituted_item {
                        substituted_items
                            .get_or_insert_with(|| args[..index].to_vec())
                            .push(substituted_item);
                    } else if let Some(substituted_items) = &mut substituted_items {
                        substituted_items.push(args[index].clone());
                    }
                }

                let _profile_context = lexical_profile_context("with_binding_list");
                let result = match (substituted_head, substituted_items) {
                    (None, None) => None,
                    (Some(head), None) => Some(record_lexical_rebuild(
                        "with_binding_list",
                        Expr::call_with_shared_args(head, args.clone()),
                    )),
                    (None, Some(args)) => Some(record_lexical_rebuild(
                        "with_binding_list",
                        Expr::call_shared(head.clone(), SharedArgs::new(args)),
                    )),
                    (Some(head), Some(args)) => Some(record_lexical_rebuild(
                        "with_binding_list",
                        Expr::call(head, args),
                    )),
                };
                completed.push(result);
            }
            LexicalWork::FinishWithBindingItem { head, args } => {
                let mut children = take_completed(&mut completed, 2).into_iter();
                let substituted_head = children.next().expect("binding-item head completes");
                let substituted_rhs = children.next().expect("binding-item RHS completes");
                debug_assert!(children.next().is_none());

                let _profile_context = lexical_profile_context("with_binding_item");
                let result = if substituted_head.is_none() && substituted_rhs.is_none() {
                    None
                } else {
                    Some(record_lexical_rebuild(
                        "with_binding_item",
                        Expr::call(
                            substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                            vec![
                                args[0].clone(),
                                substituted_rhs.unwrap_or_else(|| args[1].clone()),
                            ],
                        ),
                    ))
                };
                completed.push(result);
            }
        }
    }

    debug_assert_eq!(completed.len(), 1);
    completed.pop().expect("iterative lexical root completes")
}

fn visit_lexical_operation(
    operation: LexicalOperation,
    bindings: &LexicalBindings<'_>,
    work: &mut Vec<LexicalWork>,
    completed: &mut Vec<Option<Expr>>,
) {
    match operation {
        LexicalOperation::Expr { expr, shadowed } => {
            visit_expr(expr, shadowed, bindings, work, completed)
        }
        LexicalOperation::WithBindingList { expr, shadowed } => {
            visit_with_binding_list(expr, shadowed, work, completed)
        }
        LexicalOperation::WithBindingItem { expr, shadowed } => {
            visit_with_binding_item(expr, shadowed, work, completed)
        }
    }
}

fn visit_expr(
    expr: Expr,
    shadowed: SharedShadowSet,
    bindings: &LexicalBindings<'_>,
    work: &mut Vec<LexicalWork>,
    completed: &mut Vec<Option<Expr>>,
) {
    match &expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            completed.push(substitute_symbol_binding_changed(
                symbol,
                bindings,
                shadowed.as_ref(),
            ));
        }
        Expr::Atom(_) => completed.push(None),
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            let _profile_context = lexical_profile_context("pattern_call");
            let operations = vec![
                expr_operation(head.as_ref().clone(), &shadowed),
                expr_operation(args[1].clone(), &shadowed),
            ];
            schedule_operations(
                work,
                LexicalWork::FinishPattern {
                    head: head.clone(),
                    args: args.clone(),
                },
                operations,
            );
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed")
                || is_system_symbol_name(head, "SetDelayed"))
                && args.len() == 2 =>
        {
            let _profile_context = lexical_profile_context("assignment_like_call");
            let operations = vec![
                expr_operation(head.as_ref().clone(), &shadowed),
                expr_operation(args[0].clone(), &shadowed),
            ];
            schedule_operations(
                work,
                LexicalWork::FinishAssignmentPrefix {
                    head: head.clone(),
                    args: args.clone(),
                    shadowed,
                },
                operations,
            );
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            let _profile_context = lexical_profile_context("condition_call");
            let operations = vec![
                expr_operation(head.as_ref().clone(), &shadowed),
                expr_operation(args[0].clone(), &shadowed),
            ];
            schedule_operations(
                work,
                LexicalWork::FinishConditionPrefix {
                    head: head.clone(),
                    args: args.clone(),
                    shadowed,
                },
                operations,
            );
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "With") && args.len() == 2 => {
            let _profile_context = lexical_profile_context("with_call");
            let body_shadowed =
                Arc::new(shadowed_with_binding_symbols(shadowed.as_ref(), &args[0]));
            let operations = vec![
                expr_operation(head.as_ref().clone(), &shadowed),
                LexicalOperation::WithBindingList {
                    expr: args[0].clone(),
                    shadowed: shadowed.clone(),
                },
                LexicalOperation::Expr {
                    expr: args[1].clone(),
                    shadowed: body_shadowed,
                },
            ];
            schedule_operations(
                work,
                LexicalWork::FinishWith {
                    head: head.clone(),
                    args: args.clone(),
                },
                operations,
            );
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Module") || is_system_symbol_name(head, "Block"))
                && !args.is_empty() =>
        {
            let _profile_context = lexical_profile_context("scoped_call");
            let body_shadowed =
                Arc::new(shadowed_with_binding_symbols(shadowed.as_ref(), &args[0]));
            let (substituted_arg_indices, mut rest_operations) =
                select_arg_operations(&args[1..], bindings, &body_shadowed, head.as_ref(), 1);
            let mut operations = Vec::with_capacity(2 + rest_operations.len());
            operations.push(expr_operation(head.as_ref().clone(), &shadowed));
            operations.push(LexicalOperation::WithBindingList {
                expr: args[0].clone(),
                shadowed,
            });
            operations.append(&mut rest_operations);
            schedule_operations(
                work,
                LexicalWork::FinishScoped {
                    head: head.clone(),
                    args: args.clone(),
                    substituted_arg_indices,
                },
                operations,
            );
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") => {
            visit_function(&expr, shadowed, bindings, head, args, work);
        }
        Expr::Call { head, args } => {
            let _profile_context = lexical_profile_context("generic_call");
            let (substituted_arg_indices, mut arg_operations) =
                select_arg_operations(args, bindings, &shadowed, head.as_ref(), 0);
            let mut operations = Vec::with_capacity(1 + arg_operations.len());
            operations.push(expr_operation(head.as_ref().clone(), &shadowed));
            operations.append(&mut arg_operations);
            schedule_operations(
                work,
                LexicalWork::FinishGeneric {
                    head: head.clone(),
                    args: args.clone(),
                    substituted_arg_indices,
                },
                operations,
            );
        }
    }
}

fn visit_function(
    expr: &Expr,
    shadowed: SharedShadowSet,
    bindings: &LexicalBindings<'_>,
    head: &SharedExpr,
    args: &SharedArgs,
    work: &mut Vec<LexicalWork>,
) {
    let _profile_context = lexical_profile_context("function_call");
    match parse_function(&expr) {
        Some(FunctionSpec::SlotBased { body, .. }) => {
            let body = body.clone();
            let (substituted_arg_indices, mut rest_operations) =
                select_arg_operations(&args[1..], bindings, &shadowed, head.as_ref(), 1);
            let mut operations = Vec::with_capacity(2 + rest_operations.len());
            operations.push(expr_operation(head.as_ref().clone(), &shadowed));
            operations.push(expr_operation(body, &shadowed));
            operations.append(&mut rest_operations);
            schedule_operations(
                work,
                LexicalWork::FinishFunctionSlot {
                    head: head.clone(),
                    args: args.clone(),
                    substituted_arg_indices,
                },
                operations,
            );
        }
        Some(FunctionSpec::Explicit { params, body, .. }) => {
            let body = body.clone();
            let mut body_shadowed = shadowed.as_ref().clone();
            body_shadowed.extend(params);
            let body_shadowed = Arc::new(body_shadowed);
            let (substituted_arg_indices, mut rest_operations) =
                select_arg_operations(&args[2..], bindings, &shadowed, head.as_ref(), 2);
            let mut operations = Vec::with_capacity(2 + rest_operations.len());
            operations.push(expr_operation(head.as_ref().clone(), &shadowed));
            operations.push(LexicalOperation::Expr {
                expr: body,
                shadowed: body_shadowed,
            });
            operations.append(&mut rest_operations);
            schedule_operations(
                work,
                LexicalWork::FinishFunctionExplicit {
                    head: head.clone(),
                    args: args.clone(),
                    substituted_arg_indices,
                },
                operations,
            );
        }
        None => {
            let (substituted_arg_indices, mut arg_operations) =
                select_arg_operations(args, bindings, &shadowed, head.as_ref(), 0);
            let mut operations = Vec::with_capacity(1 + arg_operations.len());
            operations.push(expr_operation(head.as_ref().clone(), &shadowed));
            operations.append(&mut arg_operations);
            schedule_operations(
                work,
                LexicalWork::FinishGeneric {
                    head: head.clone(),
                    args: args.clone(),
                    substituted_arg_indices,
                },
                operations,
            );
        }
    }
}

fn visit_with_binding_list(
    expr: Expr,
    shadowed: SharedShadowSet,
    work: &mut Vec<LexicalWork>,
    completed: &mut Vec<Option<Expr>>,
) {
    let _profile_context = lexical_profile_context("with_binding_list");
    let Expr::Call { head, args } = &expr else {
        work.push(LexicalWork::Visit(LexicalOperation::Expr {
            expr,
            shadowed,
        }));
        return;
    };

    if !is_system_symbol_name(head, "List") {
        work.push(LexicalWork::Visit(LexicalOperation::Expr {
            expr,
            shadowed,
        }));
        return;
    }

    if args.is_empty() {
        work.push(LexicalWork::FinishWithBindingList {
            head: head.clone(),
            args: args.clone(),
        });
        work.push(LexicalWork::Visit(expr_operation(
            head.as_ref().clone(),
            &shadowed,
        )));
        return;
    }

    let mut operations = Vec::with_capacity(1 + args.len());
    operations.push(expr_operation(head.as_ref().clone(), &shadowed));
    operations.extend(
        args.iter()
            .cloned()
            .map(|item| LexicalOperation::WithBindingItem {
                expr: item,
                shadowed: shadowed.clone(),
            }),
    );
    schedule_operations(
        work,
        LexicalWork::FinishWithBindingList {
            head: head.clone(),
            args: args.clone(),
        },
        operations,
    );

    let _ = completed;
}

fn visit_with_binding_item(
    expr: Expr,
    shadowed: SharedShadowSet,
    work: &mut Vec<LexicalWork>,
    completed: &mut Vec<Option<Expr>>,
) {
    let _profile_context = lexical_profile_context("with_binding_item");
    match &expr {
        Expr::Atom(Atom::Symbol(_)) => completed.push(None),
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2
                && expr_as_symbol(&args[0]).is_some() =>
        {
            let operations = vec![
                expr_operation(head.as_ref().clone(), &shadowed),
                expr_operation(args[1].clone(), &shadowed),
            ];
            schedule_operations(
                work,
                LexicalWork::FinishWithBindingItem {
                    head: head.clone(),
                    args: args.clone(),
                },
                operations,
            );
        }
        _ => work.push(LexicalWork::Visit(LexicalOperation::Expr {
            expr,
            shadowed,
        })),
    }
}

fn expr_operation(expr: Expr, shadowed: &SharedShadowSet) -> LexicalOperation {
    LexicalOperation::Expr {
        expr,
        shadowed: shadowed.clone(),
    }
}

fn select_arg_operations(
    args: &[Expr],
    bindings: &LexicalBindings<'_>,
    shadowed: &SharedShadowSet,
    parent_head: &Expr,
    index_offset: usize,
) -> (Vec<usize>, Vec<LexicalOperation>) {
    record_profile_arg_list_start(args.len());
    let mut indices = Vec::new();
    let mut operations = Vec::new();

    for (index, arg) in args.iter().enumerate() {
        let maybe_contains_binding = {
            let _profile_site =
                lexical_profile_arg_prefilter_site(parent_head, index + index_offset);
            let _profile_context = lexical_profile_context("arg_prefilter");
            bindings.may_contain_binding_symbol(arg)
        };
        record_profile_arg_prefilter(maybe_contains_binding);
        if maybe_contains_binding {
            indices.push(index);
            operations.push(expr_operation(arg.clone(), shadowed));
        }
    }

    (indices, operations)
}

fn finish_arg_list(
    args: &[Expr],
    substituted_arg_indices: &[usize],
    child_results: Vec<Option<Expr>>,
) -> Option<Vec<Expr>> {
    debug_assert_eq!(substituted_arg_indices.len(), child_results.len());
    let profile_detail = lexical_profile_detail_enabled();
    let mut child_results = child_results.into_iter();
    let mut next_index = substituted_arg_indices.iter().copied();
    let mut selected = next_index.next();
    let mut substituted_args = None;

    for (index, arg) in args.iter().enumerate() {
        let substituted_arg = if selected == Some(index) {
            selected = next_index.next();
            let result = child_results
                .next()
                .expect("selected lexical argument completes");
            if profile_detail {
                let profile_start = lexical_profile_start();
                record_profile_arg_substitution_attempt(profile_start, result.is_some());
            }
            result
        } else {
            None
        };

        if let Some(substituted_arg) = substituted_arg {
            substituted_args
                .get_or_insert_with(|| args[..index].to_vec())
                .push(substituted_arg);
        } else if let Some(substituted_args) = &mut substituted_args {
            substituted_args.push(arg.clone());
        }
    }

    debug_assert!(selected.is_none());
    debug_assert!(next_index.next().is_none());
    debug_assert!(child_results.next().is_none());
    substituted_args
}

fn finish_generic_call(
    head: &Expr,
    args: &[Expr],
    substituted_head: Option<Expr>,
    substituted_args: Option<Vec<Expr>>,
) -> Option<Expr> {
    match (substituted_head, substituted_args) {
        (None, None) => None,
        (Some(head), None) => Some(record_lexical_rebuild(
            "generic_call",
            Expr::call_with_shared_args(head, SharedArgs::from(args.to_vec())),
        )),
        (None, Some(args)) => Some(record_lexical_rebuild(
            "generic_call",
            Expr::call_shared(SharedExpr::from(head.clone()), SharedArgs::new(args)),
        )),
        (Some(head), Some(args)) => Some(record_lexical_rebuild(
            "generic_call",
            Expr::call(head, args),
        )),
    }
}

fn schedule_operations(
    work: &mut Vec<LexicalWork>,
    finish: LexicalWork,
    operations: Vec<LexicalOperation>,
) {
    work.push(finish);
    work.extend(operations.into_iter().rev().map(LexicalWork::Visit));
}

fn take_completed(completed: &mut Vec<Option<Expr>>, child_count: usize) -> Vec<Option<Expr>> {
    let first_child = completed
        .len()
        .checked_sub(child_count)
        .expect("iterative lexical results remain balanced");
    completed.split_off(first_child)
}
