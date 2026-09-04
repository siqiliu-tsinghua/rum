use super::*;
use std::sync::Arc;

enum BindingWork {
    Visit(Expr),
    FinishCall { head: SharedExpr, args: SharedArgs },
}

pub(super) fn substitute_bindings_changed_iterative(
    expr: &Expr,
    bindings: &SymbolExprMap,
) -> Option<Expr> {
    let mut work = vec![BindingWork::Visit(expr.clone())];
    let mut completed = Vec::<Option<Expr>>::new();

    while let Some(task) = work.pop() {
        match task {
            BindingWork::Visit(expr) => match &expr {
                Expr::Atom(Atom::Symbol(symbol)) => {
                    completed.push(bindings.get(symbol).cloned());
                }
                Expr::Atom(_) => completed.push(None),
                Expr::Call { head, args } => {
                    work.push(BindingWork::FinishCall {
                        head: head.clone(),
                        args: args.clone(),
                    });
                    work.extend(args.iter().rev().cloned().map(BindingWork::Visit));
                    work.push(BindingWork::Visit(head.as_ref().clone()));
                }
            },
            BindingWork::FinishCall { head, args } => {
                let mut children = take_completed(&mut completed, 1 + args.len()).into_iter();
                let substituted_head = children.next().expect("binding head completes");
                let mut substituted_args = None;
                for (index, substituted_arg) in children.enumerate() {
                    if let Some(substituted_arg) = substituted_arg {
                        substituted_args
                            .get_or_insert_with(|| args[..index].to_vec())
                            .push(substituted_arg);
                    } else if let Some(substituted_args) = &mut substituted_args {
                        substituted_args.push(args[index].clone());
                    }
                }

                completed.push(match (substituted_head, substituted_args) {
                    (None, None) => None,
                    (Some(head), None) => Some(Expr::call_with_shared_args(head, args.clone())),
                    (None, Some(args)) => {
                        Some(Expr::call_shared(head.clone(), SharedArgs::new(args)))
                    }
                    (Some(head), Some(args)) => Some(Expr::call(head, args)),
                });
            }
        }
    }

    debug_assert_eq!(completed.len(), 1);
    completed.pop().expect("iterative binding root completes")
}

type SharedAlphaRenames = Arc<SymbolExprMap>;

enum RuleOperation {
    Expr {
        expr: Expr,
        alpha_renames: SharedAlphaRenames,
    },
    WithBindingList {
        expr: Expr,
        alpha_renames: SharedAlphaRenames,
        scoped_alpha_renames: SharedAlphaRenames,
    },
    WithBindingItem {
        expr: Expr,
        rhs_alpha_renames: SharedAlphaRenames,
        scoped_alpha_renames: SharedAlphaRenames,
    },
}

enum RuleWork {
    Visit(RuleOperation),
    FinishCall { head: SharedExpr, args: SharedArgs },
    FinishScopedCall { head: SharedExpr, args: SharedArgs },
    FinishWithFallbackCall { head: SharedExpr, args: SharedArgs },
    FinishFunctionCall { head: SharedExpr, args: SharedArgs },
    FinishWith { head: SharedExpr, args: SharedArgs },
    FinishWithBindingList { head: SharedExpr, args: SharedArgs },
    FinishWithBindingListPassthrough,
    FinishWithBindingItem { head: SharedExpr, args: SharedArgs },
    FinishWithBindingItemPassthrough,
}

pub(super) fn substitute_rule_bindings_changed_iterative(
    expr: &Expr,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
) -> Option<Expr> {
    run_rule_operation(
        RuleOperation::Expr {
            expr: expr.clone(),
            alpha_renames: Arc::new(alpha_renames.clone()),
        },
        bindings,
    )
}

pub(super) fn substitute_rule_with_binding_list_changed_iterative(
    expr: &Expr,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
    scoped_alpha_renames: &SymbolExprMap,
) -> Option<Expr> {
    run_rule_operation(
        RuleOperation::WithBindingList {
            expr: expr.clone(),
            alpha_renames: Arc::new(alpha_renames.clone()),
            scoped_alpha_renames: Arc::new(scoped_alpha_renames.clone()),
        },
        bindings,
    )
}

