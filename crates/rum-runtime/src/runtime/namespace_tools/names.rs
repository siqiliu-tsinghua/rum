mod options;
use super::regex::{RegexPattern, compile_names_regex_pattern, regex_pattern_matches};
use super::*;

struct NamesInvocation {
    patterns: Vec<NamesPattern>,
    pattern_index: Option<usize>,
    explicit_context: bool,
    explicit_options: Option<SymbolOptions>,
}

const EXTRA_SYSTEM_NAME_SYMBOLS: &[&str] = &[
    "Alternatives",
    "DialogProlog",
    "DialogSymbols",
    "Flat",
    "EvaluationMonitor",
    "Heads",
    "HoldAll",
    "HoldAllComplete",
    "HoldFirst",
    "HoldRest",
    "Listable",
    "NHoldAll",
    "NonThreadable",
    "NumericFunction",
    "OneIdentity",
    "OptionsPattern",
    "Orderless",
    "OrderlessPatternSequence",
    "PatternTest",
    "PatternSequence",
    "Protected",
    "ReadProtected",
    "RegularExpression",
    "ResolveContextAliases",
    "SequenceHold",
    "SpellingCorrection",
    "SubValuesHoldAll",
    "Temporary",
    "Verbatim",
    "StepMonitor",
    "StringExpression",
];

#[derive(Clone, Debug, PartialEq, Eq)]
enum NamesPattern {
    Wildcard(String),
    Regex(RegexPattern),
    Sequence(Vec<NamesPattern>),
    Alternatives(Vec<NamesPattern>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NamesNormalizationError {
    InvalidPattern,
}

impl NamesNormalizationError {
    fn message(self) -> &'static str {
        match self {
            Self::InvalidPattern => {
                "expected a string pattern, RegularExpression, StringExpression, Alternatives, or a list of them"
            }
        }
    }
}

impl Session {
    pub(in crate::runtime) fn eval_names(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let invocation = match self.compile_names_invocation(args) {
            Ok(invocation) => invocation,
            Err(error) => return Ok(self.invalid_call_with_arg_form("Names", args, error)),
        };
        options::read_names_options(self, args, &invocation)
    }

    pub(in crate::runtime) fn eval_symbol_name(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let [expr] = args.as_ref() else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(symbol) = expr_as_symbol(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::string(symbol.name().to_string()))
    }

