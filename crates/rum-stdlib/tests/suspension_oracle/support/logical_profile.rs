//! Exact LP01 collections. Durations, pauses and segment data never enter parity.
use rum_runtime::LogicalProfileSnapshot;
use std::collections::{BTreeMap, BTreeSet};

pub fn fingerprint(parts: &[&str]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for bytes in parts.iter().map(|p| p.as_bytes()) {
        for byte in (bytes.len() as u64).to_le_bytes().iter().chain(bytes) {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    format!("fnv1a64:{hash:016x}")
}

pub fn unhex(s: &str) -> String {
    assert_eq!(s.len() % 2, 0);
    String::from_utf8(
        s.as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect(),
    )
    .unwrap()
}

pub fn rows(text: &str, width: usize) -> Result<Vec<Vec<&str>>, String> {
    text.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|line| {
            let fields = line.split('\t').collect::<Vec<_>>();
            if fields.len() != width {
                return Err(format!(
                    "malformed requirement/receipt: expected {width} fields: {line}"
                ));
            }
            Ok(fields)
        })
        .collect()
}

pub fn required(text: &str) -> Result<Vec<Vec<&str>>, String> {
    let text = if text.starts_with("row\towner\t") {
        text.split_once('\n').unwrap().1
    } else {
        text
    };
    let rs = rows(text, 22)?;
    let mut seen = BTreeSet::new();
    for r in &rs {
        if [0, 1, 2, 3, 4, 5, 16, 17, 18, 19, 20, 21]
            .iter()
            .any(|i| r[*i].is_empty())
            || !matches!(
                r[4],
                "Middle" | "Twice" | "Decline" | "Error" | "AlreadyResumed" | "Nested"
            )
            || !r[19].starts_with("fnv1a64:")
            || !seen.insert((r[0], r[2], r[4], r[5]))
        {
            return Err(format!("malformed or duplicate requirement: {r:?}"));
        }
    }
    Ok(rs)
}

pub fn exact(expected: &[String], actual: &[String]) -> Result<(), String> {
    fn bag(rows: &[String]) -> Result<BTreeSet<&str>, String> {
        let mut set = BTreeSet::new();
        for row in rows {
            if !set.insert(row.as_str()) {
                return Err(format!("duplicate tuple/receipt: {row}"));
            }
        }
        Ok(set)
    }
    let expected = bag(expected)?;
    let actual = bag(actual)?;
    if expected != actual {
        return Err(format!(
            "missing={:?}; unexpected={:?}",
            expected.difference(&actual).take(3).collect::<Vec<_>>(),
            actual.difference(&expected).take(3).collect::<Vec<_>>()
        ));
    }
    Ok(())
}

pub fn json(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                use std::fmt::Write;
                write!(out, "\\u{:04x}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn record(tag: &str, fields: &[&str]) -> String {
    let mut fields = std::iter::once(tag)
        .chain(fields.iter().copied())
        .map(json)
        .collect::<Vec<_>>();
    format!("[{}]", fields.drain(..).collect::<Vec<_>>().join(","))
}

pub fn key_record(
    mode: &str,
    id: &str,
    delivery: &str,
    hash: &str,
    s: &LogicalProfileSnapshot,
) -> String {
    format!(
        "[\"keys\",{},{},{},{},[{}]]",
        json(mode),
        json(id),
        json(delivery),
        json(hash),
        s.logical
            .keys()
            .map(|k| json(k))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub fn differences(
    a: &LogicalProfileSnapshot,
    b: &LogicalProfileSnapshot,
) -> Vec<(String, Vec<u64>, Vec<u64>)> {
    // Both collectors have already enumerated their storage maps. Key schema is
    // independently frozen; zero-fill is only for a genuinely absent operation.
    assert_eq!(
        a.maps.keys().collect::<Vec<_>>(),
        b.maps.keys().collect::<Vec<_>>()
    );
    let keys = a
        .logical
        .keys()
        .chain(b.logical.keys())
        .collect::<BTreeSet<_>>();
    keys.into_iter()
        .filter_map(|key| {
            let width = a
                .logical
                .get(key)
                .or_else(|| b.logical.get(key))
                .unwrap()
                .len();
            let old = a
                .logical
                .get(key)
                .cloned()
                .unwrap_or_else(|| vec![0; width]);
            let new = b
                .logical
                .get(key)
                .cloned()
                .unwrap_or_else(|| vec![0; width]);
            (old != new).then(|| (key.clone(), old, new))
        })
        .collect()
}

pub fn loss_record(
    mode: &str,
    program: &str,
    key: &str,
    producer: &str,
    consumer: &str,
    old: &[u64],
    new: &[u64],
) -> Result<String, String> {
    let parts = key.split('|').collect::<Vec<_>>();
    let family = parts[0];
    let phase = if matches!(
        family,
        "runtime"
            | "definition-rule"
            | "rule-rhs"
            | "counter"
            | "cache"
            | "cache-symbol"
            | "cache-source"
    ) {
        format!("{}|{}", family, parts[1])
    } else {
        family.to_string()
    };
    let sites = rows(
        include_str!("../fixtures/logical_profile/phase-sites.tsv"),
        3,
    )
    .unwrap();
    let mut boundaries = sites
        .iter()
        .filter(|r| r[0] == phase)
        .map(|r| format!("{}@{}", r[1], r[2]))
        .collect::<Vec<_>>();
    let owner = match family {
        "runtime" if parts[1].starts_with("set.") => "LP05",
        "runtime" if parts[1].starts_with("module.") => "LP07",
        "runtime"
            if [
                "catch.",
                "while.",
                "compound.",
                "function.",
                "status_function.",
            ]
            .iter()
            .any(|p| parts[1].starts_with(p)) =>
        {
            "LP06"
        }
        "runtime"
            if ["definition.", "down_values.", "sub_values."]
                .iter()
                .any(|p| parts[1].starts_with(p)) =>
        {
            "LP04"
        }
        "runtime" => "LP03",
        "definition-rule" | "rule-rhs" | "definition-dispatch" => "LP04",
        "builtin" => "LP06",
        "counter" if parts[1] == "definition_rhs_eval" => "LP04",
        "counter" if parts[1] == "eval_module" => "LP07",
        "cache" | "cache-symbol" => "LP07",
        "cache-source" if parts[1].starts_with("definition_rhs") => "LP04",
        "cache-source" => "LP07",
        _ => return Err(format!("STOP: uninventoried counter family {key}")),
    };
    if boundaries.is_empty() {
        boundaries = match family {
            "builtin" => vec![
                "crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:185@builtin-handler".into(),
            ],
            "definition-dispatch" => vec![
                "crates/rum-runtime/src/runtime/profiling/timing.rs@definition-dispatch".into(),
            ],
            "counter" | "cache" | "cache-symbol" | "cache-source" => {
                vec![format!("brief-624/other-counters.tsv@{key}")]
            }
            _ => return Err(format!("STOP: uninventoried phase {key}")),
        };
    }
    boundaries.sort();
    boundaries.dedup();
    Ok(format!(
        "[\"loss\",{},{},{},{},{},{},{},{},{old:?},{new:?}]",
        json(mode),
        json(program),
        json(key),
        json(producer),
        json(consumer),
        json(&phase),
        json(&boundaries.join(";")),
        json(owner)
    ))
}

// The frozen JSON interns strings and integer vectors, retaining one record
// per schema/loss/receipt. This avoids repeating source routes 100,000 times.
fn unjson(s: &str) -> String {
    let mut chars = s
        .strip_prefix('"')
        .unwrap()
        .strip_suffix('"')
        .unwrap()
        .chars();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        out.push(match chars.next().unwrap() {
            '"' => '"',
            '\\' => '\\',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'u' => {
                let hex = chars.by_ref().take(4).collect::<String>();
                char::from_u32(u32::from_str_radix(&hex, 16).unwrap()).unwrap()
            }
            c => panic!("invalid frozen JSON escape {c}"),
        });
    }
    out
}
fn numbers(s: &str) -> Vec<u64> {
    let s = s.strip_prefix('[').unwrap().strip_suffix(']').unwrap();
    if s.is_empty() {
        vec![]
    } else {
        s.split(',').map(|x| x.trim().parse().unwrap()).collect()
    }
}
fn ledger_records(mode: &str, receipts: bool) -> Vec<String> {
    let mut section = "";
    let mut strings = Vec::new();
    let mut vectors = Vec::new();
    let mut out = Vec::new();
    for line in include_str!("../inventory/logical-profile-losses.json").lines() {
        let l = line.trim().trim_end_matches(',');
        match l {
            "\"strings\": [" => {
                section = "strings";
                continue;
            }
            "\"vectors\": [" => {
                section = "vectors";
                continue;
            }
            "\"records\": [" => {
                section = "records";
                continue;
            }
            "]" => {
                section = "";
                continue;
            }
            _ => {}
        }
        match section {
            "strings" => strings.push(unjson(l)),
            "vectors" => vectors.push(numbers(l)),
            "records" => {
                let r = numbers(l)
                    .into_iter()
                    .map(|n| n as usize)
                    .collect::<Vec<_>>();
                if strings[r[1]] != mode || (r[0] == 2) != receipts {
                    continue;
                }
                let row = match r[0] {
                    0 => {
                        assert_eq!(r.len(), 6);
                        let fields = r[1..5]
                            .iter()
                            .map(|i| json(&strings[*i]))
                            .collect::<Vec<_>>()
                            .join(",");
                        let keys = vectors[r[5]]
                            .iter()
                            .map(|i| json(&strings[*i as usize]))
                            .collect::<Vec<_>>()
                            .join(",");
                        format!("[\"keys\",{fields},[{keys}]]")
                    }
                    1 => {
                        assert_eq!(r.len(), 11);
                        let fields = r[1..9]
                            .iter()
                            .map(|i| json(&strings[*i]))
                            .collect::<Vec<_>>()
                            .join(",");
                        format!(
                            "[\"loss\",{fields},{:?},{:?}]",
                            vectors[r[9]], vectors[r[10]]
                        )
                    }
                    2 => {
                        assert_eq!(r.len(), 7);
                        format!(
                            "[\"receipt\",{}]",
                            r[1..]
                                .iter()
                                .map(|i| json(&strings[*i]))
                                .collect::<Vec<_>>()
                                .join(",")
                        )
                    }
                    tag => panic!("unknown frozen record {tag}"),
                };
                out.push(row);
            }
            _ => {}
        }
    }
    out
}
pub fn expected_for(mode: &str) -> Vec<String> {
    ledger_records(mode, false)
}
pub fn expected_receipts(mode: &str) -> Vec<String> {
    ledger_records(mode, true)
}

pub fn hashes_by_program(bindings: &[Vec<&str>]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for b in bindings {
        let ids = b[1].split(',').collect::<Vec<_>>();
        let hashes = b[2].split(',').collect::<Vec<_>>();
        assert_eq!(ids.len(), hashes.len(), "unbound exact source fingerprint");
        for (id, hash) in ids.into_iter().zip(hashes) {
            assert!(hash.starts_with("fnv1a64:"), "unfrozen {id}");
            if let Some(previous) = out.insert(id.into(), hash.into()) {
                assert_eq!(previous, hash);
            }
        }
    }
    out
}
