//! Test-local builtin registrations used by runtime tests.
//!
//! These helpers keep runtime tests independent from `rum-stdlib`. Some
//! handlers deliberately delegate to kernel-side data evaluators because the
//! test subject is runtime machinery. Others are small test doubles, such as
//! structural `SameQ` and structural `First`; they are not complete language
//! semantics and should stay simple.

use super::*;

macro_rules! test_arithmetic_handler {
    ($name:ident, $builtin:ident) => {
        fn $name(session: &mut Session, head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
            session.eval_arithmetic_data_builtin(ArithmeticDataBuiltin::$builtin, head, args)
        }
    };
}

macro_rules! test_control_handler {
    ($name:ident, $builtin:ident) => {
        fn $name(session: &mut Session, head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
            session.eval_control_data_builtin(ControlDataBuiltin::$builtin, head, args)
        }
    };
}

macro_rules! test_rewrite_handler {
    ($name:ident, $builtin:ident) => {
        fn $name(session: &mut Session, head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
            session.eval_rewrite_data_builtin(RewriteDataBuiltin::$builtin, head, args)
        }
    };
}

const TEST_ARITHMETIC: &str = "Test-local moved arithmetic builtin.";
const TEST_CONTROL: &str = "Test-local moved control builtin.";
const TEST_REWRITE: &str = "Test-local moved rewrite builtin.";

fn arithmetic_spec(name: &'static str, handler: BuiltinFn) -> BuiltinSpec {
    BuiltinSpec::ordinary(
        name,
        TEST_ARITHMETIC,
        handler,
        MemoryGrowthContract::Delegated,
    )
}

fn arithmetic_attrs_spec(
    name: &'static str,
    attributes: AttributeSet,
    handler: BuiltinFn,
) -> BuiltinSpec {
    BuiltinSpec::ordinary_with_attributes(
        name,
        TEST_ARITHMETIC,
        attributes,
        handler,
        MemoryGrowthContract::Delegated,
    )
}

fn control_spec(name: &'static str, handler: BuiltinFn) -> BuiltinSpec {
    BuiltinSpec::ordinary(name, TEST_CONTROL, handler, MemoryGrowthContract::Delegated)
}

fn control_hold_spec(name: &'static str, hold: HoldAttribute, handler: BuiltinFn) -> BuiltinSpec {
    BuiltinSpec::ordinary_with_hold(
        name,
        TEST_CONTROL,
        hold,
        handler,
        MemoryGrowthContract::Delegated,
    )
}

fn control_attrs_spec(
    name: &'static str,
    attributes: AttributeSet,
    handler: BuiltinFn,
) -> BuiltinSpec {
    BuiltinSpec::ordinary_with_attributes(
        name,
        TEST_CONTROL,
        attributes,
        handler,
        MemoryGrowthContract::Delegated,
    )
}

fn rewrite_spec(name: &'static str, handler: BuiltinFn) -> BuiltinSpec {
    BuiltinSpec::special(name, TEST_REWRITE, handler, MemoryGrowthContract::Delegated)
}

test_arithmetic_handler!(test_arithmetic_identity, Identity);
test_arithmetic_handler!(test_arithmetic_plus, Plus);
test_arithmetic_handler!(test_arithmetic_times, Times);
test_arithmetic_handler!(test_arithmetic_power, Power);
test_arithmetic_handler!(test_arithmetic_subtract, Subtract);
test_arithmetic_handler!(test_arithmetic_divide, Divide);
test_arithmetic_handler!(test_arithmetic_abs, Abs);
test_arithmetic_handler!(test_arithmetic_sign, Sign);
test_arithmetic_handler!(test_arithmetic_floor, Floor);
test_arithmetic_handler!(test_arithmetic_ceiling, Ceiling);
test_arithmetic_handler!(test_arithmetic_integer_part, IntegerPart);
test_arithmetic_handler!(test_arithmetic_fractional_part, FractionalPart);
test_arithmetic_handler!(test_arithmetic_minus, Minus);
test_arithmetic_handler!(test_arithmetic_round, Round);
test_arithmetic_handler!(test_arithmetic_factorial, Factorial);
test_arithmetic_handler!(test_arithmetic_factorial2, Factorial2);
test_arithmetic_handler!(test_arithmetic_total, Total);
test_arithmetic_handler!(test_arithmetic_denominator, Denominator);
test_arithmetic_handler!(test_arithmetic_numerator, Numerator);
test_arithmetic_handler!(test_arithmetic_mod, Mod);
test_arithmetic_handler!(test_arithmetic_quotient, Quotient);
test_arithmetic_handler!(test_arithmetic_power_mod, PowerMod);
test_arithmetic_handler!(test_arithmetic_gcd, GCD);
test_arithmetic_handler!(test_arithmetic_lcm, LCM);
test_arithmetic_handler!(test_arithmetic_max, Max);
test_arithmetic_handler!(test_arithmetic_min, Min);
test_arithmetic_handler!(test_arithmetic_min_max, MinMax);

