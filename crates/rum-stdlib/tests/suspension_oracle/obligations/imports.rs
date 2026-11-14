//! Standing assertions use committed TSVs only. Optional provenance auditing is
//! separate: an absent historical file reports PortBlocked, never changes a row.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use super::identity::*;

pub const INPUTS: &str = include_str!("../inventory/obligations/inputs.tsv");
pub const POPULATION: &str = include_str!("../inventory/obligations/population.tsv");
pub const ALIASES: &str = include_str!("../inventory/obligations/aliases.tsv");
pub const CROSSWALKS: &str = include_str!("../inventory/obligations/crosswalks.tsv");
pub const TRANSFERS: &str = include_str!("../inventory/obligations/transfers.tsv");
pub const OWNERS: &str = include_str!("../inventory/obligations/owners.tsv");

pub const ROW_HEADER: &str = "key\tproducer\tsignature\tarm\tconsumer\tcall_digest\toccurrence\tgroup_size\torder\ttail\tdestructors\tpass\tstrict\tfile\tline\tcolumn\tlegacy_function\ttokens\tpolicies\taliases";
pub const ALIAS_HEADER: &str = "alias\tkind\ttarget\tcohort\towner\tdisposition\tverdict\tpolicies\tkeys\tencoding\tfunction\ttokens\tprovenance\tpaired\tstrict_root\tkey_digest";
pub const TRANSFER_HEADER: &str = "alias\tsource\tcallers\troots\ttail\tdestructors\tretained";

#[derive(Clone, Default)]
pub struct AcceptedInputs {
    pub population: Population,
    pub pins: Vec<(String, usize, String)>,
    pub missing: Vec<String>,
    pub metrics: BTreeMap<String, usize>,
    pub owners: BTreeMap<String, Vec<String>>,
    pub crosswalks: Vec<Vec<String>>,
}

pub fn table<'a>(text: &'a str, header: &str) -> Result<Vec<Vec<&'a str>>, Diagnostics> {
    let mut lines = text.lines();
    if lines.next() != Some(header) {
        return Err(vec![Diagnostic::new(
            Kind::PortBlocked,
            header,
            "inventory-schema",
        )]);
    }
    let width = header.split('\t').count();
    let mut rows = Vec::new();
    let mut previous = None;
    for line in lines {
        if previous.is_some_and(|p| p >= line) {
            return Err(vec![Diagnostic::new(
                Kind::DuplicateInput,
                line,
                "unsorted-or-duplicate-tuple",
            )]);
        }
        previous = Some(line);
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != width || fields.iter().any(|s| s.contains('\r')) {
            return Err(vec![Diagnostic::new(
                Kind::PortBlocked,
                line,
                "inventory-width",
            )]);
        }
        rows.push(fields);
    }
    Ok(rows)
}

pub fn members(cell: &str) -> Result<Vec<String>, Diagnostics> {
    if cell == "-" || cell.is_empty() {
        return Ok(Vec::new());
    }
    let parts: Vec<_> = cell.split(';').map(str::to_owned).collect();
    let mut seen = BTreeSet::new();
    for part in &parts {
        if !seen.insert(part) {
            return Err(vec![Diagnostic::new(Kind::DuplicateInput, cell, part)]);
        }
    }
    if !parts.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(vec![Diagnostic::new(
            Kind::DuplicateInput,
            cell,
            "unordered-members",
        )]);
    }
    Ok(parts)
}

fn number(cell: &str) -> Result<usize, Diagnostics> {
    cell.parse()
        .map_err(|_| vec![Diagnostic::new(Kind::PortBlocked, cell, "unsigned-integer")])
}
fn optional(cell: &str) -> Option<String> {
    (cell != "-").then(|| cell.to_owned())
}

pub fn row_fields(key: &str, row: &Row) -> String {
    [
        key.to_string(),
        row.edge.producer.qualified.clone(),
        row.edge.producer.signature.clone(),
        row.edge.producer.arm.clone(),
        row.edge.consumer.function.clone(),
        row.edge.consumer.call_digest.clone(),
        row.edge.consumer.occurrence.to_string(),
        row.group_size.to_string(),
        row.order.clone(),
        row.tail.clone(),
        row.destructors.clone(),
        row.pass.clone(),
        row.strict.to_string(),
        row.span.file.clone(),
        row.span.line.to_string(),
        row.span.column.to_string(),
        row.legacy_function.clone(),
        row.tokens.clone(),
        row.policies.join(";"),
        row.aliases.join(";"),
    ]
    .join("\t")
}

