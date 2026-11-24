//! Row-specific permission to retain an actual fault-bearing observation.
use super::{Armed, AuthError, AuthResult, AuthenticatedSource, view::MutationObserver};
use crate::oracle::fixtures::obligations::guest;

pub(crate) const ROW: &str = "C10.prepared.profiled_consumer";

fn admit(row: &str, selector: &str) -> AuthResult<&'static str> {
    let specs = super::super::schema::specs();
    let Some(spec) = specs.get(row) else {
        return Err(AuthError::ObserverRow);
    };
    let plan = super::observer_policy::resolve(spec)?;
    match super::observer_policy::observation_mode(spec, plan)? {
        super::observer_policy::ObservationMode::Ordinary => Err(AuthError::ObserverRow),
        super::observer_policy::ObservationMode::AuthenticatedMutationObserver(_) => {
            if selector != row {
                return Err(AuthError::ObserverUnarmed);
            }
            Ok(spec.row)
        }
    }
}

pub(super) fn admit_for_test(row: &str, selector: &str) -> AuthResult<&'static str> {
    admit(row, selector)
}

impl AuthenticatedSource<Armed> {
    pub(crate) fn observer(&self, row: &str) -> AuthResult<MutationObserver<'_>> {
        let row = admit(row, &std::env::var("RUM_CM740_OMIT").unwrap_or_default())?;
        Ok(MutationObserver { source: self, row })
    }
}

impl MutationObserver<'_> {
    pub(crate) fn check(&self, program: &guest::Program) -> AuthResult<()> {
        admit(
            program.key.row(),
            &std::env::var("RUM_CM740_OMIT").unwrap_or_default(),
        )?;
        if program.key.row() != self.row {
            return Err(AuthError::ObserverRow);
        }
        // Reuse the admitted loader, including its use-time postimage check.
        // A caller cannot replace the row's plan after obtaining a capability.
        let programs =
            guest::programs_with_mutation(self.source).map_err(|_| AuthError::ObserverProgram)?;
        fn find<'a>(p: &'a guest::Program, id: &str) -> Option<&'a guest::Program> {
            if p.id == id {
                Some(p)
            } else {
                p.paired.as_deref().and_then(|p| find(p, id))
            }
        }
        let original = programs
            .values()
            .find_map(|p| find(p, &program.id))
            .ok_or(AuthError::ObserverProgram)?;
        if format!("{original:?}") != format!("{program:?}") {
            return Err(AuthError::ObserverProgram);
        }
        Ok(())
    }
}

#[test]
fn cm_mutation_observer_admission_controls() {
    assert_eq!(admit(ROW, ROW), Ok(ROW));
    for (name, row, selector, expected) in [
        ("wrong-row", "P0543", ROW, AuthError::ObserverRow),
        ("unarmed", ROW, "", AuthError::ObserverUnarmed),
        (
            "another-row-selector",
            ROW,
            "P0543",
            AuthError::ObserverUnarmed,
        ),
    ] {
        assert_eq!(admit(row, selector), Err(expected));
        println!(
            "CM4 OBSERVER ADMISSION NEGATIVE control={name} predicate={expected:?} semantic_catch=false"
        );
    }
    let specs = super::super::schema::specs();
    let admitted: Vec<_> = specs
        .values()
        .filter(|s| s.observer == "NativeEqual")
        .collect();
    for spec in &admitted {
        assert_eq!(admit(spec.row, spec.row), Ok(spec.row));
        assert_eq!(admit(spec.row, ""), Err(AuthError::ObserverUnarmed));
        assert_eq!(admit(spec.row, "P0543"), Err(AuthError::ObserverUnarmed));
        assert_eq!(admit("P0543", spec.row), Err(AuthError::ObserverRow));
        println!(
            "CM4 ROW OBSERVER ADMISSION PASS row={} wrong_row=ObserverRow unarmed=ObserverUnarmed another_selector=ObserverUnarmed observations_supplied=0",
            spec.row
        );
    }
    println!(
        "CM4 OBSERVER ADMISSION PASS admitted_rows={} negative_controls={} synthetic_permission_only=true",
        admitted.len(),
        3 + admitted.len() * 3
    );
}
