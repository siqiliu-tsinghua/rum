use std::sync::Arc;

trait FrameRootVisitor {
    fn visit_value(&mut self, value: u8);
}

struct AbandonmentContext<'a>(&'a mut ());

#[derive(Debug, PartialEq, Eq)]
enum FrameAbandonAction {
    Discard,
}

trait FrameAbandon {
    fn abandon(self: Arc<Self>, context: &mut AbandonmentContext<'_>) -> FrameAbandonAction;
}

trait FrameRootTrace {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor);
}

impl FrameRootTrace for u8 {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        visitor.visit_value(*self);
    }
}

impl<T: FrameRootTrace> FrameRootTrace for Option<T> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        if let Some(value) = self {
            value.trace_roots(visitor);
        }
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct Named<T> {
    first: T,
    second: u8,
}

#[derive(rum_derive::FrameRootTrace)]
struct Tuple<T>(T, u8);

#[derive(rum_derive::FrameRootTrace)]
struct Unit;

#[derive(rum_derive::PureFrameAbandon)]
struct PureNamed<T> {
    _value: T,
}

#[derive(rum_derive::PureFrameAbandon)]
enum PureVariants<T> {
    Value(T),
    Unit,
}

#[derive(rum_derive::FrameRootTrace)]
enum Variants<T> {
    Named { first: T, second: u8 },
    Tuple(T, u8),
    Unit,
}

#[derive(Default)]
struct Values(Vec<u8>);

impl FrameRootVisitor for Values {
    fn visit_value(&mut self, value: u8) {
        self.0.push(value);
    }
}

#[test]
fn derives_named_tuple_unit_and_generic_structs() {
    let mut values = Values::default();

    Named {
        first: Some(1_u8),
        second: 2,
    }
    .trace_roots(&mut values);
    Tuple(Some(3_u8), 4).trace_roots(&mut values);
    Unit.trace_roots(&mut values);

    assert_eq!(values.0, vec![1, 2, 3, 4]);
}

#[test]
fn derives_all_enum_variant_shapes() {
    let mut values = Values::default();
    let variants = [
        Variants::Named {
            first: Some(1_u8),
            second: 2,
        },
        Variants::Tuple(Some(3_u8), 4),
        Variants::Unit,
    ];

    for variant in variants {
        variant.trace_roots(&mut values);
    }

    assert_eq!(values.0, vec![1, 2, 3, 4]);
}

#[test]
fn pure_frame_abandon_discards_structs_and_enums_without_inspecting_fields() {
    let mut state = ();
    let mut context = AbandonmentContext(&mut state);

    assert_eq!(
        Arc::new(PureNamed { _value: 1_u8 }).abandon(&mut context),
        FrameAbandonAction::Discard
    );
    assert_eq!(
        Arc::new(PureVariants::Value(2_u8)).abandon(&mut context),
        FrameAbandonAction::Discard
    );
    assert_eq!(
        Arc::new(PureVariants::<u8>::Unit).abandon(&mut context),
        FrameAbandonAction::Discard
    );
    let _ = context.0;
}
