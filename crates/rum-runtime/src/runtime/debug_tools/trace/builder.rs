use super::*;

pub(super) struct StructuredTraceBuilder {
    containers: Vec<Vec<Expr>>,
    current_exprs: Vec<Option<Expr>>,
}

impl StructuredTraceBuilder {
    pub(super) fn new() -> Self {
        Self {
            containers: vec![Vec::new()],
            current_exprs: Vec::new(),
        }
    }

    pub(super) fn push_expr(&mut self, depth: usize, expr: Expr) {
        self.close_to_depth(depth);

        while self.containers.len() <= depth {
            self.containers.push(Vec::new());
        }
        if self.current_exprs.len() <= depth {
            self.current_exprs.resize(depth + 1, None);
        }

        let duplicate = self.current_exprs[depth]
            .as_ref()
            .is_some_and(|current| current == &expr);
        self.current_exprs.truncate(depth + 1);
        if duplicate {
            return;
        }

        self.containers[depth].push(Expr::call(system_expr("Hold"), vec![expr.clone()]));
        self.current_exprs[depth] = Some(expr);
    }

    pub(super) fn push_item(&mut self, depth: usize, item: Expr) {
        self.close_to_depth(depth);

        while self.containers.len() <= depth {
            self.containers.push(Vec::new());
        }

        self.containers[depth].push(item);
        self.current_exprs.truncate(depth + 1);
    }

    pub(super) fn push_evaluation_pass(&mut self, depth: usize, before: Expr, after: Expr) {
        self.close_to_depth(depth);

        if !self.container_root_matches(depth, &before) {
            self.push_expr(depth, before);
        }
        self.push_expr(depth, after);
    }

    pub(super) fn finish(mut self) -> Expr {
        self.close_to_depth(0);
        Expr::call(
            system_expr("List"),
            self.containers.pop().unwrap_or_default(),
        )
    }

    fn close_to_depth(&mut self, depth: usize) {
        while self.containers.len() > depth + 1 {
            let child = self.containers.pop().unwrap_or_default();
            if let Some(parent) = self.containers.last_mut() {
                parent.push(Expr::call(system_expr("List"), child));
            }
        }
    }

    fn container_root_matches(&self, depth: usize, target: &Expr) -> bool {
        self.containers.get(depth).and_then(|items| {
            items.iter().find_map(|item| match item {
                Expr::Call { head, args } if is_system_symbol_name(head, "Hold") => {
                    args.first().cloned()
                }
                _ => None,
            })
        }) == Some(target.clone())
    }
}

pub(super) fn trace_expr_path_depth(active_exprs: &[Expr], target: &Expr) -> Option<usize> {
    active_exprs
        .iter()
        .enumerate()
        .rev()
        .find_map(|(depth, expr)| (expr == target).then_some(depth))
}
