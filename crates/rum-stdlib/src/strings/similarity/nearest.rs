use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NearestOptions {
    pub(super) distance_function: NearestDistanceFunction,
}

pub(super) fn parse_nearest_options(options: &[Expr]) -> Option<NearestOptions> {
    let mut parsed = NearestOptions {
        distance_function: NearestDistanceFunction::Automatic,
    };

    for option in options {
        let option_items = if let Some(items) = call_items_with_head(option, &system_expr("List")) {
            items.to_vec()
        } else {
            vec![option.clone()]
        };

        for item in option_items {
            let Expr::Call { head, args } = item else {
                return None;
            };
            if !is_system_symbol_name(&head, "Rule") || args.len() != 2 {
                return None;
            }

            if !is_system_symbol_name(&args[0], "DistanceFunction") {
                return None;
            }

            parsed.distance_function = parse_nearest_distance_function(&args[1])?;
        }
    }

    Some(parsed)
}

fn parse_nearest_distance_function(expr: &Expr) -> Option<NearestDistanceFunction> {
    if is_system_symbol_name(expr, "Automatic") {
        return Some(NearestDistanceFunction::Automatic);
    }
    if is_system_symbol_name(expr, "EditDistance") {
        return Some(NearestDistanceFunction::EditDistance);
    }
    if is_system_symbol_name(expr, "HammingDistance") {
        return Some(NearestDistanceFunction::HammingDistance);
    }
    None
}
pub(super) fn parse_string_rule_dataset(expr: &Expr) -> Option<(Vec<String>, Vec<Expr>)> {
    let (keys, values) = parse_nearest_rule_pairs(expr)?;
    Some((
        keys.iter()
            .map(expr_as_string)
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .map(str::to_string)
            .collect(),
        values,
    ))
}

pub(super) fn parse_numeric_dataset(expr: &Expr) -> Option<(Vec<Rational>, Vec<Expr>)> {
    if let Some(items) = call_items_with_head(expr, &system_expr("List")) {
        let keys = items
            .iter()
            .map(expr_as_exact_rational)
            .collect::<Option<Vec<_>>>()?;
        return Some((keys, items.to_vec()));
    }

    let (keys, values) = parse_nearest_rule_pairs(expr)?;
    Some((
        keys.iter()
            .map(expr_as_exact_rational)
            .collect::<Option<Vec<_>>>()?,
        values,
    ))
}

pub(super) fn parse_nearest_rule_pairs(expr: &Expr) -> Option<(Vec<Expr>, Vec<Expr>)> {
    if let Some(entries) = association_entries_from_expr(expr) {
        let (keys, values): (Vec<_>, Vec<_>) = entries.into_iter().unzip();
        return Some((keys, values));
    }

    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Rule") && args.len() == 2 => {
            if let (Some(keys), Some(values)) = (
                call_items_with_head(&args[0], &system_expr("List")),
                call_items_with_head(&args[1], &system_expr("List")),
            ) {
                return (keys.len() == values.len()).then_some((keys.to_vec(), values.to_vec()));
            }

            Some((vec![args[0].clone()], vec![args[1].clone()]))
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut keys = Vec::with_capacity(args.len());
            let mut values = Vec::with_capacity(args.len());
            for item in args {
                let Expr::Call {
                    head: rule_head,
                    args: rule_args,
                } = item
                else {
                    return None;
                };
                if !is_system_symbol_name(rule_head, "Rule") || rule_args.len() != 2 {
                    return None;
                }
                keys.push(rule_args[0].clone());
                values.push(rule_args[1].clone());
            }
            Some((keys, values))
        }
        _ => None,
    }
}

pub(super) fn parse_nearest_result_spec(expr: &Expr) -> Option<NearestResultSpec> {
    if is_system_symbol_name(expr, "Automatic") {
        return Some(NearestResultSpec::BestTies);
    }
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(NearestResultSpec::All);
    }

    expr_as_non_negative_usize(expr).map(NearestResultSpec::Count)
}

pub(super) fn parse_nearest_selection_spec(expr: &Expr) -> Option<NearestSelectionSpec> {
    if let Some(spec) = parse_nearest_result_spec(expr) {
        return Some(NearestSelectionSpec::Results(spec));
    }

    let items = call_items_with_head(expr, &system_expr("List"))?;
    if items.len() != 2 {
        return None;
    }

    let results = parse_nearest_result_spec(&items[0])?;
    Some(NearestSelectionSpec::ResultsWithin {
        results,
        max_distance: items[1].clone(),
    })
}

