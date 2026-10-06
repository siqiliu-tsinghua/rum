//! Ordered, resumable leaf stores. Shape validation is the entry's existing
//! boundary; leaf assignment and Expr internals remain evaluator boundaries.
use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use crate::runtime::suspension::{FrameAction, SuspensionFrame};
use std::sync::Arc;

#[derive(Clone, rum_derive::FrameRootTrace)]
struct DestructuringCursor {
    lhs: SharedArgs,
    rhs: SharedArgs,
    next: usize,
}

#[derive(Clone, rum_derive::FrameRootTrace)]
struct DestructuringWork {
    current: DestructuringCursor,
    parent: Option<DestructuringCursor>,
    parents: Vec<DestructuringCursor>,
    pending_leaf: bool,
}

impl DestructuringWork {
    fn next_leaf(&mut self) -> Option<(Expr, Expr)> {
        debug_assert!(!self.pending_leaf);
        loop {
            if self.current.next == self.current.lhs.len() {
                self.current = self.parent.take()?;
                self.parent = self.parents.pop();
                continue;
            }
            let index = self.current.next;
            self.current.next += 1;
            if let Expr::Call { head, args } = &self.current.lhs[index] {
                if is_system_symbol_name(head, heads::structure::LIST) {
                    let Expr::Call { args: rhs, .. } = &self.current.rhs[index] else {
                        unreachable!("destructuring shape checked before store");
                    };
                    let child = DestructuringCursor {
                        lhs: args.clone(),
                        rhs: rhs.clone(),
                        next: 0,
                    };
                    let parent = std::mem::replace(&mut self.current, child);
                    if let Some(older) = self.parent.replace(parent) {
                        self.parents.push(older);
                    }
                    continue;
                }
            }
            // The cursor already owns the suffix before the child can suspend.
            self.pending_leaf = true;
            return Some((
                self.current.lhs[index].clone(),
                self.current.rhs[index].clone(),
            ));
        }
    }

    fn complete_leaf(&mut self) {
        debug_assert!(self.pending_leaf);
        self.pending_leaf = false;
    }
}

impl Session {
    pub(super) fn store_list_destructuring_assignments(
        &mut self,
        symbol: &str,
        lhs: Expr,
        rhs: Expr,
    ) -> Result<Expr, EvalError> {
        let Expr::Call { args: lhs, .. } = lhs else {
            unreachable!("destructuring entry requires a list");
        };
        let Expr::Call {
            args: rhs_items, ..
        } = &rhs
        else {
            unreachable!("destructuring shape checked before store");
        };
        let work = DestructuringWork {
            current: DestructuringCursor {
                lhs,
                rhs: rhs_items.clone(),
                next: 0,
            },
            parent: None,
            parents: Vec::new(),
            pending_leaf: false,
        };
        self.drive_list_destructuring(symbol, rhs, work)
    }

    fn drive_list_destructuring(
        &mut self,
        symbol: &str,
        original_rhs: Expr,
        mut work: DestructuringWork,
    ) -> Result<Expr, EvalError> {
        while let Some((lhs, rhs)) = work.next_leaf() {
            match self.finish_definition_assignment(symbol, lhs, rhs, false) {
                Ok(_) => work.complete_leaf(),
                Err(EvalError::InputRequired { request }) => {
                    return suspend_destructuring(self, symbol, original_rhs, work, request);
                }
                Err(error) => return Err(error),
            }
        }
        Ok(original_rhs)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct DestructuringSuspensionFrame {
    symbol: Symbol,
    original_rhs: Expr,
    work: DestructuringWork,
}

impl SuspensionFrame for DestructuringSuspensionFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(match outcome {
            Ok(_) => resume_destructuring(session, &self),
            Err(error) => Err(error),
        })
    }
}

#[cold]
#[inline(never)]
fn suspend_destructuring(
    session: &mut Session,
    symbol: &str,
    original_rhs: Expr,
    work: DestructuringWork,
    request: crate::RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    crate::resume_or_suspend!(insert session, DestructuringSuspensionFrame {
        symbol: Symbol::system(symbol),
        original_rhs,
        work,
    });
    Err(EvalError::InputRequired { request })
}

#[cold]
fn resume_destructuring(
    session: &mut Session,
    frame: &DestructuringSuspensionFrame,
) -> Result<Expr, EvalError> {
    // The immutable frame keeps every root visible while the working copy runs.
    let mut work = frame.work.clone();
    work.complete_leaf();
    session.drive_list_destructuring(frame.symbol.name(), frame.original_rhs.clone(), work)
}