pub(super) fn substitute_rule_with_binding_item_changed_iterative(
    expr: &Expr,
    bindings: &SymbolExprMap,
    rhs_alpha_renames: &SymbolExprMap,
    scoped_alpha_renames: &SymbolExprMap,
) -> Option<Expr> {
    run_rule_operation(
        RuleOperation::WithBindingItem {
            expr: expr.clone(),
            rhs_alpha_renames: Arc::new(rhs_alpha_renames.clone()),
            scoped_alpha_renames: Arc::new(scoped_alpha_renames.clone()),
        },
        bindings,
    )
}

#[cold]
#[inline(never)]
fn run_rule_operation(root: RuleOperation, bindings: &SymbolExprMap) -> Option<Expr> {
    let mut work = vec![RuleWork::Visit(root)];
    let mut completed = Vec::<Option<Expr>>::new();

    while let Some(task) = work.pop() {
        match task {
            RuleWork::Visit(operation) => {
                visit_rule_operation(operation, bindings, &mut work, &mut completed)
            }
            RuleWork::FinishCall { head, args } => {
                let result = finish_rule_call(&mut completed, &head, &args);
                completed.push(result);
            }
            RuleWork::FinishScopedCall { head, args } => {
                let result = finish_rule_call(&mut completed, &head, &args);
                record_iterative_rule_timing("scoped_call", result.is_some());
                completed.push(result);
            }
            RuleWork::FinishWithFallbackCall { head, args } => {
                let result = finish_rule_call(&mut completed, &head, &args);
                record_iterative_rule_timing("with_call", result.is_some());
                completed.push(result);
            }
            RuleWork::FinishFunctionCall { head, args } => {
                let result = finish_rule_call(&mut completed, &head, &args);
                record_iterative_rule_timing("function_call", result.is_some());
                completed.push(result);
            }
            RuleWork::FinishWith { head, args } => {
                let mut children = take_completed(&mut completed, 3).into_iter();
                let substituted_head = children.next().expect("rule With head completes");
                let substituted_bindings = children.next().expect("rule With bindings complete");
                let substituted_body = children.next().expect("rule With body completes");
                debug_assert!(children.next().is_none());

                let result = if substituted_head.is_none()
                    && substituted_bindings.is_none()
                    && substituted_body.is_none()
                {
                    None
                } else {
                    Some(record_rule_rhs_rebuild(
                        "rule_rhs_with_call",
                        Expr::call(
                            substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                            vec![
                                substituted_bindings.unwrap_or_else(|| args[0].clone()),
                                substituted_body.unwrap_or_else(|| args[1].clone()),
                            ],
                        ),
                    ))
                };
                record_iterative_rule_timing("with_call", result.is_some());
                completed.push(result);
            }
            RuleWork::FinishWithBindingList { head, args } => {
                let child_count = 1 + args.len();
                let mut children = take_completed(&mut completed, child_count).into_iter();
                let substituted_head = children.next().expect("rule binding-list head completes");
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

                let result = match (substituted_head, substituted_items) {
                    (None, None) => None,
                    (Some(head), None) => Some(record_rule_rhs_rebuild(
                        "rule_rhs_with_binding_list",
                        Expr::call_with_shared_args(head, args.clone()),
                    )),
                    (None, Some(args)) => Some(record_rule_rhs_rebuild(
                        "rule_rhs_with_binding_list",
                        Expr::call_shared(
                            SharedExpr::from(head.as_ref().clone()),
                            SharedArgs::new(args),
                        ),
                    )),
                    (Some(head), Some(args)) => Some(record_rule_rhs_rebuild(
                        "rule_rhs_with_binding_list",
                        Expr::call(head, args),
                    )),
                };
                record_iterative_rule_timing("with_binding_list", result.is_some());
                completed.push(result);
            }
            RuleWork::FinishWithBindingListPassthrough => {
                let result = completed
                    .pop()
                    .expect("rule binding-list passthrough completes");
                record_iterative_rule_timing("with_binding_list", result.is_some());
                completed.push(result);
            }
            RuleWork::FinishWithBindingItem { head, args } => {
                let mut children = take_completed(&mut completed, 3).into_iter();
                let substituted_head = children.next().expect("rule binding-item head completes");
                let substituted_lhs = children.next().expect("rule binding-item LHS completes");
                let substituted_rhs = children.next().expect("rule binding-item RHS completes");
                debug_assert!(children.next().is_none());

                let result = if substituted_head.is_none()
                    && substituted_lhs.is_none()
                    && substituted_rhs.is_none()
                {
                    None
                } else {
                    Some(record_rule_rhs_rebuild(
                        "rule_rhs_with_binding_item",
                        Expr::call(
                            substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                            vec![
                                substituted_lhs.unwrap_or_else(|| args[0].clone()),
                                substituted_rhs.unwrap_or_else(|| args[1].clone()),
                            ],
                        ),
                    ))
                };
                record_iterative_rule_timing("with_binding_item", result.is_some());
                completed.push(result);
            }
            RuleWork::FinishWithBindingItemPassthrough => {
                let result = completed
                    .pop()
                    .expect("rule binding-item passthrough completes");
                record_iterative_rule_timing("with_binding_item", result.is_some());
                completed.push(result);
            }
        }
    }

    debug_assert_eq!(completed.len(), 1);
    completed.pop().expect("iterative rule root completes")
}

