use rum_core::{Expr, SharedArgs};
use rum_runtime::kernel_api::{
    AbandonmentContext, BuiltinSpec, FrameAbandon, FrameAbandonAction, FrameAction, FrameRootTrace,
    FrameRootVisitor, MemoryGrowthContract, PureFrameAbandon, SuspensionFrame,
};
use rum_runtime::{EvalError, Session};
use std::sync::Arc;

// The only test-frame population. These builtins are never installed by
// install_core and only exist in falsifier sessions.
#[derive(FrameRootTrace, PureFrameAbandon)]
pub struct OracleFoldFrame {
    operator: Expr,
    items: Vec<Expr>,
    prefix: Vec<Expr>,
    next: usize,
    drop_prefix: bool,
}
impl SuspensionFrame for OracleFoldFrame {
    fn resume(self: Arc<Self>, s: &mut Session, outcome: Result<Expr, EvalError>) -> FrameAction {
        FrameAction::complete(outcome.and_then(|value| {
            let mut prefix = self.prefix.clone();
            prefix.push(value);
            fold(
                s,
                &self.operator,
                &self.items,
                self.next,
                prefix,
                self.drop_prefix,
                false,
            )
        }))
    }
}
fn list(items: Vec<Expr>) -> Expr {
    Expr::call(Expr::system_symbol("List"), items)
}
fn fold(
    s: &mut Session,
    operator: &Expr,
    items: &[Expr],
    start: usize,
    mut prefix: Vec<Expr>,
    drop_prefix: bool,
    false_tail: bool,
) -> Result<Expr, EvalError> {
    for (index, item) in items.iter().enumerate().skip(start) {
        let call = Expr::call(operator.clone(), vec![item.clone()]);
        match s.evaluate(call) {
            Ok(value) => prefix.push(value),
            Err(error @ EvalError::InputRequired { .. }) => {
                if !false_tail {
                    let frame = OracleFoldFrame {
                        operator: operator.clone(),
                        items: items.to_vec(),
                        prefix: if drop_prefix { vec![] } else { prefix },
                        next: index + 1,
                        drop_prefix,
                    };
                    rum_runtime::resume_or_suspend!(insert s, frame);
                }
                return Err(error);
            }
            Err(error) => return Err(error),
        }
    }
    Ok(list(prefix))
}
fn handler(s: &mut Session, head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let Expr::Call { args: items, .. } = &args[1] else {
        panic!("fixture requires a list")
    };
    let name = head.to_string();
    fold(
        s,
        &args[0],
        items,
        0,
        vec![],
        name == "OracleDropPrefix",
        name == "OracleFalseTail",
    )
}
pub fn install(s: &mut Session) {
    for name in ["OracleGoodFold", "OracleDropPrefix", "OracleFalseTail"] {
        s.register_builtin(BuiltinSpec::ordinary(
            name,
            "Test-only suspension falsifier",
            handler,
            MemoryGrowthContract::Delegated,
        ));
    }
}
