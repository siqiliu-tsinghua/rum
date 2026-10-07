use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    option(entry, v, "")
}
pub fn option(entry: &str, v: Variant, phase: &str) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    let (key, value) = match name {
        "Monitor" => (
            if phase.is_empty() {
                "EvaluationMonitor"
            } else {
                phase
            },
            "None",
        ),
        "Names" => (
            if phase.is_empty() {
                "SpellingCorrection"
            } else {
                phase
            },
            if phase == "ResolveContextAliases" {
                "True"
            } else {
                "False"
            },
        ),
        "DumpSave" => ("CompressionLevel", "1"),
        "Trace" | "TraceDetail" | "TracePrint" => ("Heads", "False"),
        _ => unreachable!("{entry}"),
    };
    let selected = sequence(&f, v, value);
    let options = match role {
        "defaultOption" => {
            if name.starts_with("Trace") {
                f.setup.push_str(&format!(
                    ";Unprotect[{name}];SetOptions[{name},{key}:>{selected}]"
                ));
            } else {
                // Store an ordinary unresolved value before defining its read.
                // Runtime option reads evaluate ordinary values; :> stays held.
                f.setup.push_str(&format!(";Unprotect[{name}];SetOptions[{name},{key}->Global`optionPhase];Global`optionPhase:={selected}"));
            }
            String::new()
        }
        // Unevaluated bypasses shared preparation (and Monitor's syntax read),
        // leaving the ordinary RHS for this handler's runtime option read.
        "explicitOption" => {
            if name == "DumpSave" {
                format!(",{key}->{selected}")
            } else {
                format!(",Unevaluated[{key}->{selected}]")
            }
        }
        "optionSyntax" => format!(",{}", sequence(&f, v, "EvaluationMonitor->None")),
        _ => unreachable!("{entry}"),
    };
    let expected: String;
    f.source = match name {
        "Monitor" => {
            if v != Variant::Error {
                f.expected_prints.as_mut().unwrap().push("display".into());
            }
            expected = "73".into();
            format!("Monitor[73,\"display\"{options}]")
        }
        "Names" => {
            f.setup.push_str(
                ";OracleNames`a=1;OracleNames`b=2;OracleNames`c=3;OracleNames`d=4;OracleNames`e=5",
            );
            expected="List[\"OracleNames`a\", \"OracleNames`b\", \"OracleNames`c\", \"OracleNames`d\", \"OracleNames`e\"]".into();
            format!("Names[\"OracleNames`*\"{options}]")
        }
        "DumpSave" => {
            f.files = Some(vec![]);
            f.setup.push_str(";OracleSaved`value=73");
            expected = "List[\"OracleSaved`value\"]".into();
            // Loading the written contents validates the delayed write as well.
            if v != Variant::Error {
                project(
                    &mut f,
                    "(OracleSaved`value=91;Get[\"/oracle/saved.rum\"];OracleSaved`value)",
                    "73",
                );
            }
            format!("DumpSave[\"/oracle/saved.rum\",\"OracleSaved`\"{options}]")
        }
        "Trace" | "TraceDetail" | "TracePrint" => {
            expected = if name == "TracePrint" {
                "73"
            } else if name == "TraceDetail" {
                "List[]"
            } else {
                "List[Hold[73]]"
            }
            .into();
            if name == "TracePrint" && v != Variant::Error {
                f.expected_prints.as_mut().unwrap().push("73".into());
            }
            format!("{name}[73]")
        }
        _ => unreachable!(),
    };
    finish(f, v, &expected)
}
