//! Structural comparison excludes the final scalar; pair closure remains per cell.
use super::{records::Rejection, schema};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Default)]
struct Cell {
    full: BTreeMap<bool, String>,
    structural: BTreeMap<bool, String>,
    scalar: BTreeMap<bool, String>,
    trace: BTreeMap<bool, String>,
    pauses: BTreeMap<bool, usize>,
    slots: Vec<String>,
    armed: bool,
    hits: Vec<String>,
    early_hit: bool,
}

struct Semantic {
    exact: Vec<String>,
    cells: BTreeMap<String, Cell>,
}

fn fields(line: &str) -> BTreeMap<&str, &str> {
    line.split(' ').filter_map(|c| c.split_once('=')).collect()
}

fn parse(text: &str, row: &str, arm: &str, binary: &str) -> Semantic {
    assert!(
        text.lines()
            .any(|l| l.starts_with("test result: ok. 1 passed; 0 failed;")),
        "completed semantic worker"
    );
    let identities: Vec<_> = text
        .lines()
        .filter(|l| l.starts_with("CM740_SEMANTIC_RUN_ID "))
        .collect();
    assert_eq!(identities.len(), 1);
    let id = fields(identities[0]);
    assert_eq!(id["row"], row);
    assert_eq!(id["arm"], arm);
    assert_eq!(id["binary"], binary);
    assert_eq!(
        id["schema"],
        crate::oracle::obligations::content_digest(include_bytes!("source_auth/overlays.tsv"))
    );
    if arm == "armed" {
        assert_eq!(id["selector"], row);
    }
    let mut result = Semantic {
        exact: Vec::new(),
        cells: BTreeMap::new(),
    };
    let mut selected = String::new();
    for line in text.lines() {
        let c: Vec<_> = line.split('\t').collect();
        match c[0] {
            "CM740_SEMANTIC_CASE" => {
                assert_eq!(c.len(), 3);
                assert_eq!(c[1], row);
                selected = schema::unhex(c[2]);
                assert!(
                    result
                        .cells
                        .insert(selected.clone(), Cell::default())
                        .is_none()
                );
                result.exact.push(line.into());
            }
            "CM740_SEMANTIC" => {
                assert_eq!(c.len(), 6);
                assert_eq!(c[1], row);
                assert_eq!(schema::unhex(c[2]), selected);
                let cell = result.cells.get_mut(&selected).unwrap();
                let queued = c[3].parse::<bool>().unwrap();
                assert!(cell.full.insert(queued, schema::unhex(c[4])).is_none());
                assert!(
                    cell.structural
                        .insert(queued, schema::unhex(c[5]))
                        .is_none()
                );
                result.exact.push(line.into());
            }
            "CM740_SEMANTIC_CHANNELS" => {
                assert_eq!(c.len(), 7);
                assert_eq!(c[1], row);
                assert_eq!(schema::unhex(c[2]), selected);
                let cell = result.cells.get_mut(&selected).unwrap();
                let queued = c[3].parse::<bool>().unwrap();
                assert!(cell.scalar.insert(queued, schema::unhex(c[4])).is_none());
                assert!(cell.trace.insert(queued, schema::unhex(c[5])).is_none());
                assert!(cell.pauses.insert(queued, c[6].parse().unwrap()).is_none());
                result.exact.push(line.into());
            }
            "CM740_OPTION_SLOT" => {
                assert_eq!(c.len(), 2);
                result
                    .cells
                    .get_mut(&selected)
                    .unwrap()
                    .slots
                    .push(c[1].into());
            }
            _ => {
                if line.starts_with("CM740_SELECTOR_ARMED ") {
                    assert_eq!(fields(line)["row"], row);
                    result.cells.get_mut(&selected).unwrap().armed = true;
                } else if line.starts_with("CM740_SOURCE_HIT ") {
                    let f = fields(line);
                    assert_eq!(f["row"], row);
                    assert_eq!(f["selector"], row);
                    let cell = result.cells.get_mut(&selected).unwrap();
                    cell.early_hit |= !cell.armed;
                    cell.hits.push(f["phase"].into());
                }
            }
        }
    }
    result
}

