use rum_syntax::{SyntaxExpr, SyntaxExprKind, parse_expr};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct SymbolParts(pub String, pub String, pub Option<u64>);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Key {
    Runtime(String, Option<SymbolParts>),
    Builtin(SymbolParts, Option<SymbolParts>, u16),
    Definition(String, SymbolParts, u32),
    Rhs(String, Option<SymbolParts>),
    Dispatch(String, SymbolParts, Option<SymbolParts>),
    Counter(String),
    Cache(String),
    CacheSymbol(String, SymbolParts),
    CacheSource(String, SymbolParts),
}

impl Key {
    pub fn family(&self) -> &'static str {
        match self {
            Self::Runtime(..) => "runtime",
            Self::Builtin(..) => "builtin",
            Self::Definition(..) => "definition-rule",
            Self::Rhs(..) => "rule-rhs",
            Self::Dispatch(..) => "definition-dispatch",
            Self::Counter(..) => "counter",
            Self::Cache(..) => "cache",
            Self::CacheSymbol(..) => "cache-symbol",
            Self::CacheSource(..) => "cache-source",
        }
    }
}

fn component(text: &str) -> String {
    let mut bytes = Vec::new();
    let mut input = text.bytes();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let a = char::from(input.next().expect("percent escape first digit"));
            let b = char::from(input.next().expect("percent escape second digit"));
            let decoded = (a.to_digit(16).unwrap() * 16 + b.to_digit(16).unwrap()) as u8;
            assert!(b"%|,()#".contains(&decoded), "undeclared escape");
            bytes.push(decoded);
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8(bytes).unwrap()
}

fn symbol(text: &str) -> SymbolParts {
    let inside = text.strip_prefix("s(").unwrap().strip_suffix(')').unwrap();
    let parts = inside.split(',').collect::<Vec<_>>();
    assert_eq!(parts.len(), 3);
    SymbolParts(
        component(parts[0]),
        component(parts[1]),
        if parts[2] == "n" {
            None
        } else {
            Some(parts[2].parse().unwrap())
        },
    )
}

fn source(text: &str) -> Option<SymbolParts> {
    if text == "-" {
        None
    } else {
        Some(symbol(text))
    }
}

pub(super) fn key(text: &str) -> Key {
    let p = text.split('|').collect::<Vec<_>>();
    match p.as_slice() {
        ["runtime", phase, src] => Key::Runtime((*phase).into(), source(src)),
        ["builtin", sym, src, arity] => {
            Key::Builtin(symbol(sym), source(src), arity.parse().unwrap())
        }
        ["definition-rule", kind, sym, index] => {
            Key::Definition((*kind).into(), symbol(sym), index.parse().unwrap())
        }
        ["rule-rhs", phase, src] => Key::Rhs((*phase).into(), source(src)),
        ["definition-dispatch", kind, sym, src] => {
            Key::Dispatch((*kind).into(), symbol(sym), source(src))
        }
        ["counter", name] => Key::Counter((*name).into()),
        ["cache", name] => Key::Cache((*name).into()),
        ["cache-symbol", event, sym] => Key::CacheSymbol((*event).into(), symbol(sym)),
        ["cache-source", event, sym] => Key::CacheSource((*event).into(), symbol(sym)),
        _ => panic!("unapproved key {text}"),
    }
}

