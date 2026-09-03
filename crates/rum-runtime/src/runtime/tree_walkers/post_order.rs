use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
enum PostOrderWork {
    Visit {
        expr: Expr,
        level: usize,
    },
    Finish {
        expr: Expr,
        level: usize,
        child_count: usize,
        skip_first_child_depth: bool,
    },
}

#[derive(Clone, FrameRootTrace)]
pub(in crate::runtime) struct PostOrderTraversal {
    work: Vec<PostOrderWork>,
    depths: Vec<usize>,
    include_heads: bool,
}

pub(in crate::runtime) struct PostOrderNode {
    pub(in crate::runtime) expr: Expr,
    pub(in crate::runtime) level: usize,
    pub(in crate::runtime) depth: usize,
}

impl PostOrderTraversal {
    pub(in crate::runtime) fn new(expr: Expr, include_heads: bool, level: usize) -> Self {
        Self {
            work: vec![PostOrderWork::Visit { expr, level }],
            depths: Vec::new(),
            include_heads,
        }
    }

    pub(in crate::runtime) fn next(&mut self) -> Option<PostOrderNode> {
        while let Some(work) = self.work.pop() {
            match work {
                PostOrderWork::Visit { expr, level } => match &expr {
                    Expr::Atom(_) => {
                        self.depths.push(1);
                        return Some(PostOrderNode {
                            expr,
                            level,
                            depth: 1,
                        });
                    }
                    Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
                        if let Some(entries) = association_entries_from_expr(&expr) {
                            let child_count = entries.len() + usize::from(self.include_heads);
                            self.work.push(PostOrderWork::Finish {
                                expr: expr.clone(),
                                level,
                                child_count,
                                // Association heads participate in traversal order but do not
                                // contribute to the association's negative-level depth.
                                skip_first_child_depth: self.include_heads,
                            });
                            for (_, value) in entries.into_iter().rev() {
                                self.work.push(PostOrderWork::Visit {
                                    expr: value,
                                    level: level + 1,
                                });
                            }
                            if self.include_heads {
                                self.work.push(PostOrderWork::Visit {
                                    expr: head.as_ref().clone(),
                                    level: level + 1,
                                });
                            }
                            continue;
                        }

                        self.schedule_standard_call(expr, level);
                    }
                    Expr::Call { .. } => self.schedule_standard_call(expr, level),
                },
                PostOrderWork::Finish {
                    expr,
                    level,
                    child_count,
                    skip_first_child_depth,
                } => {
                    let first_child = self
                        .depths
                        .len()
                        .checked_sub(child_count)
                        .expect("post-order child depths remain balanced");
                    let mut max_child_depth = 1usize;
                    for (index, depth) in self.depths[first_child..].iter().enumerate() {
                        if !(skip_first_child_depth && index == 0) {
                            max_child_depth = max_child_depth.max(*depth);
                        }
                    }
                    self.depths.truncate(first_child);
                    let depth = max_child_depth.saturating_add(1);
                    self.depths.push(depth);
                    return Some(PostOrderNode { expr, level, depth });
                }
            }
        }

        None
    }

    fn schedule_standard_call(&mut self, expr: Expr, level: usize) {
        let Expr::Call { head, args, .. } = &expr else {
            unreachable!("only calls reach the standard-call scheduler");
        };
        let child_count = args.len() + usize::from(self.include_heads);
        self.work.push(PostOrderWork::Finish {
            expr: expr.clone(),
            level,
            child_count,
            // Heads are selectable parts, but negative levels are measured
            // from argument leaves rather than through the head chain.
            skip_first_child_depth: self.include_heads,
        });
        for arg in args.iter().rev() {
            self.work.push(PostOrderWork::Visit {
                expr: arg.clone(),
                level: level + 1,
            });
        }
        if self.include_heads {
            self.work.push(PostOrderWork::Visit {
                expr: head.as_ref().clone(),
                level: level + 1,
            });
        }
    }
}

#[derive(Clone, FrameRootTrace)]
enum PostOrderPathWork {
    Visit {
        expr: Expr,
        level: usize,
        path: Option<usize>,
    },
    Finish {
        expr: Expr,
        level: usize,
        path: Option<usize>,
        child_count: usize,
        skip_first_child_depth: bool,
    },
}

#[derive(Clone, FrameRootTrace)]
struct PathEntry {
    parent: Option<usize>,
    component: Expr,
}

#[derive(Clone, FrameRootTrace)]
pub(in crate::runtime) struct PostOrderPathTraversal {
    work: Vec<PostOrderPathWork>,
    depths: Vec<usize>,
    paths: Vec<PathEntry>,
    include_heads: bool,
}

