//! Distinct retained owners and option-read stages attached to the same role.
use super::super::super::descriptors::{Fixture, Pause, Variant};
use super::super::primary::Descriptor;
use super::{lifecycle, support::*};

pub fn descriptors() -> Vec<Descriptor> {
    let rows: &[(&str, &str, fn(&str, Variant) -> Fixture)] = &[
        ("Block.body.single", "Block.body", block_single),
        (
            "Block.initializers.single",
            "Block.initializers",
            block_single,
        ),
        (
            "Module.body.afterInitializer",
            "Module.body",
            module_after_initializer,
        ),
        (
            "Monitor.defaultOption.step",
            "Monitor.defaultOption",
            |e, v| super::options::option(e, v, "StepMonitor"),
        ),
        (
            "Monitor.explicitOption.step",
            "Monitor.explicitOption",
            |e, v| super::options::option(e, v, "StepMonitor"),
        ),
        (
            "Names.defaultOption.aliases",
            "Names.defaultOption",
            |e, v| super::options::option(e, v, "ResolveContextAliases"),
        ),
        (
            "Names.explicitOption.aliases",
            "Names.explicitOption",
            |e, v| super::options::option(e, v, "ResolveContextAliases"),
        ),
        (
            "Names.defaultOption.ignoreCase",
            "Names.defaultOption",
            |e, v| super::options::option(e, v, "IgnoreCase"),
        ),
        (
            "Names.explicitOption.ignoreCase",
            "Names.explicitOption",
            |e, v| super::options::option(e, v, "IgnoreCase"),
        ),
        (
            "TestReport.defaultOption.time",
            "TestReport.defaultOption",
            |e, v| super::reports::option(e, v, "TimeConstraint"),
        ),
        (
            "TestReport.explicitOption.time",
            "TestReport.explicitOption",
            |e, v| super::reports::option(e, v, "TimeConstraint"),
        ),
        (
            "TestReport.defaultOption.memory",
            "TestReport.defaultOption",
            |e, v| super::reports::option(e, v, "MemoryConstraint"),
        ),
        (
            "TestReport.explicitOption.memory",
            "TestReport.explicitOption",
            |e, v| super::reports::option(e, v, "MemoryConstraint"),
        ),
        ("Reap.combiner.groups", "Reap.combiner", reap_groups),
        (
            "Sow.recipientPattern.scopes",
            "Sow.recipientPattern",
            sow_scopes,
        ),
        (
            "OptionValue.delayedValue.list",
            "OptionValue.delayedValue",
            option_list,
        ),
        (
            "OptionValue.scopedDefault.list",
            "OptionValue.scopedDefault",
            option_list,
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

fn block_single(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    f.source = if entry.ends_with("body") {
        format!(
            "Block[{{Global`local=73}},{}]",
            sequence(&f, v, "Global`local")
        )
    } else {
        format!(
            "Block[{{Global`local={}}},Global`local]",
            sequence(&f, v, "73")
        )
    };
    finish(f, v, "73")
}

fn module_after_initializer(entry: &str, v: Variant) -> Fixture {
    let mut f = super::scoping::fixture(entry, v);
    f.source = f.source.replace(
        "Global`local=73",
        "Global`local=Input[\"module initializer\"]",
    );
    let index = usize::from(v == Variant::AlreadyResumed);
    f.pauses.insert(
        index,
        Pause {
            prompt: "module initializer".into(),
            kind: rum_runtime::RuntimeInputKind::Input,
            answer: "73".into(),
        },
    );
    f.lifecycle_pauses = [1, 2];
    f
}

fn reap_groups(entry: &str, v: Variant) -> Fixture {
    let mut f = super::bodies::fixture(entry, v);
    f.source = f.source.replace("),_,Function", "),{1,2,3,4,5},Function");
    if v != Variant::Error {
        let groups = (1..=5)
            .map(|i| format!("List[List[{i}, List[{}]]]", i * 10))
            .collect::<Vec<_>>()
            .join(", ");
        f.expected_value = Some(format!("List[73, List[{groups}]]"));
    }
    f
}

fn sow_scopes(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let mut source = "Sow[73,tag]".to_string();
    let mut expected = "73".to_string();
    for i in (1..=5).rev() {
        source = format!("Reap[{source},_?Function[tag,{}]]", step(&f, v, i, "True"));
        expected = format!("List[{expected}, List[List[73]]]");
    }
    f.source = source;
    finish(f, v, &expected)
}

fn option_list(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let options = (1..=5)
        .map(|i| format!("Global`key{i}:>{}", step(&f, v, i, &i.to_string())))
        .collect::<Vec<_>>()
        .join(",");
    let keys = "{Global`key1,Global`key2,Global`key3,Global`key4,Global`key5}";
    f.setup
        .push_str(&format!(";Options[Global`listOwner]={{{options}}}"));
    f.source = if entry.ends_with("scopedDefault") {
        f.setup.push_str(&format!(
            ";Global`listOwner[OptionsPattern[]]:=OptionValue[{keys}]"
        ));
        "Global`listOwner[]".into()
    } else {
        format!("OptionValue[Global`listOwner,{keys}]")
    };
    finish(f, v, "List[1, 2, 3, 4, 5]")
}
