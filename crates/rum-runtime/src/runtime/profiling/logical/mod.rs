//! LP02 infrastructure. Production phase recorders deliberately remain legacy.
use crate::runtime::{EvalError, suspension::Slot};
use std::sync::Arc;

mod budget;
mod context;
pub(in crate::runtime) mod epoch;
pub(in crate::runtime) mod index;
mod queue_api;
mod record;
pub(in crate::runtime) mod segments;
pub(in crate::runtime) mod source_stack;
mod totals;

pub(in crate::runtime) use budget::Storage;
pub(in crate::runtime) use context::{Context, ContextGuard};
pub(in crate::runtime) use epoch::{Epoch, drop_session, reset, summary};
pub(in crate::runtime) use record::*;

pub(in crate::runtime) struct ProfileBoundary {
    pub(in crate::runtime) epoch: Arc<Epoch>,
    pub(in crate::runtime) record: RecordId,
    pub(in crate::runtime) _storage: Storage,
}

impl ProfileBoundary {
    pub(in crate::runtime) fn belongs_to(
        &self,
        control: &Arc<crate::runtime::RequestInterruptState>,
    ) -> bool {
        self.epoch
            .inspect(self.record)
            .is_some_and(|(owner, _, _)| {
                Arc::ptr_eq(&owner.control, control)
                    && owner.generation == control.logical_generation()
            })
    }
    pub(in crate::runtime) fn complete(&self, outcome: Outcome) {
        segments::finish_active(&self.epoch, self.record);
        self.epoch.close(self.record, Ok(outcome));
    }
    pub(in crate::runtime) fn abandon(&self, reason: Retire) {
        segments::finish_active(&self.epoch, self.record);
        self.epoch.close(
            self.record,
            match reason {
                Retire::Structural(crate::runtime::suspension::AbandonReason::CancelledRequest) => {
                    Ok(Outcome::Cancelled)
                }
                _ => Err(reason),
            },
        );
    }
    pub(in crate::runtime) fn policy(&self) -> Option<Policy> {
        self.epoch.inspect(self.record).map(|(_, _, p)| p)
    }
}
impl Drop for ProfileBoundary {
    fn drop(&mut self) {
        self.abandon(Retire::LastDrop);
    }
}

pub(in crate::runtime) struct Call {
    epoch: Arc<Epoch>,
    record: RecordId,
    segment: Option<segments::Segment>,
    finished: bool,
}

#[derive(Clone)]
pub(in crate::runtime) struct Frontier {
    pub(in crate::runtime) generation: crate::runtime::suspension::SuspensionGeneration,
    pub(in crate::runtime) after: Option<crate::runtime::suspension::CellId>,
    pub(in crate::runtime) control: Arc<crate::runtime::RequestInterruptState>,
    pub(in crate::runtime) request_generation: u64,
}

