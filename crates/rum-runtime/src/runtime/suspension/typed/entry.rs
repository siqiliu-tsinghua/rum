//! Private queue entries. Legacy frames retain their original allocation.
use std::mem::{size_of, size_of_val};

use super::adapter::ErasedFrame;
use super::*;
use crate::runtime::suspension::SuspensionFrame;

#[derive(Clone)]
pub(in crate::runtime::suspension) enum Entry {
    Legacy(Arc<dyn SuspensionFrame>),
    Typed(Arc<dyn ErasedFrame>),
}

pub(in crate::runtime::suspension) enum EntryRef<'a> {
    Legacy(&'a Arc<dyn SuspensionFrame>),
    Typed(&'a Arc<dyn ErasedFrame>),
}

impl Entry {
    pub(in crate::runtime::suspension) fn output_boundary(&self) -> OutputBoundary {
        match self {
            Self::Legacy(frame) => frame.output_boundary(),
            Self::Typed(frame) => frame.output_boundary(),
        }
    }

    pub(in crate::runtime::suspension) fn abandon(
        self,
        context: &mut AbandonmentContext<'_>,
    ) -> FrameAbandonAction {
        match self {
            Self::Legacy(frame) => frame.abandon(context),
            Self::Typed(frame) => frame.abandon(context),
        }
    }

    #[inline]
    pub(in crate::runtime::suspension) fn as_ref(&self) -> EntryRef<'_> {
        match self {
            Self::Legacy(frame) => EntryRef::Legacy(frame),
            Self::Typed(frame) => EntryRef::Typed(frame),
        }
    }
}

impl EntryRef<'_> {
    pub(in crate::runtime::suspension) fn output_boundary(&self) -> OutputBoundary {
        match self {
            Self::Legacy(frame) => frame.output_boundary(),
            Self::Typed(frame) => frame.output_boundary(),
        }
    }

    pub(in crate::runtime::suspension) fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        match self {
            Self::Legacy(frame) => frame.trace_roots(visitor),
            Self::Typed(frame) => frame.trace_roots(visitor),
        }
    }

    pub(in crate::runtime::suspension) fn allocation_bytes(&self) -> usize {
        let header = 2 * size_of::<usize>();
        match self {
            Self::Legacy(frame) => header.saturating_add(size_of_val(frame.as_ref())),
            Self::Typed(frame) => header
                .saturating_add(size_of_val(frame.as_ref()))
                .saturating_add(header)
                .saturating_add(size_of_val(frame.inner_frame())),
        }
    }
}
