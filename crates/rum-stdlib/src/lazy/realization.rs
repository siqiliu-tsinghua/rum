//! Output-prefix and output-budget driver. Only sources charge source work.
use super::{
    PullBudget,
    machine::drive_pull,
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
    mut cursor: DriveCursor,
) -> Result<(), EvalError> {
    loop {
        if budget.reached(items.len()) {
            return Ok(());
        }
        let Some(item) = drive_pull(machine, session, ctx, cursor)? else {
            return Ok(());
        };
        items.push(item);
        cursor = DriveCursor::Down { at: 0 };
    }
}