    pub(in crate::runtime) fn eval_name_q(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let [expr] = args.as_ref() else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(name) = expr_as_string(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(system_expr(if self.exact_symbol_name_exists(name) {
            "True"
        } else {
            "False"
        }))
    }

    pub(in crate::runtime) fn resolve_names_matching_expr(
        &self,
        expr: &Expr,
    ) -> Option<Vec<Symbol>> {
        let (patterns, explicit_context) = self.compile_names_patterns(expr)?;
        Some(
            self.names_matching_patterns(&patterns, explicit_context, false)
                .into_iter()
                .filter_map(|name| self.resolve_symbol_name(&name, false))
                .collect(),
        )
    }

    fn compile_names_patterns(&self, expr: &Expr) -> Option<(Vec<NamesPattern>, bool)> {
        if let Some(pattern) = compile_names_pattern(expr) {
            return Some((vec![pattern], names_pattern_mentions_context(expr)));
        }

        let Expr::Call { head, args } = expr else {
            return None;
        };

        if !is_system_symbol_name(head, "List") {
            return None;
        }

        let patterns = args
            .iter()
            .map(compile_names_pattern)
            .collect::<Option<Vec<_>>>()?;

        Some((patterns, names_pattern_mentions_context(expr)))
    }

    fn names_matching_patterns(
        &self,
        patterns: &[NamesPattern],
        explicit_context: bool,
        ignore_case: bool,
    ) -> Vec<String> {
        let visible_contexts = self.visible_contexts();
        let sealed_symbols_visible = self.sealed_symbols_visible();
        let mut names = BTreeSet::new();

        for symbol in self.enumerated_names_symbols() {
            let sealed_private =
                crate::runtime::scoping::packages::symbol_is_sealed_private(&symbol);
            if sealed_private && !sealed_symbols_visible {
                continue;
            }

            let candidate = if explicit_context {
                symbol.fully_qualified_name()
            } else {
                if !sealed_private
                    && symbol.context() != SYSTEM_CONTEXT
                    && symbol.context() != GLOBAL_CONTEXT
                    && !visible_contexts.contains(symbol.context())
                {
                    continue;
                }
                symbol.qualified_name()
            };

            if patterns
                .iter()
                .any(|pattern| names_pattern_matches(pattern, &candidate, ignore_case))
            {
                names.insert(candidate);
            }
        }

        names.into_iter().collect()
    }

    fn compile_names_invocation(&self, args: &[Expr]) -> Result<NamesInvocation, String> {
        let (pattern_expr, option_args) = match args {
            [] => (None, &[][..]),
            [_single] if self.compile_names_options(args).is_ok() => (None, args),
            [pattern, rest @ ..] => (Some(pattern), rest),
        };

        let (patterns, explicit_context) = match pattern_expr {
            Some(pattern) => self.compile_names_patterns(pattern).ok_or_else(|| {
                NamesNormalizationError::InvalidPattern
                    .message()
                    .to_string()
            })?,
            None => (vec![NamesPattern::Wildcard("*".to_string())], false),
        };

        let explicit_options = if option_args.is_empty() {
            None
        } else {
            Some(self.compile_names_options(option_args)?)
        };

        if let Some(option) = explicit_options
            .as_ref()
            .and_then(first_unknown_names_option)
        {
            return Err(format!("unknown option {option} for Names"));
        }

        Ok(NamesInvocation {
            patterns,
            pattern_index: pattern_expr.map(|_| 0),
            explicit_context,
            explicit_options,
        })
    }

    fn compile_names_options(&self, args: &[Expr]) -> Result<SymbolOptions, String> {
        self.compile_assigned_options_from_resolved(&Expr::call(system_expr("List"), args.to_vec()))
            .map_err(|error| error.message().to_string())
    }

    fn enumerated_names_symbols(&self) -> BTreeSet<Symbol> {
        let mut symbols = BTreeSet::new();

        symbols.extend(self.builtins.keys().copied());

        for name in rum_core::CANONICAL_CORE_HEADS {
            symbols.insert(Symbol::system(*name));
        }

        for name in EXTRA_SYSTEM_NAME_SYMBOLS {
            symbols.insert(Symbol::system(*name));
        }

        symbols.extend(self.definitions.keys().cloned());
        symbols.extend(self.attributes.keys().cloned());
        symbols.extend(self.options.keys().cloned());
        symbols.extend(self.syntax_information.keys().cloned());
        symbols.extend(self.declared_stubs.iter().cloned());
        symbols.extend(
            self.message_definitions
                .keys()
                .map(|key| key.symbol.clone()),
        );

        symbols
    }

    fn visible_contexts(&self) -> BTreeSet<&str> {
        self.context_path
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(self.current_context.as_str()))
            .collect()
    }

    pub(in crate::runtime) fn wildcard_name_pattern_matches(
        &self,
        pattern: &str,
        candidate: &str,
        ignore_case: bool,
    ) -> bool {
        let haystack = candidate.chars().collect::<Vec<_>>();
        wildcard_match_ends(pattern, &haystack, 0, ignore_case)
            .into_iter()
            .any(|end| end == haystack.len())
    }

