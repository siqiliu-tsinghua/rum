use super::*;

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) struct RuleLhsState {
    pub(super) stack: Vec<RuleLhsWork>,
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) struct RuleLhsWork {
    pub(super) original_head: Expr,
    pub(super) original_args: SharedArgs,
    pub(super) resolved_head: Option<Expr>,
    pub(super) prefix: Vec<Expr>,
    pub(super) next: usize,
    pub(super) policy: RuleLhsPolicy,
}

#[derive(Clone, Copy, rum_derive::FrameRootTrace)]
pub(super) enum RuleLhsPolicy {
    NamedInner,
    BlankArguments,
    FirstOperand,
    WrapperArguments,
    OrdinaryCall,
}

#[derive(Clone, Copy, rum_derive::FrameRootTrace)]
pub(super) enum RuleLhsDestination {
    Root,
    Head,
    Argument(usize),
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) struct RuleLhsAwait {
    pub(super) destination: RuleLhsDestination,
    pub(super) pending_expr: Expr,
}

impl RuleLhsState {
    pub(super) fn destination(&self) -> RuleLhsDestination {
        let Some(work) = self.stack.last() else {
            return RuleLhsDestination::Root;
        };
        if matches!(work.policy, RuleLhsPolicy::OrdinaryCall) && work.resolved_head.is_none() {
            RuleLhsDestination::Head
        } else {
            RuleLhsDestination::Argument(work.next)
        }
    }

    pub(super) fn pending_expr(&self, destination: RuleLhsDestination) -> &Expr {
        let work = self.stack.last().expect("a structural child has a parent");
        match destination {
            RuleLhsDestination::Head => &work.original_head,
            RuleLhsDestination::Argument(index) => &work.original_args[index],
            RuleLhsDestination::Root => unreachable!("root evaluations bypass the walk"),
        }
    }
}
