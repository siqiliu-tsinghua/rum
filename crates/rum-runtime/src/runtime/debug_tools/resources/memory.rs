use super::*;
use crate::runtime::suspension::trace::FrameRootVisitor;
use rum_core::IntValue;

mod session;

impl Session {
    pub(in crate::runtime) fn eval_byte_count(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::integer(BigInt::from(
            expr_memory_proxy_bytes(expr) as u64
        )))
    }

    pub(in crate::runtime) fn eval_memory_in_use(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        Ok(Expr::integer(BigInt::from(
            self.current_memory_proxy_bytes() as u64,
        )))
    }

    pub(in crate::runtime) fn eval_max_memory_used(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        // Sample the current proxy, then return the session high-water mark.
        let _ = self.current_memory_proxy_bytes();
        Ok(Expr::integer(BigInt::from(
            self.peak_memory_proxy_bytes.get() as u64,
        )))
    }

    pub(in crate::runtime::debug_tools::resources) fn current_memory_proxy_bytes(&self) -> usize {
        self.walk_session_memory_bytes()
    }

    pub(super) fn walked_evaluation_stack_bytes(&self) -> usize {
        self.evaluation_stack
            .iter()
            .map(render_evaluation_stack_entry_memory_proxy_bytes)
            .sum::<usize>()
            .saturating_add(vec_spare_capacity_bytes(&self.evaluation_stack))
    }

    #[cfg(test)]
    pub(in crate::runtime) fn current_memory_proxy_bytes_for_test(&self) -> usize {
        self.current_memory_proxy_bytes()
    }
}

pub(in crate::runtime) fn expr_memory_proxy_bytes(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(Atom::Integer(value)) => {
            std::mem::size_of::<Expr>().saturating_add(match value {
                IntValue::Small(_) => 0,
                IntValue::Big(value) => {
                    std::mem::size_of::<BigInt>().saturating_add(bigint_memory_proxy_bytes(value))
                }
            })
        }
        Expr::Atom(Atom::Rational(value)) => std::mem::size_of::<Expr>()
            .saturating_add(std::mem::size_of::<Rational>())
            .saturating_add(bigint_memory_proxy_bytes(value.numerator()))
            .saturating_add(bigint_memory_proxy_bytes(value.denominator())),
        Expr::Atom(Atom::String(value)) => {
            std::mem::size_of::<Expr>().saturating_add(value.capacity())
        }
        Expr::Atom(Atom::Symbol(symbol)) => {
            std::mem::size_of::<Expr>().saturating_add(symbol.context().len() + symbol.name().len())
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "ByteArray") && args.len() == 1 =>
        {
            // ByteArray physically owns base64 text in an Expr::String. Validate
            // the canonical payload to preserve the malformed-value fallback, but
            // charge that String's encoded capacity and Expr overhead just like any
            // other held string; decoded length belongs only to a native-byte
            // representation.
            let payload_bytes = args
                .first()
                .and_then(|arg| {
                    expr_as_string(arg)
                        .and_then(base64_decoded_len)
                        .map(|_| expr_memory_proxy_bytes(arg))
                })
                .unwrap_or_else(|| args.iter().map(expr_memory_proxy_bytes).sum::<usize>());
            std::mem::size_of::<Expr>()
                .saturating_add(expr_memory_proxy_bytes(head))
                .saturating_add(
                    args.capacity()
                        .saturating_sub(args.len())
                        .saturating_mul(std::mem::size_of::<Expr>()),
                )
                .saturating_add(payload_bytes)
        }
        Expr::Call { head, args } => {
            let mut total = std::mem::size_of::<Expr>()
                .saturating_add(expr_memory_proxy_bytes(head))
                .saturating_add(
                    args.capacity()
                        .saturating_sub(args.len())
                        .saturating_mul(std::mem::size_of::<Expr>()),
                );
            for arg in args {
                total = total.saturating_add(expr_memory_proxy_bytes(arg));
            }
            total
        }
    }
}

