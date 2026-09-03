use super::super::*;

pub(in crate::runtime) fn substitute_slots(expr: &Expr, args: &[Expr]) -> Expr {
    if let Some(substituted) = substitute_single_slot(expr, args) {
        return substituted;
    }

    match expr {
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call {
            head,
            args: call_args,
            ..
        } => {
            let substituted_head = substitute_slots(head, args);
            let mut substituted_args = Vec::new();

            for arg in call_args {
                let substituted_arg = substitute_slots(arg, args);

                if is_slot_sequence_expr(arg) {
                    if let Some(items) = sequence_items(&substituted_arg) {
                        substituted_args.extend(items.iter().cloned());
                    } else {
                        substituted_args.push(substituted_arg);
                    }
                } else {
                    substituted_args.push(substituted_arg);
                }
            }

            Expr::call(substituted_head, substituted_args)
        }
    }
}

pub(in crate::runtime) fn substitute_single_slot(expr: &Expr, args: &[Expr]) -> Option<Expr> {
    let Expr::Call {
        head,
        args: slot_args,
        ..
    } = expr
    else {
        return None;
    };

    if slot_args.len() != 1 {
        return None;
    }

    match (expr_as_symbol(head)?.name(), &slot_args[0]) {
        ("Slot", Expr::Atom(Atom::Integer(index))) => {
            let index = index.to_string().parse::<usize>().ok()?;
            if index == 0 {
                None
            } else {
                args.get(index - 1).cloned()
            }
        }
        ("Slot", Expr::Atom(Atom::String(name))) => lookup_named_slot(args, name),
        ("SlotSequence", Expr::Atom(Atom::Integer(index))) => {
            let index = index.to_string().parse::<usize>().ok()?;
            if index == 0 {
                None
            } else {
                Some(sequence_expr(
                    args.iter().skip(index - 1).cloned().collect(),
                ))
            }
        }
        _ => None,
    }
}

pub(in crate::runtime) fn lookup_named_slot(args: &[Expr], name: &str) -> Option<Expr> {
    let mut saw_association = false;

    for arg in args {
        let Expr::Call {
            head,
            args: call_args,
            ..
        } = arg
        else {
            continue;
        };

        if is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed") {
            if let Some(value) = lookup_named_slot_in_rule(head, call_args, name) {
                return Some(value);
            }
            continue;
        }

        if !is_system_symbol_name(head, "Association") {
            continue;
        }

        saw_association = true;

        for entry in call_args {
            let Expr::Call {
                head: rule_head,
                args: rule_args,
                ..
            } = entry
            else {
                continue;
            };

            if let Some(value) = lookup_named_slot_in_rule(rule_head, rule_args, name) {
                return Some(value);
            }
        }
    }

    saw_association.then(|| missing_key_absent_expr(Expr::string(name)))
}

fn lookup_named_slot_in_rule(head: &Expr, rule_args: &[Expr], name: &str) -> Option<Expr> {
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || rule_args.len() != 2
    {
        return None;
    }

    let matches_name = match &rule_args[0] {
        Expr::Atom(Atom::Symbol(symbol)) => symbol.name() == name,
        Expr::Atom(Atom::String(value)) => value == name,
        _ => false,
    };

    matches_name.then(|| rule_args[1].clone())
}
