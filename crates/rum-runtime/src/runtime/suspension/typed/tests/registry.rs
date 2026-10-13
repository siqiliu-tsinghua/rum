use super::super::channel::{Codec, LegacyCodec, LegacyMode, ValueMode};
use super::super::registry::*;
use super::*;
use crate::runtime::eval_call_prepare::prepared_slot_finalizer_fixture;

#[test]
fn registry_modes_are_fixed_per_slot_type() {
    fn value<T: SlotValue + Codec<Mode = ValueMode>>() {}
    fn legacy<T: SlotValue + Codec<Mode = LegacyMode>>() {}
    value::<Expr>();
    legacy::<Option<Expr>>();
    legacy::<Vec<Expr>>();
    legacy::<Vec<SymbolExprMap>>();
    legacy::<Option<Vec<Expr>>>();
    legacy::<Option<Vec<RewriteRuleInfo>>>();
    legacy::<Option<CasesLevelSpec>>();
    legacy::<bool>();
    legacy::<String>();
    legacy::<CompiledMatchOutcome>();
    legacy::<CasesTraversalResolution>();
    legacy::<TraversalWithoutLimitResolution>();
    legacy::<CasesCollectionOutcome>();
    legacy::<PositionCollectionOutcome>();
    legacy::<DeleteCasesOutcome>();
    legacy::<TraversalCountOutcome>();
    legacy::<ReplaceOutcome>();
    legacy::<ReplaceAllTraversalOutcome>();
    legacy::<Vec<ReplacePartTarget>>();
    legacy::<EvalCapturedOutcome>();
    legacy::<PreparedCallArguments>();
    legacy::<EvaluatedPartSelectors>();
}

#[test]
fn prepared_arguments_roundtrip_and_match_the_existing_finalizer_wire() {
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    for (source, count, restored) in [
        ("f[]", 0, false),
        ("f[one, two]", 2, false),
        ("f[Unevaluated[kept], two]", 2, true),
        ("f[Sequence[one, two], three]", 3, false),
        (
            "f[Sequence[], Unevaluated[kept], Sequence[one, two]]",
            3,
            true,
        ),
    ] {
        let Expr::Call { head, args } = session.lower_text(source).unwrap() else {
            panic!("fixture call")
        };
        let prepared = session
            .prepare_arguments_for_call_with_unevaluated_restore(
                head.as_ref().clone(),
                Default::default(),
                args,
            )
            .unwrap();
        assert_eq!(prepared.evaluated_args.len(), count, "{source}");
        assert_eq!(prepared.unevaluated_restore.is_some(), restored, "{source}");
        let expected_args = prepared.evaluated_args.clone();
        let expected_restore = prepared
            .unevaluated_restore
            .as_ref()
            .map(|plan| (plan.prepared_args.clone(), plan.wrappers.clone()));
        let old_finalizer_wire = prepared_slot_finalizer_fixture(&prepared);
        let wire = prepared.old_encode(CodecKey::new());
        assert_eq!(wire, old_finalizer_wire, "{source}");
        let decoded = PreparedCallArguments::old_decode(wire, CodecKey::new()).unwrap();
        assert_eq!(decoded.evaluated_args, expected_args, "{source}");
        let actual_restore = decoded
            .unevaluated_restore
            .map(|plan| (plan.prepared_args, plan.wrappers));
        assert_eq!(actual_restore, expected_restore, "{source}");
    }
    assert!(
        matches!(decode::<PreparedCallArguments>(Slot::Value(Expr::string("not a prepared call")), CodecKey::new()), Err(Slot::Value(expr)) if expr == Expr::string("not a prepared call"))
    );
}

#[test]
fn legacy_provider_retains_existing_leniency_and_wire() {
    use crate::runtime::suspension::ContinuationValue;
    for expr in [
        Expr::string("arbitrary"),
        Expr::call(crate::runtime::system_expr("List"), vec![]),
    ] {
        let old = <Option<Expr> as ContinuationValue>::from_continuation_expr(expr.clone());
        let new = decode::<Option<Expr>>(Slot::Value(expr), CodecKey::new())
            .ok()
            .unwrap();
        assert_eq!(new, old);
    }
    let input = Some(Expr::string("value"));
    let old = input.clone().into_continuation_expr();
    assert!(matches!(encode(input, CodecKey::new()), Slot::Value(expr) if expr == old));
}
