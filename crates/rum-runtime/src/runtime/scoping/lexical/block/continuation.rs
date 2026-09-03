use super::*;

fn tagged(name: &'static str, args: Vec<Expr>) -> Expr {
    Expr::call(system_expr(name), args)
}

fn tagged_args(expr: Expr, name: &'static str) -> std::vec::IntoIter<Expr> {
    let Expr::Call { head, args, .. } = expr else {
        panic!("unexpected {name} Block continuation payload")
    };
    assert!(
        is_system_symbol_name(&head, name),
        "unexpected {name} Block continuation tag: {head:?}"
    );
    args.into_iter()
}

fn symbol_expr(symbol: Symbol) -> Expr {
    Expr::Atom(Atom::Symbol(symbol))
}

fn symbol_from_expr(expr: Expr) -> Symbol {
    let Expr::Atom(Atom::Symbol(symbol)) = expr else {
        panic!("Block continuation symbol should be a symbol: {expr:?}")
    };
    symbol
}

fn string_expr(value: impl Into<String>) -> Expr {
    Expr::string(value)
}

fn string_from_expr(expr: Expr, field: &str) -> String {
    let Expr::Atom(Atom::String(value)) = expr else {
        panic!("Block continuation {field} should be a string: {expr:?}")
    };
    value
}

fn u64_expr(value: u64) -> Expr {
    string_expr(value.to_string())
}

fn u64_from_expr(expr: Expr, field: &str) -> u64 {
    string_from_expr(expr, field)
        .parse()
        .unwrap_or_else(|_| panic!("Block continuation {field} should be a u64"))
}

fn option_boxed_exprs_into_expr(values: Option<Box<[Expr]>>) -> Expr {
    values
        .map(|values| tagged("InternalContinuationBlockExprs", values.into_vec()))
        .into_continuation_expr()
}

fn option_boxed_exprs_from_expr(expr: Expr) -> Option<Box<[Expr]>> {
    Option::<Expr>::from_continuation_expr(expr).map(|values| {
        tagged_args(values, "InternalContinuationBlockExprs")
            .collect::<Vec<_>>()
            .into_boxed_slice()
    })
}

fn boxed_exprs_into_expr(values: Box<[Expr]>) -> Expr {
    tagged("InternalContinuationBlockExprs", values.into_vec())
}

