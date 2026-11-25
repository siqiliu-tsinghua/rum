use std::marker::PhantomData;

use super::action::{Action, ErasedFinalizer};
use super::channel::{CodecKey, Slot, decode, encode};
use super::*;

/// Keeps a rejected input intact until the driver records a structural fault.
pub(in crate::runtime::suspension) struct DecodeFailure {
    pub expected: &'static str,
    pub actual: Slot,
}

pub(in crate::runtime::suspension) trait ErasedFrame:
    FrameRootTrace + FrameAbandon + Send + Sync
{
    /// Consumes input, leaving the resource-free Abort even when decoding fails.
    fn resume(&self, session: &mut Session, input: &mut Result<Slot, EvalError>) -> Action;
    fn output_boundary(&self) -> OutputBoundary;
    fn inner_frame(&self) -> &dyn FrameRootTrace;
}

#[derive(rum_derive::FrameRootTrace)]
struct TypedAdapter<F: TypedFrame> {
    inner: Arc<F>,
}

/// Copied mutation attribution, absent from every live runtime build.
pub(crate) mod cm740_option_route {
    use std::cell::Cell;
    thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
    pub(crate) struct Scope(bool);
    pub(crate) fn enter(active: bool) -> Scope {
        Scope(ACTIVE.with(|c| c.replace(active)))
    }
    impl Drop for Scope {
        fn drop(&mut self) {
            ACTIVE.with(|c| c.set(self.0));
        }
    }
    pub(crate) fn is_active() -> bool {
        ACTIVE.with(|c| c.get())
    }
}
impl<F: TypedFrame> FrameAbandon for TypedAdapter<F> {
    fn abandon(self: Arc<Self>, context: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        self.inner.clone().abandon(context)
    }
}

impl<F: TypedFrame> ErasedFrame for TypedAdapter<F> {
    fn resume(&self, session: &mut Session, input: &mut Result<Slot, EvalError>) -> Action {
        if crate::cm740_probe::possibly_active() {
            return Self::resume_with_probe_state(self, session, input);
        }
        Self::resume_ordinary_body(self, session, input)
    }

    fn output_boundary(&self) -> OutputBoundary {
        self.inner.output_boundary()
    }

    fn inner_frame(&self) -> &dyn FrameRootTrace {
        self.inner.as_ref()
    }
}
impl<F: TypedFrame> TypedAdapter<F> {
    #[inline(always)]
    fn resume_ordinary_body(
        &self,
        session: &mut Session,
        input: &mut Result<Slot, EvalError>,
    ) -> Action {
        let decoded = match std::mem::replace(input, Err(EvalError::Abort)) {
            Ok(slot) => match decode::<F::Input>(slot, CodecKey::new()) {
                Ok(value) => Ok(value),
                Err(actual) => {
                    return Action::DecodeFailure(DecodeFailure {
                        expected: std::any::type_name::<F::Input>(),
                        actual,
                    });
                }
            },
            Err(error) => Err(error),
        };
        // This is the sole token mint. It stores neither this adapter nor F.
        let token = ContinueToken { proof: PhantomData };
        let observe = crate::cm740_probe::control::active("Some");
        let action = self.inner.clone().resume(session, decoded, token).inner;
        if observe {
            if let Action::Complete(Ok(Slot::Value(ref value))) = action {
                eprintln!("CM740_OPTION_SLOT\t{value}");
            }
        }
        action
    }

    #[cold]
    #[inline(never)]
    fn resume_with_probe_state(
        &self,
        session: &mut Session,
        input: &mut Result<Slot, EvalError>,
    ) -> Action {
        let _resumed = crate::cm740_probe::control::resume::<F, _>(self);
        crate::cm740_probe::class_lease::note_typed_consumer(
            self as *const Self as *const () as usize,
            std::any::type_name::<F>(),
        );
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_620(self, session, input);
        }
        Self::resume_ordinary_body(self, session, input)
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_620(
        &self,
        session: &mut Session,
        input: &mut Result<Slot, EvalError>,
    ) -> Action {
        let decoded = match std::mem::replace(input, Err(EvalError::Abort)) {
            Ok(slot) => match decode::<F::Input>(slot, CodecKey::new()) {
                Ok(value) => Ok(value),
                Err(actual) => {
                    return Action::DecodeFailure(DecodeFailure {
                        expected: std::any::type_name::<F::Input>(),
                        actual,
                    });
                }
            },
            Err(error) => Err(error),
        };
        // This is the sole token mint. It stores neither this adapter nor F.
        let token = ContinueToken { proof: PhantomData };
        let observe = std::any::type_name::<F>().ends_with("::StatusFunctionProjectionFrame")
            && crate::cm740_probe::control::active("Some");
        let _scope = crate::runtime::suspension::typed::adapter::cm740_option_route::enter(observe);
        let action = self.inner.clone().resume(session, decoded, token).inner;
        if observe {
            if let Action::Complete(Ok(Slot::Value(ref value))) = action {
                eprintln!("CM740_OPTION_SLOT\t{value}");
            }
        }
        action
    }
}

pub(in crate::runtime::suspension) fn erase<F: TypedFrame>(frame: F) -> Arc<dyn ErasedFrame> {
    Arc::new(TypedAdapter {
        inner: Arc::new(frame),
    })
}

pub(in crate::runtime::suspension) fn allocation_bytes<F: TypedFrame>() -> usize {
    4 * std::mem::size_of::<usize>()
        + std::mem::size_of::<TypedAdapter<F>>()
        + std::mem::size_of::<F>()
}

impl<F: SlotFinalizer> ErasedFinalizer for F {
    fn finish(self: Box<Self>) -> Result<Slot, EvalError> {
        SlotFinalizer::finish(self).map(|value| encode(value, CodecKey::new()))
    }
}