fn phase(row: &str, phase: &str) -> bool {
    match row {
        "Some" => phase == "actual-option-output-encode",
        "G15A" => matches!(phase, "shared-value-stage-finish" | "value-frame-finish"),
        "G15B" => phase == "CallArgument.finish_definition_lhs_active_expr",
        "C1" => matches!(
            phase,
            "finish_replace_all_application.application-evaluation-finish"
                | "finish_replace_all_unheld_application.application-evaluation-finish"
        ),
        "P0325" | "P0326" => phase == "OmitActualResumedArgumentAppend",
        _ => false,
    }
}

fn caught(row: &str, baseline: &Cell, disabled: &Cell, armed: &Cell) -> Result<(), Rejection> {
    if baseline.pauses.get(&false).copied().unwrap_or(0) == 0 {
        return Err(Rejection::BaselinePause);
    }
    if !armed.armed {
        return Err(Rejection::Unarmed);
    }
    if armed.early_hit || armed.hits.is_empty() || armed.hits.iter().any(|p| !phase(row, p)) {
        return Err(Rejection::NoHit);
    }
    let red = match row {
        "Some" => {
            if disabled.slots != ["InternalContinuationSome[5]"] {
                return Err(Rejection::RawArtifact);
            }
            armed.slots == ["5"] && baseline.scalar == armed.scalar
        }
        "C1" => baseline.scalar == armed.scalar && baseline.trace != armed.trace,
        "G15A" | "G15B" => baseline.structural != armed.structural,
        "P0325" | "P0326" => baseline.full != armed.full,
        _ => return Err(Rejection::WrongRow),
    };
    if red {
        Ok(())
    } else {
        Err(Rejection::IdentitySurvivor)
    }
}

fn pair_closed(required: &BTreeSet<String>, observed: &BTreeMap<String, bool>) -> bool {
    observed.keys().cloned().collect::<BTreeSet<_>>() == *required
        && observed.values().all(|caught| *caught)
}

#[test]
fn cm_structural_and_pair_receipts_exact() {
    let Ok(index) = std::env::var("RUM_CM740_PACKETS") else {
        println!(
            "CM4 STRUCTURAL RECEIPTS pending=4 held_pair_cells_pending=100 copied_packets_not_supplied=true"
        );
        return;
    };
    let index = super::receipts::read_index(&index);
    let configuration = if cfg!(feature = "hash") {
        "default"
    } else {
        "no-default"
    };
    for row in ["Some", "G15A", "G15B", "C1", "P0325", "P0326"] {
        let records: Vec<_> = ["real", "disabled", "armed"]
            .into_iter()
            .map(|arm| {
                let identity = &index.builds[&(configuration.into(), arm.into())];
                let (_, raw) = &index.runs[&(configuration.into(), arm.into(), row.into())];
                parse(raw, row, arm, identity.binary())
            })
            .collect();
        let expected: BTreeSet<_> = include_str!("semantic-programs.tsv")
            .lines()
            .skip(1)
            .filter_map(|line| {
                let c: Vec<_> = line.split('\t').collect();
                (c[1] == row).then(|| c[0].to_string())
            })
            .collect();
        let (real, disabled, armed) = (&records[0], &records[1], &records[2]);
        for r in &records {
            assert_eq!(r.cells.keys().cloned().collect::<BTreeSet<_>>(), expected);
        }
        assert_eq!(real.exact, disabled.exact, "record-exact disabled parity");
        let mut observed = BTreeMap::new();
        let mut no_hit = 0;
        for id in &expected {
            let (r, d, a) = (&real.cells[id], &disabled.cells[id], &armed.cells[id]);
            assert_eq!(
                r.structural[&false], r.structural[&true],
                "complete original structural control {row} {id}"
            );
            assert!(r.hits.is_empty() && d.hits.is_empty());
            let verdict = caught(row, r, d, a);
            if verdict == Err(Rejection::NoHit) {
                no_hit += 1;
            }
            observed.insert(id.clone(), verdict.is_ok());
            println!(
                "CM4 SEMANTIC CELL row={row} program={id} verdict={verdict:?} final_scalar_comparison={}",
                matches!(row, "P0325" | "P0326")
            );
        }
        if matches!(row, "P0325" | "P0326") {
            let catches = observed.values().filter(|v| **v).count();
            let closed = pair_closed(&expected, &observed);
            println!(
                "CM4 HELD PAIR BAR row={row} required=50 executed={} caught={catches} armed_without_hit={no_hit} complete_per_cell_bar={closed} owner_state=Open encoding=0",
                observed.len()
            );
            assert_eq!(expected.len(), 50);
            // The historical 45/50 packet is red. Never substitute one generic
            // own catch or silently promote the five unhit Error cells.
            assert_eq!((catches, no_hit, closed), (45, 5, false));
            for (id, ok) in &observed {
                if !ok {
                    assert!(id.contains(".Error."));
                    assert!(armed.cells[id].armed);
                }
            }
        } else {
            assert!(
                observed.values().all(|ok| *ok),
                "independent structural red {row}"
            );
            println!(
                "CM4 STRUCTURAL CONTROL PASS row={row} source_hit=true structural_red=true final_scalar_comparison=false programs={}",
                expected.len()
            );
        }
    }
}

