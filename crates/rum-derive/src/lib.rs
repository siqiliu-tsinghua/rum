//! Derive macros for rum runtime data contracts.

mod accumulator_chain;

use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::visit::Visit;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, Generics, Ident, Type, TypeParamBound,
    parse_macro_input, parse_quote,
};

/// Derives access to exactly one explicitly marked accumulator link.
///
/// Requires a named-field struct with one `#[accumulator_link]` field of type
/// `AccumulatorChain<Self>` (the concrete node spelling also works). Import
/// `AccumulatorChainNode` from the kernel API alongside this derive. This does
/// not generate `Drop` or a bounded-destruction declaration.
///
/// Missing link annotations are rejected:
///
/// ```compile_fail
/// use rum_runtime::kernel_api::{AccumulatorChain, AccumulatorChainNode};
/// #[derive(AccumulatorChainNode)]
/// struct Node { previous: AccumulatorChain<Self> }
/// ```
///
/// More than one marked link is rejected:
///
/// ```compile_fail
/// use rum_runtime::kernel_api::{AccumulatorChain, AccumulatorChainNode};
/// #[derive(AccumulatorChainNode)]
/// struct Node {
///     #[accumulator_link]
///     previous: AccumulatorChain<Self>,
///     #[accumulator_link]
///     next: AccumulatorChain<Self>,
/// }
/// ```
///
/// A raw link cannot stand in for the wrapper:
///
/// ```compile_fail
/// use rum_runtime::kernel_api::AccumulatorChainNode;
/// #[derive(AccumulatorChainNode)]
/// struct Node {
///     #[accumulator_link]
///     previous: Option<std::sync::Arc<Self>>,
/// }
/// ```
///
/// The wrapper requires the node contract even if the derive is forgotten:
///
/// ```compile_fail
/// use rum_runtime::kernel_api::AccumulatorChain;
/// struct Node { previous: AccumulatorChain<Self> }
/// ```
///
/// Deriving link access does not declare bounded payload destruction:
///
/// ```compile_fail
/// use rum_runtime::kernel_api::{AccumulatorChain, AccumulatorChainNode, BoundedDestruction};
/// #[derive(AccumulatorChainNode)]
/// struct Node {
///     #[accumulator_link]
///     previous: AccumulatorChain<Self>,
/// }
/// fn requires_bounded<T: BoundedDestruction>() {}
/// requires_bounded::<Node>();
/// ```
#[proc_macro_derive(AccumulatorChainNode, attributes(accumulator_link))]
pub fn derive_accumulator_chain_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    accumulator_chain::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derives borrowed, field-recursive frame-root traversal.
///
/// A field whose type does not implement the traversal contract is rejected by
/// the compiler:
///
/// ```compile_fail
/// trait FrameRootVisitor {}
/// trait FrameRootTrace {
///     fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor);
/// }
///
/// struct MissingTraversal;
///
/// #[derive(rum_derive::FrameRootTrace)]
/// struct Frame {
///     missing: MissingTraversal,
/// }
/// ```
///
/// Unions are intentionally unsupported:
///
/// ```compile_fail
/// trait FrameRootVisitor {}
/// trait FrameRootTrace {
///     fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor);
/// }
///
/// #[derive(rum_derive::FrameRootTrace)]
/// union UnsupportedFrame {
///     value: u64,
/// }
/// ```
#[proc_macro_derive(FrameRootTrace)]
pub fn derive_frame_root_trace(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_frame_root_trace(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Marks a suspension frame as pure computation state that is discarded on
/// abandonment without running ordinary continuation work.
#[proc_macro_derive(PureFrameAbandon)]
pub fn derive_pure_frame_abandon(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_pure_frame_abandon(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_pure_frame_abandon(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    if matches!(input.data, Data::Union(_)) {
        return Err(syn::Error::new_spanned(
            name,
            "PureFrameAbandon cannot be derived for unions; use a struct or enum",
        ));
    }

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics FrameAbandon for #name #type_generics #where_clause {
            fn abandon(
                self: ::std::sync::Arc<Self>,
                _context: &mut AbandonmentContext<'_>,
            ) -> FrameAbandonAction {
                FrameAbandonAction::Discard
            }
        }
    })
}

fn expand_frame_root_trace(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let mut generics = input.generics.clone();
    add_field_type_parameter_bounds(&mut generics, &input.data);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let body = match &input.data {
        Data::Struct(data) => trace_struct(data),
        Data::Enum(data) => trace_enum(data),
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "FrameRootTrace cannot be derived for unions; use a struct or enum with traceable fields",
            ));
        }
    };

    Ok(quote! {
        impl #impl_generics FrameRootTrace for #name #type_generics #where_clause {
            fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
                #body
            }
        }
    })
}