fn render_symbol_keyed_hash_map_memory_proxy_bytes<V, S>(values: &HashMap<Symbol, V, S>) -> usize {
    values
        .capacity()
        .saturating_mul(std::mem::size_of::<(Symbol, V)>())
        .saturating_add(
            values
                .keys()
                .map(|symbol| symbol.context().len().saturating_add(symbol.name().len()))
                .sum::<usize>(),
        )
}

fn render_symbol_hash_set_memory_proxy_bytes<S>(
    values: &std::collections::HashSet<Symbol, S>,
) -> usize {
    values
        .capacity()
        .saturating_mul(std::mem::size_of::<Symbol>())
        .saturating_add(
            values
                .iter()
                .map(|symbol| symbol.context().len().saturating_add(symbol.name().len()))
                .sum::<usize>(),
        )
}

fn render_symbol_call_effect_cache_memory_proxy_bytes(cache: &SymbolCallEffectCache) -> usize {
    cache
        .capacity()
        .saturating_mul(std::mem::size_of::<(
            SymbolCallEffectKey,
            CallEffectCacheEntry,
        )>())
        .saturating_add(
            cache
                .keys()
                .map(|key| {
                    key.symbol
                        .context()
                        .len()
                        .saturating_add(key.symbol.name().len())
                })
                .sum::<usize>(),
        )
}

fn render_call_stability_cache_memory_proxy_bytes(cache: &CallStabilityCache) -> usize {
    cache
        .capacity()
        .saturating_mul(std::mem::size_of::<(
            CallStabilityCacheKey,
            CallStabilityCacheEntry,
        )>())
        .saturating_add(
            cache
                .values()
                .map(|entry| {
                    expr_memory_proxy_bytes(entry.head.as_ref())
                        .saturating_add(render_shared_args_memory_proxy_bytes(&entry.args))
                })
                .sum::<usize>(),
        )
}

fn render_shared_args_memory_proxy_bytes(args: &SharedArgs) -> usize {
    args.iter()
        .map(expr_memory_proxy_bytes)
        .sum::<usize>()
        .saturating_add(
            args.capacity()
                .saturating_sub(args.len())
                .saturating_mul(std::mem::size_of::<Expr>()),
        )
}

fn render_installed_system_down_values_memory_proxy_bytes(
    values: &SymbolMap<Vec<(Expr, Expr)>>,
) -> usize {
    values
        .capacity()
        .saturating_mul(std::mem::size_of::<(Symbol, Vec<(Expr, Expr)>)>())
        .saturating_add(
            values
                .iter()
                .map(|(symbol, rules)| {
                    symbol
                        .context()
                        .len()
                        .saturating_add(symbol.name().len())
                        .saturating_add(
                            rules
                                .iter()
                                .map(|(lhs, rhs)| {
                                    expr_memory_proxy_bytes(lhs)
                                        .saturating_add(expr_memory_proxy_bytes(rhs))
                                })
                                .sum::<usize>(),
                        )
                        .saturating_add(vec_spare_capacity_bytes(rules))
                })
                .sum::<usize>(),
        )
}

fn render_dynamic_own_value_frame_memory_proxy_bytes(frame: &DynamicOwnValueFrame) -> usize {
    std::mem::size_of::<DynamicOwnValueFrame>()
        .saturating_add(frame.symbol.context().len())
        .saturating_add(frame.symbol.name().len())
        .saturating_add(frame.value.as_ref().map_or(0, expr_memory_proxy_bytes))
}

fn render_module_activation_memory_proxy_bytes(activation: &ModuleActivation) -> usize {
    std::mem::size_of::<ModuleActivation>()
        .saturating_add(render_symbol_vec_memory_proxy_bytes(&activation.gensyms))
}

fn render_exception_types_memory_proxy_bytes(
    exception_types: &BTreeMap<String, Option<String>>,
) -> usize {
    exception_types
        .iter()
        .map(|(name, parent)| {
            std::mem::size_of::<(String, Option<String>)>()
                .saturating_add(name.capacity())
                .saturating_add(parent.as_ref().map_or(0, String::capacity))
        })
        .sum()
}

