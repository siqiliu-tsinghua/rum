use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) struct OrderlessCandidateChoice {
    pub(super) selected_values: Vec<Expr>,
    pub(super) rest_values: Vec<Expr>,
}

impl OrderlessCandidateChoice {
    pub(super) fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationOrderlessCandidateChoice"),
            vec![
                self.selected_values.into_continuation_expr(),
                self.rest_values.into_continuation_expr(),
            ],
        )
    }

    pub(super) fn from_continuation_expr(expr: Expr) -> Self {
        let Expr::Call { head, args, .. } = expr else {
            panic!("unexpected orderless-candidate continuation payload")
        };
        assert!(
            is_system_symbol_name(&head, "InternalContinuationOrderlessCandidateChoice"),
            "unexpected orderless-candidate continuation tag: {head:?}"
        );
        let mut args = args.into_iter();
        let selected_values = Vec::<Expr>::from_continuation_expr(
            args.next()
                .expect("orderless candidate stores selected values"),
        );
        let rest_values = Vec::<Expr>::from_continuation_expr(
            args.next().expect("orderless candidate stores rest values"),
        );
        debug_assert!(args.next().is_none());
        Self {
            selected_values,
            rest_values,
        }
    }
}

pub(super) fn orderless_candidate_choices(
    pattern: &MatcherArg,
    rest_patterns: &[MatcherArg],
    value_args: &[Expr],
    allow_flat_single: bool,
    wrap_singleton: Option<&Expr>,
) -> Vec<OrderlessCandidateChoice> {
    match pattern {
        MatcherArg::Single(_) if allow_flat_single => {
            let min_rest = min_required_matcher_arg_count(rest_patterns);
            let Some(max_len) = value_args.len().checked_sub(min_rest) else {
                return Vec::new();
            };
            if max_len == 0 {
                return Vec::new();
            }

            let choices = orderless_partition_choices_by_lengths(value_args, 1..=max_len, false);
            let Some(head) = wrap_singleton else {
                return choices;
            };

            let mut wrapped_choices = Vec::with_capacity(choices.len() + value_args.len());
            for choice in choices {
                let wrapped_choice = match choice.selected_values.as_slice() {
                    [value] => Some(OrderlessCandidateChoice {
                        selected_values: vec![Expr::call(head.clone(), vec![value.clone()])],
                        rest_values: choice.rest_values.clone(),
                    }),
                    _ => None,
                };
                wrapped_choices.push(choice);
                if let Some(wrapped_choice) = wrapped_choice {
                    wrapped_choices.push(wrapped_choice);
                }
            }
            wrapped_choices
        }
        MatcherArg::Single(_) => select_orderless_value_partitions(value_args, 1),
        MatcherArg::Sequence(spec) => {
            let Some(max_len) = max_sequence_consume(
                spec,
                min_required_matcher_arg_count(rest_patterns),
                value_args.len(),
            ) else {
                return Vec::new();
            };

            orderless_partition_choices_by_lengths(
                value_args,
                sequence_candidate_lengths(spec.min_len, max_len, spec.greediness),
                false,
            )
        }
        MatcherArg::Optional(_) => Vec::new(),
        MatcherArg::PatternSequenceGroup(items) => {
            let min_len = min_required_matcher_arg_count(items);
            let max_len = max_possible_matcher_arg_count(items)
                .map(|max_len| max_len.min(value_args.len()))
                .unwrap_or(value_args.len());

            if min_len > max_len {
                return Vec::new();
            }

            orderless_partition_choices_by_lengths(value_args, min_len..=max_len, true)
        }
        MatcherArg::OrderlessPatternSequenceGroup(items) => {
            let min_len = min_required_matcher_arg_count(items);
            let max_len = max_possible_matcher_arg_count(items)
                .map(|max_len| max_len.min(value_args.len()))
                .unwrap_or(value_args.len());

            if min_len > max_len {
                return Vec::new();
            }

            orderless_partition_choices_by_lengths(value_args, min_len..=max_len, false)
        }
    }
}

fn orderless_partition_choices_by_lengths<I>(
    value_args: &[Expr],
    lengths: I,
    reverse_within_len: bool,
) -> Vec<OrderlessCandidateChoice>
where
    I: IntoIterator<Item = usize>,
{
    lengths
        .into_iter()
        .flat_map(|count| {
            let mut choices = select_orderless_value_partitions(value_args, count);
            if reverse_within_len {
                choices.reverse();
            }
            choices
        })
        .collect()
}
fn select_orderless_value_partitions(
    value_args: &[Expr],
    choose: usize,
) -> Vec<OrderlessCandidateChoice> {
    if choose > value_args.len() {
        return Vec::new();
    }

    let mut indices = Vec::new();
    let mut partitions = Vec::new();
    collect_orderless_partitions(value_args, choose, 0, &mut indices, &mut partitions);
    partitions
}

fn collect_orderless_partitions(
    value_args: &[Expr],
    choose: usize,
    start: usize,
    indices: &mut Vec<usize>,
    partitions: &mut Vec<OrderlessCandidateChoice>,
) {
    if indices.len() == choose {
        partitions.push(build_orderless_partition(value_args, indices));
        return;
    }

    let remaining = choose - indices.len();
    for index in start..=value_args.len() - remaining {
        indices.push(index);
        collect_orderless_partitions(value_args, choose, index + 1, indices, partitions);
        indices.pop();
    }
}

fn build_orderless_partition(value_args: &[Expr], indices: &[usize]) -> OrderlessCandidateChoice {
    let mut selected_values = Vec::with_capacity(indices.len());
    let mut rest_values = Vec::with_capacity(value_args.len().saturating_sub(indices.len()));
    let mut next_index = 0usize;

    for (index, value) in value_args.iter().enumerate() {
        if indices.get(next_index).copied() == Some(index) {
            selected_values.push(value.clone());
            next_index += 1;
        } else {
            rest_values.push(value.clone());
        }
    }

    OrderlessCandidateChoice {
        selected_values,
        rest_values,
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<OrderlessCandidateChoice>();
};
