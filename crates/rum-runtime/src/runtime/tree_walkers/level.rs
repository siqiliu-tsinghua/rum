mod options;
use super::*;

impl Session {
    pub(in crate::runtime) fn eval_level(
        &mut self,
        _head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Level", args, 2, 4));
        }

        self.system_option_bool(
            "Level",
            "Heads",
            false,
            options::LevelOptionWorking { args },
        )
    }
}

pub fn collect_level_parts_owned(
    expr: &Expr,
    traversal: CasesTraversalConfig,
    level: usize,
) -> Vec<Expr> {
    if let Some(collection) = collect_level_parts_recursive_bounded(expr, traversal, level, 16) {
        return collection.parts;
    }

    let mut walker = PostOrderTraversal::new(expr.clone(), traversal.include_heads, level);
    let mut parts = Vec::new();
    while let Some(node) = walker.next() {
        if traversal.levels.matches(node.level, node.depth) {
            parts.push(node.expr);
        }
    }
    parts
}

struct LevelCollection {
    parts: Vec<Expr>,
    depth: usize,
}

fn collect_level_parts_recursive_bounded(
    expr: &Expr,
    traversal: CasesTraversalConfig,
    level: usize,
    remaining_call_depth: usize,
) -> Option<LevelCollection> {
    match expr {
        Expr::Atom(_) => {
            let mut parts = Vec::new();
            if traversal.levels.matches(level, 1) {
                parts.push(expr.clone());
            }
            Some(LevelCollection { parts, depth: 1 })
        }
        Expr::Call { .. } if remaining_call_depth == 0 => None,
        Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
            let Some(entries) = association_entries_from_expr(expr) else {
                return collect_standard_call_recursive_bounded(
                    expr,
                    traversal,
                    level,
                    remaining_call_depth,
                );
            };
            let mut parts = Vec::new();
            let mut max_child_depth = 1usize;

            if traversal.include_heads {
                let head_result = collect_level_parts_recursive_bounded(
                    head,
                    traversal,
                    level + 1,
                    remaining_call_depth - 1,
                )?;
                parts.extend(head_result.parts);
            }

            for (_, value) in entries {
                let value_result = collect_level_parts_recursive_bounded(
                    &value,
                    traversal,
                    level + 1,
                    remaining_call_depth - 1,
                )?;
                max_child_depth = max_child_depth.max(value_result.depth);
                parts.extend(value_result.parts);
            }

            let depth = max_child_depth.saturating_add(1);
            if traversal.levels.matches(level, depth) {
                parts.push(expr.clone());
            }
            Some(LevelCollection { parts, depth })
        }
        Expr::Call { .. } => {
            collect_standard_call_recursive_bounded(expr, traversal, level, remaining_call_depth)
        }
    }
}

fn collect_standard_call_recursive_bounded(
    expr: &Expr,
    traversal: CasesTraversalConfig,
    level: usize,
    remaining_call_depth: usize,
) -> Option<LevelCollection> {
    let Expr::Call { head, args } = expr else {
        unreachable!("standard call collection requires a call")
    };
    let mut parts = Vec::new();
    let mut max_child_depth = 1usize;

    if traversal.include_heads {
        let head_result = collect_level_parts_recursive_bounded(
            head,
            traversal,
            level + 1,
            remaining_call_depth - 1,
        )?;
        parts.extend(head_result.parts);
    }

    for arg in args {
        let arg_result = collect_level_parts_recursive_bounded(
            arg,
            traversal,
            level + 1,
            remaining_call_depth - 1,
        )?;
        max_child_depth = max_child_depth.max(arg_result.depth);
        parts.extend(arg_result.parts);
    }

    let depth = max_child_depth.saturating_add(1);
    if traversal.levels.matches(level, depth) {
        parts.push(expr.clone());
    }
    Some(LevelCollection { parts, depth })
}

fn parse_level_level_spec(expr: &Expr) -> Option<CasesLevelSpec> {
    if expr_as_signed_isize(expr) == Some(0) {
        return Some(CasesLevelSpec::range(1, Some(0)));
    }

    parse_standard_level_spec(expr)
}
