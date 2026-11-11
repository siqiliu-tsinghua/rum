//! G15-C2 records the profiled prepared consumer, constructed by its cold helper.
use super::{
    classify::Report,
    receipts::{RECEIPTS, rows},
    scan::Population,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn transitions(report: &Report) -> Result<(), String> {
    let receipts = rows(RECEIPTS);
    let by_id: BTreeMap<_, _> = receipts.iter().map(|row| (row[1], row)).collect();
    let expected_frames = BTreeSet::from(["FG15C2Prepared"]);
    let owned = |kind| {
        receipts
            .iter()
            .filter(|row| row[0] == kind && row[2] == "G15-C2")
            .map(|row| row[1])
            .collect::<BTreeSet<_>>()
    };
    if owned("F") != expected_frames || !owned("M").is_empty() {
        return Err("G15-C2: exactly one frame receipt and no insertion receipt required".into());
    }
    for id in expected_frames {
        let frame = report
            .frames
            .iter()
            .find(|frame| frame.identity == by_id[id][4])
            .ok_or_else(|| format!("{id}: new typed frame absent"))?;
        if by_id[id][3] != "typed"
            || by_id[id][7] != "production"
            || by_id[id][8] != "F0275P"
            || frame.population != Population::Production
            || frame.kind != "TypedFrame"
            || frame.input.replace(' ', "") != "PreparedCallArguments"
            || frame.output.replace(' ', "") != "Expr"
        {
            return Err(format!(
                "{id}: production PreparedCallArguments -> Expr completion anchored to F0275P not proved"
            ));
        }
        println!(
            "G15-C2 FRAME {id} {} PreparedCallArguments -> Expr anchor=F0275P",
            frame.name
        );
    }
    if owned("Stage") != BTreeSet::from(["G15-C2"])
        || by_id["G15-C2"][3] != "implemented"
        || by_id["G15-C2"][7] != "production"
        || by_id["G15-C2"][8] != "FG15C2Prepared"
    {
        return Err("G15-C2: exact prepared-consumer typing stage receipt required".into());
    }
    println!(
        "G15-C2 TYPING frames=1 insertions=0; cold-helper construction; global modes unchanged; semantic C2/S10 closure not claimed"
    );
    Ok(())
}
