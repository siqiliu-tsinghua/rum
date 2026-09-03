use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::thread;
use std::time::Duration;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DialogConfig {
    body: Option<Expr>,
    prolog: Expr,
    symbols: Vec<(Symbol, Option<Expr>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DialogNormalizationError {
    InvalidOptions,
    InvalidSymbols,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum InputOptionsError {
    NonOption(Expr),
    InvalidOptionName(Expr),
    UnknownOption(Symbol),
}

impl DialogNormalizationError {
    fn message(self) -> &'static str {
        match self {
            Self::InvalidOptions => {
                "expected only DialogProlog and DialogSymbols option rules in the current first-pass subset"
            }
            Self::InvalidSymbols => {
                "expected DialogSymbols to be a list of symbols or symbol initializers in the current first-pass subset"
            }
        }
    }
}

impl Session {
    pub(super) fn eval_dialog(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let config = match self.compile_dialog_call(args) {
            Ok(config) => config,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Dialog", args, error.message()));
            }
        };

        let body = self.dialog_body_expr(config.body);
        let body = if is_system_symbol_name(&config.prolog, "Null") {
            body
        } else {
            Expr::call(system_expr("CompoundExpression"), vec![config.prolog, body])
        };
        let body = if config.symbols.is_empty() {
            body
        } else {
            Expr::call(
                system_expr("Block"),
                vec![render_dialog_symbols_bindings(&config.symbols), body],
            )
        };

        let body_result = self.evaluate(body);
        let outcome = crate::resume_or_suspend!(
            self,
            body_result,
            ok(value) => Ok(value),
            suspend => EvalDialogSuspensionFrame,
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_dialog_body(outcome),
        }
    }

    pub(super) fn eval_input_string(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let request = match self.compile_input_request("InputString", args, String::new()) {
            Ok(request) => request,
            Err(error) => return Ok(self.invalid_input_options_call("InputString", args, error)),
        };

        self.drive_input_request(request)
    }

    pub(super) fn eval_input(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let request = match self.compile_input_request("Input", args, "?".to_string()) {
            Ok(request) => request,
            Err(error) => return Ok(self.invalid_input_options_call("Input", args, error)),
        };

        let text_result = self.drive_input_request(request);
        let text = crate::resume_or_suspend!(
            self,
            text_result,
            ok(value) => Ok(value),
            suspend => EvalInputSuspensionFrame,
        )?;

        self.resume_input_after_text(text)
    }

    pub(super) fn eval_print(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let rendered = args
            .iter()
            .map(render_print_fragment)
            .collect::<Vec<_>>()
            .join("");
        self.push_print(rendered);
        Ok(system_expr("Null"))
    }

    pub(super) fn eval_pause(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [seconds] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Pause", args, 1));
        };

        let Some(duration) = expr_as_nonnegative_duration(seconds) else {
            let held_call = Expr::call(system_expr("Pause"), args.to_vec());
            return Ok(self.invalid_call_with_message(
                "Pause",
                args,
                "numnm",
                format!("Non-negative machine-sized number expected at position 1 in {held_call}."),
            ));
        };

        if !duration.is_zero() {
            thread::sleep(duration);
        }

        Ok(system_expr("Null"))
    }

    fn compile_dialog_call(
        &mut self,
        args: &[Expr],
    ) -> Result<DialogConfig, DialogNormalizationError> {
        let (body, option_exprs) = split_dialog_body_and_options(args);
        let explicit_options = if option_exprs.is_empty() {
            None
        } else {
            let options = self
                .compile_assigned_options_from_resolved(&Expr::call(
                    system_expr("List"),
                    option_exprs.to_vec(),
                ))
                .map_err(|_| DialogNormalizationError::InvalidOptions)?;
            Some(options)
        };

        if let Some(options) = explicit_options.as_ref() {
            for (name, _, _) in options.iter() {
                if !matches!(name.name(), "DialogProlog" | "DialogSymbols") {
                    return Err(DialogNormalizationError::InvalidOptions);
                }
            }
        }

        let prolog = self.dialog_option_raw_expr(explicit_options.as_ref(), "DialogProlog")?;
        let symbols_expr =
            self.dialog_option_raw_expr(explicit_options.as_ref(), "DialogSymbols")?;
        let symbols = match symbols_expr {
            Some((expr, _delayed)) => {
                parse_module_binding_list(&expr).ok_or(DialogNormalizationError::InvalidSymbols)?
            }
            None => Vec::new(),
        };

        Ok(DialogConfig {
            body: body.cloned(),
            prolog: prolog
                .map(|(expr, _delayed)| expr)
                .unwrap_or_else(|| system_expr("Null")),
            symbols,
        })
    }

    fn compile_input_request(
        &self,
        symbol_name: &str,
        args: &[Expr],
        default_prompt: String,
    ) -> Result<RuntimeInputRequest, InputOptionsError> {
        let (prompt, option_exprs) = split_input_prompt_and_options(args);
        let prompt = prompt.map_or(default_prompt, render_print_fragment);
        let explicit_options = self.parse_input_options(option_exprs)?;

        let mut request = match symbol_name {
            "Input" => RuntimeInputRequest::new(prompt).input(),
            _ => RuntimeInputRequest::new(prompt).input_string(),
        };
        let field_masked = self
            .raw_option_value_for_symbol(
                explicit_options.as_ref(),
                &Symbol::system(symbol_name),
                "FieldMasked",
            )
            .is_some_and(|(value, _)| is_system_symbol_name(&value, "True"));
        if field_masked {
            request = request.password();
        }

        Ok(request)
    }

    fn parse_input_options(
        &self,
        option_exprs: &[Expr],
    ) -> Result<Option<SymbolOptions>, InputOptionsError> {
        if option_exprs.is_empty() {
            return Ok(None);
        }

        let mut options = Vec::new();
        for option_expr in option_exprs {
            collect_input_option_expr(option_expr, &mut options)?;
        }

        self.compile_assigned_options_from_resolved(&Expr::call(system_expr("List"), options))
            .map(Some)
            .map_err(|_| {
                InputOptionsError::InvalidOptionName(Expr::call(
                    system_expr("List"),
                    option_exprs.to_vec(),
                ))
            })
    }

    fn drive_input_request(&mut self, request: RuntimeInputRequest) -> Result<Expr, EvalError> {
        if !self.queued_input_responses.is_empty() {
            if self.pending_suspension_generation().is_some() {
                self.abandon_pending_suspension(AbandonReason::SupersededOrQueuedInputDiscard);
            }
            let value = self
                .queued_input_responses
                .pop_front()
                .expect("a checked queued input response should remain available");
            return Ok(Expr::string(value));
        }

        self.begin_input_suspension(request.clone());

        Err(EvalError::InputRequired { request })
    }

    fn invalid_input_options_call(
        &mut self,
        symbol_name: &str,
        args: &[Expr],
        error: InputOptionsError,
    ) -> Expr {
        let held_call = Expr::call(system_expr(symbol_name), args.to_vec());
        match error {
            InputOptionsError::NonOption(offender) => self.invalid_call_with_message(
                symbol_name,
                args,
                "nonopt",
                format!(
                    "Options expected (instead of {offender}) beyond position 1 in {held_call}. An option must be a rule or a list of rules."
                ),
            ),
            InputOptionsError::InvalidOptionName(offender) => self.invalid_call_with_message(
                symbol_name,
                args,
                "optrs",
                format!(
                    "Option specification {offender} in {held_call} is not a rule for a symbol or string."
                ),
            ),
            InputOptionsError::UnknownOption(option_name) => self.invalid_call_with_message(
                symbol_name,
                args,
                "optx",
                format!("Unknown option {option_name} in {held_call}."),
            ),
        }
    }

    fn dialog_body_expr(&self, body: Option<Expr>) -> Expr {
        body.unwrap_or_else(|| Expr::call(system_expr("Input"), vec![Expr::string("")]))
    }

    fn dialog_option_raw_expr(
        &mut self,
        explicit: Option<&SymbolOptions>,
        option_name: &str,
    ) -> Result<Option<(Expr, bool)>, DialogNormalizationError> {
        Ok(self.raw_option_value_for_symbol(explicit, &Symbol::system("Dialog"), option_name))
    }

    fn finish_dialog_body(&mut self, outcome: Result<Expr, EvalError>) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Return { value }) => Ok(value),
            Err(error) => Err(error),
        }
    }

    fn resume_input_after_text(&mut self, text: Expr) -> Result<Expr, EvalError> {
        let Expr::Atom(Atom::String(source)) = text else {
            return Ok(text);
        };

        let syntax = parse_expr(&source)?;
        let expr = self.lower_syntax_expr(&syntax)?;
        let evaluated = self.evaluate(expr);
        crate::resume_or_suspend!(
            self,
            evaluated,
            ok(value) => Ok(value),
            suspend => ResumeInputAfterTextSuspensionFrame,
        )
    }
}

