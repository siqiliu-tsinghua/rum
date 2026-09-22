use super::*;

fn unique_string_prefix_parts(prefix: &str, current_context: &str) -> Option<(String, String)> {
    let (context, name_prefix) = if let Some((context, name_prefix)) = split_symbol_name(prefix) {
        if !is_valid_unique_context(context) {
            return None;
        }
        (normalize_context_name(context), name_prefix)
    } else {
        (current_context.to_string(), prefix)
    };

    is_valid_unique_symbol_prefix(name_prefix).then(|| (context, name_prefix.to_string()))
}

fn is_valid_unique_context(context: &str) -> bool {
    context
        .strip_suffix('`')
        .is_some_and(|body| !body.is_empty() && body.split('`').all(is_valid_unique_context_part))
}

fn is_valid_unique_context_part(part: &str) -> bool {
    is_valid_unique_symbol_prefix(part)
}

fn is_valid_unique_symbol_prefix(prefix: &str) -> bool {
    let mut chars = prefix.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '$')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '$')
}

fn generated_symbol_candidate(
    context: &str,
    name_prefix: &str,
    separator: &str,
    id: u64,
) -> Symbol {
    Symbol::new(context.to_string(), format!("{name_prefix}{separator}{id}"))
}

impl Session {
    pub(in crate::runtime) fn eval_unique(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (target, attributes) = match args {
            [] => (None, AttributeSet::empty()),
            [target] => (Some(target), AttributeSet::empty()),
            [target, attributes] => {
                let Some(attributes) = parse_attribute_set_expr(attributes) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Unique",
                        args,
                        "expected an attribute symbol or list of attribute symbols as the second argument",
                    ));
                };
                (Some(target), attributes)
            }
            _ => {
                return Ok(self.invalid_call_with_arg_range("Unique", args, 0, 2));
            }
        };

        let generated = match target {
            None => self
                .allocate_unique_string_symbol("$")
                .map(|symbol| Expr::Atom(Atom::Symbol(symbol))),
            Some(target) => self.allocate_unique_expr(target),
        };

        let Some(generated) = generated else {
            let offender = target.expect("invalid Unique target should have a target argument");
            return Ok(self.invalid_unique_arg(args, offender));
        };

        self.register_unique_generated_expr(&generated, attributes);
        Ok(generated)
    }

    pub(in crate::runtime) fn allocate_fresh_symbol(&mut self, symbol: &Symbol) -> Symbol {
        self.allocate_generated_symbol(symbol.context(), symbol.name(), "$")
    }

    fn allocate_unique_string_symbol(&mut self, prefix: &str) -> Option<Symbol> {
        let (context, name_prefix) = unique_string_prefix_parts(prefix, &self.current_context)?;
        Some(self.allocate_generated_symbol(&context, &name_prefix, ""))
    }

    fn allocate_unique_expr(&mut self, expr: &Expr) -> Option<Expr> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => {
                Some(Expr::Atom(Atom::Symbol(self.allocate_fresh_symbol(symbol))))
            }
            Expr::Atom(Atom::String(prefix)) => self
                .allocate_unique_string_symbol(prefix)
                .map(|symbol| Expr::Atom(Atom::Symbol(symbol))),
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
                if args.is_empty() {
                    return Some(Expr::call(system_expr("List"), Vec::new()));
                }

                if let Some(symbols) = args.iter().map(expr_as_symbol).collect::<Option<Vec<_>>>() {
                    let generated = self.allocate_fresh_symbol_batch(&symbols);
                    return Some(Expr::call(
                        system_expr("List"),
                        generated
                            .into_iter()
                            .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
                            .collect(),
                    ));
                }

                Some(Expr::call(
                    system_expr("List"),
                    args.iter()
                        .map(|arg| self.allocate_unique_expr(arg))
                        .collect::<Option<Vec<_>>>()?,
                ))
            }
            _ => None,
        }
    }

    fn allocate_fresh_symbol_batch(&mut self, symbols: &[&Symbol]) -> Vec<Symbol> {
        loop {
            self.next_generated_symbol_id += 1;
            let candidates = symbols
                .iter()
                .map(|symbol| {
                    generated_symbol_candidate(
                        symbol.context(),
                        symbol.name(),
                        "$",
                        self.next_generated_symbol_id,
                    )
                })
                .collect::<Vec<_>>();

            if candidates
                .iter()
                .all(|candidate| !self.generated_symbol_exists(candidate))
            {
                return candidates;
            }
        }
    }

    fn allocate_generated_symbol(
        &mut self,
        context: &str,
        name_prefix: &str,
        separator: &str,
    ) -> Symbol {
        loop {
            self.next_generated_symbol_id += 1;
            let candidate = generated_symbol_candidate(
                context,
                name_prefix,
                separator,
                self.next_generated_symbol_id,
            );

            if !self.generated_symbol_exists(&candidate) {
                return candidate;
            }
        }
    }

    fn register_unique_generated_expr(&mut self, expr: &Expr, attributes: AttributeSet) {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => {
                self.definitions.entry(symbol.clone()).or_default();
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::Definitions,
                );
                if !attributes.is_empty() {
                    let mut current = self.attribute_set_for_symbol(symbol);
                    for attribute in attributes.attributes() {
                        current.insert(attribute);
                    }
                    self.set_attribute_set_for_symbol(symbol.clone(), current);
                }
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
                for arg in args {
                    self.register_unique_generated_expr(arg, attributes);
                }
            }
            _ => {}
        }
    }

    fn generated_symbol_exists(&self, symbol: &Symbol) -> bool {
        self.symbol_exists(symbol)
            || self
                .message_definitions
                .keys()
                .any(|key| key.symbol == *symbol)
    }

    fn invalid_unique_arg(&mut self, args: &[Expr], arg: &Expr) -> Expr {
        self.invalid_call_with_message(
            "Unique",
            args,
            "usym",
            format!(
                "{} is not a symbol or a valid symbol name.",
                render_input_like_expr(arg)
            ),
        )
    }
}
