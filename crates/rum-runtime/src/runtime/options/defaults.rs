use super::*;

fn system_rule_option(name: &str, value: Expr) -> SymbolOptionEntry {
    SymbolOptionEntry {
        name: Symbol::system(name),
        value,
        delayed: false,
    }
}

fn bool_option(value: bool) -> Expr {
    system_expr(if value { "True" } else { "False" })
}

pub(in crate::runtime) fn default_system_options(symbol: &Symbol) -> SymbolOptions {
    SymbolOptions::from_entries(default_system_option_snapshot(symbol).entries)
}

/// The declared defaults already have distinct names. An unarmed snapshot can
/// consume them directly; ordinary callers retain the normalizing constructor.
pub(in crate::runtime) fn default_system_option_snapshot(symbol: &Symbol) -> SymbolOptions {
    if symbol.context() != SYSTEM_CONTEXT {
        return SymbolOptions::default();
    }

    let entries = match symbol.name() {
        "ArrayFilter" => vec![system_rule_option("Padding", Expr::string("Fixed"))],
        "Cases" | "DeleteCases" | "Count" | "MemberQ" | "Level" | "Depth" | "Replace" => {
            vec![system_rule_option("Heads", bool_option(false))]
        }
        "ConformSchema" | "SchemaErrors" => {
            vec![system_rule_option("AllowExtraKeys", bool_option(false))]
        }
        "FromCsv" => vec![
            system_rule_option("Delimiter", Expr::string(",")),
            system_rule_option("Headers", bool_option(false)),
        ],
        "RandomPassword" => vec![
            system_rule_option("Require", Expr::call(system_expr("List"), Vec::new())),
            system_rule_option("NoAdjacentRepeats", bool_option(false)),
            system_rule_option("Exclude", Expr::call(system_expr("List"), Vec::new())),
            system_rule_option("IgnoreCase", bool_option(false)),
        ],
        "ReplaceRepeated" => vec![system_rule_option(
            "MaxIterations",
            system_expr("Automatic"),
        )],
        "Map" | "MapIndexed" | "MapAll" | "Scan" | "Apply" | "MapApply" | "Comap"
        | "ComapApply" => vec![system_rule_option("Heads", bool_option(false))],
        "Trace" | "TraceDetail" | "TracePrint" => {
            vec![system_rule_option("Heads", bool_option(false))]
        }
        "FreeQ" | "Position" => vec![system_rule_option("Heads", bool_option(true))],
        "CheckAbort" => vec![system_rule_option(
            "PropagateAborts",
            system_expr("Automatic"),
        )],
        "Monitor" => vec![
            system_rule_option("EvaluationMonitor", system_expr("None")),
            system_rule_option("StepMonitor", system_expr("None")),
        ],
        "Dialog" => vec![
            system_rule_option("DialogProlog", system_expr("Null")),
            system_rule_option("DialogSymbols", Expr::call(system_expr("List"), Vec::new())),
        ],
        "Input" | "InputString" => vec![system_rule_option("FieldMasked", system_expr("False"))],
        "TestCreate" => vec![
            system_rule_option("TestID", system_expr("None")),
            system_rule_option(
                "ExpectedMessages",
                Expr::call(system_expr("List"), Vec::new()),
            ),
            system_rule_option("SameTest", system_expr("SameQ")),
        ],
        "TestReport" => vec![
            system_rule_option("SameTest", system_expr("SameQ")),
            system_rule_option("TimeConstraint", system_expr("Infinity")),
            system_rule_option("MemoryConstraint", system_expr("Infinity")),
        ],
        "Save" => vec![
            system_rule_option("ExcludedContexts", system_expr("Automatic")),
            system_rule_option("IncludedContexts", system_expr("All")),
        ],
        "DumpSave" => vec![
            system_rule_option("CompressionLevel", Expr::integer(BigInt::from(1))),
            system_rule_option("SymbolAttributes", bool_option(true)),
        ],
        "Once" => vec![
            system_rule_option("ExpirationDate", system_expr("None")),
            system_rule_option("PersistenceTime", system_expr("None")),
        ],
        "Names" => vec![
            system_rule_option("IgnoreCase", bool_option(false)),
            system_rule_option("ResolveContextAliases", bool_option(true)),
            system_rule_option("SpellingCorrection", bool_option(false)),
        ],
        "Nearest" => vec![system_rule_option(
            "DistanceFunction",
            system_expr("Automatic"),
        )],
        "Union" | "Intersection" | "Complement" | "DuplicateFreeQ" | "IntersectingQ"
        | "DisjointQ" | "SubsetQ" | "ContainsAll" | "ContainsAny" | "ContainsExactly"
        | "ContainsNone" | "ContainsOnly" => {
            vec![system_rule_option("SameTest", system_expr("Automatic"))]
        }
        "StringContainsQ" | "StringStartsQ" | "StringEndsQ" | "StringMatchQ" | "StringFreeQ"
        | "StringDelete" | "StringReplace" | "StringSplit" => {
            vec![system_rule_option("IgnoreCase", bool_option(false))]
        }
        "StringWrap" => vec![system_rule_option("PreserveNewlines", bool_option(false))],
        "StringIndent" => vec![system_rule_option("BlankLines", bool_option(false))],
        "AlignColumns" => vec![system_rule_option("Separator", Expr::string("  "))],
        "StringCases" => vec![
            system_rule_option("IgnoreCase", bool_option(false)),
            system_rule_option("Overlaps", bool_option(false)),
        ],
        "StringPosition" => vec![
            system_rule_option("IgnoreCase", bool_option(false)),
            system_rule_option("Overlaps", bool_option(true)),
        ],
        "StringCount" => vec![
            system_rule_option("IgnoreCase", bool_option(false)),
            system_rule_option("Overlaps", bool_option(false)),
        ],
        "TemplateApply" | "Uncompress" | "HtmlUnescape" => {
            vec![system_rule_option("Strict", bool_option(false))]
        }
        "ToCsv" => vec![
            system_rule_option("Delimiter", Expr::string(",")),
            system_rule_option("Headers", system_expr("Automatic")),
        ],
        "ToJson" => vec![system_rule_option("Indent", system_expr("None"))],
        "ToXml" => vec![system_rule_option("Indent", system_expr("None"))],
        "UrlEncode" | "UrlDecode" => vec![system_rule_option("FormEncoding", bool_option(false))],
        "UrlParse" => vec![
            system_rule_option("DefaultScheme", system_expr("None")),
            system_rule_option("DecodeQuery", bool_option(false)),
        ],
        "ParseArgs" => vec![system_rule_option("ShortOptions", bool_option(false))],
        _ => Vec::new(),
    };

    debug_assert!(entries.iter().enumerate().all(|(index, entry)| {
        entries[..index]
            .iter()
            .all(|previous| previous.name != entry.name)
    }));
    SymbolOptions { entries }
}
