use crate::s5::Workload;

pub fn workload(label: &str) -> Option<Workload> {
    Some(match label {
        "s6-rhs-single" => Workload {
            setup: "f[x_] := x /; (InputString[\"guard\"]; True);",
            body: "f[7]",
            item: "7",
            prompts: &["guard"],
        },
        "s6-rhs-twice" => Workload {
            setup: "f[x_] := (x /; (InputString[\"first\"]; True)) /; (InputString[\"later\"]; True);",
            body: "f[7]",
            item: "7",
            prompts: &["first", "later"],
        },
        "s6-cross-type" => Workload {
            setup: "f[x_ /; (InputString[\"lhs\"]; True)] := x /; (InputString[\"rhs\"]; True);",
            body: "f[7]",
            item: "7",
            prompts: &["lhs", "rhs"],
        },
        "s6-readonly-twice" => Workload {
            setup: "",
            body: "ReplacePart[{1,2}, {i_} :> (0 /; (InputString[\"first\"]; True)) /; (InputString[\"later\"]; True)]",
            item: "List[0, 0]",
            prompts: &["first", "later", "first", "later"],
        },
        "s6-string" => Workload {
            setup: "",
            body: "StringReplace[\"a\", x_ :> \"X\" /; (InputString[\"guard\"]; True)]",
            item: "\"X\"",
            prompts: &["guard"],
        },
        _ => return None,
    })
}
