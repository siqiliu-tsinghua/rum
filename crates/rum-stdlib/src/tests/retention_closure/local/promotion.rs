//! Inventory is replayed against marks earned in this process, never trusted.
use super::super::{consumers, source};
use super::*;

pub(super) const HEADER: &str = "# body\tstatus\tidentity\tsource\thash=true certificates\thash=false certificates\tpreconditions\tstrength\texclusions\n";
pub(super) const INVENTORY: &str = include_str!("../inventory/promoted.tsv");
pub(super) const OBLIGATIONS: &str = include_str!("../inventory/obligations.tsv");
pub(super) const CONSUMERS: &str = include_str!("../inventory/consumer-obligations.tsv");

pub(super) fn population(source: &Source) -> Source {
    source::test_module(
        Source::clone(source),
        "crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs",
        "rum_runtime::runtime::accounting_mode::tests",
    )
}

pub(super) fn earn(source: &Source, index: &Interfaces) -> Marks {
    let mut marks = Marks::contracts(
        contracts::read(include_str!("../inventory/local-contracts.tsv")).unwrap(),
    )
    .unwrap();
    for (_, id) in certificates::roots() {
        let f = resolve::function(source, &id).unwrap();
        marks
            .certify(index, f)
            .unwrap_or_else(|report| panic!("fresh certificate {id}: {}", report.text()));
    }
    marks
}

pub(super) fn expected(
    index: &Interfaces,
    marks: &Marks,
) -> Result<BTreeMap<String, String>, String> {
    let records = marks.production_records(index)?;
    let expected: BTreeSet<_> = types::MODES
        .iter()
        .map(|mode| {
            option_boundaries::key(
                &option_boundaries::c01(),
                &[
                    protocol::nominal(mode),
                    protocol::nominal(certificates::DESCRIPTOR),
                ],
            )
        })
        .chain([certificates::roots()[1].1.clone()])
        .collect();
    if records
        .iter()
        .map(|(i, _)| i.key.clone())
        .collect::<BTreeSet<_>>()
        != expected
    {
        return Err("missing or additional production instance mark".into());
    }
    let contracts: BTreeMap<_, _> = include_str!("../inventory/local-contracts.tsv")
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            (fields[1], fields)
        })
        .collect();
    let mut result = BTreeMap::new();
    for (label, body) in certificates::roots() {
        let mut instances = vec![];
        for (instance, dependencies) in records.iter().filter(|(i, _)| i.body == body) {
            let mut covered = BTreeMap::new();
            let mut direct_contracts = BTreeMap::new();
            for (target, stamp) in *dependencies {
                if marks.lookup(index, target)? != *stamp {
                    return Err(format!("changed direct certificate dependency {target}"));
                }
                if let Some(c) = contracts.get(target.as_str()) {
                    direct_contracts.insert(
                        target,
                        format!(
                            "{}@{stamp};precondition={};guarantee={};strength={};exclusions={}",
                            c[0], c[5], c[7], c[12], c[11]
                        ),
                    );
                } else {
                    covered.insert(target, stamp);
                }
            }
            instances.push(format!("instance={instance:?};mark={};direct-covered={covered:?};direct-contracts={direct_contracts:?}", instance.fingerprint()));
        }
        result.insert(label, instances.join(" || "));
    }
    Ok(result)
}

pub(super) fn reconcile(
    source: &Source,
    index: &Interfaces,
    marks: &Marks,
    inventory: &str,
    obligations: &str,
    consumer_inventory: &str,
) -> Result<(), String> {
    let expected = expected(index, marks)?;
    let mut seen = BTreeSet::new();
    for line in inventory
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let row: Vec<_> = line.split('\t').collect();
        if row.len() != 9 || row.iter().any(|field| field.is_empty()) {
            return Err("promotion requires nine nonempty fields including both builds".into());
        }
        let (_, id) = certificates::roots()
            .into_iter()
            .find(|(label, _)| label == row[0])
            .ok_or("unknown production promotion")?;
        if !seen.insert(row[0].to_string()) || row[1] != "LocalCovered" || row[2] != id {
            return Err("duplicate, redirected or nonlocal production promotion".into());
        }
        if row[3] != resolve::function(source, &id)?.site.display() {
            return Err("production source anchor expired".into());
        }
        if row[6] != certificates::PRECONDITIONS
            || row[7] != certificates::STRENGTH
            || row[8] != certificates::EXCLUSIONS
        {
            return Err("production preconditions, strengths or exclusions changed".into());
        }
        let active = if cfg!(feature = "hash") { 4 } else { 5 };
        if row[active] != expected[row[0]] {
            return Err(format!(
                "stale or omitted instance certificate {} hash={}",
                row[0],
                cfg!(feature = "hash")
            ));
        }
        // The other build must be populated; its exact replay is mandatory in
        // that build's gate. This run never manufactures the other build's mark.
        if !row[9 - active].contains("instance=Instance {") {
            return Err("absent other-build instance certificate".into());
        }
    }
    if seen != BTreeSet::from(["C01".into(), "C02".into()]) {
        return Err("production promotion removed or absent".into());
    }
    let mut body_rows = BTreeSet::new();
    for line in obligations
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let row: Vec<_> = line.split('\t').collect();
        if row.len() != 8 {
            return Err("malformed obligation".into());
        }
        if let Some((label, id)) = certificates::roots()
            .into_iter()
            .find(|(label, _)| label == row[0])
        {
            if row[1] != label
                || row[2] != "Candidate"
                || row[3] != "LocalCovered"
                || row[7] != id
                || !body_rows.insert(label)
            {
                return Err("production obligation removed, redirected or stale".into());
            }
        } else if row[3] != "Pending" {
            return Err("callee promotion does not cover a caller obligation".into());
        }
    }
    if body_rows.len() != 2 {
        return Err("production obligation absent".into());
    }
    consumers::reconcile(source, consumer_inventory)?;
    Ok(())
}

pub(super) struct SharedPopulation {
    pub source: Source,
    pub index: Interfaces,
}

pub(super) fn shared_population() -> &'static SharedPopulation {
    static POPULATION: std::sync::OnceLock<SharedPopulation> = std::sync::OnceLock::new();
    POPULATION.get_or_init(|| {
        let source = population(source::production());
        let index = Interfaces::new(&source);
        SharedPopulation { source, index }
    })
}

pub(in super::super) fn verify(input: &Source) {
    let shared = shared_population();
    assert!(std::ptr::eq(input, source::production()));
    let marks = earn(&shared.source, &shared.index);
    reconcile(
        &shared.source,
        &shared.index,
        &marks,
        INVENTORY,
        OBLIGATIONS,
        CONSUMERS,
    )
    .unwrap();
}
