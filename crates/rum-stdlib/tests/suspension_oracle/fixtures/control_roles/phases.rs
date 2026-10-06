//! Additional retained owners within a single catalog role. Each is attached
//! explicitly, with its own full variants and first/second-pause lifecycle.
use super::super::super::descriptors::{Fixture, Variant};
use super::super::primary::Descriptor;
use super::{lifecycle, support::*};

pub fn descriptors() -> Vec<Descriptor> {
    let rows: &[(&str, &str, fn(&str, Variant) -> Fixture)] = &[
        ("Assert.callback.resolve", "Assert.callback", assert_resolve),
        ("Catch.pattern.match", "Catch.pattern", catch_match),
        (
            "Catch.pattern.bypassPreparation",
            "Catch.pattern",
            catch_bypass_preparation,
        ),
        (
            "Catch.pattern.bypassMatch",
            "Catch.pattern",
            catch_bypass_match,
        ),
        (
            "CatchExceptions.handler.pattern",
            "CatchExceptions.handler",
            exception_pattern,
        ),
        (
            "CatchExceptions.handler.guard",
            "CatchExceptions.handler",
            exception_guard,
        ),
        (
            "CheckAbort.body.fallback",
            "CheckAbort.body",
            abort_fallback,
        ),
        (
            "CheckAbort.body.protectedFallback",
            "CheckAbort.body",
            protected_fallback,
        ),
        (
            "WithCleanup.body.initialization",
            "WithCleanup.body",
            cleanup_initialization,
        ),
        (
            "Failure.propertyParameters.styled",
            "Failure.propertyParameters",
            styled,
        ),
        (
            "Failure.propertyTemplate.styled",
            "Failure.propertyTemplate",
            styled,
        ),
    ];
    rows.iter()
        .map(|&(id, entry, factory)| Descriptor {
            id,
            entry,
            factory,
            variants: VARIANTS,
            boundary: false,
            lifecycle: lifecycle::exercise,
        })
        .collect()
}

fn assert_resolve(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.setup.push_str(&format!(
        ";$AssertFunction:={}",
        sequence(&f, v, "Function[assertion,73]")
    ));
    f.source = "Assert[False]".into();
    finish(f, v, "Null")
}

fn catch_match(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.source = format!("Catch[Throw[73,tag],(_ /; {})]", sequence(&f, v, "True"));
    finish(f, v, "73")
}

fn catch_bypass_preparation(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.setup
        .push_str(&format!(";Global`catchPattern:={}", sequence(&f, v, "tag")));
    f.source =
        "Catch[Catch[WithCleanup[Throw[73,tag],Throw[99,tag]],Global`catchPattern],tag]".into();
    finish(f, v, "73")
}

fn catch_bypass_match(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.source = format!(
        "Catch[Catch[WithCleanup[Throw[73,tag],Throw[99,tag]],(_ /; {})],tag]",
        sequence(&f, v, "True")
    );
    finish(f, v, "73")
}

fn exception_pattern(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.source = format!(
        "CatchExceptions[ThrowException[\"Exception\",<||>],{{(_?Function[x,{}]):>73}}]",
        sequence(&f, v, "True")
    );
    finish(f, v, "73")
}

fn exception_guard(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let handlers = (1..=5)
        .map(|i| {
            format!(
                "Exception[\"Exception\",_]:>(73 /; {})",
                step(&f, v, i, if i == 5 { "True" } else { "False" })
            )
        })
        .collect::<Vec<_>>();
    f.source = format!(
        "CatchExceptions[ThrowException[\"Exception\",<||>],{{{}}}]",
        handlers.join(",")
    );
    finish(f, v, "73")
}

fn abort_fallback(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.source = format!("CheckAbort[Abort[],{}]", sequence(&f, v, "73"));
    finish(f, v, "73")
}

fn protected_fallback(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.source = format!(
        "AbortProtect[CheckAbort[(Abort[];999),{}]]",
        sequence(&f, v, "73")
    );
    finish(f, v, "73")
}

fn cleanup_initialization(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.source = format!(
        "WithCleanup[{},(Global`selected=1;73),Global`cleanupCount++]",
        sequence(&f, v, "Null")
    );
    project(&mut f, "Global`cleanupCount", "1");
    project(
        &mut f,
        "Global`selected",
        if v == Variant::Error { "0" } else { "1" },
    );
    finish(f, v, "73")
}

fn styled(entry: &str, v: Variant) -> Fixture {
    let mut f = super::options_properties::fixture(entry, v);
    f.source = f.source.replace("[\"Message\"]", "[\"StyledMessage\"]");
    if v != Variant::Error {
        f.expected_value = Some("Style[\"value 73\", \"Message\"]".into());
    }
    f
}
