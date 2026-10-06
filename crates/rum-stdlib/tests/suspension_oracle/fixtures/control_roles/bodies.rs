use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let seq = sequence(&f, v, "73");
    let mut expected = "73";
    f.source = match entry {
        "CompoundExpression.sequence" => seq,
        "AbortProtect.body" => {
            project(
                &mut f,
                "(Global`abortSuffix=0;CheckAbort[(Abort[];Global`abortSuffix=1),73];Global`abortSuffix)",
                "0",
            );
            format!("AbortProtect[{seq}]")
        }
        "Catch.body" => format!("Catch[({seq};Throw[73,tag]),tag]"),
        "CatchExceptions.body" => format!(
            "CatchExceptions[({seq};ThrowException[\"Exception\",<||>]),{{Exception[\"Exception\",_]:>73}}]"
        ),
        "CatchExceptions.handler" => format!(
            "CatchExceptions[ThrowException[\"Exception\",<||>],{{Exception[\"Exception\",_]:>{seq}}}]"
        ),
        "CheckAbort.body" => format!("CheckAbort[({seq};Abort[]),73]"),
        "Enclose.body" => format!("Enclose[({seq};Confirm[$Failed]),Function[failure,73]]"),
        "WithCleanup.body" => {
            project(&mut f, "Global`cleanupCount", "1");
            format!("WithCleanup[{seq},Global`cleanupCount++]")
        }
        "WithCleanup.cleanup" => {
            project(&mut f, "Global`selected", "1");
            format!("WithCleanup[(Global`selected=1;73),{seq}]")
        }
        "Assert.callback" => {
            f.setup
                .push_str(&format!(";$AssertFunction=Function[assertion,{seq}]"));
            expected = "Null";
            "Assert[False]".into()
        }
        "Assert.test" => {
            f.setup.push_str(";$AssertFunction=Function[assertion,999]");
            expected = "Null";
            format!("Assert[{}]", sequence(&f, v, "True"))
        }
        "Confirm.test" => format!(
            "Enclose[Confirm[{}],Function[failure,73]]",
            sequence(&f, v, "$Failed")
        ),
        "ConfirmAssert.test" => {
            expected = "Null";
            format!("ConfirmAssert[{}]", sequence(&f, v, "True"))
        }
        "ConfirmBy.test" => {
            project(
                &mut f,
                "Global`selected",
                if v == Variant::Error { "0" } else { "1" },
            );
            format!("ConfirmBy[{seq},Function[x,(Global`selected++;x==73)]]")
        }
        "ConfirmBy.callback" => format!("ConfirmBy[73,Function[x,{}]]", sequence(&f, v, "x==73")),
        "ConfirmMatch.test" => {
            project(
                &mut f,
                "Global`selected",
                if v == Variant::Error { "0" } else { "1" },
            );
            format!("ConfirmMatch[{seq},(_Integer /; (Global`selected++;True))]")
        }
        "ConfirmMatch.pattern" => format!("ConfirmMatch[73,(_ /; {})]", sequence(&f, v, "True")),
        "ConfirmQuiet.test" => {
            project(
                &mut f,
                "Global`selected",
                if v == Variant::Error { "0" } else { "1" },
            );
            format!(
                "Enclose[ConfirmQuiet[({seq};Message[Global`probe::tag];73)],Function[failure,(Global`selected++;73)]]"
            )
        }
        "Catch.pattern" => {
            f.setup
                .push_str(&format!(";Global`catchPattern:={}", sequence(&f, v, "tag")));
            "Catch[Throw[73,tag],Global`catchPattern]".into()
        }
        "BlockRandom.body" => {
            // The next five draws must equal a saved same-seed control after
            // completion/error, cancellation or reset; no random leaf is ignored.
            f.setup.push_str(";SeedRandom[589];Global`expectedRandom=Table[RandomInteger[1000000],{5}];SeedRandom[589]");
            project(
                &mut f,
                "Table[RandomInteger[1000000],{5}]==Global`expectedRandom",
                "True",
            );
            let draws = (1..=5)
                .map(|i| step(&f, v, i, "(RandomInteger[1000000];73)"))
                .collect::<Vec<_>>();
            format!("BlockRandom[{}]", draws.join(";"))
        }
        _ => unreachable!("{entry}"),
    };
    finish(f, v, expected)
}
