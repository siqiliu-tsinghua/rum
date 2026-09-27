use super::*;

#[test]
fn option_rule_c02_non_call_immediate_and_delayed() {
    let mut s = Session::new();
    let atom = Expr::string("raw option");
    let immediate = Expr::call(
        system_expr("Rule"),
        vec![system_expr("IgnoreCase"), system_expr("True")],
    );
    let delayed = Expr::call(
        system_expr("RuleDelayed"),
        vec![system_expr("IgnoreCase"), system_expr("True")],
    );
    for armed in [false, true] {
        s.current_memory_limit = if armed { Some(1_000_000_000) } else { None };
        if armed {
            s.note_accounting_arm();
        }
        assert_eq!(s.evaluate_runtime_option_expr(&atom).unwrap(), atom);
        assert_eq!(
            s.evaluate_runtime_option_expr(&immediate).unwrap(),
            immediate
        );
        assert_eq!(s.evaluate_runtime_option_expr(&delayed).unwrap(), immediate);
        if armed {
            assert_eq!(s.retention_snapshot().unwrap().live, 0);
        }
    }
}
