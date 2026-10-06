use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let expected;
    f.source = match entry {
        "CheckAbort.defaultOption" => {
            f.setup.push_str(&format!(
                ";Unprotect[CheckAbort];SetOptions[CheckAbort,PropagateAborts:>{}]",
                sequence(&f, v, "False")
            ));
            expected = "73";
            "CheckAbort[Abort[],73]".into()
        }
        "CheckAbort.explicitOption" => {
            expected = "73";
            format!(
                "CheckAbort[Abort[],73,PropagateAborts->{}]",
                sequence(&f, v, "False")
            )
        }
        "Failure.propertyParameters" => {
            expected = "\"value 73\"";
            format!(
                "Failure[\"tag\",<|\"MessageTemplate\"->\"value `1`\",\"MessageParameters\":>{}|>][\"Message\"]",
                sequence(&f, v, "{73}")
            )
        }
        "Failure.propertyTemplate" => {
            // Keep MessageName held as stored data. The delayed message text is
            // fetched only while rendering the Failure virtual Message property.
            f.setup.push_str(&format!(
                ";Global`controlMessage::tag:={}",
                sequence(&f, v, "\"value `1`\"")
            ));
            expected = "\"value 73\"";
            "Failure[\"tag\",<|\"MessageTemplate\":>Global`controlMessage::tag,\"MessageParameters\"->{73}|>][\"Message\"]".into()
        }
        _ => unreachable!(),
    };
    finish(f, v, expected)
}