fn boxed_exprs_from_expr(expr: Expr) -> Box<[Expr]> {
    tagged_args(expr, "InternalContinuationBlockExprs")
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn attributes_into_expr(attributes: AttributeSet) -> Expr {
    tagged(
        "InternalContinuationBlockAttributes",
        attributes.iter().map(Attribute::expr).collect(),
    )
}

fn attributes_from_expr(expr: Expr) -> AttributeSet {
    let mut attributes = AttributeSet::empty();
    for attribute_expr in tagged_args(expr, "InternalContinuationBlockAttributes") {
        let attribute = Attribute::from_expr(&attribute_expr)
            .unwrap_or_else(|| panic!("invalid Block attribute continuation: {attribute_expr:?}"));
        attributes.insert(attribute);
    }
    attributes
}

fn up_value_into_expr(value: UpValue) -> Expr {
    tagged(
        "InternalContinuationBlockUpValue",
        vec![
            value.lhs,
            value.rhs,
            value.delayed.into_continuation_expr(),
            value
                .rhs_may_splice_without_bindings
                .into_continuation_expr(),
            matcher_node_into_continuation_expr(value.matcher),
            u64_expr(value.matcher_attribute_revision),
            boxed_exprs_into_expr(value.guards),
        ],
    )
}

fn up_value_from_expr(expr: Expr) -> UpValue {
    let mut args = tagged_args(expr, "InternalContinuationBlockUpValue");
    let value = UpValue {
        lhs: args.next().expect("Block up-value stores its lhs"),
        rhs: args.next().expect("Block up-value stores its rhs"),
        delayed: bool::from_continuation_expr(
            args.next().expect("Block up-value stores its delayed flag"),
        ),
        rhs_may_splice_without_bindings: bool::from_continuation_expr(
            args.next()
                .expect("Block up-value stores its splice-without-bindings flag"),
        ),
        matcher: matcher_node_from_continuation_expr(
            args.next().expect("Block up-value stores its matcher"),
        ),
        matcher_attribute_revision: u64_from_expr(
            args.next()
                .expect("Block up-value stores its attribute revision"),
            "up-value attribute revision",
        ),
        guards: boxed_exprs_from_expr(args.next().expect("Block up-value stores its guards")),
    };
    debug_assert!(args.next().is_none());
    value
}

fn down_value_into_expr(value: DownValue) -> Expr {
    tagged(
        "InternalContinuationBlockDownValue",
        vec![
            value.lhs,
            value.rhs,
            value.delayed.into_continuation_expr(),
            value.lhs_is_ground.into_continuation_expr(),
            value
                .rhs_may_splice_without_bindings
                .into_continuation_expr(),
            matcher_node_into_continuation_expr(value.matcher),
            u64_expr(value.matcher_attribute_revision),
            boxed_exprs_into_expr(value.guards),
        ],
    )
}

fn down_value_from_expr(expr: Expr) -> DownValue {
    let mut args = tagged_args(expr, "InternalContinuationBlockDownValue");
    let value = DownValue {
        lhs: args.next().expect("Block down-value stores its lhs"),
        rhs: args.next().expect("Block down-value stores its rhs"),
        delayed: bool::from_continuation_expr(
            args.next()
                .expect("Block down-value stores its delayed flag"),
        ),
        lhs_is_ground: bool::from_continuation_expr(
            args.next()
                .expect("Block down-value stores its ground-lhs flag"),
        ),
        rhs_may_splice_without_bindings: bool::from_continuation_expr(
            args.next()
                .expect("Block down-value stores its splice-without-bindings flag"),
        ),
        matcher: matcher_node_from_continuation_expr(
            args.next().expect("Block down-value stores its matcher"),
        ),
        matcher_attribute_revision: u64_from_expr(
            args.next()
                .expect("Block down-value stores its attribute revision"),
            "down-value attribute revision",
        ),
        guards: boxed_exprs_from_expr(args.next().expect("Block down-value stores its guards")),
    };
    debug_assert!(args.next().is_none());
    value
}

fn sub_value_into_expr(value: SubValue) -> Expr {
    tagged(
        "InternalContinuationBlockSubValue",
        vec![
            value.lhs,
            value.rhs,
            value.delayed.into_continuation_expr(),
            value
                .rhs_may_splice_without_bindings
                .into_continuation_expr(),
            matcher_node_into_continuation_expr(value.matcher),
            u64_expr(value.matcher_attribute_revision),
            boxed_exprs_into_expr(value.guards),
        ],
    )
}

fn sub_value_from_expr(expr: Expr) -> SubValue {
    let mut args = tagged_args(expr, "InternalContinuationBlockSubValue");
    let value = SubValue {
        lhs: args.next().expect("Block sub-value stores its lhs"),
        rhs: args.next().expect("Block sub-value stores its rhs"),
        delayed: bool::from_continuation_expr(
            args.next()
                .expect("Block sub-value stores its delayed flag"),
        ),
        rhs_may_splice_without_bindings: bool::from_continuation_expr(
            args.next()
                .expect("Block sub-value stores its splice-without-bindings flag"),
        ),
        matcher: matcher_node_from_continuation_expr(
            args.next().expect("Block sub-value stores its matcher"),
        ),
        matcher_attribute_revision: u64_from_expr(
            args.next()
                .expect("Block sub-value stores its attribute revision"),
            "sub-value attribute revision",
        ),
        guards: boxed_exprs_from_expr(args.next().expect("Block sub-value stores its guards")),
    };
    debug_assert!(args.next().is_none());
    value
}

fn definition_into_expr(mut definition: SymbolDefinition) -> Expr {
    let own_value = definition.own_value.take();
    let delayed_own_value = definition.delayed_own_value.take();
    let own_value_guards = definition.own_value_guards.take();
    let up_values = std::mem::take(&mut definition.up_values)
        .into_iter()
        .map(up_value_into_expr)
        .collect();
    let down_values = std::mem::take(&mut definition.down_values)
        .into_iter()
        .map(down_value_into_expr)
        .collect();
    let sub_values = std::mem::take(&mut definition.sub_values)
        .into_iter()
        .map(sub_value_into_expr)
        .collect();
    tagged(
        "InternalContinuationBlockDefinition",
        vec![
            own_value.into_continuation_expr(),
            delayed_own_value.into_continuation_expr(),
            option_boxed_exprs_into_expr(own_value_guards),
            tagged("InternalContinuationBlockUpValues", up_values),
            tagged("InternalContinuationBlockDownValues", down_values),
            tagged("InternalContinuationBlockSubValues", sub_values),
        ],
    )
}

fn definition_from_expr(expr: Expr) -> SymbolDefinition {
    let mut args = tagged_args(expr, "InternalContinuationBlockDefinition");
    let own_value = Option::<Expr>::from_continuation_expr(
        args.next().expect("Block definition stores its own value"),
    );
    let delayed_own_value = Option::<Expr>::from_continuation_expr(
        args.next()
            .expect("Block definition stores its delayed own value"),
    );
    let own_value_guards = option_boxed_exprs_from_expr(
        args.next()
            .expect("Block definition stores its own-value guards"),
    );
    let up_values = tagged_args(
        args.next().expect("Block definition stores its up-values"),
        "InternalContinuationBlockUpValues",
    )
    .map(up_value_from_expr)
    .collect();
    let down_values = tagged_args(
        args.next()
            .expect("Block definition stores its down-values"),
        "InternalContinuationBlockDownValues",
    )
    .map(down_value_from_expr)
    .collect();
    let sub_values = tagged_args(
        args.next().expect("Block definition stores its sub-values"),
        "InternalContinuationBlockSubValues",
    )
    .map(sub_value_from_expr)
    .collect();
    debug_assert!(args.next().is_none());
    let mut definition = SymbolDefinition::default();
    definition.own_value = own_value;
    definition.delayed_own_value = delayed_own_value;
    definition.own_value_guards = own_value_guards;
    definition.up_values = up_values;
    definition.down_values = down_values;
    definition.sub_values = sub_values;
    definition
}

fn message_definition_into_expr(definition: MessageDefinition) -> Expr {
    tagged(
        "InternalContinuationBlockMessageDefinition",
        vec![
            definition.immediate.into_continuation_expr(),
            definition.delayed.into_continuation_expr(),
        ],
    )
}

fn message_definition_from_expr(expr: Expr) -> MessageDefinition {
    let mut args = tagged_args(expr, "InternalContinuationBlockMessageDefinition");
    let definition = MessageDefinition {
        immediate: Option::<Expr>::from_continuation_expr(
            args.next()
                .expect("Block message definition stores its immediate value"),
        ),
        delayed: Option::<Expr>::from_continuation_expr(
            args.next()
                .expect("Block message definition stores its delayed value"),
        ),
    };
    debug_assert!(args.next().is_none());
    definition
}

fn block_symbol_state_into_expr(state: BlockSymbolState) -> Expr {
    let messages = state
        .messages
        .into_iter()
        .map(|(key, definition)| {
            tagged(
                "InternalContinuationBlockMessage",
                vec![
                    symbol_expr(key.symbol),
                    string_expr(key.tag),
                    message_definition_into_expr(definition),
                ],
            )
        })
        .collect();
    tagged(
        "InternalContinuationBlockSymbolState",
        vec![
            state
                .definition
                .map(definition_into_expr)
                .into_continuation_expr(),
            state
                .attributes
                .map(attributes_into_expr)
                .into_continuation_expr(),
            state
                .options
                .map(SymbolOptions::into_continuation_expr)
                .into_continuation_expr(),
            state.syntax_information.into_continuation_expr(),
            tagged("InternalContinuationBlockMessages", messages),
        ],
    )
}

fn block_symbol_state_from_expr(expr: Expr) -> BlockSymbolState {
    let mut args = tagged_args(expr, "InternalContinuationBlockSymbolState");
    let state = BlockSymbolState {
        definition: Option::<Expr>::from_continuation_expr(
            args.next().expect("Block state stores its definition"),
        )
        .map(definition_from_expr),
        attributes: Option::<Expr>::from_continuation_expr(
            args.next().expect("Block state stores its attributes"),
        )
        .map(attributes_from_expr),
        options: Option::<Expr>::from_continuation_expr(
            args.next().expect("Block state stores its options"),
        )
        .map(SymbolOptions::from_continuation_expr),
        syntax_information: Option::<Expr>::from_continuation_expr(
            args.next()
                .expect("Block state stores its syntax information"),
        ),
        messages: tagged_args(
            args.next().expect("Block state stores its messages"),
            "InternalContinuationBlockMessages",
        )
        .map(|message| {
            let mut fields = tagged_args(message, "InternalContinuationBlockMessage");
            let key = MessageNameKey {
                symbol: symbol_from_expr(fields.next().expect("Block message stores its symbol")),
                tag: string_from_expr(
                    fields.next().expect("Block message stores its tag"),
                    "message tag",
                ),
            };
            let definition = message_definition_from_expr(
                fields.next().expect("Block message stores its definition"),
            );
            debug_assert!(fields.next().is_none());
            (key, definition)
        })
        .collect(),
    };
    debug_assert!(args.next().is_none());
    state
}

impl BlockSymbolState {
    pub(in crate::runtime) fn into_block_continuation_expr(self) -> Expr {
        block_symbol_state_into_expr(self)
    }

    pub(in crate::runtime) fn from_block_continuation_expr(expr: Expr) -> Self {
        block_symbol_state_from_expr(expr)
    }
}

fn block_state_map_into_expr(states: BTreeMap<Symbol, BlockSymbolState>) -> Expr {
    tagged(
        "InternalContinuationBlockStateMap",
        states
            .into_iter()
            .map(|(symbol, state)| {
                tagged(
                    "InternalContinuationBlockStateEntry",
                    vec![symbol_expr(symbol), block_symbol_state_into_expr(state)],
                )
            })
            .collect(),
    )
}

fn block_state_map_from_expr(expr: Expr) -> BTreeMap<Symbol, BlockSymbolState> {
    tagged_args(expr, "InternalContinuationBlockStateMap")
        .map(|entry| {
            let mut args = tagged_args(entry, "InternalContinuationBlockStateEntry");
            let symbol =
                symbol_from_expr(args.next().expect("Block state entry stores its symbol"));
            let state = block_symbol_state_from_expr(
                args.next().expect("Block state entry stores its state"),
            );
            debug_assert!(args.next().is_none());
            (symbol, state)
        })
        .collect()
}

fn block_items_into_expr(items: Vec<(Symbol, Option<Expr>)>) -> Expr {
    tagged(
        "InternalContinuationBlockItems",
        items
            .into_iter()
            .map(|(symbol, init)| {
                tagged(
                    "InternalContinuationBlockItem",
                    vec![symbol_expr(symbol), init.into_continuation_expr()],
                )
            })
            .collect(),
    )
}

fn block_items_from_expr(expr: Expr) -> Vec<(Symbol, Option<Expr>)> {
    tagged_args(expr, "InternalContinuationBlockItems")
        .map(|item| {
            let mut args = tagged_args(item, "InternalContinuationBlockItem");
            let symbol = symbol_from_expr(args.next().expect("Block item stores its symbol"));
            let init = Option::<Expr>::from_continuation_expr(
                args.next().expect("Block item stores its initializer"),
            );
            debug_assert!(args.next().is_none());
            (symbol, init)
        })
        .collect()
}

impl SuspendedSingleBlockScope {
    pub(super) fn into_continuation_expr(self, outcome: Result<Expr, EvalError>) -> Expr {
        let saved_state = Arc::try_unwrap(self.saved_state)
            .unwrap_or_else(|_| panic!("completed single Block state must be unique"));
        tagged(
            "InternalContinuationSingleBlockScope",
            vec![
                symbol_expr(self.symbol),
                self.body,
                block_symbol_state_into_expr(saved_state),
                usize_into_continuation_expr(self.dynamic_depth),
                result_into_continuation_expr(outcome),
            ],
        )
    }

    pub(super) fn from_continuation_expr(
        expr: Expr,
    ) -> (
        Symbol,
        Expr,
        BlockSymbolState,
        usize,
        Result<Expr, EvalError>,
    ) {
        let mut args = tagged_args(expr, "InternalContinuationSingleBlockScope");
        let values = (
            symbol_from_expr(args.next().expect("single Block stores its symbol")),
            args.next().expect("single Block stores its body"),
            block_symbol_state_from_expr(args.next().expect("single Block stores its saved state")),
            usize_from_continuation_expr(
                args.next().expect("single Block stores its dynamic depth"),
            ),
            result_from_continuation_expr(
                args.next()
                    .expect("single Block stores its evaluation result"),
            ),
        );
        debug_assert!(args.next().is_none());
        values
    }
}

impl SuspendedBlockScope {
    pub(super) fn into_continuation_expr(self, outcome: Result<Expr, EvalError>) -> Expr {
        let saved_state = Arc::try_unwrap(self.saved_state)
            .unwrap_or_else(|_| panic!("completed Block state must be unique"));
        tagged(
            "InternalContinuationBlockScope",
            vec![
                block_state_map_into_expr(saved_state),
                usize_into_continuation_expr(self.dynamic_depth),
                result_into_continuation_expr(outcome),
            ],
        )
    }

    pub(super) fn from_continuation_expr(
        expr: Expr,
    ) -> (
        BTreeMap<Symbol, BlockSymbolState>,
        usize,
        Result<Expr, EvalError>,
    ) {
        let mut args = tagged_args(expr, "InternalContinuationBlockScope");
        let values = (
            block_state_map_from_expr(args.next().expect("Block scope stores its saved state")),
            usize_from_continuation_expr(
                args.next().expect("Block scope stores its dynamic depth"),
            ),
            result_from_continuation_expr(
                args.next()
                    .expect("Block scope stores its evaluation result"),
            ),
        );
        debug_assert!(args.next().is_none());
        values
    }
}

impl SuspendedBlockInitializers {
    pub(super) fn into_continuation_expr(self, outcome: Result<Expr, EvalError>) -> Expr {
        let items = Arc::try_unwrap(self.items)
            .unwrap_or_else(|_| panic!("completed Block initializer items must be unique"));
        let saved_state = Arc::try_unwrap(self.saved_state)
            .unwrap_or_else(|_| panic!("completed Block initializer state must be unique"));
        tagged(
            "InternalContinuationBlockInitializers",
            vec![
                block_items_into_expr(items),
                self.body,
                block_state_map_into_expr(saved_state),
                usize_into_continuation_expr(self.dynamic_depth),
                usize_into_continuation_expr(self.index),
                result_into_continuation_expr(outcome),
            ],
        )
    }

    pub(super) fn from_continuation_expr(
        expr: Expr,
    ) -> (
        Vec<(Symbol, Option<Expr>)>,
        Expr,
        BTreeMap<Symbol, BlockSymbolState>,
        usize,
        usize,
        Result<Expr, EvalError>,
    ) {
        let mut args = tagged_args(expr, "InternalContinuationBlockInitializers");
        let values = (
            block_items_from_expr(args.next().expect("Block initializers store their items")),
            args.next().expect("Block initializers store their body"),
            block_state_map_from_expr(
                args.next()
                    .expect("Block initializers store their saved state"),
            ),
            usize_from_continuation_expr(
                args.next()
                    .expect("Block initializers store their dynamic depth"),
            ),
            usize_from_continuation_expr(
                args.next().expect("Block initializers store their cursor"),
            ),
            result_from_continuation_expr(
                args.next()
                    .expect("Block initializers store their evaluation result"),
            ),
        );
        debug_assert!(args.next().is_none());
        values
    }
}
