use rum_core::SharedArgs;

use rum_core::{BigInt, Expr, Rational};
use rum_runtime::kernel_api::{
    CancellationObserver, EvalError, Session, compare_expr_canonical, expr_as_exact_rational,
    expr_as_non_negative_usize, expr_as_symbol, is_system_symbol_name, list_items, system_expr,
    truthiness_symbol,
};

use crate::structure_ops::next_permutation;

#[derive(Clone, Copy)]
enum PullBudget {
    AtMost(usize),
    UntilExhausted,
}

impl PullBudget {
    fn reached(self, pulled: usize) -> bool {
        match self {
            Self::AtMost(limit) => pulled >= limit,
            Self::UntilExhausted => false,
        }
    }
}

struct PullContext {
    iteration_limit: usize,
    source_pulls: usize,
    cancellation: CancellationObserver,
}

impl PullContext {
    fn new(session: &Session) -> Self {
        Self {
            iteration_limit: session.resource_limits().iteration_work_limit,
            source_pulls: 0,
            cancellation: session.cancellation_observer(),
        }
    }

    fn charge_source_pull(&mut self) -> Result<(), EvalError> {
        if self.cancellation.is_cancelled() {
            return Err(EvalError::Cancelled);
        }
        if self.source_pulls >= self.iteration_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: self.iteration_limit,
            });
        }

        self.source_pulls += 1;
        Ok(())
    }
}

enum PullStepper {
    Range {
        current: Rational,
        step: Rational,
    },
    Permutations {
        current: Vec<Expr>,
        exhausted: bool,
    },
    Subsets {
        elements: Vec<Expr>,
        k: usize,
        combo: Vec<usize>,
    },
    Tuples {
        inner_lists: Vec<Expr>,
        digits: Vec<usize>,
        exhausted: bool,
    },
    NestList {
        function: Expr,
        current: Expr,
    },
    Map {
        function: Expr,
        inner: Box<PullStepper>,
    },
    Select {
        predicate: Expr,
        inner: Box<PullStepper>,
    },
    Take {
        inner: Box<PullStepper>,
        remaining: usize,
    },
    Drop {
        inner: Box<PullStepper>,
        remaining: usize,
    },
    FoldList {
        function: Expr,
        accumulator: Expr,
        emitted_init: bool,
        inner: Box<PullStepper>,
    },
}

impl PullStepper {
    fn next(
        &mut self,
        session: &mut Session,
        ctx: &mut PullContext,
    ) -> Result<Option<Expr>, EvalError> {
        match self {
            Self::Range { current, step } => {
                ctx.charge_source_pull()?;
                let value = Expr::rational(current.clone());
                *current = current.add(step);
                Ok(Some(value))
            }
            Self::Permutations { current, exhausted } => {
                if *exhausted {
                    return Ok(None);
                }
                ctx.charge_source_pull()?;
                let permutation = Expr::call(system_expr("List"), current.clone());
                *exhausted = !next_permutation(current);
                Ok(Some(permutation))
            }
            Self::Subsets { elements, k, combo } => {
                if *k > elements.len() {
                    return Ok(None);
                }
                ctx.charge_source_pull()?;
                let subset = Expr::call(
                    system_expr("List"),
                    combo.iter().map(|index| elements[*index].clone()).collect(),
                );
                advance_subset_cursor(elements.len(), k, combo);
                Ok(Some(subset))
            }
            Self::Tuples {
                inner_lists,
                digits,
                exhausted,
            } => {
                if *exhausted {
                    return Ok(None);
                }
                ctx.charge_source_pull()?;
                let tuple = Expr::call(
                    system_expr("List"),
                    inner_lists
                        .iter()
                        .zip(digits.iter())
                        .map(|(list, digit)| {
                            list_items(list).expect("tuple factors were validated")[*digit].clone()
                        })
                        .collect(),
                );
                *exhausted = advance_tuple_cursor(inner_lists, digits);
                Ok(Some(tuple))
            }
            Self::NestList { function, current } => {
                ctx.charge_source_pull()?;
                let value = current.clone();
                *current = session.evaluate(Expr::call(function.clone(), vec![value.clone()]))?;
                Ok(Some(value))
            }
            Self::Map { function, inner } => {
                let Some(value) = inner.next(session, ctx)? else {
                    return Ok(None);
                };
                Ok(Some(
                    session.evaluate(Expr::call(function.clone(), vec![value]))?,
                ))
            }
            Self::Select { predicate, inner } => loop {
                let Some(value) = inner.next(session, ctx)? else {
                    return Ok(None);
                };
                let selected =
                    session.evaluate(Expr::call(predicate.clone(), vec![value.clone()]))?;
                if truthiness_symbol(&selected) == Some(true) {
                    return Ok(Some(value));
                }
            },
            Self::Take { inner, remaining } => {
                if *remaining == 0 {
                    return Ok(None);
                }
                *remaining -= 1;
                inner.next(session, ctx)
            }
            Self::Drop { inner, remaining } => {
                while *remaining > 0 {
                    let Some(_) = inner.next(session, ctx)? else {
                        *remaining = 0;
                        return Ok(None);
                    };
                    *remaining -= 1;
                }
                inner.next(session, ctx)
            }
            Self::FoldList {
                function,
                accumulator,
                emitted_init,
                inner,
            } => {
                if !*emitted_init {
                    *emitted_init = true;
                    return Ok(Some(accumulator.clone()));
                }
                let Some(value) = inner.next(session, ctx)? else {
                    return Ok(None);
                };
                *accumulator = session.evaluate(Expr::call(
                    function.clone(),
                    vec![accumulator.clone(), value],
                ))?;
                Ok(Some(accumulator.clone()))
            }
        }
    }
}