#[test]
fn cm_structural_predicate_negative_controls() {
    for row in ["Some", "G15A", "G15B", "C1"] {
        let mut before = Cell::default();
        before.pauses.insert(false, 1);
        before.scalar.insert(false, "5".into());
        before.structural.insert(false, "finish".into());
        before.trace.insert(false, "finished-pass".into());
        let mut disabled = before.clone();
        disabled.slots.push("InternalContinuationSome[5]".into());
        let mut after = before.clone();
        after.armed = true;
        let phase = match row {
            "Some" => "actual-option-output-encode",
            "G15A" => "shared-value-stage-finish",
            "G15B" => "CallArgument.finish_definition_lhs_active_expr",
            _ => "finish_replace_all_application.application-evaluation-finish",
        };
        after.hits.push(phase.into());
        after.slots.push("5".into());
        after.structural.insert(false, "omitted-finish".into());
        after.trace.insert(false, "omitted-pass".into());
        assert_eq!(caught(row, &before, &disabled, &after), Ok(()));
        let mut survivor = after.clone();
        survivor.structural = before.structural.clone();
        survivor.trace = before.trace.clone();
        survivor.slots = disabled.slots.clone();
        assert_eq!(
            caught(row, &before, &disabled, &survivor),
            Err(Rejection::IdentitySurvivor)
        );
        let mut borrowed = after.clone();
        borrowed.hits = vec!["another-row-phase".into()];
        assert_eq!(
            caught(row, &before, &disabled, &borrowed),
            Err(Rejection::NoHit)
        );
        println!(
            "CM4 STRUCTURAL PREDICATE NEGATIVE row={row} identity_survivor=IdentitySurvivor borrowed_phase=NoHit final_scalar_comparison=false synthetic=true"
        );
    }
    for row in ["P0325", "P0326"] {
        let required: BTreeSet<_> = (0..50).map(|i| format!("{row}:{i}")).collect();
        let mut complete: BTreeMap<_, _> = required.iter().map(|k| (k.clone(), true)).collect();
        assert!(pair_closed(&required, &complete));
        for key in required.iter().take(5) {
            complete.insert(key.clone(), false);
        }
        assert!(!pair_closed(&required, &complete));
        println!(
            "CM4 HELD PAIR NEGATIVE row={row} required=50 caught=45 armed_without_hit=5 complete_per_cell_bar=false synthetic=true"
        );
    }
}
