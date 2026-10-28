use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use crate::runtime::suspension::{FrameAction, SuspensionFrame};
use std::mem::size_of;
use std::sync::Arc;

fn dispatch_size(name: &str) -> usize {
    SymbolDefinition::occurrence_dispatch_layout()
        .into_iter()
        .find(|(label, _)| *label == name)
        .unwrap()
        .1
}

#[test]
fn dispatch_metadata_and_lazy_index_have_exact_separate_charges() {
    let mut session = Session::empty();
    let key = Symbol::from_name("indexed");
    let mut value = SymbolDefinition::default();
    for i in 0..32 {
        let lhs = Expr::call(
            Expr::Atom(Atom::Symbol(key.clone())),
            vec![Expr::small_integer(i)],
        );
        value.down_values.push(DownValue {
            matcher: MatcherNode::Exact(lhs.clone()),
            lhs,
            rhs: Expr::small_integer(i),
            delayed: false,
            lhs_is_ground: true,
            rhs_may_splice_without_bindings: false,
            matcher_attribute_revision: 0,
            guards: Box::new([]),
        });
    }
    value.note_down_values_rebuilt(&key);
    assert_eq!(value.down_value_shallow_signatures().unwrap().len(), 32);
    let metadata = dispatch_size("DownValueDispatchMetadata");
    assert_eq!(value.occurrence_dispatch_bytes(), Bytes::Exact(metadata));
    let expr_bytes = 3 * size_of::<Expr>() + key.context().len() + key.name().len();
    let expected_rules = 32 * (size_of::<DownValue>() + expr_bytes + size_of::<Expr>());
    assert_eq!(
        definition(&value),
        Bytes::Exact(size_of::<SymbolDefinition>() + expected_rules + metadata)
    );
    session.definitions.insert(key.clone(), value);
    let before = assert_model(&session).field("definitions");
    assert_eq!(
        before,
        Bytes::Exact(
            key.to_string().len() + size_of::<SymbolDefinition>() + expected_rules + metadata
        )
    );
    let value = session.definitions.get(&key).unwrap();
    assert_eq!(
        value.occurrence_dispatch_bytes(),
        Bytes::Exact(metadata),
        "observing must not initialize the index"
    );
    value
        .start_indexed_down_value_scan(&value.down_values[0].lhs, 0)
        .unwrap();
    let index = dispatch_size("DownValueDispatchIndex")
        + HashMap::<u64, usize>::with_capacity(32).capacity() * size_of::<(u64, usize)>()
        + 32 * size_of::<usize>();
    assert_eq!(
        value.occurrence_dispatch_bytes(),
        Bytes::Exact(metadata + index)
    );
    assert_eq!(
        value.down_value_dispatch_memory_proxy_bytes(),
        metadata + index
    );
    assert_eq!(
        assert_model(&session).field("definitions"),
        before.add(Bytes::Exact(index))
    );
}

#[test]
fn tagged_dispatch_indexes_are_charged_for_both_value_tables() {
    let mut session = Session::empty();
    let mut value = SymbolDefinition::default();
    for i in 0..32 {
        let lhs = Expr::small_integer(i);
        value.up_values.push(UpValue {
            lhs: lhs.clone(),
            rhs: lhs.clone(),
            delayed: false,
            rhs_may_splice_without_bindings: false,
            matcher: MatcherNode::Exact(lhs.clone()),
            matcher_attribute_revision: 0,
            guards: Box::new([]),
        });
        value.sub_values.push(SubValue {
            lhs: lhs.clone(),
            rhs: lhs.clone(),
            delayed: false,
            rhs_may_splice_without_bindings: false,
            matcher: MatcherNode::Exact(lhs),
            matcher_attribute_revision: 0,
            guards: Box::new([]),
        });
    }
    assert_eq!(value.occurrence_dispatch_bytes(), Bytes::Exact(0));
    value.prepare_up_value_store(&Expr::small_integer(0));
    let base = dispatch_size("DownValueDispatchMetadata") + dispatch_size("DownValueDispatchIndex");
    let tagged = dispatch_size("TaggedValueStoreIndex")
        + HashMap::<u64, usize>::with_capacity(32).capacity() * size_of::<(u64, usize)>()
        + 32 * size_of::<usize>();
    assert_eq!(
        value.occurrence_dispatch_bytes(),
        Bytes::Exact(base + tagged)
    );
    value.prepare_sub_value_store(&Expr::small_integer(0));
    assert_eq!(
        value.occurrence_dispatch_bytes(),
        Bytes::Exact(base + 2 * tagged)
    );
    assert_eq!(
        value.down_value_dispatch_memory_proxy_bytes(),
        base + 2 * tagged
    );
    let rules = 32 * (size_of::<UpValue>() + size_of::<SubValue>() + 4 * size_of::<Expr>());
    assert_eq!(
        definition(&value),
        Bytes::Exact(size_of::<SymbolDefinition>() + rules + base + 2 * tagged)
    );
    session
        .definitions
        .insert(Symbol::from_name("tagged"), value);
    assert_model(&session);
}

