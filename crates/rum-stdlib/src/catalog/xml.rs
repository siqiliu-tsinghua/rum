use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use crate::xml;

pub(crate) const XML_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "FromXml",
        "Parses XML text into a symbolic XmlElement tree, rejecting DTDs and unsupported XML surface.",
        AttributeSet::empty().with(Attribute::Protected),
        xml::eval_from_xml,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "xml",
            template: "Reports an XML decoding or argument failure.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "ToXml",
        "Renders a well-formed XmlElement tree as XML text.",
        AttributeSet::empty().with(Attribute::Protected),
        xml::eval_to_xml,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "xml",
            template: "Reports an XML encoding or argument failure.",
        },
    ]),
    BuiltinSpec::with_attributes(
        "XmlElement",
        "Represents an inert XML element as XmlElement[tag, attrs, children].",
        AttributeSet::empty().with(Attribute::Protected),
        MemoryGrowthContract::NonAmplifying,
    ),
];