fn visit_rule_operation(
    operation: RuleOperation,
    bindings: &SymbolExprMap,
    work: &mut Vec<RuleWork>,
    completed: &mut Vec<Option<Expr>>,
) {
    match operation {
        RuleOperation::Expr {
            expr,
            alpha_renames,
        } => visit_rule_expr(expr, alpha_renames, bindings, work, completed),
        RuleOperation::WithBindingList {
            expr,
            alpha_renames,
            scoped_alpha_renames,
        } => visit_rule_with_binding_list(expr, alpha_renames, scoped_alpha_renames, work),
        RuleOperation::WithBindingItem {
            expr,
            rhs_alpha_renames,
            scoped_alpha_renames,
        } => visit_rule_with_binding_item(expr, rhs_alpha_renames, scoped_alpha_renames, work),
    }
}

fn visit_rule_expr(
    expr: Expr,
    alpha_renames: SharedAlphaRenames,
    bindings: &SymbolExprMap,
    work: &mut Vec<RuleWork>,
    completed: &mut Vec<Option<Expr>>,
) {
    match &expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            let result = if let Some(value) = bindings.get(symbol) {
                record_rule_rhs_binding_hit(value);
                record_profile_rule_rhs_binding_value(value);
                Some(value.clone())
            } else if let Some(value) = alpha_renames.get(symbol) {
                record_rule_rhs_binding_hit(value);
                Some(value.clone())
            } else {
                None
            };
            completed.push(result);
        }
        Expr::Atom(_) => completed.push(None),
        Expr::Call { head, args } if is_system_symbol_name(head, "Module") && args.len() == 2 => {
            let operation_alpha_renames = if let Some(local_symbols) =
                parse_module_binding_list(&args[0]).map(scoped_binding_symbols)
            {
                Arc::new(alpha_renames_with_fresh_locals(
                    alpha_renames.as_ref(),
                    bindings,
                    local_symbols,
                ))
            } else {
                alpha_renames.clone()
            };
            schedule_rule_call(
                work,
                RuleWork::FinishScopedCall {
                    head: head.clone(),
                    args: args.clone(),
                },
                head,
                args,
                alpha_renames,
                |_, _| operation_alpha_renames.clone(),
            );
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "With") && args.len() == 2 => {
            let Some(local_symbols) = parse_with_binding_list(&args[0]).map(with_binding_symbols)
            else {
                schedule_rule_call(
                    work,
                    RuleWork::FinishWithFallbackCall {
                        head: head.clone(),
                        args: args.clone(),
                    },
                    head,
                    args,
                    alpha_renames.clone(),
                    |_, alpha_renames| alpha_renames.clone(),
                );
                return;
            };
            let scoped_alpha_renames = Arc::new(alpha_renames_with_fresh_locals(
                alpha_renames.as_ref(),
                bindings,
                local_symbols,
            ));
            let operations = vec![
                rule_expr_operation(head.as_ref().clone(), &alpha_renames),
                RuleOperation::WithBindingList {
                    expr: args[0].clone(),
                    alpha_renames: alpha_renames.clone(),
                    scoped_alpha_renames: scoped_alpha_renames.clone(),
                },
                RuleOperation::Expr {
                    expr: args[1].clone(),
                    alpha_renames: scoped_alpha_renames,
                },
            ];
            schedule_rule_operations(
                work,
                RuleWork::FinishWith {
                    head: head.clone(),
                    args: args.clone(),
                },
                operations,
            );
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") => {
            let scoped_alpha_renames = explicit_function_parameters_from_args(args).map(|params| {
                Arc::new(alpha_renames_with_fresh_locals(
                    alpha_renames.as_ref(),
                    bindings,
                    params,
                ))
            });
            schedule_rule_call(
                work,
                RuleWork::FinishFunctionCall {
                    head: head.clone(),
                    args: args.clone(),
                },
                head,
                args,
                alpha_renames.clone(),
                |index, alpha_renames| {
                    if index <= 1 {
                        scoped_alpha_renames
                            .as_ref()
                            .unwrap_or(alpha_renames)
                            .clone()
                    } else {
                        alpha_renames.clone()
                    }
                },
            );
        }
        Expr::Call { head, args } => schedule_rule_call(
            work,
            RuleWork::FinishCall {
                head: head.clone(),
                args: args.clone(),
            },
            head,
            args,
            alpha_renames.clone(),
            |_, alpha_renames| alpha_renames.clone(),
        ),
    }
}