pub(crate) fn eval_lazy_drop(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [lazy_list, count_expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyDrop", args, 2));
    };
    let (Some(descriptor), Some(count)) = (
        lazy_list_descriptor(lazy_list),
        expr_as_non_negative_usize(count_expr),
    ) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazyDrop",
            args,
            "expected a LazyList and a non-negative integer",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyDrop"),
        vec![descriptor.clone(), Expr::integer(BigInt::from(count))],
    )))
}

pub(crate) fn eval_lazy_fold_list(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [function, init, lazy_list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyFoldList", args, 3));
    };
    let Some(descriptor) = lazy_list_descriptor(lazy_list) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazyFoldList",
            args,
            "expected a LazyList as the third argument",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyFoldList"),
        vec![function.clone(), init.clone(), descriptor.clone()],
    )))
}

pub(crate) fn eval_lazy_map(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [function, lazy_list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyMap", args, 2));
    };
    let Some(descriptor) = lazy_list_descriptor(lazy_list) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazyMap",
            args,
            "expected a LazyList as the second argument",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyMap"),
        vec![function.clone(), descriptor.clone()],
    )))
}

pub(crate) fn eval_lazy_nest_list(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [function, init] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyNestList", args, 2));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyNestList"),
        vec![function.clone(), init.clone()],
    )))
}

pub(crate) fn eval_lazy_permutations(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyPermutations", args, 1));
    };
    if list_items(list).is_none() {
        return Ok(session.invalid_call_with_arg_form("LazyPermutations", args, "expected a List"));
    }

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyPermutations"),
        vec![list.clone()],
    )))
}

pub(crate) fn eval_lazy_range(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() > 2 {
        return Ok(session.invalid_call_with_arg_range("LazyRange", args, 0, 2));
    }

    let start = args
        .first()
        .cloned()
        .unwrap_or_else(|| Expr::small_integer(1));
    let step = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| Expr::small_integer(1));

    if expr_as_exact_rational(&start).is_none() || expr_as_exact_rational(&step).is_none() {
        return Ok(session.invalid_call_with_arg_form(
            "LazyRange",
            args,
            "start and step must be Integer or Rational values",
        ));
    }

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyRange"),
        vec![start, step],
    )))
}

pub(crate) fn eval_lazy_select(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [predicate, lazy_list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazySelect", args, 2));
    };
    let Some(descriptor) = lazy_list_descriptor(lazy_list) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazySelect",
            args,
            "expected a LazyList as the second argument",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazySelect"),
        vec![predicate.clone(), descriptor.clone()],
    )))
}

pub(crate) fn eval_lazy_subsets(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazySubsets", args, 1));
    };
    if list_items(list).is_none() {
        return Ok(session.invalid_call_with_arg_form("LazySubsets", args, "expected a List"));
    }

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazySubsets"),
        vec![list.clone()],
    )))
}

