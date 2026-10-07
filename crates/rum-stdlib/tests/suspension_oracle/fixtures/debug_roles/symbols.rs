use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let mut expected = "73";
    f.source = match entry {
        "Check.body" => format!(
            "Check[{};Message[Global`probe::tag];73,91]",
            sequence(&f, v, "Null")
        ),
        "Quiet.body" => format!(
            "Quiet[{};Message[Global`probe::tag];73]",
            sequence(&f, v, "Null")
        ),
        "Message.template" => {
            f.setup.push_str(&format!(
                ";Global`probe::tag:={}",
                sequence(&f, v, "\"oracle `1`\"")
            ));
            expected = "Null";
            "Message[Global`probe::tag,73]".into()
        }
        "OptionValue.delayedValue" | "OptionValue.scopedDefault" => {
            f.setup.push_str(&format!(
                ";Options[Global`optionOwner]={{Global`answer:>{}}}",
                sequence(&f, v, "73")
            ));
            if entry.ends_with("scopedDefault") {
                f.setup
                    .push_str(";Global`optionOwner[OptionsPattern[]]:=OptionValue[Global`answer]");
                "Global`optionOwner[]".into()
            } else {
                "OptionValue[Global`optionOwner,Global`answer]".into()
            }
        }
        "Unset.target" => {
            f.setup.push_str(";Global`target[73]=91");
            project(
                &mut f,
                "Global`target[73]",
                if v == Variant::Error {
                    "91"
                } else {
                    "target[73]"
                },
            );
            expected = "Null";
            format!("Unset[Global`target[{}]]", sequence(&f, v, "73"))
        }
        "SetAttributes.attributes" | "ClearAttributes.attributes" => {
            let name = entry.split_once('.').unwrap().0;
            if name == "ClearAttributes" {
                f.setup
                    .push_str(";SetAttributes[Global`attrTarget,{Flat,Orderless}]");
            }
            project(
                &mut f,
                "Attributes[Global`attrTarget]",
                match (name, v == Variant::Error) {
                    ("SetAttributes", false) => "List[Flat]",
                    ("SetAttributes", true) => "List[]",
                    (_, false) => "List[Orderless]",
                    (_, true) => "List[Orderless, Flat]",
                },
            );
            expected = "Null";
            format!(
                "{name}[Global`attrTarget,Unevaluated[{}]]",
                sequence(&f, v, "{Flat}")
            )
        }
        "SetOptions.optionSyntax" => {
            f.setup
                .push_str(";Options[Global`optionTarget]={Global`answer->19}");
            expected = "List[Rule[answer, 73]]";
            project(
                &mut f,
                "OptionValue[Global`optionTarget,Global`answer]",
                if v == Variant::Error { "19" } else { "73" },
            );
            format!(
                "SetOptions[Global`optionTarget,Unevaluated[{}]]",
                sequence(&f, v, "Global`answer->73")
            )
        }
        _ => unreachable!("{entry}"),
    };
    if entry == "Check.body" {
        expected = "91";
    }
    finish(f, v, expected)
}