fn visit_rule_with_binding_list(
    expr: Expr,
    alpha_renames: SharedAlphaRenames,
    scoped_alpha_renames: SharedAlphaRenames,
    work: &mut Vec<RuleWork>,
) {
    let Expr::Call { head, args } = &expr else {
        work.push(RuleWork::FinishWithBindingListPassthrough);
        work.push(RuleWork::Visit(RuleOperation::Expr {
            expr,
            alpha_renames,
        }));
        return;
    };

    if !is_system_symbol_name(head, "List") {
        work.push(RuleWork::FinishWithBindingListPassthrough);
        work.push(RuleWork::Visit(RuleOperation::Expr {
            expr,
            alpha_renames,
        }));
        return;
    }

    let mut operations = Vec::with_capacity(1 + args.len());
    operations.push(rule_expr_operation(head.as_ref().clone(), &alpha_renames));
    let mut item_rhs_alpha_renames = alpha_renames;
    for item in args.iter() {
        operations.push(RuleOperation::WithBindingItem {
            expr: item.clone(),
            rhs_alpha_renames: item_rhs_alpha_renames.clone(),
            scoped_alpha_renames: scoped_alpha_renames.clone(),
        });
        if let Some((symbol, _)) = parse_with_binding_item(item) {
            if let Some(alpha) = scoped_alpha_renames.get(&symbol) {
                let mut next_alpha_renames = item_rhs_alpha_renames.as_ref().clone();
                next_alpha_renames.insert(symbol, alpha.clone());
                item_rhs_alpha_renames = Arc::new(next_alpha_renames);
            }
        }
    }
    schedule_rule_operations(
        work,
        RuleWork::FinishWithBindingList {
            head: head.clone(),
            args: args.clone(),
        },
        operations,
    );
}

