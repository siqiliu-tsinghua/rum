use super::*;

const INLINE_REPLACE_DEPTH_FRAMES: usize = 16;

enum ReplaceDepthChildren {
    Standard,
    Association(Vec<Expr>),
}

struct ReplaceDepthFrame {
    expr: Expr,
    children: ReplaceDepthChildren,
    next_child: usize,
}

impl ReplaceDepthFrame {
    fn new(expr: Expr) -> Self {
        let children = if association_entries_from_expr(&expr).is_some() {
            ReplaceDepthChildren::Association(
                association_entries_from_expr(&expr)
                    .expect("association shape checked above")
                    .into_iter()
                    .map(|(_, value)| value)
                    .collect(),
            )
        } else {
            ReplaceDepthChildren::Standard
        };
        Self {
            expr,
            children,
            next_child: 0,
        }
    }

    fn next_child(&mut self, include_heads: bool) -> Option<Expr> {
        let head_offset = usize::from(include_heads);
        if include_heads && self.next_child == 0 {
            self.next_child += 1;
            let Expr::Call { head, .. } = &self.expr else {
                unreachable!("only calls have depth frames")
            };
            return Some(head.as_ref().clone());
        }

        let child_index = self.next_child.checked_sub(head_offset)?;
        let child = match &self.children {
            ReplaceDepthChildren::Standard => {
                let Expr::Call { args, .. } = &self.expr else {
                    unreachable!("only calls have depth frames")
                };
                args.get(child_index).cloned()
            }
            ReplaceDepthChildren::Association(values) => values.get(child_index).cloned(),
        };
        if child.is_some() {
            self.next_child += 1;
        }
        child
    }
}

struct ReplaceDepthStack {
    inline: [Option<ReplaceDepthFrame>; INLINE_REPLACE_DEPTH_FRAMES],
    inline_len: usize,
    overflow: Vec<ReplaceDepthFrame>,
}

impl ReplaceDepthStack {
    fn new(root: ReplaceDepthFrame) -> Self {
        let mut inline = std::array::from_fn(|_| None);
        inline[0] = Some(root);
        Self {
            inline,
            inline_len: 1,
            overflow: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.inline_len + self.overflow.len()
    }

    fn last_mut(&mut self) -> Option<&mut ReplaceDepthFrame> {
        if let Some(frame) = self.overflow.last_mut() {
            return Some(frame);
        }
        self.inline
            .get_mut(self.inline_len.checked_sub(1)?)?
            .as_mut()
    }

    fn push(&mut self, frame: ReplaceDepthFrame) {
        if self.inline_len < INLINE_REPLACE_DEPTH_FRAMES && self.overflow.is_empty() {
            self.inline[self.inline_len] = Some(frame);
            self.inline_len += 1;
        } else {
            self.overflow.push(frame);
        }
    }

    fn pop(&mut self) {
        if self.overflow.pop().is_some() {
            return;
        }
        self.inline_len = self
            .inline_len
            .checked_sub(1)
            .expect("depth stack pop remains balanced");
        self.inline[self.inline_len] = None;
    }
}

pub(super) fn replace_expr_depth_for_heads(expr: &Expr, include_heads: bool) -> usize {
    replace_expr_depth_iterative(expr, include_heads)
}

pub(super) fn replace_expr_depth(expr: &Expr) -> usize {
    replace_expr_depth_iterative(expr, false)
}

pub(super) fn replace_expr_depth_with_heads(expr: &Expr) -> usize {
    replace_expr_depth_iterative(expr, true)
}

fn replace_expr_depth_iterative(expr: &Expr, include_heads: bool) -> usize {
    if matches!(expr, Expr::Atom(_)) {
        return 1;
    }

    let mut current = expr;
    let mut prefix_depth = 0usize;
    loop {
        let Expr::Call { head, args } = current else {
            return prefix_depth.saturating_add(1);
        };
        if is_system_symbol_name(head, "Association") {
            return prefix_depth
                .saturating_add(replace_expr_depth_branching(current, include_heads));
        }

        let mut only_call_child = if include_heads && matches!(head.as_ref(), Expr::Call { .. }) {
            Some(head.as_ref())
        } else {
            None
        };
        for arg in args.iter() {
            if !matches!(arg, Expr::Call { .. }) {
                continue;
            }
            if only_call_child.is_some() {
                return prefix_depth
                    .saturating_add(replace_expr_depth_branching(current, include_heads));
            }
            only_call_child = Some(arg);
        }

        let Some(child) = only_call_child else {
            return prefix_depth.saturating_add(2);
        };
        prefix_depth = prefix_depth.saturating_add(1);
        current = child;
    }
}

fn replace_expr_depth_branching(expr: &Expr, include_heads: bool) -> usize {
    let mut stack = ReplaceDepthStack::new(ReplaceDepthFrame::new(expr.clone()));
    let mut max_depth = 2usize;
    while !stack.overflow.is_empty() || stack.inline_len != 0 {
        let child_depth = stack.len().saturating_add(1);
        let child = stack
            .last_mut()
            .expect("non-empty depth stack has a current frame")
            .next_child(include_heads);
        let Some(child) = child else {
            stack.pop();
            continue;
        };

        match child {
            Expr::Atom(_) => max_depth = max_depth.max(child_depth),
            child @ Expr::Call { .. } => {
                max_depth = max_depth.max(child_depth.saturating_add(1));
                stack.push(ReplaceDepthFrame::new(child));
            }
        }
    }
    max_depth
}