fn trace_struct(data: &DataStruct) -> proc_macro2::TokenStream {
    match &data.fields {
        Fields::Named(fields) => {
            let visits = fields.named.iter().map(|field| {
                let name = field
                    .ident
                    .as_ref()
                    .expect("named fields should carry identifiers");
                quote!(FrameRootTrace::trace_roots(&self.#name, visitor);)
            });
            quote!(#(#visits)*)
        }
        Fields::Unnamed(fields) => {
            let visits = fields.unnamed.iter().enumerate().map(|(index, _)| {
                let index = syn::Index::from(index);
                quote!(FrameRootTrace::trace_roots(&self.#index, visitor);)
            });
            quote!(#(#visits)*)
        }
        Fields::Unit => quote!(),
    }
}

fn trace_enum(data: &DataEnum) -> proc_macro2::TokenStream {
    let arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Named(fields) => {
                let bindings = fields.named.iter().enumerate().map(|(index, field)| {
                    let field_name = field
                        .ident
                        .as_ref()
                        .expect("named fields should carry identifiers");
                    let binding = format_ident!("__frame_root_trace_field_{index}");
                    quote!(#field_name: #binding)
                });
                let visits = fields.named.iter().enumerate().map(|(index, _)| {
                    let binding = format_ident!("__frame_root_trace_field_{index}");
                    quote!(FrameRootTrace::trace_roots(#binding, visitor);)
                });
                quote! {
                    Self::#variant_name { #(#bindings),* } => {
                        #(#visits)*
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let bindings = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(index, _)| format_ident!("__frame_root_trace_field_{index}"));
                let visits = fields.unnamed.iter().enumerate().map(|(index, _)| {
                    let binding = format_ident!("__frame_root_trace_field_{index}");
                    quote!(FrameRootTrace::trace_roots(#binding, visitor);)
                });
                quote! {
                    Self::#variant_name(#(#bindings),*) => {
                        #(#visits)*
                    }
                }
            }
            Fields::Unit => quote!(Self::#variant_name => {}),
        }
    });

    quote! {
        match self {
            #(#arms),*
        }
    }
}

fn add_field_type_parameter_bounds(generics: &mut Generics, data: &Data) {
    let type_parameter_names = generics
        .type_params()
        .map(|parameter| parameter.ident.clone())
        .collect::<HashSet<_>>();
    let mut used_type_parameters = HashSet::new();

    for field_type in field_types(data) {
        let mut visitor = TypeParameterUseVisitor {
            type_parameter_names: &type_parameter_names,
            used_type_parameters: &mut used_type_parameters,
        };
        visitor.visit_type(field_type);
    }

    for parameter in generics.type_params_mut() {
        if used_type_parameters.contains(&parameter.ident)
            && !parameter.bounds.iter().any(is_frame_root_trace_bound)
        {
            parameter.bounds.push(parse_quote!(FrameRootTrace));
        }
    }
}

fn field_types(data: &Data) -> Vec<&Type> {
    match data {
        Data::Struct(data) => data.fields.iter().map(|field| &field.ty).collect(),
        Data::Enum(data) => data
            .variants
            .iter()
            .flat_map(|variant| variant.fields.iter().map(|field| &field.ty))
            .collect(),
        Data::Union(data) => data.fields.named.iter().map(|field| &field.ty).collect(),
    }
}

fn is_frame_root_trace_bound(bound: &TypeParamBound) -> bool {
    let TypeParamBound::Trait(bound) = bound else {
        return false;
    };

    bound
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "FrameRootTrace")
}

struct TypeParameterUseVisitor<'a> {
    type_parameter_names: &'a HashSet<Ident>,
    used_type_parameters: &'a mut HashSet<Ident>,
}

impl<'ast> Visit<'ast> for TypeParameterUseVisitor<'_> {
    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        if path.qself.is_none() {
            if let Some(first) = path.path.segments.first() {
                if self.type_parameter_names.contains(&first.ident) {
                    self.used_type_parameters.insert(first.ident.clone());
                }
            }
        }

        syn::visit::visit_type_path(self, path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn expansion_covers_all_struct_and_enum_field_shapes() {
        let named: DeriveInput = parse_quote!(
            struct Named {
                first: u8,
                second: u16,
            }
        );
        let tuple: DeriveInput = parse_quote!(
            struct Tuple(u8, u16);
        );
        let unit: DeriveInput = parse_quote!(
            struct Unit;
        );
        let variants: DeriveInput = parse_quote! {
            enum Variants {
                Named { first: u8, second: u16 },
                Tuple(u8, u16),
                Unit,
            }
        };

        assert_eq!(trace_call_count(&named), 2);
        assert_eq!(trace_call_count(&tuple), 2);
        assert_eq!(trace_call_count(&unit), 0);
        assert_eq!(trace_call_count(&variants), 4);
    }

    #[test]
    fn generic_bounds_are_added_only_for_parameters_used_by_fields() {
        let input: DeriveInput = parse_quote! {
            struct Generic<T, U, V, const N: usize> {
                direct: T,
                nested: Option<U>,
                fixed: [u8; N],
            }
        };
        let mut generics = input.generics.clone();

        add_field_type_parameter_bounds(&mut generics, &input.data);

        let bounds = generics
            .type_params()
            .map(|parameter| {
                (
                    parameter.ident.to_string(),
                    parameter.bounds.iter().any(is_frame_root_trace_bound),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            bounds,
            vec![
                ("T".to_string(), true),
                ("U".to_string(), true),
                ("V".to_string(), false),
            ]
        );
    }

    #[test]
    fn an_existing_frame_root_trace_bound_is_not_duplicated() {
        let input: DeriveInput = parse_quote! {
            struct Generic<T: FrameRootTrace> {
                value: T,
            }
        };
        let mut generics = input.generics.clone();

        add_field_type_parameter_bounds(&mut generics, &input.data);

        let parameter = generics.type_params().next().unwrap();
        assert_eq!(
            parameter
                .bounds
                .iter()
                .filter(|bound| is_frame_root_trace_bound(bound))
                .count(),
            1
        );
    }

    #[test]
    fn unions_are_rejected_with_a_clear_error() {
        let input: DeriveInput = parse_quote! {
            union Unsupported {
                integer: u64,
                float: f64,
            }
        };

        let error = expand_frame_root_trace(&input).unwrap_err();

        assert_eq!(
            error.to_string(),
            "FrameRootTrace cannot be derived for unions; use a struct or enum with traceable fields"
        );
    }

    #[test]
    fn pure_frame_abandon_preserves_generics_and_where_clauses() {
        let input: DeriveInput = parse_quote! {
            struct Generic<T>
            where
                T: Clone,
            {
                value: T,
            }
        };

        let expanded = expand_pure_frame_abandon(&input).unwrap().to_string();

        assert!(expanded.contains("impl < T > FrameAbandon for Generic < T >"));
        assert!(expanded.contains("where T : Clone"));
        assert!(expanded.contains("FrameAbandonAction :: Discard"));
    }

    fn trace_call_count(input: &DeriveInput) -> usize {
        expand_frame_root_trace(input)
            .unwrap()
            .to_string()
            .matches("FrameRootTrace :: trace_roots")
            .count()
    }
}
