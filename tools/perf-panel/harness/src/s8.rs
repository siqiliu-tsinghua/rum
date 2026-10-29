use crate::s5::Workload;

pub fn workload(label: &str) -> Option<Workload> {
    Some(match label {
        "s8-replaceall-node" => Workload {
            setup: "",
            body: "f[1] /. x_Integer /; (InputString[\"guard\"];True) :> x+1",
            item: "f[2]",
            prompts: &["guard"],
        },
        "s8-replaceall-atom" => Workload {
            setup: "",
            body: "1 /. x_Integer /; (InputString[\"guard\"];True) :> x+1",
            item: "2",
            prompts: &["guard"],
        },
        "s8-replaceall-twice" => Workload {
            setup: "",
            body: "{1,2} /. x_Integer /; (InputString[\"guard\"];True) :> x+1",
            item: "List[2, 3]",
            prompts: &["guard", "guard"],
        },
        "s8-repeated-guard" => Workload {
            setup: "",
            body: "ReplaceRepeated[f[1], f[1] /; (InputString[\"guard\"];True) :> f[0]]",
            item: "f[0]",
            prompts: &["guard"],
        },
        "s8-replace-decline" => Workload {
            setup: "",
            body: "Replace[3,{x_ /; (InputString[\"first\"];False) :> 0,x_ :> 9}]",
            item: "9",
            prompts: &["first"],
        },
        "s8-replace-both" => Workload {
            setup: "",
            body: "Replace[3,{x_ /; (InputString[\"first\"];False) :> 0,x_ /; (InputString[\"later\"];True) :> 9}]",
            item: "9",
            prompts: &["first", "later"],
        },
        "s8-replaceat-guard" => Workload {
            setup: "",
            body: "ReplaceAt[{1,2,3},x_ /; (InputString[\"guard\"];True) :> x+1,2]",
            item: "List[1, 3, 3]",
            prompts: &["guard"],
        },
        "s8-count-target" => Workload {
            setup: "",
            body: "Count[(InputString[\"target\"];{1}),_Integer]",
            item: "1",
            prompts: &["target"],
        },
        "s8-count-pattern" => Workload {
            setup: "",
            body: "Count[{1},(InputString[\"pattern\"];_Integer)]",
            item: "1",
            prompts: &["pattern"],
        },
        "s8-memberq-target" => Workload {
            setup: "",
            body: "MemberQ[(InputString[\"target\"];{1}),_Integer]",
            item: "True",
            prompts: &["target"],
        },
        "s8-memberq-pattern" => Workload {
            setup: "",
            body: "MemberQ[{1},(InputString[\"pattern\"];_Integer)]",
            item: "True",
            prompts: &["pattern"],
        },
        "s8-freeq-target" => Workload {
            setup: "",
            body: "FreeQ[(InputString[\"target\"];{1}),_Integer]",
            item: "False",
            prompts: &["target"],
        },
        "s8-freeq-pattern" => Workload {
            setup: "",
            body: "FreeQ[{1},(InputString[\"pattern\"];_Integer)]",
            item: "False",
            prompts: &["pattern"],
        },
        "s8-count-level" => Workload {
            setup: "",
            body: "Count[{1},_Integer,(InputString[\"level\"];{1})]",
            item: "1",
            prompts: &["level"],
        },
        "s8-count-guard-twice" => Workload {
            setup: "",
            body: "Count[{1,2},x_Integer /; (InputString[\"guard\"];True)]",
            item: "2",
            prompts: &["guard", "guard"],
        },
        "s8-position-target" => Workload {
            setup: "",
            body: "Position[(InputString[\"target\"];{1}),1]",
            item: "List[List[1]]",
            prompts: &["target"],
        },
        "s8-position-pattern" => Workload {
            setup: "",
            body: "Position[{1},(InputString[\"pattern\"];1)]",
            item: "List[List[1]]",
            prompts: &["pattern"],
        },
        "s8-position-level" => Workload {
            setup: "",
            body: "Position[{1},1,(InputString[\"level\"];{1})]",
            item: "List[List[1]]",
            prompts: &["level"],
        },
        "s8-position-guard-twice" => Workload {
            setup: "",
            body: "Position[{1,2},x_Integer /; (InputString[\"guard\"];True)]",
            item: "List[List[1], List[2]]",
            prompts: &["guard", "guard"],
        },
        _ => return None,
    })
}