pub(crate) fn eval_lazy_table(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [body, spec] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyTable", args, 2));
    };
    let Expr::Call {
        head: spec_head,
        args: spec_args,
    } = spec
    else {
        return Ok(invalid_lazy_table_spec(session, args));
    };
    if !is_system_symbol_name(spec_head, "List") {
        return Ok(invalid_lazy_table_spec(session, args));
    }

    let one = || Expr::small_integer(1);
    let (iterator, [imin, imax, step]) = match spec_args.as_slice() {
        [iterator, imax] => (iterator, [one(), imax.clone(), one()]),
        [iterator, imin, imax] => (iterator, [imin.clone(), imax.clone(), one()]),
        [iterator, imin, imax, step] => (iterator, [imin.clone(), imax.clone(), step.clone()]),
        _ => return Ok(invalid_lazy_table_spec(session, args)),
    };
    if expr_as_symbol(iterator).is_none() {
        return Ok(invalid_lazy_table_spec(session, args));
    }

    let imin = session.evaluate(imin)?;
    let imax = session.evaluate(imax)?;
    let step = session.evaluate(step)?;
    let (Some(imin_value), Some(imax_value), Some(step_value)) = (
        expr_as_exact_rational(&imin),
        expr_as_exact_rational(&imax),
        expr_as_exact_rational(&step),
    ) else {
        return Ok(invalid_lazy_table_spec(session, args));
    };
    let Some(count) = lazy_table_count(&imin_value, &imax_value, &step_value) else {
        return Ok(invalid_lazy_table_spec(session, args));
    };

    let function = Expr::call(
        system_expr("Function"),
        vec![iterator.clone(), body.clone()],
    );
    let range = Expr::call(system_expr("LazyRange"), vec![imin, step]);
    let map = Expr::call(system_expr("LazyMap"), vec![function, range]);
    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyTake"),
        vec![map, Expr::integer(BigInt::from(count))],
    )))
}

pub(crate) fn eval_lazy_take(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [lazy_list, count_expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyTake", args, 2));
    };
    let (Some(descriptor), Some(count)) = (
        lazy_list_descriptor(lazy_list),
        parse_lazy_take_count(count_expr),
    ) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazyTake",
            args,
            "expected a LazyList and a non-negative integer or UpTo[n]",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyTake"),
        vec![descriptor.clone(), Expr::integer(BigInt::from(count))],
    )))
}

pub(crate) fn eval_lazy_tuples(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let list_of_lists = match args {
        [list_of_lists]
            if list_items(list_of_lists)
                .is_some_and(|items| items.iter().all(|item| list_items(item).is_some())) =>
        {
            list_of_lists.clone()
        }
        [list, count] => {
            let (Some(_), Some(count)) = (list_items(list), expr_as_non_negative_usize(count))
            else {
                return Ok(session.invalid_call_with_arg_form(
                    "LazyTuples",
                    args,
                    "expected a list of lists, or a list and a non-negative integer",
                ));
            };
            Expr::call(system_expr("List"), vec![list.clone(); count])
        }
        [_] => {
            return Ok(session.invalid_call_with_arg_form(
                "LazyTuples",
                args,
                "expected a list of lists, or a list and a non-negative integer",
            ));
        }
        _ => {
            return Ok(session.invalid_call_with_message(
                "LazyTuples",
                args,
                "argx",
                format!("expected 1 or 2 argument(s), got {}", args.len()),
            ));
        }
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyTuples"),
        vec![list_of_lists],
    )))
}

pub(crate) fn is_lazy_list(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, .. } if is_system_symbol_name(head, "LazyList")
    )
}

pub(crate) fn lazy_drop(lazy_list: &Expr, count: usize) -> Option<Expr> {
    let descriptor = lazy_list_descriptor(lazy_list)?;
    Some(wrap_descriptor(Expr::call(
        system_expr("LazyDrop"),
        vec![descriptor.clone(), Expr::integer(BigInt::from(count))],
    )))
}

pub(crate) fn realize_all(
    session: &mut Session,
    lazy_list: &Expr,
) -> Result<Option<Expr>, EvalError> {
    realize(session, lazy_list, PullBudget::UntilExhausted)
}

pub(crate) fn realize_at_most(
    session: &mut Session,
    lazy_list: &Expr,
    limit: usize,
) -> Result<Option<Expr>, EvalError> {
    realize(session, lazy_list, PullBudget::AtMost(limit))
}

fn realize(
    session: &mut Session,
    lazy_list: &Expr,
    budget: PullBudget,
) -> Result<Option<Expr>, EvalError> {
    let Some(descriptor) = lazy_list_descriptor(lazy_list) else {
        return Ok(None);
    };
    let Some(mut stepper) = stepper_from_descriptor(descriptor) else {
        return Ok(None);
    };
    let mut ctx = PullContext::new(session);
    let mut items = Vec::new();

    loop {
        if budget.reached(items.len()) {
            return Ok(Some(Expr::call(system_expr("List"), items)));
        }
        let Some(item) = stepper.next(session, &mut ctx)? else {
            return Ok(Some(Expr::call(system_expr("List"), items)));
        };
        items.push(item);
    }
}

fn lazy_list_descriptor(expr: &Expr) -> Option<&Expr> {
    if !is_lazy_list(expr) {
        return None;
    }
    let Expr::Call { args, .. } = expr else {
        return None;
    };
    let [descriptor] = args.as_slice() else {
        return None;
    };
    Some(descriptor)
}

fn parse_lazy_take_count(expr: &Expr) -> Option<usize> {
    if let Some(count) = expr_as_non_negative_usize(expr) {
        return Some(count);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "UpTo") {
        return None;
    }
    let [count] = args.as_slice() else {
        return None;
    };
    expr_as_non_negative_usize(count)
}