pub(in crate::runtime) fn error_outcome(error: &crate::runtime::EvalError) -> Outcome {
    use crate::runtime::EvalError;
    match error {
        EvalError::Return { .. } => Outcome::Return,
        EvalError::Thrown { .. } | EvalError::ThrownBypassingCatch { .. } => Outcome::Throw,
        EvalError::Cancelled => Outcome::Cancelled,
        _ => Outcome::Error,
    }
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
impl crate::runtime::Session {
    /// Isolated native test span; no production phase uses the LP02 engine.
    #[doc(hidden)]
    pub fn logical_profile_test_span(
        &mut self,
        phase: &'static str,
        expr: crate::runtime::Expr,
    ) -> Result<crate::runtime::Expr, crate::runtime::EvalError> {
        if !super::lexical_profile_enabled() {
            return self.evaluate(expr);
        }
        let call = Call::start(
            self,
            AggregateKey::Runtime {
                phase,
                source: crate::runtime::evaluation_profile_current_source_symbol(),
            },
            Policy::Expr,
        );
        let outcome = self.evaluate(expr);
        if let Some(call) = call {
            match &outcome {
                Err(crate::runtime::EvalError::InputRequired { .. }) => {
                    if let Some(boundary) = call.pause() {
                        self.logical_insert_boundary(boundary, None, false);
                    }
                }
                Ok(_) => call.finish(Outcome::Ok),
                Err(error) => call.finish(error_outcome(error)),
            }
        }
        outcome
    }

    #[doc(hidden)]
    pub fn logical_profile_test_source_scope(
        &mut self,
        source: crate::runtime::Symbol,
        expr: crate::runtime::Expr,
    ) -> Result<crate::runtime::Expr, crate::runtime::EvalError> {
        let _guard = crate::runtime::evaluation_profile_definition_source(source);
        self.evaluate(expr)
    }

    #[doc(hidden)]
    pub fn logical_profile_test_rows() -> Vec<(String, [u64; 10])> {
        epoch::current()
            .map(|e| {
                e.rows()
                    .into_iter()
                    .map(|(key, c)| {
                        (
                            format!("{key:?}"),
                            [
                                c.calls,
                                c.ok,
                                c.err,
                                c.some,
                                c.none,
                                c.return_err,
                                c.throw_err,
                                c.cancelled,
                                c.abandoned,
                                c.open,
                            ],
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    #[doc(hidden)]
    pub fn logical_profile_test_leases() -> usize {
        epoch::current()
            .map(|e| e.contexts.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap_or(0)
    }

    #[doc(hidden)]
    pub fn logical_profile_test_assert_quiescent() {
        if let Some(epoch) = epoch::current() {
            epoch.assert_quiescent();
        }
    }

    #[doc(hidden)]
    pub fn logical_profile_test_reset() {
        epoch::reset();
    }

    #[doc(hidden)]
    pub fn logical_profile_test_reasons() -> [u64; 11] {
        let mut reasons = [0; 11];
        if let Some(epoch) = epoch::current() {
            for (_, counts) in epoch.rows() {
                for (target, value) in reasons.iter_mut().zip(counts.abandonment_reasons) {
                    *target += value;
                }
            }
        }
        reasons
    }

    #[doc(hidden)]
    pub fn logical_profile_test_pauses() -> u64 {
        epoch::current()
            .map(|e| e.rows().iter().map(|(_, c)| c.pauses).sum())
            .unwrap_or(0)
    }
}

impl Call {
    /// The caller invokes this only inside its existing enabled slow path.
    pub(in crate::runtime) fn start(
        session: &crate::runtime::Session,
        key: AggregateKey,
        policy: Policy,
    ) -> Option<Self> {
        let control = session.request_interrupt_control.borrow();
        let (epoch, owner, parent) = match segments::owner(&control) {
            Some((epoch, owner, parent)) => (epoch, owner, Some(parent)),
            None => (
                epoch::current()?,
                Owner {
                    control: Arc::clone(&control),
                    generation: control.logical_generation(),
                    root: epoch::next_id(),
                },
                None,
            ),
        };
        let record = epoch.open(owner, parent, key, policy)?;
        let segment = segments::enter(&epoch, record)?;
        Some(Self {
            epoch,
            record,
            segment: Some(segment),
            finished: false,
        })
    }
    pub(in crate::runtime) fn finish(mut self, outcome: Outcome) {
        self.segment.take();
        self.epoch.close(self.record, Ok(outcome));
        self.finished = true;
    }
    pub(in crate::runtime) fn pause(mut self) -> Option<Arc<ProfileBoundary>> {
        if let Some(segment) = &self.segment {
            segment.pause();
        }
        self.segment.take();
        let storage = self
            .epoch
            .storage(std::mem::size_of::<ProfileBoundary>() + 16)?;
        self.finished = true;
        Some(Arc::new(ProfileBoundary {
            epoch: Arc::clone(&self.epoch),
            record: self.record,
            _storage: storage,
        }))
    }
}
impl Drop for Call {
    fn drop(&mut self) {
        self.segment.take();
        if !self.finished {
            self.epoch.close(self.record, Err(Retire::LastDrop));
        }
    }
}

pub(in crate::runtime) fn executing_epoch() -> Option<Arc<Epoch>> {
    segments::executing_epoch()
}

impl ProfileBoundary {
    /// A raw read only. This path never obtains a codec key or a resume token.
    pub(in crate::runtime) fn observe(&self, result: &Result<Slot, EvalError>) {
        use crate::runtime::suspension::ContinuationState;
        let Some(policy) = self.policy() else {
            return;
        };
        let outcome = match result {
            Err(EvalError::InputRequired { .. }) => return,
            Err(EvalError::Return { .. }) => Outcome::Return,
            Err(EvalError::Thrown { .. } | EvalError::ThrownBypassingCatch { .. }) => {
                Outcome::Throw
            }
            Err(EvalError::Cancelled) => Outcome::Cancelled,
            Err(_) => Outcome::Error,
            Ok(slot) => match (policy, slot) {
                (Policy::Incoming(outcome), Slot::Value(_)) => outcome,
                (Policy::Expr, Slot::Value(_)) => Outcome::Ok,
                (Policy::ApplicableExpr, Slot::Value(_)) => Outcome::Some,
                (Policy::OptionalExpr, Slot::State(ContinuationState::OptionalExpr(value))) => {
                    if value.is_some() {
                        Outcome::Some
                    } else {
                        Outcome::None
                    }
                }
                (Policy::Bool, Slot::State(ContinuationState::Bool(_)))
                | (Policy::Prepared, Slot::State(ContinuationState::PreparedArguments(_)))
                | (Policy::Match, Slot::State(ContinuationState::MatchOutcome(_))) => Outcome::Ok,
                _ => {
                    self.epoch.fault();
                    return;
                }
            },
        };
        self.complete(if let Policy::Incoming(incoming) = policy {
            incoming
        } else {
            outcome
        });
    }
}

#[cfg(test)]
pub(in crate::runtime) use super::tests::engine_support as test_support;
