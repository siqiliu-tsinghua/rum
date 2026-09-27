//! Single declaration source for installer registration and the lint's exact
//! certified option domain. The kernel knows TypeIds, never builtin variants.
use rum_runtime::kernel_api::OptionReentryDeclaration;

pub(crate) const DECLARATION: &[OptionReentryDeclaration] =
    &[OptionReentryDeclaration::of::<
        crate::option_consumers::OptionConsumer,
    >("rum_stdlib::option_consumers::OptionConsumer")];