fn invalid_lazy_table_spec(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_arg_form(
        "LazyTable",
        args,
        "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step",
    )
}

fn lazy_table_count(imin: &Rational, imax: &Rational, step: &Rational) -> Option<usize> {
    if step.is_zero() {
        return None;
    }

    let zero = BigInt::from(0_u8);
    let (span, step_magnitude) = if step.numerator() > &zero {
        (imax.sub(imin), step.clone())
    } else {
        (imin.sub(imax), step.negated())
    };
    if span.numerator() < &zero {
        return Some(0);
    }

    let quotient = span.div(&step_magnitude)?;
    let count = quotient.numerator() / quotient.denominator() + BigInt::from(1_u8);
    expr_as_non_negative_usize(&Expr::integer(count))
}

fn advance_subset_cursor(item_count: usize, k: &mut usize, combo: &mut Vec<usize>) {
    if *k == item_count {
        *k += 1;
        combo.clear();
        return;
    }

    if let Some(index) = (0..combo.len())
        .rev()
        .find(|index| combo[*index] < item_count - *k + *index)
    {
        combo[index] += 1;
        for next in index + 1..combo.len() {
            combo[next] = combo[next - 1] + 1;
        }
    } else {
        *k += 1;
        *combo = (0..*k).collect();
    }
}

fn advance_tuple_cursor(inner_lists: &[Expr], digits: &mut [usize]) -> bool {
    for index in (0..digits.len()).rev() {
        digits[index] += 1;
        if digits[index]
            < list_items(&inner_lists[index])
                .expect("tuple factors were validated")
                .len()
        {
            return false;
        }
        digits[index] = 0;
    }
    true
}

fn stepper_from_descriptor(descriptor: &Expr) -> Option<PullStepper> {
    let Expr::Call { head, args } = descriptor else {
        return None;
    };

    if is_system_symbol_name(head, "LazyRange") {
        let [start, step] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::Range {
            current: expr_as_exact_rational(start)?,
            step: expr_as_exact_rational(step)?,
        });
    }
    if is_system_symbol_name(head, "LazyPermutations") {
        let [list] = args.as_slice() else {
            return None;
        };
        let mut current = list_items(list)?.to_vec();
        current.sort_by(compare_expr_canonical);
        return Some(PullStepper::Permutations {
            current,
            exhausted: false,
        });
    }
    if is_system_symbol_name(head, "LazySubsets") {
        let [list] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::Subsets {
            elements: list_items(list)?.to_vec(),
            k: 0,
            combo: Vec::new(),
        });
    }
    if is_system_symbol_name(head, "LazyTuples") {
        let [list_of_lists] = args.as_slice() else {
            return None;
        };
        let inner_lists = list_items(list_of_lists)?;
        if !inner_lists.iter().all(|list| list_items(list).is_some()) {
            return None;
        }
        let exhausted = inner_lists
            .iter()
            .any(|list| list_items(list).is_some_and(<[Expr]>::is_empty));
        return Some(PullStepper::Tuples {
            digits: vec![0; inner_lists.len()],
            inner_lists: inner_lists.to_vec(),
            exhausted,
        });
    }
    if is_system_symbol_name(head, "LazyNestList") {
        let [function, init] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::NestList {
            function: function.clone(),
            current: init.clone(),
        });
    }
    if is_system_symbol_name(head, "LazyMap") {
        let [function, inner] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::Map {
            function: function.clone(),
            inner: Box::new(stepper_from_descriptor(inner)?),
        });
    }
    if is_system_symbol_name(head, "LazySelect") {
        let [predicate, inner] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::Select {
            predicate: predicate.clone(),
            inner: Box::new(stepper_from_descriptor(inner)?),
        });
    }
    if is_system_symbol_name(head, "LazyTake") {
        let [inner, count] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::Take {
            inner: Box::new(stepper_from_descriptor(inner)?),
            remaining: expr_as_non_negative_usize(count)?,
        });
    }
    if is_system_symbol_name(head, "LazyDrop") {
        let [inner, count] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::Drop {
            inner: Box::new(stepper_from_descriptor(inner)?),
            remaining: expr_as_non_negative_usize(count)?,
        });
    }
    if is_system_symbol_name(head, "LazyFoldList") {
        let [function, init, inner] = args.as_slice() else {
            return None;
        };
        return Some(PullStepper::FoldList {
            function: function.clone(),
            accumulator: init.clone(),
            emitted_init: false,
            inner: Box::new(stepper_from_descriptor(inner)?),
        });
    }

    None
}

fn wrap_descriptor(descriptor: Expr) -> Expr {
    Expr::call(system_expr("LazyList"), vec![descriptor])
}
