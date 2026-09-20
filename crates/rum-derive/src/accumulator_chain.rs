use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericArgument, Meta, PathArguments, Type};

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            input,
            "AccumulatorChainNode requires a named-field struct",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            input,
            "AccumulatorChainNode requires a named-field struct",
        ));
    };
    let mut marked = None;
    for field in &fields.named {
        for attribute in &field.attrs {
            if !attribute.path().is_ident("accumulator_link") {
                continue;
            }
            if !matches!(attribute.meta, Meta::Path(_)) {
                return Err(syn::Error::new_spanned(
                    attribute,
                    "use #[accumulator_link] without arguments",
                ));
            }
            if marked.replace(field).is_some() {
                return Err(syn::Error::new_spanned(
                    attribute,
                    "AccumulatorChainNode requires exactly one #[accumulator_link] field",
                ));
            }
        }
    }
    let field = marked.ok_or_else(|| {
        syn::Error::new_spanned(
            input,
            "AccumulatorChainNode requires exactly one #[accumulator_link] field",
        )
    })?;
    let supported = match &field.ty {
        Type::Path(path) if path.qself.is_none() => path.path.segments.last().is_some_and(|segment| {
            segment.ident == "AccumulatorChain"
                && matches!(&segment.arguments, PathArguments::AngleBracketed(args)
                    if args.args.len() == 1 && matches!(args.args.first(), Some(GenericArgument::Type(_))))
        }),
        _ => false,
    };
    if !supported {
        return Err(syn::Error::new_spanned(
            &field.ty,
            "the marked link must have type AccumulatorChain<Self> (or AccumulatorChain<NodeType>)",
        ));
    }
    let name = &input.ident;
    let member = &field.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics AccumulatorChainNode for #name #type_generics #where_clause {
            fn take_accumulator_link(&mut self) -> ::std::option::Option<::std::sync::Arc<Self>> {
                self.#member.take().into_inner()
            }
        }
    })
}