test_control_handler!(test_control_compound_expression, CompoundExpression);
test_control_handler!(test_control_if, If);
test_control_handler!(test_control_which, Which);
test_control_handler!(test_control_switch, Switch);
test_control_handler!(test_control_do, Do);
test_control_handler!(test_control_product, Product);
test_control_handler!(test_control_sum, Sum);
test_control_handler!(test_control_while, While);
test_control_handler!(test_control_until, Until);
test_control_handler!(test_control_for, For);
test_control_handler!(test_control_table, Table);
test_control_handler!(test_control_abort, Abort);
test_control_handler!(test_control_exit, Exit);
test_control_handler!(test_control_check_abort, CheckAbort);
test_control_handler!(test_control_abort_protect, AbortProtect);
test_control_handler!(test_control_with_cleanup, WithCleanup);
test_control_handler!(test_control_failsafe, Failsafe);
test_control_handler!(test_control_assert, Assert);
test_control_handler!(test_control_enclose, Enclose);
test_control_handler!(test_control_confirm, Confirm);
test_control_handler!(test_control_confirm_by, ConfirmBy);
test_control_handler!(test_control_confirm_match, ConfirmMatch);
test_control_handler!(test_control_confirm_quiet, ConfirmQuiet);
test_control_handler!(test_control_confirm_assert, ConfirmAssert);
test_control_handler!(test_control_return, Return);
test_control_handler!(test_control_break, Break);
test_control_handler!(test_control_continue, Continue);
test_control_handler!(test_control_throw, Throw);
test_control_handler!(test_control_catch, Catch);
test_control_handler!(test_control_failure, Failure);
test_control_handler!(test_control_failure_q, FailureQ);
test_control_handler!(test_control_success, Success);
test_control_handler!(test_control_terminated_evaluation, TerminatedEvaluation);

test_rewrite_handler!(test_rewrite_match_q, MatchQ);
test_rewrite_handler!(test_rewrite_cases, Cases);
test_rewrite_handler!(test_rewrite_delete_cases, DeleteCases);
test_rewrite_handler!(test_rewrite_replace, Replace);
test_rewrite_handler!(test_rewrite_replace_all, ReplaceAll);
test_rewrite_handler!(test_rewrite_replace_list, ReplaceList);
test_rewrite_handler!(test_rewrite_replace_repeated, ReplaceRepeated);

