//! Pure owner transactions. Validation never writes an inventory or the input state.
use std::collections::{BTreeMap, BTreeSet};

use super::super::{bindings::TupleKey, content_digest, owners::OwnerState};

pub(super) type Ids = BTreeSet<String>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Requirement {
    pub owner: String,
    pub tuples: BTreeSet<TupleKey>,
    pub source: String,
    pub tail: String,
    pub per_cell_mutants: bool,
    pub alternative: Option<String>,
    pub dependencies: Ids,
    pub crosswalks: Ids,
    pub certificates: Ids,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Duty {
    pub owner: String,
    pub subject: String,
    pub bar: String,
    pub dependencies: Ids,
    pub certificate: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Loss {
    pub owner: String,
    pub row: String,
    pub tuple: TupleKey,
    pub program: String,
    pub difference: String,
    pub promotion: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct Snapshot {
    pub owners: BTreeMap<String, OwnerState>,
    pub requirements: BTreeMap<String, Requirement>,
    pub owned_rows: BTreeMap<String, String>,
    pub active_rows: Ids,
    pub tuple_duties: BTreeMap<String, Duty>,
    pub source_duties: BTreeMap<String, Duty>,
    pub open_tuple_duties: Ids,
    pub open_source_duties: Ids,
    pub losses: BTreeMap<String, Loss>,
    pub crosswalks: BTreeMap<String, String>,
    pub certificates: BTreeMap<String, String>,
    pub encoding: BTreeSet<TupleKey>,
    pub executed_encoding: BTreeSet<TupleKey>,
    pub receipts: BTreeMap<String, String>,
}

impl Snapshot {
    pub fn fingerprint(&self) -> String {
        content_digest(format!("{self:?}").as_bytes())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Observation {
    Same,
    Diff(String),
}

/// A validator input, never a source-mutation receipt or an execution receipt.
/// Tests label their fabricated evidence as synthetic and never export it.
#[derive(Clone, Debug, Default)]
pub(super) struct Evidence {
    pub rows: BTreeMap<String, RowEvidence>,
    pub tuples: BTreeMap<TupleKey, Observation>,
    pub tuple_duties: BTreeMap<String, Duty>,
    pub source_duties: BTreeMap<String, Duty>,
    pub losses: BTreeMap<String, Observation>,
    pub crosswalks: BTreeMap<String, String>,
    pub certificates: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub(super) struct RowEvidence {
    pub row: String,
    pub source: String,
    pub tail: String,
    pub real_tuples: Vec<TupleKey>,
    pub caught_tuples: Vec<TupleKey>,
    pub alternative: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) struct Promotion {
    pub id: String,
    pub owner: String,
    pub before_hash: String,
    pub exact_rows: Vec<String>,
    pub tuple_duties: Ids,
    pub source_duties: Ids,
    pub losses_removed: Ids,
    pub crosswalks: BTreeMap<String, String>,
    pub certificates: BTreeMap<String, String>,
    pub after: Snapshot,
    pub after_hash: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PromotionError {
    BeforeHash,
    AfterHash,
    Owner,
    DoneWithDuty,
    RowSet,
    TupleDuty,
    SourceDuty,
    WrongRow,
    Source,
    Dependency,
    MissingRealTuple,
    DuplicateTuple,
    OwnMutation,
    PartialHeldPair,
    Alternative,
    NewDiff,
    LossSet,
    LossStillDiff,
    StaleNowSame,
    ChangedLoss,
    Crosswalk,
    Certificate,
    Encoding,
    Receipt,
    WrongDelta,
}

pub(super) struct AtomicDelta {
    before_hash: String,
    after: Snapshot,
}

pub(super) fn receipt_hash(p: &Promotion) -> String {
    content_digest(
        format!(
            "{}\0{}\0{}\0{:?}\0{:?}\0{:?}\0{:?}\0{:?}\0{:?}",
            p.id,
            p.owner,
            p.before_hash,
            p.exact_rows,
            p.tuple_duties,
            p.source_duties,
            p.losses_removed,
            p.crosswalks,
            p.certificates
        )
        .as_bytes(),
    )
}

impl AtomicDelta {
    pub fn apply(self, state: &mut Snapshot) -> Result<(), PromotionError> {
        if state.fingerprint() != self.before_hash {
            return Err(PromotionError::BeforeHash);
        }
        *state = self.after;
        Ok(())
    }
}

fn owned(duties: &BTreeMap<String, Duty>, open: &Ids, owner: &str) -> Ids {
    duties
        .iter()
        .filter(|(id, d)| d.owner == owner && open.contains(*id))
        .map(|(id, _)| id.clone())
        .collect()
}

fn integrity(state: &Snapshot) -> Result<(), PromotionError> {
    for (open, duties, error) in [
        (
            &state.open_tuple_duties,
            &state.tuple_duties,
            PromotionError::TupleDuty,
        ),
        (
            &state.open_source_duties,
            &state.source_duties,
            PromotionError::SourceDuty,
        ),
    ] {
        if open.iter().any(|id| {
            duties
                .get(id)
                .is_none_or(|duty| !state.owners.contains_key(&duty.owner))
        }) {
            return Err(error);
        }
    }
    for (row, owner) in &state.owned_rows {
        if !state.owners.contains_key(owner)
            || state
                .requirements
                .get(row)
                .is_none_or(|r| &r.owner != owner)
        {
            return Err(PromotionError::Owner);
        }
    }
    for (owner, status) in &state.owners {
        if matches!(status, OwnerState::Done(_))
            && (state.owned_rows.values().any(|v| v == owner)
                || !owned(&state.tuple_duties, &state.open_tuple_duties, owner).is_empty()
                || !owned(&state.source_duties, &state.open_source_duties, owner).is_empty()
                || state.losses.values().any(|l| &l.owner == owner))
        {
            return Err(PromotionError::DoneWithDuty);
        }
    }
    Ok(())
}

fn exact_tuples(values: &[TupleKey]) -> Result<BTreeSet<TupleKey>, PromotionError> {
    let unique: BTreeSet<_> = values.iter().cloned().collect();
    if unique.len() != values.len() {
        return Err(PromotionError::DuplicateTuple);
    }
    Ok(unique)
}

pub(super) fn validate_promotion(
    before: &Snapshot,
    p: &Promotion,
    evidence: &Evidence,
) -> Result<AtomicDelta, PromotionError> {
    use PromotionError as E;
    if before.fingerprint() != p.before_hash {
        return Err(E::BeforeHash);
    }
    if p.after.fingerprint() != p.after_hash {
        return Err(E::AfterHash);
    }
    integrity(before)?;
    integrity(&p.after)?;
    if before.owners.get(&p.owner) != Some(&OwnerState::Open) {
        return Err(E::Owner);
    }
    if before.encoding != p.after.encoding
        || before.executed_encoding != p.after.executed_encoding
        || evidence.tuples.keys().any(|k| before.encoding.contains(k))
    {
        return Err(E::Encoding);
    }
    let rows: Ids = before
        .owned_rows
        .iter()
        .filter(|(_, owner)| *owner == &p.owner)
        .map(|(row, _)| row.clone())
        .collect();
    if p.exact_rows != rows.iter().cloned().collect::<Vec<_>>()
        || evidence.rows.keys().cloned().collect::<Ids>() != rows
    {
        return Err(E::RowSet);
    }
    let tuple_duties = owned(&before.tuple_duties, &before.open_tuple_duties, &p.owner);
    let source_duties = owned(&before.source_duties, &before.open_source_duties, &p.owner);
    for (required, proposed, observed, all, error) in [
        (
            &tuple_duties,
            &p.tuple_duties,
            &evidence.tuple_duties,
            &before.tuple_duties,
            E::TupleDuty,
        ),
        (
            &source_duties,
            &p.source_duties,
            &evidence.source_duties,
            &before.source_duties,
            E::SourceDuty,
        ),
    ] {
        if required != proposed
            || observed.keys().cloned().collect::<Ids>() != *required
            || observed.iter().any(|(id, duty)| all.get(id) != Some(duty))
        {
            return Err(error);
        }
    }
    let mut required_tuples = BTreeSet::new();
    let mut crosswalks = Ids::new();
    let mut certificates = Ids::new();
    for (all, required) in [
        (&before.tuple_duties, &tuple_duties),
        (&before.source_duties, &source_duties),
    ] {
        for id in required {
            let duty = &all[id];
            certificates.insert(duty.certificate.clone());
            if duty.dependencies.iter().any(|row| {
                !before.requirements.contains_key(row)
                    || (!rows.contains(row) && before.owned_rows.contains_key(row))
            }) {
                return Err(E::Dependency);
            }
        }
    }
    for row in &rows {
        let required = before.requirements.get(row).ok_or(E::RowSet)?;
        let observed = &evidence.rows[row];
        if observed.row != *row {
            return Err(E::WrongRow);
        }
        if required.owner != p.owner
            || observed.source != required.source
            || observed.tail != required.tail
        {
            return Err(E::Source);
        }
        if observed.alternative != required.alternative {
            return Err(E::Alternative);
        }
        crosswalks.extend(required.crosswalks.iter().cloned());
        certificates.extend(required.certificates.iter().cloned());
        if required.dependencies.iter().any(|dep| {
            !before.requirements.contains_key(dep)
                || (!rows.contains(dep) && before.owned_rows.contains_key(dep))
        }) {
            return Err(E::Dependency);
        }
        // Accepted proof dispositions are retained, never converted to SAME.
        if required.alternative.is_some() {
            if !required
                .certificates
                .contains(required.alternative.as_ref().unwrap())
            {
                return Err(E::Certificate);
            }
            if !observed.real_tuples.is_empty() || !observed.caught_tuples.is_empty() {
                return Err(E::Alternative);
            }
            continue;
        }
        if exact_tuples(&observed.real_tuples)? != required.tuples {
            return Err(E::MissingRealTuple);
        }
        required_tuples.extend(required.tuples.iter().cloned());
        let caught = exact_tuples(&observed.caught_tuples)?;
        if caught.is_empty() || !caught.is_subset(&required.tuples) {
            return Err(E::OwnMutation);
        }
        if required.per_cell_mutants && caught != required.tuples {
            return Err(E::PartialHeldPair);
        }
    }
    for row in &before.active_rows {
        let required = before.requirements.get(row).ok_or(E::RowSet)?;
        if required.alternative.is_some() {
            return Err(E::Alternative);
        }
        required_tuples.extend(required.tuples.iter().cloned());
    }
    if evidence.tuples.keys().cloned().collect::<BTreeSet<_>>() != required_tuples {
        return Err(E::MissingRealTuple);
    }
    if evidence.tuples.values().any(|v| v != &Observation::Same) {
        return Err(E::NewDiff);
    }
    let losses: Ids = before
        .losses
        .iter()
        .filter(|(_, l)| l.owner == p.owner)
        .map(|(id, _)| id.clone())
        .collect();
    if p.losses_removed != losses
        || evidence.losses.keys().cloned().collect::<Ids>()
            != before.losses.keys().cloned().collect()
    {
        return Err(E::LossSet);
    }
    for (id, loss) in &before.losses {
        let observed = &evidence.losses[id];
        if losses.contains(id) {
            if !rows.contains(&loss.row)
                || loss.tuple.row() != loss.row
                || evidence.tuples.get(&loss.tuple) != Some(&Observation::Same)
                || loss.promotion != p.id
                || observed != &Observation::Same
            {
                return Err(E::LossStillDiff);
            }
        } else if observed == &Observation::Same {
            return Err(E::StaleNowSame);
        } else if observed != &Observation::Diff(loss.difference.clone()) {
            return Err(E::ChangedLoss);
        }
    }
    if p.crosswalks.keys().cloned().collect::<Ids>() != crosswalks
        || p.crosswalks != evidence.crosswalks
        || p.crosswalks
            .iter()
            .any(|(id, value)| before.crosswalks.get(id) != Some(value))
    {
        return Err(E::Crosswalk);
    }
    if p.certificates.keys().cloned().collect::<Ids>() != certificates
        || p.certificates != evidence.certificates
        || p.certificates
            .iter()
            .any(|(id, value)| before.certificates.get(id) != Some(value))
    {
        return Err(E::Certificate);
    }
    if p.id.is_empty() || before.receipts.contains_key(&p.id) {
        return Err(E::Receipt);
    }
    let mut expected = before.clone();
    expected.owned_rows.retain(|row, _| !rows.contains(row));
    expected.active_rows.extend(
        rows.iter()
            .filter(|row| before.requirements[*row].alternative.is_none())
            .cloned(),
    );
    expected
        .open_tuple_duties
        .retain(|id| !tuple_duties.contains(id));
    expected
        .open_source_duties
        .retain(|id| !source_duties.contains(id));
    expected.losses.retain(|id, _| !losses.contains(id));
    expected
        .owners
        .insert(p.owner.clone(), OwnerState::Done(p.id.clone()));
    expected.receipts.insert(p.id.clone(), receipt_hash(p));
    if p.after != expected {
        return Err(E::WrongDelta);
    }
    Ok(AtomicDelta {
        before_hash: p.before_hash.clone(),
        after: expected,
    })
}