pub(in crate::runtime) struct PostOrderPathNode {
    pub(in crate::runtime) expr: Expr,
    pub(in crate::runtime) level: usize,
    pub(in crate::runtime) depth: usize,
    pub(in crate::runtime) path: Option<usize>,
}

impl PostOrderPathTraversal {
    pub(in crate::runtime) fn new(expr: Expr, include_heads: bool, level: usize) -> Self {
        Self {
            work: vec![PostOrderPathWork::Visit {
                expr,
                level,
                path: None,
            }],
            depths: Vec::new(),
            paths: Vec::new(),
            include_heads,
        }
    }

    pub(in crate::runtime) fn next(&mut self) -> Option<PostOrderPathNode> {
        while let Some(work) = self.work.pop() {
            match work {
                PostOrderPathWork::Visit { expr, level, path } => match &expr {
                    Expr::Atom(_) => {
                        self.depths.push(1);
                        return Some(PostOrderPathNode {
                            expr,
                            level,
                            depth: 1,
                            path,
                        });
                    }
                    Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
                        if let Some(entries) = association_entries_from_expr(&expr) {
                            let child_count = entries.len() + usize::from(self.include_heads);
                            self.work.push(PostOrderPathWork::Finish {
                                expr: expr.clone(),
                                level,
                                path,
                                child_count,
                                skip_first_child_depth: self.include_heads,
                            });
                            for (key, value) in entries.into_iter().rev() {
                                let child_path = self
                                    .append_path(path, Expr::call(system_expr("Key"), vec![key]));
                                self.work.push(PostOrderPathWork::Visit {
                                    expr: value,
                                    level: level + 1,
                                    path: Some(child_path),
                                });
                            }
                            if self.include_heads {
                                let child_path =
                                    self.append_path(path, Expr::integer(BigInt::from(0usize)));
                                self.work.push(PostOrderPathWork::Visit {
                                    expr: head.as_ref().clone(),
                                    level: level + 1,
                                    path: Some(child_path),
                                });
                            }
                            continue;
                        }

                        self.schedule_standard_call(expr, level, path);
                    }
                    Expr::Call { .. } => self.schedule_standard_call(expr, level, path),
                },
                PostOrderPathWork::Finish {
                    expr,
                    level,
                    path,
                    child_count,
                    skip_first_child_depth,
                } => {
                    let first_child = self
                        .depths
                        .len()
                        .checked_sub(child_count)
                        .expect("post-order path child depths remain balanced");
                    let mut max_child_depth = 1usize;
                    for (index, depth) in self.depths[first_child..].iter().enumerate() {
                        if !(skip_first_child_depth && index == 0) {
                            max_child_depth = max_child_depth.max(*depth);
                        }
                    }
                    self.depths.truncate(first_child);
                    let depth = max_child_depth.saturating_add(1);
                    self.depths.push(depth);
                    return Some(PostOrderPathNode {
                        expr,
                        level,
                        depth,
                        path,
                    });
                }
            }
        }

        None
    }

    pub(in crate::runtime) fn position_expr(&self, mut path: Option<usize>) -> Expr {
        let mut components = Vec::new();
        while let Some(index) = path {
            let entry = self
                .paths
                .get(index)
                .expect("post-order position path index remains valid");
            components.push(entry.component.clone());
            path = entry.parent;
        }
        components.reverse();
        Expr::call(system_expr("List"), components)
    }

    fn schedule_standard_call(&mut self, expr: Expr, level: usize, path: Option<usize>) {
        let Expr::Call { head, args, .. } = &expr else {
            unreachable!("only calls reach the path standard-call scheduler");
        };
        let child_count = args.len() + usize::from(self.include_heads);
        self.work.push(PostOrderPathWork::Finish {
            expr: expr.clone(),
            level,
            path,
            child_count,
            skip_first_child_depth: self.include_heads,
        });
        for (index, arg) in args.iter().enumerate().rev() {
            let child_path =
                self.append_path(path, Expr::integer(BigInt::from(index.saturating_add(1))));
            self.work.push(PostOrderPathWork::Visit {
                expr: arg.clone(),
                level: level + 1,
                path: Some(child_path),
            });
        }
        if self.include_heads {
            let child_path = self.append_path(path, Expr::integer(BigInt::from(0usize)));
            self.work.push(PostOrderPathWork::Visit {
                expr: head.as_ref().clone(),
                level: level + 1,
                path: Some(child_path),
            });
        }
    }

    fn append_path(&mut self, parent: Option<usize>, component: Expr) -> usize {
        let index = self.paths.len();
        self.paths.push(PathEntry { parent, component });
        index
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<PostOrderWork>();
    assert_traces::<PostOrderTraversal>();
    assert_traces::<PostOrderPathWork>();
    assert_traces::<PathEntry>();
    assert_traces::<PostOrderPathTraversal>();
};