pub(in crate::runtime) fn register_test_arithmetic_builtins(session: &mut Session) {
    let specs = [
        arithmetic_spec("Identity", test_arithmetic_identity),
        arithmetic_attrs_spec(
            "Plus",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::Orderless)
                .with(Attribute::Flat)
                .with(Attribute::OneIdentity),
            test_arithmetic_plus,
        ),
        arithmetic_attrs_spec(
            "Times",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::Orderless)
                .with(Attribute::Flat)
                .with(Attribute::OneIdentity),
            test_arithmetic_times,
        ),
        arithmetic_attrs_spec(
            "Power",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_power,
        ),
        arithmetic_spec("Subtract", test_arithmetic_subtract),
        arithmetic_spec("Divide", test_arithmetic_divide),
        arithmetic_attrs_spec(
            "Abs",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_abs,
        ),
        arithmetic_attrs_spec(
            "Sign",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::NumericFunction)
                .with(Attribute::Protected),
            test_arithmetic_sign,
        ),
        arithmetic_attrs_spec(
            "Floor",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::NumericFunction)
                .with(Attribute::Protected),
            test_arithmetic_floor,
        ),
        arithmetic_attrs_spec(
            "Ceiling",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::NumericFunction)
                .with(Attribute::Protected),
            test_arithmetic_ceiling,
        ),
        arithmetic_attrs_spec(
            "IntegerPart",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::NumericFunction)
                .with(Attribute::Protected),
            test_arithmetic_integer_part,
        ),
        arithmetic_attrs_spec(
            "FractionalPart",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::NumericFunction)
                .with(Attribute::Protected),
            test_arithmetic_fractional_part,
        ),
        arithmetic_attrs_spec(
            "Minus",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::NumericFunction)
                .with(Attribute::Protected),
            test_arithmetic_minus,
        ),
        arithmetic_attrs_spec(
            "Round",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_round,
        ),
        arithmetic_attrs_spec(
            "Factorial",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_factorial,
        ),
        arithmetic_attrs_spec(
            "Factorial2",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_factorial2,
        ),
        arithmetic_spec("Total", test_arithmetic_total),
        arithmetic_attrs_spec(
            "Denominator",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_denominator,
        ),
        arithmetic_attrs_spec(
            "Numerator",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_numerator,
        ),
        arithmetic_attrs_spec(
            "Mod",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_mod,
        ),
        arithmetic_attrs_spec(
            "Quotient",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_quotient,
        ),
        arithmetic_attrs_spec(
            "PowerMod",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_power_mod,
        ),
        arithmetic_attrs_spec(
            "GCD",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_gcd,
        ),
        arithmetic_attrs_spec(
            "LCM",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_lcm,
        ),
        arithmetic_spec("Max", test_arithmetic_max),
        arithmetic_spec("Min", test_arithmetic_min),
        arithmetic_attrs_spec(
            "MinMax",
            AttributeSet::from_attribute(Attribute::Protected),
            test_arithmetic_min_max,
        ),
    ];

    for spec in specs {
        session.register_builtin(spec);
    }
}

pub(in crate::runtime) fn register_test_compound_expression_builtin(session: &mut Session) {
    session.register_builtin(BuiltinSpec::ordinary_with_hold(
        "CompoundExpression",
        "Test-local control builtin.",
        HoldAttribute::HoldAll,
        test_control_compound_expression,
        MemoryGrowthContract::Delegated,
    ));
}

pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    session.register_builtin(BuiltinSpec::ordinary_with_hold(
        "If",
        "Test-local control builtin.",
        HoldAttribute::HoldRest,
        test_control_if,
        MemoryGrowthContract::Delegated,
    ));
}

pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    session.register_builtin(BuiltinSpec::ordinary(
        "Return",
        "Test-local control builtin.",
        test_control_return,
        MemoryGrowthContract::NonAmplifying,
    ));
}

pub(in crate::runtime) fn register_test_sameq_builtin(session: &mut Session) {
    session.register_builtin(BuiltinSpec::ordinary(
        "SameQ",
        "Test-local structural equality builtin.",
        test_same_q,
        MemoryGrowthContract::NonAmplifying,
    ));
}

pub(in crate::runtime) fn register_test_first_builtin(session: &mut Session) {
    session.register_builtin(BuiltinSpec::ordinary_with_hold(
        "First",
        "Test-local structural first builtin.",
        HoldAttribute::HoldRest,
        test_first,
        MemoryGrowthContract::NonAmplifying,
    ));
}

pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    register_test_compound_expression_builtin(session);
    register_test_if_builtin(session);
    register_test_return_builtin(session);
    register_test_sameq_builtin(session);
}