fn render_source_unit_memory_proxy_bytes(
    unit: &crate::runtime::scoping::packages::SourceUnit,
) -> usize {
    std::mem::size_of::<crate::runtime::scoping::packages::SourceUnit>()
        .saturating_add(unit.package_context.capacity())
        .saturating_add(expr_memory_proxy_bytes(&unit.held_wrapper))
        .saturating_add(render_symbol_vec_memory_proxy_bytes(&unit.public))
        .saturating_add(render_symbol_vec_memory_proxy_bytes(&unit.package_scope))
        .saturating_add(
            unit.private_index
                .iter()
                .map(|(name, symbol)| {
                    std::mem::size_of::<(String, Symbol)>()
                        .saturating_add(name.capacity())
                        .saturating_add(symbol.context().len())
                        .saturating_add(symbol.name().len())
                })
                .sum::<usize>(),
        )
}

fn render_symbol_vec_memory_proxy_bytes(values: &Vec<Symbol>) -> usize {
    values
        .iter()
        .map(render_symbol_memory_proxy_bytes)
        .sum::<usize>()
        .saturating_add(vec_spare_capacity_bytes(values))
}

fn render_file_io_provider_memory_proxy_bytes(provider: &FileIoProvider) -> usize {
    arc_allocation_memory_proxy_bytes(provider.as_file_io())
}

fn render_entropy_provider_memory_proxy_bytes(provider: &EntropyProvider) -> usize {
    arc_allocation_memory_proxy_bytes(provider.as_entropy_source())
}

fn arc_allocation_memory_proxy_bytes<T: ?Sized>(value: &T) -> usize {
    2usize
        .saturating_mul(std::mem::size_of::<usize>())
        .saturating_add(std::mem::size_of_val(value))
}

fn render_definition_memory_proxy_bytes(definition: &SymbolDefinition) -> usize {
    std::mem::size_of::<SymbolDefinition>()
        .saturating_add(
            definition
                .own_value
                .as_ref()
                .map_or(0, expr_memory_proxy_bytes),
        )
        .saturating_add(
            definition
                .delayed_own_value
                .as_ref()
                .map_or(0, expr_memory_proxy_bytes),
        )
        .saturating_add(
            definition
                .own_value_guards()
                .iter()
                .map(expr_memory_proxy_bytes)
                .sum::<usize>(),
        )
        .saturating_add(
            definition
                .up_values
                .iter()
                .map(render_upvalue_memory_proxy_bytes)
                .sum::<usize>(),
        )
        .saturating_add(
            definition
                .down_values
                .iter()
                .map(render_downvalue_memory_proxy_bytes)
                .sum::<usize>(),
        )
        .saturating_add(definition.down_value_dispatch_memory_proxy_bytes())
        .saturating_add(
            definition
                .sub_values
                .iter()
                .map(render_subvalue_memory_proxy_bytes)
                .sum::<usize>(),
        )
}

fn render_upvalue_memory_proxy_bytes(value: &UpValue) -> usize {
    std::mem::size_of::<UpValue>()
        .saturating_add(expr_memory_proxy_bytes(&value.lhs))
        .saturating_add(expr_memory_proxy_bytes(&value.rhs))
        .saturating_add(
            value
                .guards
                .iter()
                .map(expr_memory_proxy_bytes)
                .sum::<usize>(),
        )
}

fn render_downvalue_memory_proxy_bytes(value: &DownValue) -> usize {
    std::mem::size_of::<DownValue>()
        .saturating_add(expr_memory_proxy_bytes(&value.lhs))
        .saturating_add(expr_memory_proxy_bytes(&value.rhs))
        .saturating_add(
            value
                .guards
                .iter()
                .map(expr_memory_proxy_bytes)
                .sum::<usize>(),
        )
}

fn render_subvalue_memory_proxy_bytes(value: &SubValue) -> usize {
    std::mem::size_of::<SubValue>()
        .saturating_add(expr_memory_proxy_bytes(&value.lhs))
        .saturating_add(expr_memory_proxy_bytes(&value.rhs))
        .saturating_add(
            value
                .guards
                .iter()
                .map(expr_memory_proxy_bytes)
                .sum::<usize>(),
        )
}