fn split_input_prompt_and_options(args: &[Expr]) -> (Option<&Expr>, &[Expr]) {
    match args {
        [] => (None, &[]),
        [first, ..] if is_input_option_start_expr(first) => (None, args),
        [first, rest @ ..] => (Some(first), rest),
    }
}

fn is_input_option_start_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args, .. }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            matches!(&args[0], Expr::Atom(Atom::Symbol(_)))
                || matches!(&args[0], Expr::Atom(Atom::String(_)))
        }
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => {
            !args.is_empty() && args.iter().all(is_input_option_start_expr)
        }
        _ => false,
    }
}

fn collect_input_option_expr(
    option_expr: &Expr,
    options: &mut Vec<Expr>,
) -> Result<(), InputOptionsError> {
    match option_expr {
        Expr::Call { head, args, .. }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            let option_name = input_option_name(&args[0])
                .ok_or_else(|| InputOptionsError::InvalidOptionName(option_expr.clone()))?;
            if option_name.name() != "FieldMasked" {
                return Err(InputOptionsError::UnknownOption(option_name));
            }
            options.push(option_expr.clone());
            Ok(())
        }
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => {
            for item in args {
                if let Err(error) = collect_input_option_expr(item, options) {
                    return Err(match error {
                        InputOptionsError::NonOption(_) => {
                            InputOptionsError::NonOption(option_expr.clone())
                        }
                        other => other,
                    });
                }
            }
            Ok(())
        }
        _ => Err(InputOptionsError::NonOption(option_expr.clone())),
    }
}

