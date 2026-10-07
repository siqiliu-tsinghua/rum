//! The debug, symbols, contexts and scoping drain. Contracts are the rum
//! evaluation/embedding/scoping guides and each builtin's generated reference.
mod audit;
mod bodies;
mod contexts;
mod falsifiers;
mod lifecycle;
mod options;
mod phases;
mod reports;
mod scoping;
mod support;
mod symbols;
mod trace;
use super::super::descriptors::{Fixture, Variant};
use super::primary::Descriptor;

pub fn descriptors() -> Vec<Descriptor> {
    let mut rows = Vec::new();
    for &(entry, factory) in &[
        (
            "Block.body",
            scoping::fixture as fn(&str, Variant) -> Fixture,
        ),
        ("Block.initializers", scoping::fixture),
        ("Module.body", scoping::fixture),
        ("Module.initializers", scoping::fixture),
        ("With.body", scoping::fixture),
        ("With.initializers", scoping::fixture),
        ("DefinePackage.body", scoping::fixture),
        ("Dialog.body", scoping::fixture),
        ("Evaluate.body", scoping::fixture),
        ("ReleaseHold.body", scoping::fixture),
        ("Input.request", scoping::fixture),
        ("Input.parsedAnswer", scoping::fixture),
        ("InputString.request", scoping::fixture),
        ("Check.body", symbols::fixture),
        ("Quiet.body", symbols::fixture),
        ("Message.template", symbols::fixture),
        ("OptionValue.delayedValue", symbols::fixture),
        ("OptionValue.scopedDefault", symbols::fixture),
        ("Unset.target", symbols::fixture),
        ("AbsoluteTiming.body", bodies::fixture),
        ("Timing.body", bodies::fixture),
        ("RepeatedTiming.body", bodies::fixture),
        ("MemoryConstrained.body", bodies::fixture),
        ("MemoryConstrained.fallback", bodies::fixture),
        ("TimeConstrained.body", bodies::fixture),
        ("TimeConstrained.fallback", bodies::fixture),
        ("Monitor.body", bodies::fixture),
        ("EchoFunction.callback", bodies::fixture),
        ("Reap.body", bodies::fixture),
        ("Reap.combiner", bodies::fixture),
        ("Sow.recipientPattern", bodies::fixture),
        ("Monitor.defaultOption", options::fixture),
        ("Monitor.explicitOption", options::fixture),
        ("Monitor.optionSyntax", options::fixture),
        ("Trace.defaultOption", options::fixture),
        ("TraceDetail.defaultOption", options::fixture),
        ("TracePrint.defaultOption", options::fixture),
        ("Names.defaultOption", options::fixture),
        ("Names.explicitOption", options::fixture),
        ("DumpSave.defaultOption", options::fixture),
        ("DumpSave.explicitOption", options::fixture),
        ("Trace.body", trace::fixture),
        ("TraceDetail.body", trace::fixture),
        ("TracePrint.body", trace::fixture),
        ("Trace.filter", trace::fixture),
        ("TraceDetail.filter", trace::fixture),
        ("TracePrint.filter", trace::fixture),
        ("TestReport.defaultOption", reports::fixture),
        ("TestReport.explicitOption", reports::fixture),
        ("Once.body", contexts::fixture),
        ("Get.fileForms", contexts::fixture),
        ("Needs.fileForms", contexts::fixture),
        ("BeginPackage.fileForms", contexts::fixture),
        ("BeginPackage.dependency", contexts::fixture),
        ("DeclarePackage.fileForms", contexts::fixture),
        ("SetAttributes.attributes", symbols::fixture),
        ("ClearAttributes.attributes", symbols::fixture),
        ("SetOptions.optionSyntax", symbols::fixture),
    ] {
        rows.push(Descriptor {
            id: entry,
            entry,
            variants: support::VARIANTS,
            factory,
            boundary: false,
            lifecycle: lifecycle::exercise,
        });
    }
    rows.extend(phases::descriptors());
    rows
}
