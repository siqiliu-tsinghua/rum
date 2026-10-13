use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    parsed(entry, v, false)
}
pub fn wrapped_uncompress(entry: &str, v: Variant) -> Fixture {
    parsed(entry, v, true)
}
fn parsed(entry: &str, v: Variant, wrapped_uncompress: bool) -> Fixture {
    let mut f = base(entry, v);
    let body = sequence(&f, v, "7");
    let wrapper = "Function[Global`x,(Global`suffix++;Global`x+100)]";
    let mut suffix = "1";
    match entry {
        "ToExpression.parsedSource" => {
            let source = body.replace('\\', "\\\\").replace('"', "\\\"");
            f.source = format!("ToExpression[\"{source}\",InputForm,{wrapper}]");
        }
        "ToExpression.resultHead" => {
            let body = sequence(&f, v, "Global`x+100");
            f.source = format!(
                "ToExpression[\"7\",InputForm,Function[Global`x,({body};Global`suffix++;Global`x+100)]]"
            );
            if v == Variant::Error {
                suffix = "0";
            }
        }
        "Uncompress.parsedSource" => {
            f.setup.push_str(&format!(
                ";Global`compressed=Compress[Global`decoded[]];Global`decoded[]:={body}"
            ));
            f.source = if wrapped_uncompress {
                format!("Uncompress[Global`compressed,{wrapper}]")
            } else {
                "Uncompress[Global`compressed]".into()
            };
            if !wrapped_uncompress {
                suffix = "0";
            }
        }
        _ => unreachable!("{entry}"),
    }
    if v == Variant::Error {
        suffix = "0";
    }
    project(&mut f, "Global`suffix", suffix);
    finish(
        f,
        v,
        if entry.starts_with("Uncompress") && !wrapped_uncompress {
            "7"
        } else {
            "107"
        },
    )
}