fn input_option_name(expr: &Expr) -> Option<Symbol> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.clone()),
        Expr::Atom(Atom::String(name)) => resolve_unqualified_symbol(name),
        _ => None,
    }
}

fn split_dialog_body_and_options(args: &[Expr]) -> (Option<&Expr>, &[Expr]) {
    match args {
        [] => (None, &[]),
        [first, ..] if is_dialog_option_expr(first) => (None, args),
        [first, rest @ ..] => (Some(first), rest),
    }
}

fn is_dialog_option_expr(expr: &Expr) -> bool {
    let Expr::Call { head, args, .. } = expr else {
        return false;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return false;
    }

    matches!(
        &args[0],
        Expr::Atom(Atom::Symbol(name)) if matches!(name.name(), "DialogProlog" | "DialogSymbols")
    ) || matches!(
        &args[0],
        Expr::Atom(Atom::String(name)) if matches!(name.as_str(), "DialogProlog" | "DialogSymbols")
    )
}

fn render_dialog_symbols_bindings(items: &[(Symbol, Option<Expr>)]) -> Expr {
    Expr::call(
        system_expr("List"),
        items
            .iter()
            .map(|(symbol, init)| match init {
                Some(init) => Expr::call(
                    system_expr("Rule"),
                    vec![Expr::Atom(Atom::Symbol(symbol.clone())), init.clone()],
                ),
                None => Expr::Atom(Atom::Symbol(symbol.clone())),
            })
            .collect(),
    )
}

fn render_print_fragment(expr: &Expr) -> String {
    match expr {
        Expr::Atom(Atom::String(value)) => value.clone(),
        other => render_input_like_expr(other),
    }
}

fn expr_as_nonnegative_duration(expr: &Expr) -> Option<Duration> {
    let seconds = match expr {
        Expr::Atom(Atom::Integer(value)) => value.to_string().parse::<f64>().ok()?,
        Expr::Atom(Atom::Rational(value)) => {
            let numerator = value.numerator().to_string().parse::<f64>().ok()?;
            let denominator = value.denominator().to_string().parse::<f64>().ok()?;
            numerator / denominator
        }
        _ => return None,
    };

    Duration::try_from_secs_f64(seconds).ok()
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalDialogSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for EvalDialogSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = (|| session.finish_dialog_body(outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalInputSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for EvalInputSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(text) => (|| session.resume_input_after_text(text))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeInputAfterTextSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for ResumeInputAfterTextSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
