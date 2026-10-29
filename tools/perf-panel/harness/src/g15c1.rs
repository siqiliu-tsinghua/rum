use crate::s5::Workload;

pub fn workload(label: &str) -> Option<Workload> {
    Some(match label {
        "c1-all-guard" => Workload {
            setup: "",
            body: "ReplaceAll[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)]",
            item: "2",
            prompts: &["guard"],
        },
        "c1-all-test" => Workload {
            setup: "",
            body: "ReplaceAll[1, (x_Integer ? (Function[z, (InputString[\"test\"]; True)])) :> (x + 1)]",
            item: "2",
            prompts: &["test"],
        },
        "c1-all-guard-value" => Workload {
            setup: "",
            body: "ReplaceAll[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (InputString[\"value\"]; x + 1)]",
            item: "2",
            prompts: &["guard", "value"],
        },
        "c1-all-two-guards" => Workload {
            setup: "",
            body: "ReplaceAll[{1, 2}, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)]",
            item: "List[2, 3]",
            prompts: &["guard", "guard"],
        },
        "c1-unheld-guard" => Workload {
            setup: "",
            body: "ReplaceAllUnheld[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)]",
            item: "2",
            prompts: &["guard"],
        },
        "c1-unheld-test" => Workload {
            setup: "",
            body: "ReplaceAllUnheld[1, (x_Integer ? (Function[z, (InputString[\"test\"]; True)])) :> (x + 1)]",
            item: "2",
            prompts: &["test"],
        },
        "c1-unheld-guard-value" => Workload {
            setup: "",
            body: "ReplaceAllUnheld[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (InputString[\"value\"]; x + 1)]",
            item: "2",
            prompts: &["guard", "value"],
        },
        "c1-unheld-two-guards" => Workload {
            setup: "",
            body: "ReplaceAllUnheld[{1, 2}, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)]",
            item: "List[2, 3]",
            prompts: &["guard", "guard"],
        },
        _ => return None,
    })
}