pub(in crate::runtime) fn register_test_control_builtins(session: &mut Session) {
    let specs = [
        control_hold_spec(
            "CompoundExpression",
            HoldAttribute::HoldAll,
            test_control_compound_expression,
        ),
        control_hold_spec("If", HoldAttribute::HoldRest, test_control_if),
        control_hold_spec("Which", HoldAttribute::HoldAll, test_control_which),
        control_hold_spec("Switch", HoldAttribute::HoldRest, test_control_switch),
        control_hold_spec("Do", HoldAttribute::HoldAll, test_control_do),
        control_attrs_spec(
            "Product",
            AttributeSet::from_hold(HoldAttribute::HoldAll)
                .with(Attribute::Protected)
                .with(Attribute::ReadProtected),
            test_control_product,
        ),
        control_attrs_spec(
            "Sum",
            AttributeSet::from_hold(HoldAttribute::HoldAll)
                .with(Attribute::Protected)
                .with(Attribute::ReadProtected),
            test_control_sum,
        ),
        control_hold_spec("While", HoldAttribute::HoldAll, test_control_while),
        control_hold_spec("Until", HoldAttribute::HoldAll, test_control_until),
        control_hold_spec("For", HoldAttribute::HoldAll, test_control_for),
        control_hold_spec("Table", HoldAttribute::HoldAll, test_control_table),
        control_spec("Abort", test_control_abort),
        control_spec("Exit", test_control_exit),
        control_hold_spec(
            "CheckAbort",
            HoldAttribute::HoldAll,
            test_control_check_abort,
        ),
        control_hold_spec(
            "AbortProtect",
            HoldAttribute::HoldAll,
            test_control_abort_protect,
        ),
        control_hold_spec(
            "WithCleanup",
            HoldAttribute::HoldAll,
            test_control_with_cleanup,
        ),
        control_spec("Failsafe", test_control_failsafe),
        control_hold_spec("Assert", HoldAttribute::HoldAll, test_control_assert),
        BuiltinSpec::new(
            "$AssertFunction",
            "Test-local moved control-support symbol.",
            MemoryGrowthContract::NonAmplifying,
        ),
        BuiltinSpec::new(
            "PropagateAborts",
            "Test-local moved control-support symbol.",
            MemoryGrowthContract::Delegated,
        ),
        control_hold_spec("Enclose", HoldAttribute::HoldAll, test_control_enclose),
        control_hold_spec("Confirm", HoldAttribute::HoldAll, test_control_confirm),
        control_hold_spec("ConfirmBy", HoldAttribute::HoldAll, test_control_confirm_by),
        control_hold_spec(
            "ConfirmMatch",
            HoldAttribute::HoldAll,
            test_control_confirm_match,
        ),
        control_hold_spec(
            "ConfirmQuiet",
            HoldAttribute::HoldAll,
            test_control_confirm_quiet,
        ),
        control_hold_spec(
            "ConfirmAssert",
            HoldAttribute::HoldAll,
            test_control_confirm_assert,
        ),
        control_spec("Return", test_control_return),
        control_spec("Break", test_control_break),
        control_spec("Continue", test_control_continue),
        control_spec("Throw", test_control_throw),
        control_hold_spec("Catch", HoldAttribute::HoldAll, test_control_catch),
        control_spec("Failure", test_control_failure),
        control_spec("FailureQ", test_control_failure_q),
        control_spec("Success", test_control_success),
        control_spec("TerminatedEvaluation", test_control_terminated_evaluation),
    ];

    for spec in specs {
        session.register_builtin(spec);
    }
}

pub(in crate::runtime) fn register_test_rewrite_builtins(session: &mut Session) {
    let specs = [
        rewrite_spec("MatchQ", test_rewrite_match_q),
        rewrite_spec("Cases", test_rewrite_cases),
        rewrite_spec("DeleteCases", test_rewrite_delete_cases),
        rewrite_spec("Replace", test_rewrite_replace),
        rewrite_spec("ReplaceList", test_rewrite_replace_list),
        rewrite_spec("ReplaceAll", test_rewrite_replace_all),
        rewrite_spec("ReplaceRepeated", test_rewrite_replace_repeated),
    ];

    for spec in specs {
        session.register_builtin(spec);
    }
}

pub(in crate::runtime) fn session_with_test_compound_expression_builtin() -> Session {
    let mut session = Session::new();
    register_test_compound_expression_builtin(&mut session);
    session
}

pub(in crate::runtime) fn session_with_test_control_builtins() -> Session {
    let mut session = Session::new();
    register_test_control_builtins(&mut session);
    register_test_rewrite_builtins(&mut session);
    session
}

pub(in crate::runtime) fn session_with_test_rewrite_builtins() -> Session {
    let mut session = Session::new();
    register_test_rewrite_builtins(&mut session);
    session
}

fn test_same_q(_session: &mut Session, _head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(system_expr(
        if args.windows(2).all(|pair| pair[0] == pair[1]) {
            heads::structure::TRUE
        } else {
            heads::structure::FALSE
        },
    ))
}

fn test_first(session: &mut Session, head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    match expr {
        Expr::Call { args, .. } if !args.is_empty() => session.evaluate(args[0].clone()),
        _ => Ok(Expr::call(head.clone(), args.to_vec())),
    }
}