    fn exact_symbol_name_exists(&self, raw: &str) -> bool {
        if let Some((context, name)) = split_symbol_name(raw) {
            let context = normalize_context_name(context);
            let symbol = self.active_symbol_for_name(&context, name);
            return self.symbol_exists(&symbol);
        }

        self.resolve_symbol_name(raw, false).is_some()
    }
}

fn compile_names_pattern(expr: &Expr) -> Option<NamesPattern> {
    if let Expr::Atom(Atom::String(value)) = expr {
        return Some(NamesPattern::Wildcard(value.clone()));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if is_system_symbol_name(head, "RegularExpression") && args.len() == 1 {
        let Expr::Atom(Atom::String(value)) = &args[0] else {
            return None;
        };

        return compile_names_regex_pattern(value).map(NamesPattern::Regex);
    }

    if is_system_symbol_name(head, "StringExpression") && !args.is_empty() {
        return args
            .iter()
            .map(compile_names_pattern)
            .collect::<Option<Vec<_>>>()
            .map(NamesPattern::Sequence);
    }

    if is_system_symbol_name(head, "Alternatives") && !args.is_empty() {
        return args
            .iter()
            .map(compile_names_pattern)
            .collect::<Option<Vec<_>>>()
            .map(NamesPattern::Alternatives);
    }

    None
}

fn names_pattern_mentions_context(expr: &Expr) -> bool {
    match expr {
        Expr::Atom(Atom::String(value)) => value.contains('`'),
        Expr::Call { args, .. } => args.iter().any(names_pattern_mentions_context),
        _ => false,
    }
}

fn first_unknown_names_option(explicit: &SymbolOptions) -> Option<Symbol> {
    let defaults = default_system_options(&Symbol::system("Names"));
    explicit
        .iter()
        .find(|(name, _, _)| !defaults.contains_key(name) && !defaults.contains_name(name.name()))
        .map(|(name, _, _)| name.clone())
}

fn names_pattern_matches(pattern: &NamesPattern, candidate: &str, ignore_case: bool) -> bool {
    let haystack = candidate.chars().collect::<Vec<_>>();
    names_pattern_match_ends(pattern, &haystack, 0, ignore_case)
        .into_iter()
        .any(|end| end == haystack.len())
}

fn names_pattern_match_ends(
    pattern: &NamesPattern,
    haystack: &[char],
    start: usize,
    ignore_case: bool,
) -> Vec<usize> {
    match pattern {
        NamesPattern::Wildcard(pattern) => {
            wildcard_match_ends(pattern, haystack, start, ignore_case)
        }
        NamesPattern::Regex(pattern) => {
            if start == 0 && regex_pattern_matches(pattern, haystack, ignore_case) {
                vec![haystack.len()]
            } else {
                Vec::new()
            }
        }
        NamesPattern::Sequence(parts) => {
            let mut current = vec![start];

            for part in parts {
                let mut next = Vec::new();

                for end in current {
                    next.extend(names_pattern_match_ends(part, haystack, end, ignore_case));
                }

                next.sort_unstable();
                next.dedup();

                if next.is_empty() {
                    return next;
                }

                current = next;
            }

            current
        }
        NamesPattern::Alternatives(parts) => {
            let mut ends = Vec::new();

            for part in parts {
                ends.extend(names_pattern_match_ends(part, haystack, start, ignore_case));
            }

            ends.sort_unstable();
            ends.dedup();
            ends
        }
    }
}

fn wildcard_match_ends(
    pattern: &str,
    haystack: &[char],
    start: usize,
    ignore_case: bool,
) -> Vec<usize> {
    fn recur(
        pattern: &[char],
        pi: usize,
        haystack: &[char],
        hi: usize,
        ignore_case: bool,
        memo: &mut BTreeMap<(usize, usize), Vec<usize>>,
    ) -> Vec<usize> {
        if let Some(cached) = memo.get(&(pi, hi)) {
            return cached.clone();
        }

        let result = if pi == pattern.len() {
            vec![hi]
        } else if pattern[pi] == '*' {
            let mut ends = Vec::new();

            for next_hi in hi..=haystack.len() {
                ends.extend(recur(pattern, pi + 1, haystack, next_hi, ignore_case, memo));
            }

            ends.sort_unstable();
            ends.dedup();
            ends
        } else if hi < haystack.len()
            && wildcard_chars_match(pattern[pi], haystack[hi], ignore_case)
        {
            recur(pattern, pi + 1, haystack, hi + 1, ignore_case, memo)
        } else {
            Vec::new()
        };

        memo.insert((pi, hi), result.clone());
        result
    }

    let pattern = pattern.chars().collect::<Vec<_>>();
    let mut memo = BTreeMap::new();
    recur(&pattern, 0, haystack, start, ignore_case, &mut memo)
}

fn wildcard_chars_match(pattern: char, candidate: char, ignore_case: bool) -> bool {
    if ignore_case {
        pattern.eq_ignore_ascii_case(&candidate)
    } else {
        pattern == candidate
    }
}
