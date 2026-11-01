use super::*;
use crate::runtime::accumulator_chain::{AccumulatorChain, AccumulatorChainNode};
use crate::runtime::{RequestInterruptState, Symbol};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct EpochId(pub(in crate::runtime) u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) struct RecordId(pub(in crate::runtime) u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct SegmentId(pub(in crate::runtime) u64);

#[derive(Clone)]
pub(in crate::runtime) struct Owner {
    pub(in crate::runtime) control: Arc<RequestInterruptState>,
    pub(in crate::runtime) generation: u64,
    pub(in crate::runtime) root: u64,
}
impl Owner {
    pub(in crate::runtime) fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.control, &other.control)
            && self.generation == other.generation
            && self.root == other.root
    }
}

#[allow(dead_code)] // LP03-LP07 adopt these keys and policies separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) enum AggregateKey {
    Runtime {
        phase: &'static str,
        source: Option<Symbol>,
    },
    Builtin {
        symbol: Symbol,
        source: Option<Symbol>,
        arity: u16,
    },
    Definition {
        kind: &'static str,
        symbol: Symbol,
        index: u32,
    },
    RuleRhs {
        phase: &'static str,
        source: Option<Symbol>,
    },
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum Policy {
    Expr,
    OptionalExpr,
    Bool,
    Prepared,
    Match,
    ApplicableExpr,
    Incoming(Outcome),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum Outcome {
    Ok,
    Some,
    None,
    Error,
    Return,
    Throw,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum Retire {
    Structural(crate::runtime::suspension::AbandonReason),
    SessionDrop,
    ProfileReset,
    DiagnosticCap,
    InstrumentationFault,
    LastDrop,
}
impl Retire {
    pub(in crate::runtime) fn index(self) -> usize {
        use crate::runtime::suspension::AbandonReason;
        match self {
            Self::Structural(reason) => match reason {
                AbandonReason::ContinuationSlotMismatch => 0,
                AbandonReason::CancelledRequest => 1,
                AbandonReason::FailedSnapshotRestore => 2,
                AbandonReason::HelperBestEffortDiscard => 3,
                AbandonReason::CheckpointRollbackDiscard => 4,
                AbandonReason::SupersededOrQueuedInputDiscard => 5,
            },
            Self::SessionDrop => 6,
            Self::ProfileReset => 7,
            Self::DiagnosticCap => 8,
            Self::InstrumentationFault => 9,
            Self::LastDrop => 10,
        }
    }
}

pub(in crate::runtime) struct Record {
    pub owner: Owner,
    pub parent: Option<RecordId>,
    pub policy: Policy,
    pub path: Arc<RecordPath>,
}

#[derive(rum_derive::AccumulatorChainNode)]
pub(in crate::runtime) struct RecordPath {
    pub(super) record: RecordId,
    #[accumulator_link]
    pub(super) parent: AccumulatorChain<Self>,
    pub(super) depth: usize,
    pub(super) owner: Owner,
    pub(super) key: AggregateKey,
    pub(super) state: std::sync::Mutex<super::totals::State>,
    pub(super) _storage: Storage,
}

#[cfg(test)]
impl RecordPath {
    /// Build a charged ancestry without an index retaining every node. The
    /// teardown tests must leave only the suffix's last one or two holders.
    pub(in crate::runtime) fn test_chain(epoch: &Epoch, owner: Owner, depth: usize) -> Arc<Self> {
        let mut parent = None;
        for index in 0..depth {
            parent = Some(Arc::new(Self {
                record: RecordId(index as u64),
                parent: AccumulatorChain::new(parent),
                depth: index + 1,
                owner: owner.clone(),
                key: AggregateKey::Runtime {
                    phase: "chain-drop",
                    source: None,
                },
                state: std::sync::Mutex::new(super::totals::State::default()),
                _storage: epoch
                    .storage(std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>())
                    .unwrap(),
            }));
        }
        parent.unwrap()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::runtime) struct Counts {
    pub calls: u64,
    pub ok: u64,
    pub err: u64,
    pub some: u64,
    pub none: u64,
    pub return_err: u64,
    pub throw_err: u64,
    pub cancelled: u64,
    pub abandoned: u64,
    pub open: u64,
    pub pauses: u64,
    pub input_required: u64,
    pub segments: u64,
    pub nanos: u64,
    pub self_nanos: u64,
    pub abandonment_reasons: [u64; 11],
}

impl Counts {
    pub(in crate::runtime) fn finish(&mut self, outcome: Result<Outcome, Retire>) {
        self.open = self.open.saturating_sub(1);
        self.calls += 1;
        match outcome {
            Err(reason) => {
                self.abandoned += 1;
                self.abandonment_reasons[reason.index()] += 1;
            }
            Ok(Outcome::Ok) => self.ok += 1,
            Ok(Outcome::Some) => {
                self.ok += 1;
                self.some += 1;
            }
            Ok(Outcome::None) => {
                self.ok += 1;
                self.none += 1;
            }
            Ok(error) => {
                self.err += 1;
                match error {
                    Outcome::Return => self.return_err += 1,
                    Outcome::Throw => self.throw_err += 1,
                    Outcome::Cancelled => self.cancelled += 1,
                    _ => {}
                }
            }
        }
    }
}
