use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;
pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let name = entry.split_once('.').unwrap().0;
    let mut expected;
    if entry.ends_with("default") {
        let fields = (1..=5)
            .map(|i| {
                format!(
                    "\"k{i}\"->Optional[_Integer,{}]",
                    step(&f, v, i, &i.to_string())
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        f.source = format!("{name}[<|\"items\"-><||>|>,<|\"items\"-><|{fields}|>|>]");
        expected = format!(
            "Association[Rule[\"items\", Association[{}]]]",
            (1..=5)
                .map(|i| format!("Rule[\"k{i}\", {i}]"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    } else {
        f.setup.push_str(&format!(
            ";Global`oracleSchemaPredicate[x_]:={}",
            visits(
                &f,
                v,
                if v == Variant::Decline {
                    "x!=2"
                } else {
                    "True"
                }
            )
        ));
        f.source = format!(
            "{name}[<|\"items\"->{{1,2,3,4,5}}|>,<|\"items\"->{{_?Global`oracleSchemaPredicate}}|>]"
        );
        expected = "Association[Rule[\"items\", List[1, 2, 3, 4, 5]]]".into();
    }
    if name == "SchemaErrors" {
        expected = "List[]".into();
    }
    if v == Variant::Decline {
        let message = "expected to match _?oracleSchemaPredicate, got 2";
        if name == "SchemaErrors" {
            expected = format!(
                "List[Association[Rule[\"Path\", \"items[2]\"], Rule[\"Message\", \"{message}\"]]]"
            );
        } else {
            expected = "$Failed".into();
            f.expected_messages = Some(vec![rum_runtime::RuntimeMessage {
                symbol: name.into(),
                tag: "schema".into(),
                text: format!("items[2]: {message}"),
            }]);
        }
    }
    finish(f, v, &expected)
}