pub(super) fn list(expr: &SyntaxExpr) -> &[SyntaxExpr] {
    match &expr.kind {
        SyntaxExprKind::List(items) => items,
        other => panic!("expected list: {other:?}"),
    }
}
pub(super) fn string(expr: &SyntaxExpr) -> &str {
    match &expr.kind {
        SyntaxExprKind::String(text) => text,
        other => panic!("expected string: {other:?}"),
    }
}
fn number(expr: &SyntaxExpr) -> u64 {
    match &expr.kind {
        SyntaxExprKind::Integer(text) => text.parse().unwrap(),
        other => panic!("expected integer: {other:?}"),
    }
}
fn vector(expr: &SyntaxExpr) -> Vec<u64> {
    list(expr).iter().map(number).collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Row {
    pub vector: Vec<u64>,
    pub accounting: [u64; 3],
    pub telemetry: Vec<u64>,
    pub durations: Vec<u64>,
}

#[derive(Debug)]
pub(super) struct Snapshot {
    pub label: String,
    pub epoch: Option<u64>,
    pub validity: String,
    pub reason: Option<String>,
    pub mode: String,
    pub faults: u64,
    pub keys: Vec<String>,
    pub rows: BTreeMap<Key, Row>,
}

pub(super) fn parse(text: &str) -> Snapshot {
    assert!(!text.contains('\n'), "raw newline in diagnostic");
    let expr = parse_expr(text).expect("schema-v2 must parse; never skip malformed output");
    let top = list(&expr);
    let has_reason = top.len() == 18;
    assert!(matches!(top.len(), 16 | 18));
    let fields = top
        .chunks_exact(2)
        .map(|pair| string(&pair[0]))
        .collect::<Vec<_>>();
    let mut names = vec!["schema", "label", "epoch", "validity"];
    if has_reason {
        names.push("reason");
    }
    names.extend(["mode", "logical", "telemetry", "durations"]);
    assert_eq!(fields, names);
    let values = top
        .chunks_exact(2)
        .map(|pair| (string(&pair[0]), &pair[1]))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(number(values["schema"]), 2);
    let epoch = match &values["epoch"].kind {
        SyntaxExprKind::Symbol(text) if text == "Null" => None,
        _ => Some(number(values["epoch"])),
    };
    let telemetry = list(values["telemetry"]);
    assert_eq!(telemetry.len(), 4);
    assert_eq!(string(&telemetry[0]), "faults");
    assert_eq!(string(&telemetry[2]), "rows");
    let durations = list(values["durations"]);
    assert_eq!(durations.len(), 4);
    assert_eq!(string(&durations[0]), "basis");
    assert_eq!(
        string(&durations[1]),
        "epoch-accrued+observing-thread-running"
    );
    assert_eq!(string(&durations[2]), "rows");
    let mut keys = Vec::new();
    let mut rows = BTreeMap::new();
    for row in list(values["logical"]) {
        let row = list(row);
        assert_eq!(row.len(), 9);
        assert_eq!(
            [
                string(&row[1]),
                string(&row[3]),
                string(&row[5]),
                string(&row[7])
            ],
            ["vector", "open", "abandoned", "cancelled"]
        );
        keys.push(string(&row[0]).to_owned());
        assert!(
            rows.insert(
                key(string(&row[0])),
                Row {
                    vector: vector(&row[2]),
                    accounting: [number(&row[4]), number(&row[6]), number(&row[8])],
                    telemetry: Vec::new(),
                    durations: Vec::new(),
                }
            )
            .is_none(),
            "duplicate decoded key"
        );
    }
    assert!(
        keys.windows(2)
            .all(|pair| pair[0].as_bytes() < pair[1].as_bytes()),
        "encoded byte order / duplicate key"
    );
    for (is_time, section) in [(false, &telemetry[3]), (true, &durations[3])] {
        let entries = list(section);
        assert_eq!(entries.len(), keys.len());
        for (expected, row) in keys.iter().zip(entries) {
            let row = list(row);
            assert_eq!(row.len(), 2);
            assert_eq!(string(&row[0]), expected);
            let output = rows.get_mut(&key(expected)).unwrap();
            if is_time {
                output.durations = vector(&row[1]);
            } else {
                output.telemetry = vector(&row[1]);
            }
        }
    }
    Snapshot {
        label: string(values["label"]).into(),
        epoch,
        validity: string(values["validity"]).into(),
        reason: values.get("reason").map(|v| string(v).into()),
        mode: string(values["mode"]).into(),
        faults: number(&telemetry[1]),
        keys,
        rows,
    }
}
