use std::sync::Arc;

use rum_runtime::kernel_api::{AccumulatorChain, AccumulatorChainNode};

#[derive(AccumulatorChainNode)]
struct PreviousNode {
    #[accumulator_link]
    previous: AccumulatorChain<PreviousNode>,
    value: String,
}

// Payload-first, next-linked, qualified wrapper spelling, and generics all use
// the same accessor. The payload deliberately does not implement Clone.
#[derive(AccumulatorChainNode)]
struct NextNode<'a, T, const N: usize>
where
    T: AsRef<str>,
{
    payload: T,
    borrowed: &'a [u8; N],
    #[accumulator_link]
    next: rum_runtime::kernel_api::AccumulatorChain<Self>,
}

struct NonClone(String);

impl AsRef<str> for NonClone {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[test]
fn accessor_detaches_the_original_link_and_leaves_payloads_movable() {
    let tail = Arc::new(PreviousNode {
        previous: AccumulatorChain::default(),
        value: "tail".into(),
    });
    let mut head = PreviousNode {
        previous: AccumulatorChain::new(Some(tail.clone())),
        value: "head".into(),
    };

    let detached = head.take_accumulator_link().unwrap();
    assert!(head.previous.is_none());
    assert!(head.take_accumulator_link().is_none());
    assert!(Arc::ptr_eq(&detached, &tail));
    drop(tail);
    // Acquisition would fail if the accessor retained or cloned an owning link.
    let PreviousNode { previous, value } = Arc::into_inner(detached).unwrap();
    assert!(previous.is_none());
    assert_eq!(value, "tail");
    let PreviousNode { value, .. } = head;
    assert_eq!(value, "head");
}

#[test]
fn next_link_supports_non_clone_payloads_lifetimes_and_const_generics() {
    let borrowed = [1, 2, 3];
    let tail = Arc::new(NextNode {
        payload: NonClone("tail".into()),
        borrowed: &borrowed,
        next: AccumulatorChain::default(),
    });
    let mut head = NextNode {
        payload: NonClone("head".into()),
        borrowed: &borrowed,
        next: AccumulatorChain::new(Some(tail)),
    };

    let detached = head.take_accumulator_link().unwrap();
    assert!(head.next.is_none());
    let NextNode {
        payload,
        borrowed: actual,
        next,
    } = Arc::into_inner(detached).unwrap();
    assert_eq!(payload.as_ref(), "tail");
    assert_eq!(actual, &borrowed);
    assert!(next.is_none());
    let NextNode { payload, .. } = head;
    assert_eq!(payload.as_ref(), "head");
}

// Exercise each unsupported syntax arrangement at the diagnostic boundary.
#[path = "../src/accumulator_chain.rs"]
mod expansion;

#[test]
fn unsupported_arrangements_report_specific_diagnostics() {
    let one_link = "AccumulatorChainNode requires exactly one #[accumulator_link] field";
    let named = "AccumulatorChainNode requires a named-field struct";
    for (source, expected) in [
        ("struct Node { previous: AccumulatorChain<Self> }", one_link),
        (
            "struct Node { #[accumulator_link] a: AccumulatorChain<Self>, #[accumulator_link] b: AccumulatorChain<Self> }",
            one_link,
        ),
        (
            "struct Node { #[accumulator_link] #[accumulator_link] a: AccumulatorChain<Self> }",
            one_link,
        ),
        (
            "struct Node(#[accumulator_link] AccumulatorChain<Self>);",
            named,
        ),
        ("struct Node;", named),
        ("enum Node { End }", named),
        ("union Node { value: usize }", named),
        (
            "struct Node { #[accumulator_link(previous)] a: AccumulatorChain<Self> }",
            "use #[accumulator_link] without arguments",
        ),
        (
            "struct Node { #[accumulator_link] a: Option<Arc<Self>> }",
            "the marked link must have type AccumulatorChain<Self> (or AccumulatorChain<NodeType>)",
        ),
    ] {
        let input = syn::parse_str(source).unwrap();
        let error = expansion::expand(&input).unwrap_err();
        assert_eq!(error.to_string(), expected, "{source}");
        assert!(error.into_compile_error().to_string().contains(expected));
    }
}
