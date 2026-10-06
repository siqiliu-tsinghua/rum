use super::*;
use crate::runtime::kernel_api::{
    FrameRootTrace, FrameRootVisitor, FrozenOptionReadContinuation, OptionReadContinuation,
};
use std::sync::Arc;

#[derive(Clone, Copy, FrameRootTrace)]
enum NamesOptionStage {
    Spelling,
    Aliases,
    IgnoreCase,
}

struct NamesOptionWorking<'a> {
    args: &'a [Expr],
    patterns: &'a [NamesPattern],
    pattern_index: Option<usize>,
    explicit_context: bool,
    explicit: Option<&'a SymbolOptions>,
    stage: NamesOptionStage,
}

#[derive(FrameRootTrace)]
struct NamesOptionRead {
    args: SharedArgs,
    pattern_index: Option<usize>,
    explicit_context: bool,
    explicit: Option<Arc<SymbolOptions>>,
    stage: NamesOptionStage,
}

pub(super) fn read_names_options(
    session: &mut Session,
    args: &[Expr],
    invocation: &NamesInvocation,
) -> Result<Expr, EvalError> {
    NamesOptionWorking {
        args,
        patterns: &invocation.patterns,
        pattern_index: invocation.pattern_index,
        explicit_context: invocation.explicit_context,
        explicit: invocation.explicit_options.as_ref(),
        stage: NamesOptionStage::Spelling,
    }
    .read(session)
}

impl NamesOptionWorking<'_> {
    fn read(self, session: &mut Session) -> Result<Expr, EvalError> {
        let name = match self.stage {
            NamesOptionStage::Spelling => "SpellingCorrection",
            NamesOptionStage::Aliases => "ResolveContextAliases",
            NamesOptionStage::IgnoreCase => "IgnoreCase",
        };
        session.runtime_option_expr_for_symbol(self.explicit, &Symbol::system("Names"), name, self)
    }
}

impl OptionReadContinuation<Option<Expr>> for NamesOptionWorking<'_> {
    type Frozen = NamesOptionRead;
    fn finish(mut self, session: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        let (name, supported, next) = match self.stage {
            NamesOptionStage::Spelling => {
                ("SpellingCorrection", "False", NamesOptionStage::Aliases)
            }
            NamesOptionStage::Aliases => (
                "ResolveContextAliases",
                "True",
                NamesOptionStage::IgnoreCase,
            ),
            NamesOptionStage::IgnoreCase => {
                return Ok(string_list_expr(session.names_matching_patterns(
                    self.patterns,
                    self.explicit_context,
                    names_ignore_case(value),
                )));
            }
        };
        if let Some(value) = value {
            if !is_system_symbol_name(&value, supported) {
                session.push_message(
                    "Names", "optval",
                    format!("Unsupported value {value} for option {name} in Names; only {supported} is currently supported."),
                );
                return Ok(Expr::call(system_expr("Names"), self.args.to_vec()));
            }
        }
        self.stage = next;
        self.read(session)
    }
    fn freeze(self) -> Self::Frozen {
        NamesOptionRead {
            args: SharedArgs::new(self.args.to_vec()),
            pattern_index: self.pattern_index,
            explicit_context: self.explicit_context,
            explicit: self.explicit.map(|options| Arc::new(options.clone())),
            stage: self.stage,
        }
    }
}

impl FrozenOptionReadContinuation<Option<Expr>> for NamesOptionRead {
    fn finish(&self, session: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        // The source selection was validated before any option ran. Rebuild only
        // pure pattern data; no option normalization or guest evaluation repeats.
        let patterns = match self.pattern_index {
            None => vec![NamesPattern::Wildcard("*".to_string())],
            Some(index) => {
                let source = &self.args[index];
                if let Some(pattern) = compile_names_pattern(source) {
                    vec![pattern]
                } else {
                    let Expr::Call { args, .. } = source else {
                        unreachable!("validated Names pattern list");
                    };
                    args.iter()
                        .map(|expr| compile_names_pattern(expr).expect("validated Names pattern"))
                        .collect()
                }
            }
        };
        NamesOptionWorking {
            args: &self.args,
            patterns: &patterns,
            pattern_index: self.pattern_index,
            explicit_context: self.explicit_context,
            explicit: self.explicit.as_deref(),
            stage: self.stage,
        }
        .finish(session, value)
    }
}

fn names_ignore_case(value: Option<Expr>) -> bool {
    value.is_some_and(|value| is_system_symbol_name(&value, "True"))
}
