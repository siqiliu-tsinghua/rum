use crate::s5::Workload;

pub fn workload(label: &str) -> Option<Workload> {
    Some(match label {
        "g15-parameters" => Workload {
            setup: "",
            body: "Failure[\"t\", <|\"MessageTemplate\" -> \"m `1`\", \"MessageParameters\" :> (InputString[\"projection\"]; {1})|>][\"Message\"]",
            item: "\"m 1\"",
            prompts: &["projection"],
        },
        "g15-template" => Workload {
            setup: "g15msg::m := (InputString[\"template\"]; \"m `1`\");",
            body: "Failure[\"t\", <|\"MessageTemplate\" :> g15msg::m, \"MessageParameters\" -> {1}|>][\"Message\"]",
            item: "\"m 1\"",
            prompts: &["template"],
        },
        "g15-styled" => Workload {
            setup: "",
            body: "Failure[\"t\", <|\"MessageTemplate\" -> \"m `1`\", \"MessageParameters\" :> (InputString[\"projection\"]; {1})|>][\"StyledMessage\"]",
            item: "Style[\"m 1\", \"Message\"]",
            prompts: &["projection"],
        },
        "g15-raw-failure" => Workload {
            setup: "",
            body: "Failure[\"t\", <|\"x\" :> (InputString[\"value\"]; 17)|>][\"x\"]",
            item: "17",
            prompts: &["value"],
        },
        "g15-raw-success" => Workload {
            setup: "",
            body: "Success[\"t\", <|\"x\" :> (InputString[\"value\"]; 17)|>][\"x\"]",
            item: "17",
            prompts: &["value"],
        },
        "g15-projection-value" => Workload {
            setup: "",
            body: "Failure[\"t\", <|\"MessageTemplate\" :> (InputString[\"value\"]; 19), \"MessageParameters\" :> (InputString[\"projection\"]; {})|>][\"Message\"]",
            item: "19",
            prompts: &["projection", "value"],
        },
        "g15-enclose" => Workload {
            setup: "",
            body: "Enclose[Throw[Failure[\"t\", <|\"MessageTemplate\" -> \"m `1`\", \"MessageParameters\" :> (InputString[\"projection\"]; {1})|>], t], \"Message\", t]",
            item: "\"m 1\"",
            prompts: &["projection"],
        },
        "g15-confirm" => Workload {
            setup: "",
            body: "Enclose[Confirm[Failure[\"t\", <|\"MessageTemplate\" -> \"m `1`\", \"MessageParameters\" :> (InputString[\"projection\"]; {1})|>]], \"Message\"]",
            item: "\"m 1\"",
            prompts: &["projection"],
        },
        _ => return None,
    })
}
