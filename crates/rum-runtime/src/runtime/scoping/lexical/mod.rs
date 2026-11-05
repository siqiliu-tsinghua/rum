use super::*;

mod block;

mod module;

mod with;

#[derive(Clone, Debug, PartialEq, Eq)]
struct WithBindingsConfig {
    items: Vec<(Symbol, Expr)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScopedBindingsConfig {
    items: Vec<(Symbol, Option<Expr>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScopingBindingNormalizationError {
    InvalidWithBindings,
    InvalidScopedBindings,
}

impl ScopingBindingNormalizationError {
    fn message(self) -> &'static str {
        match self {
            Self::InvalidWithBindings => {
                "expected a list of symbol assignments or rules as the first argument"
            }
            Self::InvalidScopedBindings => {
                "expected a list of symbols or symbol initializers as the first argument"
            }
        }
    }
}

impl Session {
    fn compile_with_call_from_raw_args(
        &self,
        args: &[Expr],
    ) -> Result<WithBindingsConfig, ScopingBindingNormalizationError> {
        Ok(WithBindingsConfig {
            items: parse_with_binding_list(&args[0])
                .ok_or(ScopingBindingNormalizationError::InvalidWithBindings)?,
        })
    }

    fn compile_scoped_call_from_raw_args(
        &self,
        args: &[Expr],
    ) -> Result<ScopedBindingsConfig, ScopingBindingNormalizationError> {
        Ok(ScopedBindingsConfig {
            items: parse_module_binding_list(&args[0])
                .ok_or(ScopingBindingNormalizationError::InvalidScopedBindings)?,
        })
    }
}
