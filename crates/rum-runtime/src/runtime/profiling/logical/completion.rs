use super::*;

impl ProfileBoundary {
    pub(super) fn has_runtime_key(&self, expected: &str) -> bool {
        self.epoch.path(self.record).is_some_and(
            |p| matches!(p.key, AggregateKey::Runtime {phase, ..} if phase == expected),
        )
    }

    pub(super) fn has_runtime_key_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        access: &crate::runtime::profiling::lease::Admitted,
        expected: &str,
    ) -> bool {
        self.epoch
            .path_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                Some(access),
                self.record,
            )
            .is_some_and(
                |p| matches!(p.key, AggregateKey::Runtime {phase, ..} if phase == expected),
            )
    }
}

impl Call {
    pub(super) fn defer_definition(mut self) {
        self.inner.epoch.observe_silent_retention(self.inner.record);
        self.inner.segment.take();
        self.inner.epoch.defer_definition(self.inner.record);
        self.inner.finished = true;
    }
}
impl crate::runtime::Session {
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn flush_definition_boundaries(&mut self) {
        // Direct finish has no enclosing segment (`executing_epoch` is None).
        // The deferred record was opened on the current epoch by `Call::start`.
        let Some(epoch) = executing_epoch()
            .or_else(super::epoch::existing)
            .filter(|epoch| epoch.is_valid())
        else {
            return;
        };
        let control = self.request_interrupt_control.borrow().clone();
        while let Some(boundary) = take_deferred_definition_boundary(&epoch, &control) {
            self.logical_insert_boundary(boundary, None, false);
        }
    }
}

/// Drain one record for the exact request control and generation. Both the
/// Session publisher and the profiled queue join use the same charged owner.
pub(in crate::runtime) fn take_deferred_definition_boundary(
    epoch: &Arc<Epoch>,
    control: &Arc<crate::runtime::RequestInterruptState>,
) -> Option<Arc<ProfileBoundary>> {
    let record = epoch.take_deferred_definition(control)?;
    let storage = epoch.storage(std::mem::size_of::<ProfileBoundary>() + 16)?;
    Some(Arc::new(ProfileBoundary {
        epoch: Arc::clone(epoch),
        record,
        _storage: storage.into(),
    }))
}

impl ProfileBoundary {
    pub(in crate::runtime) fn observe_decoded<T: 'static>(&self, result: &Result<T, EvalError>) {
        let Some(policy) = self.policy() else {
            return;
        };
        let outcome = match result {
            Err(EvalError::InputRequired { .. }) => return,
            Err(error) => error_outcome(error),
            Ok(value) => {
                let any = value as &dyn std::any::Any;
                match policy {
                    Policy::Expr if any.is::<Expr>() => Outcome::Ok,
                    Policy::Expr
                        if self.has_runtime_key("definition.rhs.total")
                            && any.is::<Option<Expr>>() =>
                    {
                        Outcome::Ok
                    }
                    Policy::ApplicableExpr if any.is::<Expr>() => Outcome::Some,
                    Policy::Incoming(incoming) if any.is::<Expr>() => incoming,
                    Policy::OptionalExpr if any.is::<Option<Expr>>() => {
                        if any.downcast_ref::<Option<Expr>>().unwrap().is_some() {
                            Outcome::Some
                        } else {
                            Outcome::None
                        }
                    }
                    Policy::OptionalExpr
                        if any.is::<Expr>()
                            && self.epoch.path(self.record).is_some_and(|path| {
                                matches!(path.key, AggregateKey::Definition { .. })
                            }) =>
                    {
                        Outcome::Some
                    }
                    Policy::OptionalExpr
                        if self.has_runtime_key("call.stage.prepare_arguments")
                            && any
                                .is::<crate::runtime::eval_call_prepare::PreparedCallArguments>(
                                ) =>
                    {
                        Outcome::None
                    }
                    Policy::Prepared
                        if any.is::<crate::runtime::eval_call_prepare::PreparedCallArguments>() =>
                    {
                        Outcome::Ok
                    }
                    Policy::Match
                        if any.is::<crate::runtime::pattern_matching::CompiledMatchOutcome>() =>
                    {
                        Outcome::Ok
                    }
                    Policy::Bool if any.is::<bool>() => Outcome::Ok,
                    _ => {
                        self.epoch.fault();
                        return;
                    }
                }
            }
        };
        self.complete(if let Policy::Incoming(incoming) = policy {
            incoming
        } else {
            outcome
        });
    }
}

impl ProfileBoundary {
    pub(in crate::runtime) fn begin_prepared_completion<T: 'static>(
        &self,
        outcome: &Result<T, EvalError>,
    ) -> Option<segments::Segment> {
        if !self.has_runtime_key("call.stage.prepare_arguments") {
            return None;
        }
        let value = outcome.as_ref().ok()? as &dyn std::any::Any;
        if !value.is::<crate::runtime::eval_call_prepare::PreparedCallArguments>() {
            return None;
        }
        let path = self.epoch.path(self.record)?;
        segments::restore(&self.epoch, &path)
    }
}
#[inline]
pub(in crate::runtime) fn finish_prepared_completion(session: &crate::runtime::Session) {
    if super::super::lexical_profile_enabled() {
        finish_prepared_completion_slow(session);
    }
}
#[cold]
#[inline(never)]
fn finish_prepared_completion_slow(session: &crate::runtime::Session) {
    let control = session.request_interrupt_control.borrow();
    let Some((epoch, _, record)) = segments::owner(&control) else {
        return;
    };
    let Some(path) = epoch.path(record) else {
        return;
    };
    if !matches!(
        path.key,
        AggregateKey::Runtime {
            phase: "call.stage.prepare_arguments",
            ..
        }
    ) {
        epoch.fault();
        return;
    }
    segments::finish_active(&epoch, record);
    epoch.close(record, Ok(Outcome::None));
}

impl ProfileBoundary {
    pub(in crate::runtime) fn is_prepared_completion<T: 'static>(
        &self,
        outcome: &Result<T, EvalError>,
    ) -> bool {
        self.has_runtime_key("call.stage.prepare_arguments")
            && outcome.as_ref().is_ok_and(|value| {
                let any = value as &dyn std::any::Any;
                any.is::<crate::runtime::eval_call_prepare::PreparedCallArguments>()
            })
    }
}
