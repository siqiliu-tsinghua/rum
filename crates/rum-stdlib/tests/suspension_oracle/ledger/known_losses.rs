use super::super::{bindings, content_digest, imports::AcceptedInputs, policy::table};
use std::collections::BTreeSet;

const PROMOTION: &str = "docs/dev/codex/brief-622-g15c1-replaceall-application-tail.md:21065";
pub const PROMOTION_HASH: &str = "5c88efea8c602e0320c5148ae0e10c8aba786a376b24cc99f262892f33cdc8e1";
const MISSING: &str = "continuation1/new-lost-witnesses.json";
const HEADER: &str = "row\tstatus\towner\ttuple\tcontext\tprogram_sha256\tsetup_sha256\tanswers_sha256\tsource\tsource_sha256\trequests\tdifference_hex\traw\traw_sha256\tpromotion\tpromotion_sha256\tduty\tmissing";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LossError {
    Inventory,
    Duplicate,
    Population,
    Seal,
    Identity,
    MissingBaseline,
    ChangedBaseline,
    NowSame,
    Promotion,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructuredDifference {
    pub channel: String,
    pub queued: String,
    pub resumed: String,
    pub complete_queued: String,
    pub complete_resumed: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LossRecord {
    pub row: String,
    pub status: String,
    pub owner: String,
    pub tuple: String,
    pub context: String,
    pub program: String,
    pub setup: String,
    pub answers: String,
    pub source: String,
    pub source_hash: String,
    pub requests: Option<usize>,
    pub difference: Option<StructuredDifference>,
    pub raw: String,
    pub raw_hash: String,
    pub promotion: String,
    pub promotion_hash: String,
    pub duty: String,
    pub missing: String,
}

fn unhex(s: &str) -> Result<String, LossError> {
    if s.len() % 2 != 0 {
        return Err(LossError::Inventory);
    }
    let bytes = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| LossError::Inventory))
        .collect::<Result<Vec<_>, _>>()?;
    String::from_utf8(bytes).map_err(|_| LossError::Inventory)
}

pub fn committed() -> Result<Vec<LossRecord>, LossError> {
    let text = include_str!("../inventory/obligations/known-losses.tsv");
    let seals = bindings::provenance().map_err(|_| LossError::Inventory)?;
    let seal: Vec<_> = seals.iter().filter(|r| r[0] == "loss-table").collect();
    if seal.len() != 1
        || seal[0][3] != content_digest(text.as_bytes())
        || seal[0][5] != PROMOTION_HASH
    {
        return Err(LossError::Seal);
    }
    let mut result = Vec::new();
    for r in table(text, HEADER).map_err(|_| LossError::Inventory)? {
        let difference = if r[11] == "-" {
            None
        } else {
            let decoded = unhex(r[11])?;
            let fields: Vec<_> = decoded.split('\0').collect();
            if fields.len() != 5 {
                return Err(LossError::Inventory);
            }
            Some(StructuredDifference {
                channel: fields[0].into(),
                queued: fields[1].into(),
                resumed: fields[2].into(),
                complete_queued: fields[3].into(),
                complete_resumed: fields[4].into(),
            })
        };
        result.push(LossRecord {
            row: r[0].into(),
            status: r[1].into(),
            owner: r[2].into(),
            tuple: r[3].into(),
            context: r[4].into(),
            program: r[5].into(),
            setup: r[6].into(),
            answers: r[7].into(),
            source: r[8].into(),
            source_hash: r[9].into(),
            requests: if r[10] == "-" {
                None
            } else {
                Some(r[10].parse().map_err(|_| LossError::Inventory)?)
            },
            difference,
            raw: r[12].into(),
            raw_hash: r[13].into(),
            promotion: r[14].into(),
            promotion_hash: r[15].into(),
            duty: r[16].into(),
            missing: r[17].into(),
        });
    }
    validate(&result)?;
    Ok(result)
}

fn promotion(record: &LossRecord, acknowledged: Option<&str>) -> bool {
    ["P0941", "P0946"].contains(&record.row.as_str())
        && record.status == "Promoted"
        && record.promotion == PROMOTION
        && record.promotion_hash == PROMOTION_HASH
        && acknowledged == Some(PROMOTION_HASH)
}

pub fn validate(records: &[LossRecord]) -> Result<(), LossError> {
    let mut seen = BTreeSet::new();
    for r in records {
        if !seen.insert(r.row.as_str()) {
            return Err(LossError::Duplicate);
        }
    }
    if seen
        != ["P0755", "P0883", "P0941", "P0946", "P1489"]
            .into_iter()
            .collect()
    {
        return Err(LossError::Population);
    }
    let inputs = AcceptedInputs::committed().map_err(|_| LossError::Inventory)?;
    for r in records {
        let a = &inputs.population.aliases[&r.row];
        if a.cohort != "prior"
            || a.verdict != "Lost"
            || a.owner != r.owner
            || a.function != r.source
            || content_digest(a.tokens.as_bytes()) != r.source_hash
        {
            return Err(LossError::Identity);
        }
        if ["P0941", "P0946"].contains(&r.row.as_str()) {
            if !promotion(r, Some(PROMOTION_HASH)) {
                return Err(LossError::Promotion);
            }
            if r.missing != MISSING
                || r.duty != "PortBlocked(original-baseline)"
                || r.difference.is_some()
                || r.program != "-"
                || r.requests.is_some()
                || r.raw != MISSING
            {
                return Err(LossError::MissingBaseline);
            }
        } else {
            if r.status != "Lost"
                || r.context != "guest_heads"
                || r.duty != "Deferred(G13-transport,bar)"
                || r.requests != Some(1)
                || r.promotion != "-"
                || r.promotion_hash != "-"
                || r.missing != "-"
                || [&r.program, &r.setup, &r.answers, &r.raw_hash]
                    .iter()
                    .any(|h| h.len() != 64)
            {
                return Err(LossError::Identity);
            }
            let d = r.difference.as_ref().ok_or(LossError::MissingBaseline)?;
            if d.channel != "final_outcome.value.text"
                || d.queued == d.resumed
                || d.complete_queued == d.complete_resumed
            {
                return Err(LossError::NowSame);
            }
        }
    }
    Ok(())
}

/// This predicate accepts an observed structured difference, not a SAME flag.
/// The imported encoding baselines are not executed by this brief.
pub fn compare(
    baseline: &LossRecord,
    observed: &LossRecord,
    acknowledged: Option<&str>,
) -> Result<(), LossError> {
    if observed.difference.is_none() {
        return if promotion(baseline, acknowledged) {
            if observed == baseline {
                Ok(())
            } else {
                Err(LossError::ChangedBaseline)
            }
        } else {
            Err(LossError::NowSame)
        };
    }
    if baseline.status == "Promoted" {
        return Err(LossError::Promotion);
    }
    if baseline != observed {
        return Err(LossError::ChangedBaseline);
    }
    Ok(())
}