pub(super) fn nearest_strings(
    values: &[String],
    target: &str,
    spec: NearestSelectionSpec,
    distance_function: NearestDistanceFunction,
) -> Option<Vec<String>> {
    let distances = string_distances(values, target, distance_function)?;
    let filtered = filter_usize_distances(distances, &spec)?;
    Some(
        select_nearest_indices_by_usize(&filtered, nearest_result_spec(&spec))
            .into_iter()
            .map(|index| values[filtered[index].0].clone())
            .collect(),
    )
}

fn string_distances(
    values: &[String],
    target: &str,
    distance_function: NearestDistanceFunction,
) -> Option<Vec<usize>> {
    let target_chars = target.chars().collect::<Vec<_>>();
    values
        .iter()
        .map(|value| {
            let value_chars = value.chars().collect::<Vec<_>>();
            match distance_function {
                NearestDistanceFunction::Automatic | NearestDistanceFunction::EditDistance => {
                    Some(levenshtein_distance(&value_chars, &target_chars))
                }
                NearestDistanceFunction::HammingDistance => {
                    hamming_distance(&value_chars, &target_chars)
                }
            }
        })
        .collect::<Option<Vec<_>>>()
}

pub(super) fn nearest_values_for_strings(
    keys: &[String],
    values: &[Expr],
    target: &str,
    spec: NearestSelectionSpec,
    distance_function: NearestDistanceFunction,
) -> Option<Vec<Expr>> {
    let distances = string_distances(keys, target, distance_function)?;
    let filtered = filter_usize_distances(distances, &spec)?;
    Some(
        select_nearest_indices_by_usize(&filtered, nearest_result_spec(&spec))
            .into_iter()
            .map(|index| values[filtered[index].0].clone())
            .collect(),
    )
}

pub(super) fn nearest_values_for_numbers(
    keys: &[Rational],
    values: &[Expr],
    target: &Rational,
    spec: NearestSelectionSpec,
) -> Vec<Expr> {
    let distances = keys
        .iter()
        .map(|value| rational_abs_difference(value, target))
        .collect::<Vec<_>>();
    let Some(filtered) = filter_rational_distances(distances, &spec) else {
        return Vec::new();
    };
    select_nearest_indices_by_rational(&filtered, nearest_result_spec(&spec))
        .into_iter()
        .map(|index| values[filtered[index].0].clone())
        .collect()
}

fn nearest_result_spec(spec: &NearestSelectionSpec) -> NearestResultSpec {
    match spec {
        NearestSelectionSpec::Results(results)
        | NearestSelectionSpec::ResultsWithin { results, .. } => *results,
    }
}

fn filter_usize_distances(
    distances: Vec<usize>,
    spec: &NearestSelectionSpec,
) -> Option<Vec<(usize, usize)>> {
    let max_distance = match spec {
        NearestSelectionSpec::Results(_) => None,
        NearestSelectionSpec::ResultsWithin { max_distance, .. } => {
            if is_system_symbol_name(max_distance, "Infinity") {
                None
            } else {
                Some(expr_as_non_negative_usize(max_distance)?)
            }
        }
    };

    Some(
        distances
            .into_iter()
            .enumerate()
            .filter(|(_, distance)| max_distance.is_none_or(|bound| *distance <= bound))
            .collect(),
    )
}

fn filter_rational_distances(
    distances: Vec<Rational>,
    spec: &NearestSelectionSpec,
) -> Option<Vec<(usize, Rational)>> {
    let max_distance = match spec {
        NearestSelectionSpec::Results(_) => None,
        NearestSelectionSpec::ResultsWithin { max_distance, .. } => {
            if is_system_symbol_name(max_distance, "Infinity") {
                None
            } else {
                Some(expr_as_exact_rational(max_distance)?)
            }
        }
    };

    Some(
        distances
            .into_iter()
            .enumerate()
            .filter(|(_, distance)| {
                max_distance.as_ref().is_none_or(|bound| {
                    compare_rationals_local(distance, bound) != std::cmp::Ordering::Greater
                })
            })
            .collect(),
    )
}

pub(super) fn rational_abs_difference(lhs: &Rational, rhs: &Rational) -> Rational {
    let difference = lhs.sub(rhs);
    if compare_rationals_local(&difference, &Rational::zero()) == std::cmp::Ordering::Less {
        difference.negated()
    } else {
        difference
    }
}

pub(super) fn compare_rationals_local(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    // Rational normalizes denominators to positive values, so equal
    // denominators can be ordered without allocating cross-products.
    if lhs.denominator() == rhs.denominator() {
        return lhs.numerator().cmp(rhs.numerator());
    }

    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}
