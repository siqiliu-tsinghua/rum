//! Exact record comparison and the own-row catch predicate.
use super::schema::MutationSpec;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Rejection {
    WrongRow,
    SourceIdentity,
    Unarmed,
    NoHit,
    IdentitySurvivor,
    BaselinePause,
    MissingRealTuple,
    DisabledDifference,
    CompileOnly,
    EarlierGuard,
    NativeDifference,
    PublicOutput,
    BuildIdentity,
    RawArtifact,
}

#[derive(Clone, Debug)]
pub(super) struct Records {
    pub row: String,
    pub source_key: String,
    pub tail_digest: String,
    pub source_schema: String,
    pub arm: String,
    pub binary: String,
    pub selector: String,
    pub success: bool,
    pub cases: Vec<String>,
    pub root_programs: Vec<(String, String)>,
    pub constituents: Vec<(String, String, usize)>,
    pub exact: Vec<String>,
    pub disabled_parity: Vec<String>,
    pub observations: Vec<(String, String, String, String, usize)>,
    pub hits: Vec<(String, String, String)>,
    pub armed: BTreeSet<String>,
    pub hit_before_arm: bool,
    pub predicates: Vec<(String, String, bool, bool)>,
}

fn fields(line: &str) -> std::collections::BTreeMap<&str, &str> {
    line.split(' ').filter_map(|c| c.split_once('=')).collect()
}

