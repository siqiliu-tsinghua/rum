use super::*;

impl Session {
    pub(in crate::runtime) fn eval_on(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if starts_with_assert_switch(args) {
            self.asserts_enabled = true;
            if args.len() == 1 {
                return Ok(system_expr("Null"));
            }
        }

        self.eval_message_switch("On", args, true)
    }

    pub(in crate::runtime) fn eval_off(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if starts_with_assert_switch(args) {
            self.asserts_enabled = false;
            if args.len() == 1 {
                return Ok(system_expr("Null"));
            }
        }

        self.eval_message_switch("Off", args, false)
    }

    fn eval_message_switch(
        &mut self,
        symbol: &str,
        args: &[Expr],
        enabled: bool,
    ) -> Result<Expr, EvalError> {
        let Some(selectors) = normalize_message_switch_selectors(args) else {
            return Ok(self.invalid_call_with_arg_form(
                symbol,
                args,
                "expected one or more symbols, MessageName expressions, \"symbol::tag\" strings, or lists of those",
            ));
        };

        if selectors.is_empty() {
            return Ok(self.invalid_call_with_message(
                symbol,
                args,
                "argx",
                "expected at least 1 argument",
            ));
        }

        if enabled {
            self.disabled_message_selectors
                .retain(|existing| !selectors.iter().any(|selector| selector.covers(existing)));
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::DisabledMessageSelectors,
            );
        } else {
            for selector in selectors {
                if !self.disabled_message_selectors.contains(&selector) {
                    self.push_accounted_disabled_message_selectors(selector);
                }
            }
        }

        Ok(system_expr("Null"))
    }
}

fn normalize_message_switch_selectors(args: &[Expr]) -> Option<Vec<MessageSelector>> {
    let mut selectors = Vec::new();

    for arg in args {
        selectors.extend(parse_message_selectors(arg)?);
    }

    Some(selectors)
}

fn starts_with_assert_switch(args: &[Expr]) -> bool {
    matches!(args.first(), Some(arg) if is_system_symbol_name(arg, "Assert"))
}