fn visit_rule_with_binding_item(
    expr: Expr,
    rhs_alpha_renames: SharedAlphaRenames,
    scoped_alpha_renames: SharedAlphaRenames,
    work: &mut Vec<RuleWork>,
) {
    match &expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2
                && expr_as_symbol(&args[0]).is_some() =>
        {
            let operations = vec![
                rule_expr_operation(head.as_ref().clone(), &rhs_alpha_renames),
                rule_expr_operation(args[0].clone(), &scoped_alpha_renames),
                rule_expr_operation(args[1].clone(), &rhs_alpha_renames),
            ];
            schedule_rule_operations(
                work,
                RuleWork::FinishWithBindingItem {
                    head: head.clone(),
                    args: args.clone(),
                },
                operations,
            );
        }
        _ => {
            work.push(RuleWork::FinishWithBindingItemPassthrough);
            work.push(RuleWork::Visit(RuleOperation::Expr {
                expr,
                alpha_renames: rhs_alpha_renames,
            }));
        }
    }
}

fn schedule_rule_call<F>(
    work: &mut Vec<RuleWork>,
    finish: RuleWork,
    head: &SharedExpr,
    args: &SharedArgs,
    head_alpha_renames: SharedAlphaRenames,
    mut arg_alpha_renames: F,
) where
    F: FnMut(usize, &SharedAlphaRenames) -> SharedAlphaRenames,
{
    let mut operations = Vec::with_capacity(1 + args.len());
    operations.push(rule_expr_operation(
        head.as_ref().clone(),
        &head_alpha_renames,
    ));
    operations.extend(
        args.iter()
            .enumerate()
            .map(|(index, arg)| RuleOperation::Expr {
                expr: arg.clone(),
                alpha_renames: arg_alpha_renames(index, &head_alpha_renames),
            }),
    );
    schedule_rule_operations(work, finish, operations);
}

fn rule_expr_operation(expr: Expr, alpha_renames: &SharedAlphaRenames) -> RuleOperation {
    RuleOperation::Expr {
        expr,
        alpha_renames: alpha_renames.clone(),
    }
}

fn finish_rule_call(
    completed: &mut Vec<Option<Expr>>,
    head: &SharedExpr,
    args: &SharedArgs,
) -> Option<Expr> {
    let mut children = take_completed(completed, 1 + args.len()).into_iter();
    let substituted_head = children.next().expect("rule call head completes");
    let mut substituted_args = None;

    for (index, substituted_arg) in children.enumerate() {
        let arg = &args[index];
        if let Some(substituted_arg) = substituted_arg {
            let output_args = substituted_args.get_or_insert_with(|| {
                record_rule_rhs_preserved_args(&args[..index]);
                args[..index].to_vec()
            });
            let bound_sequence_items = if matches!(arg, Expr::Atom(Atom::Symbol(_))) {
                sequence_items(&substituted_arg)
            } else {
                None
            };
            if let Some(sequence_items) = bound_sequence_items {
                record_substitution_sharing_counter(
                    &RULE_RHS_SPLICE_FROM_BINDING_COUNT,
                    "rule_rhs_splices_from_binding",
                );
                output_args.extend(sequence_items.iter().cloned());
            } else {
                output_args.push(substituted_arg);
            }
        } else if let Some(substituted_args) = &mut substituted_args {
            record_rule_rhs_preserved_arg(arg);
            substituted_args.push(arg.clone());
        }
    }

    let start = rule_rhs_timing_start();
    let result = rebuild_rule_call(head.as_ref(), args, substituted_head, substituted_args);
    record_profile_rule_rhs_timing("call_args", start, Some(result.is_some()));
    result
}

fn record_iterative_rule_timing(phase: &'static str, changed: bool) {
    let start = rule_rhs_timing_start();
    record_profile_rule_rhs_timing(phase, start, Some(changed));
}

fn schedule_rule_operations(
    work: &mut Vec<RuleWork>,
    finish: RuleWork,
    operations: Vec<RuleOperation>,
) {
    work.push(finish);
    work.extend(operations.into_iter().rev().map(RuleWork::Visit));
}

fn take_completed(completed: &mut Vec<Option<Expr>>, child_count: usize) -> Vec<Option<Expr>> {
    let first_child = completed
        .len()
        .checked_sub(child_count)
        .expect("iterative binding results remain balanced");
    completed.split_off(first_child)
}