impl Records {
    pub fn parse(text: &str) -> Result<Self, Rejection> {
        let ids: Vec<_> = text
            .lines()
            .filter(|l| l.starts_with("CM740_RUN_ID "))
            .collect();
        if ids.is_empty() {
            return Err(Rejection::RawArtifact);
        }
        let id = fields(ids[0]);
        let profiles = ids
            .iter()
            .map(|line| fields(line).get("profile").copied().unwrap_or(""))
            .collect::<BTreeSet<_>>();
        if (ids.len() != 1
            && (id.get("row") != Some(&"P0248")
                || !matches!(id.get("arm"), Some(&"real" | &"disabled"))
                || ids.len() != 2
                || profiles.len() != 2))
            || profiles.iter().any(|p| !matches!(*p, "on" | "off"))
        {
            return Err(Rejection::RawArtifact);
        }
        for line in &ids {
            let other = fields(line);
            for key in [
                "row",
                "source_key",
                "tail_digest",
                "schema",
                "arm",
                "binary",
                "selector",
            ] {
                if id.get(key) != other.get(key) {
                    return Err(Rejection::BuildIdentity);
                }
            }
        }
        let results: Vec<_> = text
            .lines()
            .filter(|l| l.starts_with("test result:"))
            .collect();
        let get = |key| {
            id.get(key)
                .map(|v| v.to_string())
                .ok_or(Rejection::RawArtifact)
        };
        let mut r = Self {
            row: get("row")?,
            source_key: get("source_key")?,
            tail_digest: get("tail_digest")?,
            source_schema: get("schema")?,
            arm: get("arm")?,
            binary: get("binary")?,
            selector: get("selector")?,
            success: results.len() == ids.len()
                && results
                    .iter()
                    .all(|l| l.starts_with("test result: ok. 1 passed; 0 failed;")),
            cases: Vec::new(),
            root_programs: Vec::new(),
            constituents: Vec::new(),
            exact: Vec::new(),
            disabled_parity: Vec::new(),
            observations: Vec::new(),
            hits: Vec::new(),
            armed: BTreeSet::new(),
            hit_before_arm: false,
            predicates: Vec::new(),
        };
        let mut profile = "";
        let mut duration_bindings = BTreeSet::new();
        for line in text.lines() {
            if line.starts_with("CM740_RUN_ID ") {
                profile = fields(line)["profile"];
            } else if line.starts_with("CM740_SOURCE_HIT ") {
                let f = fields(line);
                let get = |key| {
                    f.get(key)
                        .map(|v| v.to_string())
                        .ok_or(Rejection::RawArtifact)
                };
                r.hit_before_arm |= !r.armed.contains(&get("row")?);
                r.hits.push((get("row")?, get("phase")?, get("selector")?));
            } else if line.starts_with("CM740_SELECTOR_ARMED ") {
                let f = fields(line);
                r.armed
                    .insert(f.get("row").ok_or(Rejection::RawArtifact)?.to_string());
            } else if line.starts_with("CM740_OWN_PREDICATE ") {
                let f = fields(line);
                let get = |key| f.get(key).copied().ok_or(Rejection::RawArtifact);
                r.predicates.push((
                    get("row")?.into(),
                    get("predicate")?.into(),
                    get("public_output_exact")? == "true",
                    get("native_difference")? == "true",
                ));
            } else if line.starts_with("CM740_DURATION_PARITY\t") {
                let c: Vec<_> = line.split('\t').collect();
                let index = r.exact.len().checked_sub(1).ok_or(Rejection::RawArtifact)?;
                if c.len() != 3
                    || !super::duration_parity::applies(&r.row)
                    || !matches!(r.arm.as_str(), "real" | "disabled")
                    || !duration_bindings.insert(index)
                    || crate::oracle::obligations::content_digest(r.exact[index].as_bytes()) != c[1]
                {
                    return Err(Rejection::RawArtifact);
                }
                let comparison = super::schema::unhex(c[2]);
                let raw: Vec<_> = r.exact[index].split('\t').collect();
                let encoded: Vec<_> = comparison.split('\t').collect();
                if raw.len() != 6
                    || encoded.len() != 6
                    || raw[0] != "CM740_OBSERVATION"
                    || [raw[0], raw[1], raw[4], raw[5]]
                        != [encoded[0], encoded[1], encoded[4], encoded[5]]
                {
                    return Err(Rejection::RawArtifact);
                }
                if r.row == "P0515" && r.exact[index] != comparison {
                    let (key, program, _) = r.constituents.last().ok_or(Rejection::RawArtifact)?;
                    if *key != super::schema::unhex(raw[1])
                        || !super::duration_parity::declared_p0515_leaf(key, program)
                    {
                        return Err(Rejection::RawArtifact);
                    }
                }
                r.disabled_parity[index] = comparison;
            } else if line.starts_with("CM740_CASE\t")
                || line.starts_with("CM740_CONSTITUENT\t")
                || line.starts_with("CM740_OBSERVATION\t")
                || line.starts_with("CM740_LIFECYCLE\t")
            {
                let c: Vec<_> = line.split('\t').collect();
                r.exact.push(line.into());
                r.disabled_parity.push(line.into());
                if c[0] == "CM740_CASE" && c.len() == 3 {
                    let key = super::schema::unhex(c[1]);
                    let route = key.split('\t').nth(3).ok_or(Rejection::RawArtifact)?;
                    if route.split(';').any(|s| s == "PROFILED=false") != (profile == "off") {
                        return Err(Rejection::RawArtifact);
                    }
                    r.cases.push(key);
                    r.root_programs
                        .push((super::schema::unhex(c[1]), super::schema::unhex(c[2])));
                } else if c[0] == "CM740_CONSTITUENT" && c.len() == 4 {
                    r.constituents.push((
                        super::schema::unhex(c[1]),
                        super::schema::unhex(c[2]),
                        c[3].parse().map_err(|_| Rejection::RawArtifact)?,
                    ));
                } else if c[0] == "CM740_OBSERVATION" && c.len() == 6 {
                    r.observations.push((
                        super::schema::unhex(c[1]),
                        super::schema::unhex(c[2]),
                        super::schema::unhex(c[3]),
                        c[4].into(),
                        c[5].parse().map_err(|_| Rejection::RawArtifact)?,
                    ));
                } else if !matches!(c[0], "CM740_CONSTITUENT" | "CM740_LIFECYCLE") {
                    return Err(Rejection::RawArtifact);
                }
            }
        }
        // Earlier P0515 packets contain only raw records. Their comparison stays byte-exact.
        // Once a packet supplies a duration view, every observation must bind its own view.
        if (r.row == "P0492" || !duration_bindings.is_empty())
            && matches!(r.arm.as_str(), "real" | "disabled")
            && duration_bindings.len() != r.observations.len()
        {
            return Err(Rejection::RawArtifact);
        }
        Ok(r)
    }
}

pub(super) fn validate_constituents(
    records: &Records,
    programs: &std::collections::BTreeMap<String, Vec<String>>,
) -> Result<(), Rejection> {
    let mut roots = Vec::new();
    let mut expected = Vec::new();
    for key in &records.cases {
        let ids = programs.get(key).ok_or(Rejection::MissingRealTuple)?;
        let first = ids.first().ok_or(Rejection::MissingRealTuple)?;
        roots.push((key.clone(), first.clone()));
        expected.extend(
            ids.iter()
                .enumerate()
                .map(|(i, id)| (key.clone(), id.clone(), i)),
        );
    }
    if records.root_programs != roots
        || records.constituents != expected
        || records.observations.len() != expected.len()
        || records
            .observations
            .iter()
            .zip(&expected)
            .any(|(o, c)| o.0 != c.0)
    {
        return Err(Rejection::MissingRealTuple);
    }
    Ok(())
}

/// Pure predicate result, also used by synthetic falsifiers. This is not an
/// authenticated build, execution receipt or source-mutation catch.
pub(super) struct OwnPredicate {
    row: String,
    real_keys: usize,
    hits: usize,
}
impl OwnPredicate {
    pub fn row(&self) -> &str {
        &self.row
    }
    pub fn real_keys(&self) -> usize {
        self.real_keys
    }
    pub fn hits(&self) -> usize {
        self.hits
    }
}

