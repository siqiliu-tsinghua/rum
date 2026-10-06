use super::*;

impl CandidateWork {
    pub(super) fn current(&self) -> Option<&Expr> {
        match self {
            Self::Single(expr) => expr.as_ref(),
            Self::Parts(parts) => parts.as_slice().first(),
        }
    }

    fn take_candidate(&mut self) -> Option<Expr> {
        match self {
            Self::Single(expr) => expr.take(),
            Self::Parts(parts) => parts.next(),
        }
    }
}

#[inline(always)]
fn select_candidates(expr: Expr, work: &FilterWork) -> CandidateWork {
    let levels = match &work.levels {
        Some(levels) => levels.clone(),
        None if work.include_heads && work.pattern.is_some() => CasesLevelSpec::range(0, None),
        None => return CandidateWork::Single(Some(expr)),
    };
    let traversal = CasesTraversalConfig {
        levels,
        max_matches: if work.mode == TraceOutput::Detail && work.pattern.is_none() {
            Some(1)
        } else {
            None
        },
        include_heads: work.include_heads,
    };
    CandidateWork::Parts(collect_level_parts_owned(&expr, traversal, 0).into_iter())
}

pub(super) fn start_leaf(work: &mut FilterWork, original: Expr) -> Option<LeafWork> {
    // With no filter there is no guest match or unfinished leaf search.
    if work.pattern.is_none() && work.levels.is_none() {
        if work.mode == TraceOutput::Detail {
            work.current.emitted.push(original);
        } else {
            push_unique_trace_item(&mut work.current.emitted, original);
        }
        return None;
    }
    let (expr, underlying) = if work.mode == TraceOutput::Detail {
        let Some(exprs) = trace_detail_underlying_exprs(&original) else {
            if work.pattern.is_none() && work.levels.is_none() {
                work.current.emitted.push(original);
            }
            return None;
        };
        let underlying = exprs.into_iter();
        let expr = underlying.as_slice().first()?.clone();
        (expr, underlying)
    } else {
        let Some(expr) = unwrap_trace_hold(&original) else {
            push_unique_trace_item(&mut work.current.emitted, original);
            return None;
        };
        (expr, Vec::new().into_iter())
    };
    let patterns = match &work.pattern {
        Some(pattern) => match pattern {
            Expr::Call { head, .. } if is_system_symbol_name(head, "List") => {
                PatternWork::Parts(normalize_trace_patterns(pattern))
            }
            _ => PatternWork::Single,
        },
        None => PatternWork::Parts(Vec::new()),
    };
    Some(LeafWork {
        original,
        underlying,
        field_index: 0,
        patterns,
        pattern_index: 0,
        candidates: select_candidates(expr, work),
        candidate_index: 0,
        selected: Vec::new(),
    })
}

impl LeafWork {
    // A true detail or whole-leaf match finishes the original item immediately.
    // Otherwise advance the pattern, or consume exactly one original part.
    #[inline(always)]
    pub(super) fn apply_match(&mut self, matched: bool, detail: bool) -> bool {
        if matched && (detail || matches!(self.candidates, CandidateWork::Single(_))) {
            return true;
        }
        self.pattern_index += 1;
        if matched || self.pattern_index >= self.patterns.pattern_count() {
            let candidate = self.candidates.take_candidate().expect("pending candidate");
            if matched {
                push_unique_trace_expr(&mut self.selected, candidate);
            }
            self.candidate_index += 1;
            self.pattern_index = 0;
        }
        false
    }

    fn next_field(&mut self, work: &FilterWork) -> bool {
        if work.mode != TraceOutput::Detail {
            return false;
        }
        self.underlying.next();
        self.field_index += 1;
        let Some(expr) = self.underlying.as_slice().first() else {
            return false;
        };
        self.candidates = select_candidates(expr.clone(), work);
        self.candidate_index = 0;
        self.pattern_index = 0;
        true
    }
}

impl Session {
    pub(super) fn match_trace_leaf(
        &mut self,
        work: &FilterWork,
        leaf: &mut LeafWork,
    ) -> Result<bool, EvalError> {
        loop {
            let Some(candidate) = leaf.candidates.current() else {
                if leaf.next_field(work) {
                    continue;
                }
                return Ok(false);
            };
            let matched = if work.pattern.is_none() {
                true
            } else if leaf.patterns.pattern_count() == 0 {
                false
            } else {
                // The driver owns work and leaf across this one may-suspend edge.
                self.match_expr_owned(
                    leaf.patterns
                        .pattern_at(leaf.pattern_index, work.pattern.as_ref())
                        .clone(),
                    candidate.clone(),
                    SymbolExprMap::default(),
                )?
                .matched
            };
            if leaf.apply_match(matched, work.mode == TraceOutput::Detail) {
                return Ok(true);
            }
        }
    }
}

#[inline(always)]
pub(super) fn finish_leaf(work: &mut FilterWork, leaf: LeafWork, whole_match: bool) {
    if whole_match {
        // Single candidates are exactly the original held expression. Keeping
        // that Hold also preserves its sharing without a temporary output Vec.
        if work.mode == TraceOutput::Detail {
            work.current.emitted.push(leaf.original);
        } else {
            push_unique_trace_item(&mut work.current.emitted, leaf.original);
        }
    } else if work.mode != TraceOutput::Detail {
        for candidate in leaf.selected {
            push_unique_trace_item(
                &mut work.current.emitted,
                Expr::call(system_expr("Hold"), vec![candidate]),
            );
        }
    }
}