fn render_symbol_options_memory_proxy_bytes(options: &SymbolOptions) -> usize {
    std::mem::size_of::<SymbolOptions>().saturating_add(
        options
            .iter()
            .map(|(name, value, _)| {
                std::mem::size_of::<SymbolOptionEntry>().saturating_add(
                    name.to_string()
                        .len()
                        .saturating_add(expr_memory_proxy_bytes(value)),
                )
            })
            .sum::<usize>()
            .saturating_add(
                options
                    .capacity()
                    .saturating_sub(options.len())
                    .saturating_mul(std::mem::size_of::<SymbolOptionEntry>()),
            ),
    )
}

fn render_symbol_memory_proxy_bytes(symbol: &Symbol) -> usize {
    std::mem::size_of::<Symbol>()
        .saturating_add(symbol.context().len())
        .saturating_add(symbol.name().len())
}

fn render_symbol_generation_memory_proxy_bytes(entry: (&(String, String), &u64)) -> usize {
    let ((context, name), _generation) = entry;
    std::mem::size_of::<((String, String), u64)>()
        .saturating_add(context.capacity())
        .saturating_add(name.capacity())
}

fn render_context_frame_memory_proxy_bytes(frame: &ContextFrame) -> usize {
    std::mem::size_of::<ContextFrame>()
        .saturating_add(frame.previous_context.capacity().saturating_add(
            render_string_vec_memory_proxy_bytes(&frame.previous_context_path),
        ))
        .saturating_add(frame.opened_context.capacity())
}

pub(super) fn render_message_selector_memory_proxy_bytes(selector: &MessageSelector) -> usize {
    std::mem::size_of::<MessageSelector>().saturating_add(
        selector
            .symbol
            .capacity()
            .saturating_add(selector.tag.as_ref().map_or(0, String::capacity)),
    )
}

pub(super) fn render_runtime_message_memory_proxy_bytes(message: &RuntimeMessage) -> usize {
    std::mem::size_of::<RuntimeMessage>().saturating_add(
        message
            .symbol
            .capacity()
            .saturating_add(message.tag.capacity())
            .saturating_add(message.text.capacity()),
    )
}

pub(super) fn render_reap_scope_memory_proxy_bytes(scope: &ReapScope) -> usize {
    let patterns = scope
        .patterns
        .iter()
        .map(expr_memory_proxy_bytes)
        .sum::<usize>();
    let handler = scope.handler.as_ref().map_or(0, expr_memory_proxy_bytes);
    let buckets = scope
        .buckets_by_pattern
        .iter()
        .map(|group| {
            group
                .iter()
                .map(|bucket| {
                    expr_memory_proxy_bytes(&bucket.tag).saturating_add(
                        bucket
                            .values
                            .iter()
                            .map(expr_memory_proxy_bytes)
                            .sum::<usize>(),
                    )
                })
                .sum::<usize>()
        })
        .sum::<usize>();

    std::mem::size_of::<ReapScope>()
        .saturating_add(patterns)
        .saturating_add(handler)
        .saturating_add(buckets)
}

pub(super) fn render_monitor_scope_memory_proxy_bytes(scope: &MonitorScope) -> usize {
    std::mem::size_of::<MonitorScope>()
        .saturating_add(expr_memory_proxy_bytes(&scope.display_expr))
        .saturating_add(
            scope
                .evaluation_monitor
                .as_ref()
                .map_or(0, expr_memory_proxy_bytes),
        )
        .saturating_add(
            scope
                .step_monitor
                .as_ref()
                .map_or(0, expr_memory_proxy_bytes),
        )
        .saturating_add(scope.last_display.as_ref().map_or(0, String::capacity))
}

pub(super) fn render_trace_event_memory_proxy_bytes(event: &TraceEvent) -> usize {
    match event {
        TraceEvent::EvaluationPass { before, after } => std::mem::size_of::<TraceEvent>()
            .saturating_add(expr_memory_proxy_bytes(before))
            .saturating_add(expr_memory_proxy_bytes(after)),
        TraceEvent::DefinitionStored { symbol, value, .. } => std::mem::size_of::<TraceEvent>()
            .saturating_add(symbol.capacity())
            .saturating_add(expr_memory_proxy_bytes(value)),
        TraceEvent::RuleApplied {
            input,
            rule,
            result,
            ..
        } => std::mem::size_of::<TraceEvent>().saturating_add(
            expr_memory_proxy_bytes(input)
                .saturating_add(expr_memory_proxy_bytes(rule))
                .saturating_add(expr_memory_proxy_bytes(result)),
        ),
    }
}

