use super::equality;

#[test]
fn part_none() {
    equality(
        "part.none",
        r###"sweepList={0};"###,
        r###"{sweepList[[(InputString["sweep"];1)]]=System`InternalContinuationNone[],sweepList[[1]]}"###,
        &["sweep"],
        Some("List[InternalContinuationNone[], InternalContinuationNone[]]"),
        "List[]",
    );
}

#[test]
fn assoc_none() {
    equality(
        "assoc.none",
        r###"swA=<|"k"->0|>;"###,
        r###"{swA[[(InputString["sweep"];System`Key["k"])]]=System`InternalContinuationNone[],swA[[System`Key["k"]]]}"###,
        &["sweep"],
        Some("List[InternalContinuationNone[], InternalContinuationNone[]]"),
        "List[]",
    );
}

#[test]
fn part_some() {
    equality(
        "part.some",
        r###"sweepList={0};"###,
        r###"{sweepList[[(InputString["sweep"];1)]]=System`InternalContinuationSome[5],sweepList[[1]]}"###,
        &["sweep"],
        Some("List[InternalContinuationSome[5], InternalContinuationSome[5]]"),
        "List[]",
    );
}

#[test]
fn assoc_some() {
    equality(
        "assoc.some",
        r###"swA=<|"k"->0|>;"###,
        r###"{swA[[(InputString["sweep"];System`Key["k"])]]=System`InternalContinuationSome[5],swA[[System`Key["k"]]]}"###,
        &["sweep"],
        Some("List[InternalContinuationSome[5], InternalContinuationSome[5]]"),
        "List[]",
    );
}

#[test]
fn part_empty_some() {
    equality(
        "part.empty_some",
        r###"sweepList={0};"###,
        r###"{sweepList[[(InputString["sweep"];1)]]=System`InternalContinuationSome[],sweepList[[1]]}"###,
        &["sweep"],
        Some("List[InternalContinuationSome[], InternalContinuationSome[]]"),
        "List[]",
    );
}

#[test]
fn assoc_empty_some() {
    equality(
        "assoc.empty_some",
        r###"swA=<|"k"->0|>;"###,
        r###"{swA[[(InputString["sweep"];System`Key["k"])]]=System`InternalContinuationSome[],swA[[System`Key["k"]]]}"###,
        &["sweep"],
        Some("List[InternalContinuationSome[], InternalContinuationSome[]]"),
        "List[]",
    );
}