pub(super) fn verify_own_omission(
    spec: &MutationSpec,
    required: &BTreeSet<String>,
    selected: &str,
    real: &Records,
    disabled: &Records,
    armed: &Records,
) -> Result<OwnPredicate, Rejection> {
    for r in [real, disabled, armed] {
        if r.row != spec.row {
            return Err(Rejection::WrongRow);
        }
        if r.source_key != spec.source_key
            || r.tail_digest != spec.tail_digest
            || r.source_schema != spec.source_schema
        {
            return Err(Rejection::SourceIdentity);
        }
        if !r.success {
            return Err(Rejection::CompileOnly);
        }
    }
    if real.arm != "real"
        || disabled.arm != "disabled"
        || armed.arm != "armed"
        || real.binary == disabled.binary
        || real.binary == armed.binary
        || disabled.binary == armed.binary
    {
        return Err(Rejection::BuildIdentity);
    }
    let keys = real.cases.iter().cloned().collect::<BTreeSet<_>>();
    if &keys != required
        || keys.len() != real.cases.len()
        || !required.contains(selected)
        || armed.cases != [selected]
        || real
            .observations
            .iter()
            .map(|o| o.0.clone())
            .collect::<BTreeSet<_>>()
            != *required
    {
        return Err(Rejection::MissingRealTuple);
    }
    let parity = if super::duration_parity::applies(spec.row) {
        disabled.disabled_parity == real.disabled_parity
            && real.disabled_parity.len() == real.exact.len()
            && disabled.disabled_parity.len() == disabled.exact.len()
    } else {
        disabled.exact == real.exact
    };
    if !parity || disabled.cases != real.cases || !disabled.hits.is_empty() {
        return Err(Rejection::DisabledDifference);
    }
    if real.observations.is_empty()
        || real
            .observations
            .iter()
            .any(|o| o.3 != "Ok(())" || o.4 == 0)
    {
        return Err(Rejection::BaselinePause);
    }
    if !real.hits.is_empty() {
        return Err(Rejection::SourceIdentity);
    }
    if armed.selector != spec.selector || !armed.armed.contains(spec.row) {
        return Err(Rejection::Unarmed);
    }
    if armed.hit_before_arm
        || armed.hits.is_empty()
        || armed.hits.iter().any(|(row, phase, selector)| {
            row != spec.row || selector != spec.selector || !spec.phases.contains(&phase.as_str())
        })
    {
        return Err(Rejection::NoHit);
    }
    let baseline: Vec<_> = real
        .observations
        .iter()
        .filter(|o| o.0 == selected)
        .collect();
    if baseline.len() != armed.observations.len() || baseline.is_empty() {
        return Err(Rejection::MissingRealTuple);
    }
    if armed
        .observations
        .iter()
        .zip(&baseline)
        .all(|(a, b)| a.1 == b.1 && a.2 == b.2)
    {
        return Err(Rejection::IdentitySurvivor);
    }
    if armed.predicates.len() != baseline.len()
        || armed
            .predicates
            .iter()
            .any(|(row, _, _, _)| row != spec.row)
    {
        return Err(Rejection::WrongRow);
    }
    if armed
        .predicates
        .iter()
        .zip(&armed.observations)
        .any(|((_, p, _, _), o)| *p != o.3 || o.0 != selected)
    {
        return Err(Rejection::RawArtifact);
    }
    let own = |p: &str| {
        matches!(
            p,
            "Err(Output)"
                | "Err(Projection)"
                | "Err(Native)"
                | "Err(PrintPrefix)"
                | "Err(MessagePrefix)"
                | "Err(TracePrefix)"
                | "Err(Pending)"
                | "Err(Queue)"
                | "Err(Fresh)"
        )
    };
    if armed
        .predicates
        .iter()
        .any(|(_, p, _, _)| p != "Ok(())" && !own(p))
    {
        return Err(Rejection::EarlierGuard);
    }
    if !armed.predicates.iter().any(|(_, p, _, _)| own(p)) {
        return Err(Rejection::IdentitySurvivor);
    }
    if spec.observer != "Strict" {
        if !armed
            .predicates
            .iter()
            .any(|(_, p, _, native)| p == "Err(Native)" && *native)
        {
            return Err(Rejection::NativeDifference);
        }
        if armed.predicates.iter().any(|(_, _, public, _)| !*public) {
            return Err(Rejection::PublicOutput);
        }
    }
    Ok(OwnPredicate {
        row: spec.row.into(),
        real_keys: required.len(),
        hits: armed.hits.len(),
    })
}
