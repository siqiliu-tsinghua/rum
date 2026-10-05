//! Held pattern preparation: the parent work stack is also the suspension state.
use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

mod continuation;
mod state;
#[cfg(test)]
mod tests;

use continuation::MatcherPatternSuspensionFrame;
use state::*;

// Local transitions never own another transition or survive a guest call.
enum MatcherPatternStep {
    Visit(Expr),
    PreparedCall { head: Expr, args: Vec<Expr> },
    Done(Expr),
}

impl Session {
    pub(super) fn resolve_held_matcher_pattern(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        let root = strip_hold_pattern(&expr);
        self.drive_matcher_pattern(
            MatcherPatternState { stack: Vec::new() },
            MatcherPatternStep::Visit(root.clone()),
            Some(root),
        )
    }

    fn drive_matcher_pattern(
        &mut self,
        mut state: MatcherPatternState,
        mut step: MatcherPatternStep,
        root: Option<&Expr>,
    ) -> Result<Expr, EvalError> {
        loop {
            let value = match step {
                MatcherPatternStep::Done(value) => value,
                MatcherPatternStep::PreparedCall { head, args } => {
                    // The call's work record is already popped. The callee owns the
                    // moved arguments; only the parent and destination survive input.
                    match self.resume_call_from_prepared_args(head, args) {
                        Ok(value) => value,
                        Err(error @ EvalError::InputRequired { .. }) => {
                            let destination = state.destination();
                            return self.suspend_matcher_pattern(
                                state,
                                Some(MatcherPatternAwait::PreparedCall { destination }),
                                None,
                                error,
                            );
                        }
                        Err(error) => return Err(error),
                    }
                }
                MatcherPatternStep::Visit(expr) => {
                    if !contains_pattern_syntax(&expr) {
                        match self.evaluate(expr) {
                            Ok(value) => value,
                            Err(error @ EvalError::InputRequired { .. }) => {
                                return self.suspend_matcher_pattern(state, None, root, error);
                            }
                            Err(error) => return Err(error),
                        }
                    } else {
                        match expr {
                            Expr::Call { head, args } if !matches_pattern_head(&head) => {
                                step = MatcherPatternStep::Visit(head.as_ref().clone());
                                state.stack.push(MatcherPatternWork {
                                    original_head: head.as_ref().clone(),
                                    prefix: Vec::with_capacity(args.len()),
                                    original_args: args,
                                    resolved_head: None,
                                    next: 0,
                                });
                                continue;
                            }
                            // The policy is selected from raw syntax. Recognized
                            // pattern calls, including their operands, stay opaque.
                            value => value,
                        }
                    }
                }
            };
            let destination = state.destination();
            // Every path, including a resumed root, normalizes here once. An
            // ordinary root need not make an acceptance/Done round trip first.
            if matches!(destination, MatcherPatternDestination::Root) {
                return Ok(self.normalize_match_expr(&value));
            }
            step = accept_matcher_pattern_value(&mut state, destination, value);
        }
    }

    #[cold]
    fn suspend_matcher_pattern(
        &mut self,
        state: MatcherPatternState,
        pending: Option<MatcherPatternAwait>,
        root: Option<&Expr>,
        error: EvalError,
    ) -> Result<Expr, EvalError> {
        let pending = match pending {
            Some(pending) => pending,
            None => {
                let destination = state.destination();
                // Parents retain raw children until acceptance. The entry borrow
                // supplies an ordinary root only; it never escapes this capture.
                let pending_expr = match destination {
                    MatcherPatternDestination::Root => root.expect("root entry retains its source"),
                    MatcherPatternDestination::Head => {
                        &state.stack.last().expect("parent").original_head
                    }
                    MatcherPatternDestination::Argument(index) => {
                        &state.stack.last().expect("parent").original_args[index]
                    }
                }
                .clone();
                MatcherPatternAwait::Evaluate {
                    destination,
                    pending_expr,
                }
            }
        };
        crate::resume_or_suspend!(insert self, MatcherPatternSuspensionFrame { state, pending });
        Err(error)
    }
}

fn accept_matcher_pattern_value(
    state: &mut MatcherPatternState,
    destination: MatcherPatternDestination,
    value: Expr,
) -> MatcherPatternStep {
    if matches!(destination, MatcherPatternDestination::Root) {
        return MatcherPatternStep::Done(value);
    }
    let work = state
        .stack
        .last_mut()
        .expect("child destination has a parent");
    match destination {
        MatcherPatternDestination::Root => unreachable!(),
        MatcherPatternDestination::Head => work.resolved_head = Some(value),
        MatcherPatternDestination::Argument(index) => {
            debug_assert_eq!(work.next, index);
            work.prefix.push(value);
            work.next = index + 1;
        }
    }
    if work.next < work.original_args.len() {
        return MatcherPatternStep::Visit(work.original_args[work.next].clone());
    }
    let work = state.stack.pop().expect("completed parent");
    MatcherPatternStep::PreparedCall {
        head: work
            .resolved_head
            .expect("head is resolved before arguments"),
        args: work.prefix,
    }
}

fn contains_pattern_syntax(expr: &Expr) -> bool {
    let Expr::Call { head, args } = strip_hold_pattern(expr) else {
        return false;
    };
    if matches_pattern_head(head) {
        return true;
    }
    let mut pending = Vec::new();
    let mut current = args.iter();
    if matches!(head.as_ref(), Expr::Call { .. }) {
        if !current.as_slice().is_empty() {
            pending.push(current);
        }
        current = std::slice::from_ref(head.as_ref()).iter();
    }
    loop {
        match current.next() {
            Some(expr) => {
                if let Expr::Call { head, args } = strip_hold_pattern(expr) {
                    if matches_pattern_head(head) {
                        return true;
                    }
                    if !current.as_slice().is_empty() {
                        pending.push(current);
                    }
                    current = args.iter();
                    if matches!(head.as_ref(), Expr::Call { .. }) {
                        if !current.as_slice().is_empty() {
                            pending.push(current);
                        }
                        current = std::slice::from_ref(head.as_ref()).iter();
                    }
                }
            }
            None => match pending.pop() {
                Some(next) => current = next,
                None => return false,
            },
        }
    }
}

pub(super) fn matches_pattern_head(head: &Expr) -> bool {
    // OptionsPattern is recognized by the sequence matcher, but the existing
    // MatchQ resolution policy does not treat it as an opaque whole-pattern
    // wrapper. Keep that matcher behavior unchanged while sharing the registry.
    matcher_pattern_head(head)
        .is_some_and(|pattern_head| pattern_head != MatcherPatternHead::OptionsPattern)
}
