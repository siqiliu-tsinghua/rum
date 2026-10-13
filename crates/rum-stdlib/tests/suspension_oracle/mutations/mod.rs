mod capture_output;
mod driver;
use super::{
    catalog::{self, CatalogError},
    descriptors::{Checks, Fixture, Pause, accept},
    support::observe::Rejection,
};
use rum_runtime::RuntimeInputKind;

fn fixture(name: &str, twice: bool) -> Fixture {
    let mut f = Fixture::new(
        name,
        &format!(
            "{name}[Function[x,(AppendTo[Global`log,x];If[Or[x==2,{}],InputString[\"fold callback\"]];x)],{{1,2,3,4,5}}]",
            if twice { "x==4" } else { "False" }
        ),
    );
    f.configure = driver::install;
    f.pauses = (0..if twice { 2 } else { 1 })
        .map(|_| Pause {
            prompt: "fold callback".into(),
            kind: RuntimeInputKind::InputString,
            answer: "answer".into(),
        })
        .collect();
    f.expected_value = Some("List[1, 2, 3, 4, 5]".into());
    f.expected_log = Some("List[1, 2, 3, 4, 5]".into());
    f
}

#[test]
fn omitted_registration() {
    let s = super::support::session::new();
    let installed: Vec<_> = s
        .registered_builtin_specs()
        .map(|s| format!("System`{}", s.name))
        .collect();
    // Also exercised against the reviewed manifest by catalog acceptance.
    let complete = super::manifest::catalog_rows()
        .into_iter()
        .filter(|r| super::manifest::active(r[1]))
        .map(|r| r[0].to_owned())
        .collect::<Vec<_>>();
    let omitted: Vec<_> = installed
        .iter()
        .filter(|n| *n != "System`Scan")
        .cloned()
        .collect();
    let mut extra = complete.clone();
    extra.push("System`OracleUndeclared".into());
    for (i, m, e) in [
        (
            &omitted,
            &complete,
            CatalogError::MissingInstalled("System`Scan".into()),
        ),
        (
            &complete,
            &omitted,
            CatalogError::UndeclaredInstalled("System`Scan".into()),
        ),
        (
            &complete,
            &extra,
            CatalogError::MissingInstalled("System`OracleUndeclared".into()),
        ),
    ] {
        assert_eq!(catalog::reconcile(i, m, false), Ok(()));
        println!("WEAK omitted_registration: PASS");
        let actual = catalog::reconcile(i, m, true);
        assert_eq!(actual, Err(e));
        println!("STRICT omitted_registration: {actual:?}");
    }
    assert_eq!(catalog::reconcile(&installed, &complete, true), Ok(()));
    println!("CONTROL omitted_registration: PASS");
}

#[test]
fn non_suspending_fixture() {
    let good = fixture("OracleGoodFold", false);
    let mut bad = good.clone();
    bad.source = bad
        .source
        .replace("InputString[\"fold callback\"]", "\"answer\"");
    let weak = Checks {
        transcript: false,
        queue: false,
        ..Checks::ALL
    };
    accept(&bad, weak).unwrap();
    println!("WEAK non_suspending_fixture: PASS (equal final values)");
    let actual = accept(&bad, Checks::ALL).unwrap_err();
    assert_eq!(
        actual,
        Rejection::PauseCount {
            expected: 1,
            actual: 0
        }
    );
    println!("STRICT non_suspending_fixture: {actual:?}");
    accept(&good, Checks::ALL).unwrap();
    println!("CONTROL non_suspending_fixture: PASS");
}

#[test]
fn false_tail() {
    let bad = fixture("OracleFalseTail", false);
    accept(
        &bad,
        Checks {
            equality: false,
            expectations: false,
            ..Checks::ALL
        },
    )
    .unwrap();
    println!("WEAK false_tail: PASS");
    let actual = accept(&bad, Checks::ALL).unwrap_err();
    assert_eq!(actual, Rejection::Value);
    println!("STRICT false_tail: {actual:?}");
    accept(&fixture("OracleGoodFold", false), Checks::ALL).unwrap();
    let mut no_pause = bad;
    no_pause.source = no_pause
        .source
        .replace("InputString[\"fold callback\"]", "\"answer\"");
    no_pause.pauses.clear();
    accept(&no_pause, Checks::ALL).unwrap();
    println!("CONTROL false_tail: PASS (framed and no-pause)");
}

#[test]
fn missing_prefix() {
    let bad = fixture("OracleDropPrefix", true);
    accept(
        &bad,
        Checks {
            equality: false,
            expectations: false,
            ..Checks::ALL
        },
    )
    .unwrap();
    println!("WEAK missing_prefix: PASS (both prompts reached)");
    let actual = accept(&bad, Checks::ALL).unwrap_err();
    assert_eq!(actual, Rejection::Value);
    println!("STRICT missing_prefix: {actual:?}");
    accept(&fixture("OracleGoodFold", true), Checks::ALL).unwrap();
    println!("CONTROL missing_prefix: PASS");
}

#[test]
fn owed_entry_gains_descriptor() {
    let declared: std::collections::BTreeSet<String> =
        ["Scan.callback".to_string()].into_iter().collect();
    let covered = declared.clone();
    let owed = declared.clone();
    assert_eq!(
        catalog::reconcile_owed(&declared, &covered, &owed, false),
        Ok(())
    );
    println!("WEAK stale_owed: PASS");
    let actual = catalog::reconcile_owed(&declared, &covered, &owed, true);
    assert_eq!(actual, Err(CatalogError::StaleOwed("Scan.callback".into())));
    println!("STRICT stale_owed: {actual:?}");
    assert_eq!(
        catalog::reconcile_owed(&declared, &covered, &Default::default(), true),
        Ok(())
    );
    println!("CONTROL stale_owed: PASS");
}

#[test]
fn entry_missing_from_both() {
    let declared: std::collections::BTreeSet<String> =
        ["Scan.callback".to_string()].into_iter().collect();
    assert_eq!(
        catalog::reconcile_owed(&declared, &Default::default(), &Default::default(), false),
        Ok(())
    );
    println!("WEAK missing_owed: PASS");
    let actual = catalog::reconcile_owed(&declared, &Default::default(), &Default::default(), true);
    assert_eq!(
        actual,
        Err(CatalogError::MissingOwed("Scan.callback".into()))
    );
    println!("STRICT missing_owed: {actual:?}");
    assert_eq!(
        catalog::reconcile_owed(&declared, &Default::default(), &declared, true),
        Ok(())
    );
    println!("CONTROL missing_owed: PASS");
}