impl AcceptedInputs {
    pub fn committed() -> Result<Self, Diagnostics> {
        let mut result = Self::default();
        for r in table(INPUTS, "kind\tname\tbytes_or_count\tsha256")? {
            match r[0] {
                "pin" => result.pins.push((r[1].into(), number(r[2])?, r[3].into())),
                "missing" => result.missing.push(r[1].into()),
                "count" => {
                    if result.metrics.insert(r[1].into(), number(r[2])?).is_some() {
                        return Err(vec![Diagnostic::new(Kind::DuplicateInput, r[1], "metric")]);
                    }
                }
                _ => {
                    return Err(vec![Diagnostic::new(
                        Kind::UnknownConstruct,
                        r[0],
                        "input-kind",
                    )]);
                }
            }
        }
        for r in table(POPULATION, ROW_HEADER)? {
            let row = Row {
                edge: EdgeKey {
                    producer: ProducerId {
                        qualified: r[1].into(),
                        signature: r[2].into(),
                        arm: r[3].into(),
                    },
                    consumer: ConsumerSiteId {
                        function: r[4].into(),
                        call_digest: r[5].into(),
                        occurrence: number(r[6])?,
                    },
                },
                group_size: number(r[7])?,
                order: r[8].into(),
                tail: r[9].into(),
                destructors: r[10].into(),
                pass: r[11].into(),
                strict: r[12] == "true",
                span: SourceSpan {
                    file: r[13].into(),
                    line: number(r[14])?,
                    column: number(r[15])?,
                },
                legacy_function: r[16].into(),
                tokens: r[17].into(),
                policies: members(r[18])?,
                aliases: members(r[19])?,
            };
            if row.edge.key() != r[0]
                || row.edge.consumer.call_digest != digest(row.tokens.as_bytes())
            {
                return Err(vec![Diagnostic::new(
                    Kind::PinMismatch,
                    r[0],
                    "source-key-or-token-digest",
                )]);
            }
            if result.population.rows.insert(r[0].into(), row).is_some() {
                return Err(vec![Diagnostic::new(
                    Kind::DuplicateInput,
                    r[0],
                    "edge-key",
                )]);
            }
        }
        for r in table(ALIASES, ALIAS_HEADER)? {
            let alias = AliasRecord {
                alias: r[0].into(),
                kind: r[1].into(),
                target: optional(r[2]),
                cohort: r[3].into(),
                owner: r[4].into(),
                disposition: r[5].into(),
                verdict: r[6].into(),
                policies: members(r[7])?,
                keys: number(r[8])?,
                encoding: number(r[9])?,
                function: r[10].into(),
                tokens: r[11].into(),
                provenance: r[12].into(),
                paired: r[13] == "true",
                strict_root: r[14] == "true",
                key_digest: r[15].into(),
            };
            if !matches!(
                alias.kind.as_str(),
                "row" | "retired" | "duplicate" | "transfer"
            ) {
                return Err(vec![Diagnostic::new(
                    Kind::UnknownConstruct,
                    r[0],
                    "alias-kind",
                )]);
            }
            if result
                .population
                .aliases
                .insert(r[0].into(), alias)
                .is_some()
            {
                return Err(vec![Diagnostic::new(
                    Kind::DuplicateInput,
                    r[0],
                    "accepted-alias",
                )]);
            }
        }
        for r in table(TRANSFERS, TRANSFER_HEADER)? {
            result.population.transfers.push(Transfer {
                alias: r[0].into(),
                source: optional(r[1]),
                callers: members(r[2])?.into_iter().collect(),
                roots: members(r[3])?.into_iter().collect(),
                tail: r[4].into(),
                destructors: r[5].into(),
                retained: r[6] == "true",
            });
        }
        for r in table(
            OWNERS,
            "owner\tkind\tstate\tdeferred\tequal\tlost\tsubject\tsha256",
        )? {
            if result
                .owners
                .insert(r[0].into(), r[1..].iter().map(|s| (*s).into()).collect())
                .is_some()
            {
                return Err(vec![Diagnostic::new(Kind::DuplicateInput, r[0], "owner")]);
            }
        }
        result.crosswalks = table(CROSSWALKS, "version\talias\ttarget\tkind\tprovenance")?
            .into_iter()
            .map(|r| r.into_iter().map(str::to_owned).collect())
            .collect();
        let mut identities = BTreeSet::new();
        for row in &result.crosswalks {
            if !identities.insert((&row[0], &row[1])) {
                return Err(vec![Diagnostic::new(
                    Kind::DuplicateInput,
                    &row[1],
                    "crosswalk-version-identity",
                )]);
            }
        }
        Ok(result)
    }

    pub fn audit_pins(&self, root: &Path) -> Diagnostics {
        let mut out = Vec::new();
        for (path, bytes, hash) in &self.pins {
            match std::fs::read(root.join(path)) {
                Ok(data) => {
                    if data.len() != *bytes || digest(&data) != *hash {
                        out.push(Diagnostic::new(
                            Kind::PinMismatch,
                            path,
                            "historical-content",
                        ));
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => out.push(Diagnostic::new(
                    Kind::PortBlocked,
                    path,
                    "missing-provenance",
                )),
                Err(e) => out.push(Diagnostic::new(Kind::PortBlocked, path, e.to_string())),
            }
        }
        for path in &self.missing {
            out.push(Diagnostic::new(
                Kind::PortBlocked,
                path,
                "absent-unrestorable-input",
            ));
        }
        out.sort();
        out
    }
}
