//! Expand only in the private owner's test module. This keeps accounting
//! here while permitting direct field reads, without changing production
//! visibility or relying on parent-relative source paths in the inventory.

macro_rules! define_oracle {
    () => {
        use super::*;
        use crate::runtime::debug_tools::resources::occurrence::{Bytes, expression};

        impl SuspendedEvaluation {
            pub(in crate::runtime) fn occurrence_bytes(&self) -> Bytes {
                let queue = match &self.frames {
                    FrameQueue::Legacy(frames) => Bytes::Exact(frames.capacity())
                        .multiply(size_of::<Queued<Arc<DynSuspensionFrame>>>()),
                    FrameQueue::Mixed(frames) => {
                        Bytes::Exact(frames.capacity()).multiply(size_of::<Queued<Entry>>())
                    }
                };
                let mut bytes = Bytes::Exact(size_of::<Self>()).add(queue);
                // As in the specified legacy model, saved-stack spare capacity is
                // omitted; the live Session evaluation_stack does charge its spare.
                for entry in &self.active_stack {
                    bytes = bytes
                        .add(Bytes::Exact(size_of::<EvaluationStackEntry>()))
                        .add(expression(&entry.expr));
                }
                let mut roots = OccurrenceRoots { bytes };
                self.frames.visit(|frame| {
                    match &frame {
                        EntryRef::EvaluationReturn => {}
                        EntryRef::Legacy(owner) => {
                            roots.bytes = roots
                                .bytes
                                .add(Bytes::Exact(size_of::<usize>()).multiply(2))
                                .add(Bytes::Exact(size_of_val(owner.as_ref())));
                        }
                        EntryRef::Typed(adapter) => {
                            // Enumerate each owning allocation independently.
                            roots.bytes = roots
                                .bytes
                                .add(Bytes::Exact(size_of::<usize>()).multiply(2))
                                .add(Bytes::Exact(size_of_val(adapter.as_ref())))
                                .add(Bytes::Exact(size_of::<usize>()).multiply(2))
                                .add(Bytes::Exact(size_of_val(adapter.inner_frame())));
                        }
                    }
                    // The structural protocol is shared, but the expression arithmetic
                    // and the frame/container accounting are independent of memory.rs.
                    frame.trace_roots(&mut roots);
                });
                roots.bytes
            }

            pub(in crate::runtime) fn occurrence_frame_capacity(&self) -> usize {
                self.frames.capacity()
            }
        }

        struct OccurrenceRoots {
            bytes: Bytes,
        }

        impl FrameRootVisitor for OccurrenceRoots {
            fn visit_expr(&mut self, expr: &Expr) {
                self.bytes = self.bytes.add(expression(expr));
            }

            fn visit_symbol(&mut self, symbol: &Symbol) {
                self.bytes = self
                    .bytes
                    .add(Bytes::Exact(size_of::<Symbol>()))
                    .add(Bytes::Exact(symbol.context().len()))
                    .add(Bytes::Exact(symbol.name().len()));
            }
        }
    };
}

pub(in crate::runtime) use define_oracle;
