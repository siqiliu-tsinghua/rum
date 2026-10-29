use crate::s5::Workload;

pub fn workload(label: &str) -> Option<Workload> {
    Some(match label {
        "s7-list-part" => Workload {
            setup: "",
            body: "l={0};l[[(InputString[\"selector\"];1)]]=9;l[[1]]",
            item: "9",
            prompts: &["selector"],
        },
        "s7-assoc-part" => Workload {
            setup: "",
            body: "a=<|\"k\"->0|>;a[[(InputString[\"selector\"];\"k\")]]=9;a[\"k\"]",
            item: "9",
            prompts: &["selector"],
        },
        "s7-assoc-call" => Workload {
            setup: "",
            body: "a=<|\"k\"->0|>;a[(InputString[\"selector\"];\"k\")]=9;a[\"k\"]",
            item: "9",
            prompts: &["selector"],
        },
        "s7-assoc-call-thrice" => Workload {
            setup: "",
            body: "a=<|\"k\"-><|\"j\"-><|\"z\"->0|>|>|>;a[(InputString[\"first\"];\"k\"),(InputString[\"later\"];\"j\"),(InputString[\"last\"];\"z\")]=9;a[[\"k\",\"j\",\"z\"]]",
            item: "9",
            prompts: &["first", "later", "last"],
        },
        "s7-held-twice" => Workload {
            setup: "",
            body: "x={{1,2},{3,4}};x[[(InputString[\"first\"];1),(InputString[\"later\"];2)]]+=10;x[[1,2]]",
            item: "12",
            prompts: &["first", "later"],
        },
        "s7-held-rhs" => Workload {
            setup: "",
            body: "x={1};x[[1]] //= Function[z,(InputString[\"rhs\"];z+2)];x[[1]]",
            item: "3",
            prompts: &["rhs"],
        },
        "s7-associate-argument" => Workload {
            setup: "",
            body: "a=<||>;AssociateTo[a,(InputString[\"argument\"];\"k\"->9)];a[\"k\"]",
            item: "9",
            prompts: &["argument"],
        },
        "s7-keydrop-argument" => Workload {
            setup: "",
            body: "a=<|\"k\"->1|>;KeyDropFrom[a,(InputString[\"argument\"];\"k\")];Length[a]",
            item: "0",
            prompts: &["argument"],
        },
        "s7-associate-target" => Workload {
            setup: "s7a[i_]:=(InputString[\"target\"];<|\"k\"->1|>);",
            body: "Length[AssociateTo[s7a[s5Tick],\"j\"->2]]",
            item: "2",
            prompts: &["target"],
        },
        "s7-keydrop-target" => Workload {
            setup: "s7a[i_]:=(InputString[\"target\"];<|\"k\"->1,\"j\"->2|>);",
            body: "Length[KeyDropFrom[s7a[s5Tick],\"j\"]]",
            item: "1",
            prompts: &["target"],
        },
        "s7-associate-held" => Workload {
            setup: "",
            body: "a=<|\"k\"-><|\"j\"->1|>|>;AssociateTo[a[[(InputString[\"selector\"];\"k\")]],\"m\"->9];a[[\"k\",\"m\"]]",
            item: "9",
            prompts: &["selector"],
        },
        "s7-keydrop-held" => Workload {
            setup: "",
            body: "a=<|\"k\"-><|\"j\"->1,\"m\"->2|>|>;KeyDropFrom[a[[(InputString[\"selector\"];System`Key[\"k\"])]],\"j\"];Length[a[\"k\"]]",
            item: "1",
            prompts: &["selector"],
        },
        _ => return None,
    })
}
