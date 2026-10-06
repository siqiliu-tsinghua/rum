use super::super::primary::Descriptor;

pub fn descriptors() -> Vec<Descriptor> {
    let mut rows = Vec::new();
    macro_rules! phase {
        ($id:literal, $entry:literal, $factory:expr) => {
            rows.push(Descriptor {
                id: $id,
                entry: $entry,
                variants: super::variants($entry),
                factory: $factory,
                boundary: false,
                lifecycle: super::lifecycle::exercise,
            });
        };
    }
    phase!(
        "Intersection.sameTest/deduplicate",
        "Intersection.sameTest",
        |e, v| super::collections::set_fixture(e, v, true)
    );
    phase!(
        "Complement.sameTest/deduplicate",
        "Complement.sameTest",
        |e, v| super::collections::set_fixture(e, v, true)
    );
    phase!(
        "GatherBy.keyFunction/function-list",
        "GatherBy.keyFunction",
        |e, v| super::keys::phase(e, v, true)
    );
    phase!(
        "SortBy.keyFunction/ordering",
        "SortBy.keyFunction",
        |e, v| super::keys::phase(e, v, true)
    );
    phase!(
        "Differences.arithmetic/order-two",
        "Differences.arithmetic",
        |e, v| super::structure::differences(e, v, true)
    );
    phase!(
        "LexicographicOrder.comparator/reverse",
        "LexicographicOrder.comparator",
        |e, v| super::ordering::lexicographic(e, v, "reverse")
    );
    phase!(
        "LexicographicOrder.comparator/pair-forward",
        "LexicographicOrder.comparator",
        |e, v| super::ordering::lexicographic(e, v, "pair-forward")
    );
    phase!(
        "LexicographicOrder.comparator/pair-reverse",
        "LexicographicOrder.comparator",
        |e, v| super::ordering::lexicographic(e, v, "pair-reverse")
    );
    phase!(
        "LexicographicOrder.comparator/cross",
        "LexicographicOrder.comparator",
        |e, v| super::ordering::lexicographic(e, v, "cross")
    );
    phase!(
        "LexicographicOrder.comparator/pair-cross",
        "LexicographicOrder.comparator",
        |e, v| super::ordering::lexicographic(e, v, "pair-cross")
    );
    rows
}
