The public API boundary is executable from an outside crate. Plain registration
does not grant admission; installing a host declaration asserts trust outside the
repository certificate domain.

```rust
// An outside crate, using only the public registration and consumer APIs.
use rum_core::Expr;
use rum_runtime::kernel_api::{
    EvalError, FrameRootTrace, FrameRootVisitor, OptionReentry, OptionReentryDeclaration,
    RetainedOptions, Session, system_expr,
};

#[derive(Clone, Copy, FrameRootTrace)]
struct Outside;

impl OptionReentry for Outside {
    fn symbol_name(self) -> &'static str {
        "Outside544"
    }
    fn resume(
        self,
        _: &mut Session,
        _: &Expr,
        _: &[Expr],
        options: RetainedOptions<'_>,
    ) -> Result<Expr, EvalError> {
        assert_eq!(options.len(), 1);
        Ok(options[0].clone())
    }
}

const HOST_ASSERTION: &[OptionReentryDeclaration] =
    &[OptionReentryDeclaration::of::<Outside>("host::Outside544")];

fn main() {
    let mut session = Session::new();
    let head = system_expr("Outside544");
    let args = [Expr::small_integer(544)];
    let run = |session: &mut Session| {
        session
            .merged_option_exprs_for_system_symbol(Outside, &head, &[], &args)
            .unwrap()
    };
    session.register_option_reentry::<Outside>();
    assert_eq!(
        session.option_reentry_registration::<Outside>(),
        (true, None)
    );
    assert_eq!(run(&mut session), system_expr("$Failed"));
    assert_eq!(session.messages().last().unwrap().tag, "optreentry");
    // Rust visibility cannot make this downstream installer seam unforgeable.
    // This deliberately leaves the repository declaration's lint domain.
    session.install_option_reentry_domain(HOST_ASSERTION);
    assert_eq!(
        session.option_reentry_registration::<Outside>(),
        (true, Some("host::Outside544"))
    );
    assert_eq!(run(&mut session), args[0]);
    session.install_option_reentry_domain(&[]);
    assert_eq!(run(&mut session), system_expr("$Failed"));
}
```
