use super::*;
use std::vec::IntoIter;

// Only remaining tails are frozen. Ordinals never index these shortened tails.
#[derive(Clone, rum_derive::FrameRootTrace)]
pub(in super::super) struct TraceFilterState {
    pub(super) mode: TraceOutput,
    pub(super) original_value: Expr,
    pub(super) pattern: Option<Expr>,
    pub(super) levels: Option<CasesLevelSpec>,
    pub(super) include_heads: bool,
    pub(super) current: TraceFilterLevel,
    pub(super) parents: Vec<TraceFilterLevel>,
    pub(super) pending: TraceFilterLeaf,
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) struct TraceFilterLevel {
    remaining: Vec<Expr>,
    emitted: Vec<Expr>,
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) struct TraceFilterLeaf {
    original: Expr,
    underlying: Vec<Expr>,
    field_index: usize,
    patterns: Vec<Expr>,
    pattern_index: usize,
    candidates: TraceFilterCandidates,
    candidate_index: usize,
    selected: Vec<Expr>,
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) enum TraceFilterCandidates {
    Single(Expr),
    Parts(Vec<Expr>),
}

pub(super) struct FilterWork {
    pub(super) mode: TraceOutput,
    pub(super) original_value: Expr,
    pub(super) pattern: Option<Expr>,
    pub(super) levels: Option<CasesLevelSpec>,
    pub(super) include_heads: bool,
    pub(super) current: LevelWork,
    pub(super) parents: Vec<LevelWork>,
}

pub(super) struct LevelWork {
    pub(super) remaining: IntoIter<Expr>,
    pub(super) emitted: Vec<Expr>,
}

pub(super) struct LeafWork {
    pub(super) original: Expr,
    pub(super) underlying: IntoIter<Expr>,
    pub(super) field_index: usize,
    pub(super) patterns: PatternWork,
    pub(super) pattern_index: usize,
    pub(super) candidates: CandidateWork,
    pub(super) candidate_index: usize,
    pub(super) selected: Vec<Expr>,
}

pub(super) enum PatternWork {
    Single,
    Parts(Vec<Expr>),
}

impl PatternWork {
    pub(super) fn pattern_count(&self) -> usize {
        match self {
            Self::Single => 1,
            Self::Parts(parts) => parts.len(),
        }
    }

    pub(super) fn pattern_at<'a>(&'a self, index: usize, original: Option<&'a Expr>) -> &'a Expr {
        match self {
            Self::Single => original.expect("single filter pattern"),
            Self::Parts(parts) => &parts[index],
        }
    }
}

pub(super) enum CandidateWork {
    Single(Option<Expr>),
    Parts(IntoIter<Expr>),
}

impl LevelWork {
    pub(super) fn from_items(items: Vec<Expr>) -> Self {
        Self {
            remaining: items.into_iter(),
            emitted: Vec::new(),
        }
    }

    fn freeze_level(self) -> TraceFilterLevel {
        TraceFilterLevel {
            remaining: self.remaining.collect(),
            emitted: self.emitted,
        }
    }
}

impl TraceFilterLevel {
    fn thaw_level(&self) -> LevelWork {
        LevelWork {
            remaining: self.remaining.clone().into_iter(),
            emitted: self.emitted.clone(),
        }
    }
}

impl FilterWork {
    #[cold]
    pub(super) fn freeze_filter(self, pending: LeafWork) -> TraceFilterState {
        let patterns = match pending.patterns {
            PatternWork::Single => {
                normalize_trace_patterns(self.pattern.as_ref().expect("pending pattern"))
            }
            PatternWork::Parts(patterns) => patterns,
        };
        TraceFilterState {
            mode: self.mode,
            original_value: self.original_value,
            pattern: self.pattern,
            levels: self.levels,
            include_heads: self.include_heads,
            current: self.current.freeze_level(),
            parents: self
                .parents
                .into_iter()
                .map(LevelWork::freeze_level)
                .collect(),
            pending: TraceFilterLeaf {
                original: pending.original,
                underlying: pending.underlying.collect(),
                field_index: pending.field_index,
                patterns,
                pattern_index: pending.pattern_index,
                candidates: match pending.candidates {
                    CandidateWork::Single(expr) => {
                        TraceFilterCandidates::Single(expr.expect("pending candidate"))
                    }
                    CandidateWork::Parts(parts) => TraceFilterCandidates::Parts(parts.collect()),
                },
                candidate_index: pending.candidate_index,
                selected: pending.selected,
            },
        }
    }
}

impl TraceFilterState {
    #[cold]
    pub(super) fn thaw_filter(&self) -> (FilterWork, LeafWork) {
        let work = FilterWork {
            mode: self.mode,
            original_value: self.original_value.clone(),
            pattern: self.pattern.clone(),
            levels: self.levels.clone(),
            include_heads: self.include_heads,
            current: self.current.thaw_level(),
            parents: self
                .parents
                .iter()
                .map(TraceFilterLevel::thaw_level)
                .collect(),
        };
        let leaf = LeafWork {
            original: self.pending.original.clone(),
            underlying: self.pending.underlying.clone().into_iter(),
            field_index: self.pending.field_index,
            patterns: PatternWork::Parts(self.pending.patterns.clone()),
            pattern_index: self.pending.pattern_index,
            candidates: match &self.pending.candidates {
                TraceFilterCandidates::Single(expr) => CandidateWork::Single(Some(expr.clone())),
                TraceFilterCandidates::Parts(parts) => {
                    CandidateWork::Parts(parts.clone().into_iter())
                }
            },
            candidate_index: self.pending.candidate_index,
            selected: self.pending.selected.clone(),
        };
        (work, leaf)
    }
}
