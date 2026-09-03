use super::model::{ParsedTestReportObject, TestCaseSpec};
use super::*;

pub(in crate::runtime::debug_tools::test_report) fn parse_test_case_exprs(
    expr: &Expr,
) -> Option<Vec<Expr>> {
    if association_entries_from_expr(expr).is_some() {
        return Some(vec![expr.clone()]);
    }

    list_items(expr).map(|items| items.to_vec())
}

pub(in crate::runtime::debug_tools::test_report) fn parse_test_case(
    expr: &Expr,
) -> Option<TestCaseSpec> {
    let entries = association_entries_from_expr(expr)?;
    let mut input = None;
    let mut expected = None;
    let mut test_id = None;
    let mut expected_messages = None;
    let mut same_test = None;

    for (key, value) in entries {
        match key {
            Expr::Atom(Atom::String(name)) if name == "Input" => {
                input = unwrap_hold_complete(&value);
            }
            Expr::Atom(Atom::String(name)) if name == "ExpectedOutput" => {
                expected = unwrap_hold_complete(&value);
            }
            Expr::Atom(Atom::String(name)) if name == "TestID" => {
                test_id = Some(value);
            }
            Expr::Atom(Atom::String(name)) if name == "ExpectedMessages" => {
                expected_messages = unwrap_hold_complete(&value).map(|held| {
                    list_items(&held)
                        .map(|items| items.to_vec())
                        .unwrap_or_else(|| vec![held])
                });
            }
            Expr::Atom(Atom::String(name)) if name == "SameTest" => {
                same_test = Some(value);
            }
            _ => {}
        }
    }

    Some(TestCaseSpec {
        input: input?,
        expected: expected?,
        test_id,
        expected_messages,
        same_test,
    })
}

pub(in crate::runtime::debug_tools::test_report) fn parse_test_report_objects(
    expr: &Expr,
) -> Option<Vec<ParsedTestReportObject>> {
    if let Some(report) = parse_test_report_object(expr) {
        return Some(vec![report]);
    }

    let items = list_items(expr)?;
    items.iter().map(parse_test_report_object).collect()
}

pub(in crate::runtime::debug_tools::test_report) fn parse_test_create_options(
    expr: &Expr,
) -> Option<Vec<(String, Expr)>> {
    let items = if let Some(items) = list_items(expr) {
        items.to_vec()
    } else {
        vec![expr.clone()]
    };

    let mut options = Vec::new();
    for item in items {
        match item {
            Expr::Call { head, args }
                if (is_system_symbol_name(&head, "Rule")
                    || is_system_symbol_name(&head, "RuleDelayed"))
                    && args.len() == 2 =>
            {
                let name = match &args[0] {
                    Expr::Atom(Atom::Symbol(symbol)) => symbol.name().to_string(),
                    Expr::Atom(Atom::String(name)) => name.clone(),
                    _ => return None,
                };
                options.push((name, args[1].clone()));
            }
            _ => return None,
        }
    }
    Some(options)
}

fn parse_test_report_object(expr: &Expr) -> Option<ParsedTestReportObject> {
    let tests_run = association_property_by_name(expr, "TestsRunCount")
        .and_then(|value| expr_as_non_negative_usize(&value))?;
    let tests_succeeded = association_property_by_name(expr, "TestsSucceededCount")
        .and_then(|value| expr_as_non_negative_usize(&value))?;
    let tests_failed = association_property_by_name(expr, "TestsFailedCount")
        .and_then(|value| expr_as_non_negative_usize(&value))?;
    let test_results = association_property_by_name(expr, "TestResults")
        .and_then(|value| list_items(&value).map(|items| items.to_vec()))?;

    Some(ParsedTestReportObject {
        tests_run,
        tests_succeeded,
        tests_failed,
        test_results,
    })
}

fn unwrap_hold_complete(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Hold")
                || is_system_symbol_name(head, "HoldComplete")
                || is_system_symbol_name(head, "HoldForm"))
                && args.len() == 1 =>
        {
            Some(args[0].clone())
        }
        _ => None,
    }
}