#[test]
fn association_indexes_keep_stale_storage_until_the_owner_prunes_it() {
    let mut session = Session::empty();
    let association = render_association(
        (0..32)
            .map(|i| (Expr::small_integer(i), Expr::small_integer(i)))
            .collect(),
    );
    for _ in 0..2 {
        session.apply_association_to_args(&association, &[Expr::small_integer(1)]);
    }
    let (capacity, indexes) = session.association_lookup_cache.occurrence_capacities();
    assert_eq!(indexes.len(), 1);
    let entry_size = AssociationLookupCache::occurrence_layout()[1].1;
    let expected = capacity * entry_size
        + indexes[0].0 * size_of::<(u64, usize)>()
        + indexes[0].1 * size_of::<usize>();
    let work = session.association_lookup_work();
    let before = assert_model(&session).field("association_lookup_cache");
    assert_eq!(before, Bytes::Exact(expected));
    drop(association);
    assert_eq!(
        assert_model(&session).field("association_lookup_cache"),
        before
    );
    assert_eq!(session.association_lookup_work(), work);
    assert_eq!(
        session.association_lookup_cache.occurrence_capacities(),
        (capacity, indexes)
    );
}

#[test]
fn call_cache_roots_are_additional_occurrences_of_retained_values() {
    let mut session = Session::empty();
    let head = SharedExpr::new(Expr::small_integer(0));
    let mut values = Vec::with_capacity(5);
    values.push(Expr::string(reserved("value", 71)));
    let args = SharedArgs::new(values);
    session.once_values.insert(
        String::from("call"),
        Expr::call_shared(head.clone(), args.clone()),
    );
    session.call_stability_cache.insert(
        CallStabilityCacheKey::new(&head, &args),
        CallStabilityCacheEntry {
            generation: 0,
            head,
            args,
            stable: true,
            bloom: StabilityBloom::default(),
            dirty_snapshot: StabilityBloom::default(),
        },
    );
    let payload = 6 * size_of::<Expr>() + 71;
    let expected = session.call_stability_cache.capacity()
        * size_of::<(CallStabilityCacheKey, CallStabilityCacheEntry)>()
        + payload;
    let model = assert_model(&session);
    assert_eq!(model.field("call_stability_cache"), Bytes::Exact(expected));
    assert_eq!(
        model.field("once_values"),
        Bytes::Exact(4 + size_of::<Expr>() + payload)
    );
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct PayloadFrame {
    roots: Vec<Expr>,
    bare_symbol: Symbol,
    outcome: Result<Expr, EvalError>,
    omitted_text: String,
}

impl SuspensionFrame for PayloadFrame {
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(outcome)
    }
}

