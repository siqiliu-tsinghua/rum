//! Borrowed, iterative validation of the entire inert descriptor.
use super::state::{PullMachine, PullSource, PullStage};
use rum_core::Expr;
use rum_runtime::kernel_api::{
    compare_expr_canonical, expr_as_exact_rational, expr_as_non_negative_usize, expr_as_symbol,
    list_items,
};

pub(super) fn machine_from_descriptor(mut descriptor: &Expr) -> Option<PullMachine> {
    let mut stages = Vec::new();
    let source = loop {
        let Expr::Call { head, args } = descriptor else {
            return None;
        };
        let symbol = expr_as_symbol(head)?;
        if !symbol.is_system() {
            return None;
        }
        match symbol.name() {
            "LazyRange" => {
                let [start, step] = args.as_slice() else {
                    return None;
                };
                break PullSource::Range {
                    current: expr_as_exact_rational(start)?,
                    step: expr_as_exact_rational(step)?,
                };
            }
            "LazyPermutations" => {
                let [list] = args.as_slice() else {
                    return None;
                };
                let mut current = list_items(list)?.to_vec();
                current.sort_by(compare_expr_canonical);
                break PullSource::Permutations {
                    current,
                    exhausted: false,
                };
            }
            "LazySubsets" => {
                let [list] = args.as_slice() else {
                    return None;
                };
                break PullSource::Subsets {
                    elements: list_items(list)?.to_vec(),
                    k: 0,
                    combo: Vec::new(),
                };
            }
            "LazyTuples" => {
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
                break PullSource::Tuples {
                    digits: vec![0; inner_lists.len()],
                    inner_lists: inner_lists.to_vec(),
                    exhausted,
                };
            }
            "LazyNestList" => {
                let [function, init] = args.as_slice() else {
                    return None;
                };
                break PullSource::NestList {
                    function: function.clone(),
                    current: init.clone(),
                };
            }
            "LazyMap" => {
                let [function, inner] = args.as_slice() else {
                    return None;
                };
                push_stage(
                    &mut stages,
                    descriptor,
                    PullStage::Map {
                        function: function.clone(),
                    },
                );
                descriptor = inner;
                continue;
            }
            "LazySelect" => {
                let [predicate, inner] = args.as_slice() else {
                    return None;
                };
                push_stage(
                    &mut stages,
                    descriptor,
                    PullStage::Select {
                        predicate: predicate.clone(),
                    },
                );
                descriptor = inner;
                continue;
            }
            "LazyTake" => {
                let [inner, count] = args.as_slice() else {
                    return None;
                };
                push_stage(
                    &mut stages,
                    descriptor,
                    PullStage::Take {
                        remaining: expr_as_non_negative_usize(count)?,
                    },
                );
                descriptor = inner;
                continue;
            }
            "LazyDrop" => {
                let [inner, count] = args.as_slice() else {
                    return None;
                };
                push_stage(
                    &mut stages,
                    descriptor,
                    PullStage::Drop {
                        remaining: expr_as_non_negative_usize(count)?,
                    },
                );
                descriptor = inner;
                continue;
            }
            "LazyFoldList" => {
                let [function, init, inner] = args.as_slice() else {
                    return None;
                };
                push_stage(
                    &mut stages,
                    descriptor,
                    PullStage::FoldList {
                        function: function.clone(),
                        accumulator: init.clone(),
                        emitted_init: false,
                    },
                );
                descriptor = inner;
                continue;
            }
            _ => return None,
        }
    };
    Some(PullMachine { source, stages })
}

// Keep source-only construction allocation-free and use one slot for a single
// adapter. Only when that slot fills do we size the remaining inert tail. This
// avoids repeated geometric allocation bytes without an unconditional prepass.
fn push_stage(stages: &mut Vec<PullStage>, descriptor: &Expr, stage: PullStage) {
    if stages.len() == stages.capacity() {
        let additional = if stages.is_empty() {
            1
        } else {
            remaining_stages(descriptor)
        };
        stages.reserve_exact(additional);
    }
    stages.push(stage);
}

fn remaining_stages(mut descriptor: &Expr) -> usize {
    let mut count = 0;
    loop {
        let Expr::Call { head, args } = descriptor else {
            break;
        };
        let Some(symbol) = expr_as_symbol(head) else {
            break;
        };
        if !symbol.is_system() {
            break;
        }
        let child = match (symbol.name(), args.as_slice()) {
            ("LazyMap" | "LazySelect", [_, child])
            | ("LazyTake" | "LazyDrop", [child, _])
            | ("LazyFoldList", [_, _, child]) => child,
            _ => break,
        };
        count += 1;
        descriptor = child;
    }
    count
}
