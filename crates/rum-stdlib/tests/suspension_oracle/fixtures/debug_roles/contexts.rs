use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    if entry == "Once.body" {
        let call = format!("Once[{}]", sequence(&f, v, "73"));
        f.source = format!("{{{call},{call}}}");
        return finish(f, v, "List[73, 73]");
    }
    f.files = Some(vec![]);
    let mut expected = "73";
    let forms = (1..=5)
        .map(|i| step(&f, v, i, "73"))
        .collect::<Vec<_>>()
        .join("\n");
    if entry == "BeginPackage.dependency" {
        // Five dependencies are five separate loads. Suspension occurs in
        // dependency two/four, after earlier dependency commits.
        for i in 1..=5 {
            let source = step(&f, v, i, "73");
            f.files
                .as_mut()
                .unwrap()
                .push((format!("/oracle/OracleDep{i}.rum"), source));
        }
        f.source="BeginPackage[\"OracleOuter`\",{\"OracleDep1`\",\"OracleDep2`\",\"OracleDep3`\",\"OracleDep4`\",\"OracleDep5`\"}]".into();
        expected = "\"OracleOuter`\"";
    } else {
        f.files.as_mut().unwrap().push((
            "/oracle/OracleLoaded.rum".into(),
            format!("{forms}\nOracleLoaded`value=73"),
        ));
        f.source = match entry {
            "Get.fileForms" => "Get[\"/oracle/OracleLoaded.rum\"]".into(),
            "Needs.fileForms" => {
                expected = "Null";
                "Needs[\"OracleLoaded`\"]".into()
            }
            "BeginPackage.fileForms" => {
                expected = "\"OracleOuter`\"";
                "BeginPackage[\"OracleOuter`\",{\"OracleLoaded`\"}]".into()
            }
            "DeclarePackage.fileForms" => {
                f.setup
                    .push_str(";DeclarePackage[\"OracleLoaded`\",{\"value\"}]");
                "OracleLoaded`value".into()
            }
            _ => unreachable!("{entry}"),
        };
        project(
            &mut f,
            "Length[OwnValues[OracleLoaded`value]]",
            if v == Variant::Error { "0" } else { "1" },
        );
    }
    project(&mut f, "$InputFileName", "\"\"");
    project(
        &mut f,
        "$Context",
        if entry.starts_with("BeginPackage.") && v != Variant::Error {
            "\"OracleOuter`\""
        } else {
            "\"Global`\""
        },
    );
    let package = if entry == "BeginPackage.dependency" {
        "OracleDep5`"
    } else {
        "OracleLoaded`"
    };
    project(
        &mut f,
        &format!("MemberQ[$Packages,\"{package}\"]"),
        if v != Variant::Error && entry != "Get.fileForms" {
            "True"
        } else {
            "False"
        },
    );
    // Failed loads restore their transaction's definitions, including the log.
    if v == Variant::Error {
        f.expected_log = Some(
            if entry == "BeginPackage.dependency" {
                "List[1, 2, 3, 4]"
            } else {
                "List[]"
            }
            .into(),
        );
        f.expected_projections[0] = if entry == "BeginPackage.dependency" {
            "4"
        } else {
            "0"
        }
        .into();
    }
    let mut f = finish(f, v, expected);
    if v == Variant::Error {
        f.expected_error = None;
        f.expected_value = Some(
            if entry == "DeclarePackage.fileForms" {
                "OracleLoaded`value"
            } else {
                "$Failed"
            }
            .into(),
        );
    }
    f
}