#[test]
fn suspended_frames_charge_storage_repeated_roots_and_exception_payloads() {
    let mut session = Session::empty();
    let root = Expr::call(
        Expr::small_integer(0),
        vec![Expr::string(reserved("root", 67))],
    );
    let root_bytes = 3 * size_of::<Expr>() + 67;
    session.evaluation_stack = vec![
        EvaluationStackEntry {
            expr: root.clone(),
            epoch: 0
        };
        3
    ];
    session.begin_input_suspension(RuntimeInputRequest::new(reserved("prompt", 73)));
    session.pending_request = Some(PendingRequest::Resumable(PendingReport {
        mark: crate::runtime::model::OutputMark::START,
        reported: ReportedOutput::at(crate::runtime::model::OutputMark::START),
    }));
    let bare_symbol = Symbol::from_name("frameRoot");
    let symbol_bytes = size_of::<Symbol>() + bare_symbol.context().len() + bare_symbol.name().len();
    let mut roots = Vec::with_capacity(11);
    roots.extend([root.clone(), root.clone()]);
    session.insert_suspension_frame(PayloadFrame {
        roots,
        bare_symbol,
        outcome: Err(EvalError::ExceptionThrown { exception: root }),
        omitted_text: reserved("opaque", 4096),
    });
    let owner = session.pending_suspension.as_ref().unwrap();
    assert_eq!(owner.active_stack.len(), 3);
    let expected = size_of::<SuspendedEvaluation>()
        + 3 * (size_of::<EvaluationStackEntry>() + root_bytes)
        + owner.occurrence_frame_capacity() * size_of::<(usize, usize, Arc<dyn SuspensionFrame>)>()
        + 2 * size_of::<usize>()
        + size_of::<PayloadFrame>()
        + 3 * root_bytes
        + symbol_bytes;
    let model = assert_model(&session);
    assert_eq!(model.field("pending_suspension"), Bytes::Exact(expected));
    assert_eq!(
        model.field("pending_request"),
        Bytes::Exact(size_of::<PendingRequest>())
    );
    assert_eq!(
        model.field("pending_input_request"),
        Bytes::Exact(size_of::<RuntimeInputRequest>() + 73 + 1)
    );
    assert_eq!(
        model.field("evaluation_stack"),
        Bytes::Exact(3 * (size_of::<EvaluationStackEntry>() + root_bytes))
    );
}

#[test]
fn live_frames_messages_and_traces_preserve_conservative_inline_terms() {
    let mut session = Session::empty();
    let expr = Expr::small_integer(4);
    let key = Symbol::from_name("dynamic");
    session.dynamic_own_values = Vec::with_capacity(3);
    session.dynamic_own_values.push(DynamicOwnValueFrame {
        symbol: key.clone(),
        value: Some(expr.clone()),
    });
    session.monitor_scopes = Vec::with_capacity(2);
    session.monitor_scopes.push(MonitorScope {
        display_expr: expr.clone(),
        evaluation_monitor: Some(expr.clone()),
        step_monitor: Some(expr.clone()),
        last_display: Some(reserved("display", 91)),
    });
    session.reap_scopes = Vec::with_capacity(4);
    session.reap_scopes.push(ReapScope {
        patterns: vec![expr.clone()],
        pattern_list_form: false,
        handler: Some(expr.clone()),
        buckets_by_pattern: vec![ReapBucketGroup {
            buckets: vec![ReapBucket {
                tag: expr.clone(),
                values: vec![expr.clone(), expr.clone()],
            }],
            tag_positions: Some(HashMap::from([(expr.clone(), 0)])),
        }],
    });
    session.messages = crate::runtime::model::OutputLog::with_capacity(5);
    let message_stamp = session.next_output_stamp();
    session.messages.push(
        message_stamp,
        RuntimeMessage {
            symbol: reserved("s", 17),
            tag: reserved("t", 19),
            text: reserved("message", 101),
        },
    );
    session.trace_events = crate::runtime::model::OutputLog::with_capacity(6);
    let trace_stamp = session.next_output_stamp();
    session.trace_events.push(
        trace_stamp,
        (
            TraceEvent::EvaluationPass {
                before: expr.clone(),
                after: expr,
            },
            0,
        ),
    );
    let model = assert_model(&session);
    assert_eq!(
        model.field("dynamic_own_values"),
        Bytes::Exact(
            3 * size_of::<DynamicOwnValueFrame>()
                + key.context().len()
                + key.name().len()
                + size_of::<Expr>()
        )
    );
    assert_eq!(
        model.field("monitor_scopes"),
        Bytes::Exact(2 * size_of::<MonitorScope>() + 3 * size_of::<Expr>() + 91)
    );
    assert_eq!(
        model.field("reap_scopes"),
        Bytes::Exact(4 * size_of::<ReapScope>() + 5 * size_of::<Expr>())
    );
    assert_eq!(
        model.field("messages"),
        Bytes::Exact(
            5 * size_of::<(crate::runtime::model::OutputStamp, RuntimeMessage)>() + 17 + 19 + 101
        )
    );
    assert_eq!(
        model.field("trace_events"),
        Bytes::Exact(
            6 * size_of::<(crate::runtime::model::OutputStamp, (TraceEvent, usize))>()
                + 2 * size_of::<Expr>()
        )
    );
}

