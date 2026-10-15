//! S2 receipts prove driver mechanisms; each production family keeps its step.
use super::{
    receipts::{RECEIPTS, rows},
    scan::{Facts, Population},
};

pub(super) fn compatible_driver(facts: &Facts) -> Result<(), String> {
    let receipts = rows(RECEIPTS);
    for (id, file, function, witness) in [
        (
            "I0001",
            "typed/witness.rs",
            "suspend",
            "macro_is_cold_and_a_witnessed_chain_returns_its_last_output",
        ),
        (
            "I0002",
            "typed/chain.rs",
            "suspend",
            "macro_is_cold_and_a_witnessed_chain_returns_its_last_output",
        ),
        (
            "I0003",
            "typed/resume_token.rs",
            "suspend",
            "repeat_token_freezes_fresh_state_and_keeps_the_deferred_output_type",
        ),
        (
            "I0007",
            "typed/action.rs",
            "evaluate",
            "evaluate_successor_and_nested_owner_precede_the_unrun_suffix",
        ),
        (
            "I0008",
            "typed/action.rs",
            "finalize",
            "driver_drops_both_owning_arcs_before_running_the_finalizer",
        ),
        (
            "I0009",
            "core.rs",
            "drive_resuming_frames",
            "completion_exact_decode_and_probe_survive_rollback_and_replacement",
        ),
    ] {
        let suffix = format!("runtime/suspension/{file}");
        let live: Vec<_> = facts
            .functions
            .iter()
            .filter(|f| {
                f.site.file.ends_with(&suffix)
                    && f.name == function
                    && f.population == Population::Production
            })
            .collect();
        if live.len() != 1
            || !facts
                .functions
                .iter()
                .any(|f| f.name == witness && f.population == Population::TestSupport)
        {
            return Err(format!(
                "{id}: live mechanism or executable witness missing"
            ));
        }
        if !receipts
            .iter()
            .any(|r| r[0] == "Route" && r[1] == id && r[2] == "S2" && r[3] == "mechanism-live")
        {
            return Err(format!("{id}: S2 mechanism receipt missing"));
        }
        println!(
            "S2 ROUTE {id} mechanism={} witness={witness}; S3 types evaluation; S4 types argument and traversal resolution; later families remain untyped",
            live[0].identity
        );
    }
    if !facts
        .uses
        .iter()
        .any(|s| s.name == "resume_typed_or_suspend" && s.population == Population::TestSupport)
    {
        return Err("I0001: no actual typed macro producer witness".into());
    }
    for (id, step) in [("I0005", "S18"), ("I0006", "S14"), ("I0010", "S24")] {
        if !receipts
            .iter()
            .any(|r| r[0] == "Route" && r[1] == id && r[2] == step && r[3] == "untyped")
        {
            return Err(format!("{id}: family owner changed"));
        }
        println!("S2 ROUTE {id} remains untyped; family={step}");
    }
    if !receipts
        .iter()
        .any(|r| r[1] == "I0004" && r[2] == "S5" && r[3] == "implemented")
    {
        return Err("I0004: S5 fallback witness receipt missing".into());
    }
    let transform = receipts.iter().find(|r| r[1] == "T0010").unwrap();
    if transform[3] != "observed"
        || !facts
            .uses
            .iter()
            .any(|s| super::receipts::use_identity(s) == transform[4])
    {
        return Err("T0010: public CLI Result mapping observation disappeared".into());
    }
    println!("S2 T0010 observed unchanged: public CLI Result mapping; no slot conversion");
    Ok(())
}
