//! Decode resumed cursors before the shared indexed output loop.
use super::{
    PullBudget,
    machine::{drive_machine, exhausted},
    state::{DriveCursor, PullContext, PullMachine},
};
use rum_core::Expr;
use rum_runtime::kernel_api::{EvalError, Session};

pub(super) fn drive_items(
    machine: &mut PullMachine,
    session: &mut Session,
    ctx: &mut PullContext,
    items: &mut Vec<Expr>,
    budget: PullBudget,
    cursor: DriveCursor,
) -> Result<(), EvalError> {
    if budget.reached(items.len()) {
        return Ok(());
    }
    let (boundary, carried) = match cursor {
        DriveCursor::Down { at } => (at, None),
        DriveCursor::Up { boundary, result } => match result {
            Some(value) => (boundary, Some(value)),
            None => return exhausted(machine, boundary),
        },
    };
    drive_machine(machine, session, ctx, items, budget, boundary, carried)
}