#[test]
fn session_memory_layout_baseline() {
    let mut sizes = vec![
        ("Session (test)", size_of::<Session>()),
        ("Expr", size_of::<Expr>()),
        ("SharedExpr", size_of::<SharedExpr>()),
        ("SharedArgs", size_of::<SharedArgs>()),
        ("Symbol", size_of::<Symbol>()),
        ("BigInt", size_of::<BigInt>()),
        ("Rational", size_of::<Rational>()),
        ("ResourceLimits", size_of::<ResourceLimits>()),
        ("EvaluationStackEntry", size_of::<EvaluationStackEntry>()),
        ("ContextFrame", size_of::<ContextFrame>()),
        ("DynamicOwnValueFrame", size_of::<DynamicOwnValueFrame>()),
        ("ModuleActivation", size_of::<ModuleActivation>()),
        ("OptionValueScope", size_of::<OptionValueScope>()),
        ("SymbolOptions", size_of::<SymbolOptions>()),
        ("SymbolOptionEntry", size_of::<SymbolOptionEntry>()),
        ("ReapScope", size_of::<ReapScope>()),
        ("ReapBucket", size_of::<ReapBucket>()),
        ("ReapBucketGroup", size_of::<ReapBucketGroup>()),
        ("MonitorScope", size_of::<MonitorScope>()),
        ("RuntimeMessage", size_of::<RuntimeMessage>()),
        ("TraceEvent", size_of::<TraceEvent>()),
        ("RuntimeInputRequest", size_of::<RuntimeInputRequest>()),
        ("PendingRequest", size_of::<PendingRequest>()),
        ("SuspendedEvaluation", size_of::<SuspendedEvaluation>()),
        (
            "Arc<dyn SuspensionFrame>",
            size_of::<Arc<dyn SuspensionFrame>>(),
        ),
        ("PayloadFrame (test)", size_of::<PayloadFrame>()),
        ("RequestInterruptState", size_of::<RequestInterruptState>()),
        ("SymbolDefinition", size_of::<SymbolDefinition>()),
        ("UpValue", size_of::<UpValue>()),
        ("DownValue", size_of::<DownValue>()),
        ("SubValue", size_of::<SubValue>()),
        ("CallStabilityCacheKey", size_of::<CallStabilityCacheKey>()),
        (
            "CallStabilityCacheEntry",
            size_of::<CallStabilityCacheEntry>(),
        ),
        ("SymbolCallEffectKey", size_of::<SymbolCallEffectKey>()),
        ("CallEffectCacheEntry", size_of::<CallEffectCacheEntry>()),
        (
            "SourceUnit",
            size_of::<crate::runtime::scoping::packages::SourceUnit>(),
        ),
    ];
    sizes.extend(AssociationLookupCache::occurrence_layout());
    sizes.extend(SymbolDefinition::occurrence_dispatch_layout());
    sizes.sort_unstable();
    for (name, bytes) in sizes {
        println!("{name}\t{bytes}");
        assert!(bytes > 0);
    }
}