fn bigint_memory_proxy_bytes(value: &BigInt) -> usize {
    value.to_signed_bytes_le().len().max(1)
}

fn base64_decoded_len(encoded: &str) -> Option<usize> {
    if encoded.is_empty() {
        return Some(0);
    }
    if encoded.len() % 4 != 0 {
        return None;
    }

    let mut padding = 0usize;
    for (index, byte) in encoded.as_bytes().iter().copied().enumerate() {
        let is_padding = byte == b'=';
        let is_base64 = matches!(
            byte,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/'
        );
        if !is_padding && !is_base64 {
            return None;
        }
        if is_padding {
            padding += 1;
            if index < encoded.len().saturating_sub(2) {
                return None;
            }
        } else if padding != 0 {
            return None;
        }
    }

    if padding > 2 {
        return None;
    }

    Some(encoded.len() / 4 * 3usize - padding)
}

fn render_input_request_memory_proxy_bytes(request: &RuntimeInputRequest) -> usize {
    std::mem::size_of::<RuntimeInputRequest>()
        .saturating_add(request.prompt.capacity())
        .saturating_add(1)
}

fn render_suspended_evaluation_memory_proxy_bytes(suspended: &SuspendedEvaluation) -> usize {
    let mut roots = SuspensionMemoryRootVisitor::default();
    suspended.trace_frame_roots(&mut roots);

    std::mem::size_of::<SuspendedEvaluation>()
        .saturating_add(
            suspended
                .active_stack
                .iter()
                .map(render_evaluation_stack_entry_memory_proxy_bytes)
                .sum::<usize>(),
        )
        .saturating_add(suspended.frame_storage_memory_proxy_bytes())
        .saturating_add(roots.bytes)
}

#[derive(Default)]
struct SuspensionMemoryRootVisitor {
    bytes: usize,
}

impl FrameRootVisitor for SuspensionMemoryRootVisitor {
    fn visit_expr(&mut self, expr: &Expr) {
        self.bytes = self.bytes.saturating_add(expr_memory_proxy_bytes(expr));
    }

    fn visit_symbol(&mut self, symbol: &Symbol) {
        self.bytes = self
            .bytes
            .saturating_add(render_symbol_memory_proxy_bytes(symbol));
    }
}

fn render_evaluation_stack_entry_memory_proxy_bytes(entry: &EvaluationStackEntry) -> usize {
    std::mem::size_of::<EvaluationStackEntry>().saturating_add(expr_memory_proxy_bytes(&entry.expr))
}

fn render_option_value_scope_memory_proxy_bytes(scope: &OptionValueScope) -> usize {
    std::mem::size_of::<OptionValueScope>()
        .saturating_add(scope.target().to_string().len())
        .saturating_add(render_symbol_options_memory_proxy_bytes(scope.explicit()))
}

fn vec_spare_capacity_bytes<T>(values: &Vec<T>) -> usize {
    values
        .capacity()
        .saturating_sub(values.len())
        .saturating_mul(std::mem::size_of::<T>())
}

fn render_string_vec_memory_proxy_bytes(values: &Vec<String>) -> usize {
    values
        .iter()
        .map(String::capacity)
        .sum::<usize>()
        .saturating_add(
            values
                .capacity()
                .saturating_mul(std::mem::size_of::<String>()),
        )
}

fn render_path_buf_vec_memory_proxy_bytes(values: &Vec<PathBuf>) -> usize {
    values
        .iter()
        .map(|path| path.as_os_str().len())
        .sum::<usize>()
        .saturating_add(
            values
                .capacity()
                .saturating_mul(std::mem::size_of::<PathBuf>()),
        )
}

fn render_string_vecdeque_memory_proxy_bytes(values: &VecDeque<String>) -> usize {
    values
        .iter()
        .map(String::capacity)
        .sum::<usize>()
        .saturating_add(
            values
                .capacity()
                .saturating_mul(std::mem::size_of::<String>()),
        )
}
