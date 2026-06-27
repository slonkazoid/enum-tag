use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::DeriveInput;
use syn::Meta;
use syn::Result;

/// Wrapper around `enum_tag_impl` for error conversions.
pub fn enum_tag(input: DeriveInput) -> TokenStream2 {
    match enum_tag_impl(input) {
        Ok(result) => result,
        Err(error) => error.to_compile_error(),
    }
}

/// Implements the `#derive(EnumTag)` functionality on the given `input`.
///
/// This creates an C-like `enum` that has the same variants as `input` but
/// no data attached to them as well as a trait implementation for the `EnumTag`
/// trait in order to make it possible to create instances of the C-like `enum` type.
///
/// All generated code is enclosed in a `const` block for proc. macro hygiene.
fn enum_tag_impl(input: DeriveInput) -> Result<TokenStream2> {
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let data = match &input.data {
        syn::Data::Enum(data) => data,
        syn::Data::Struct(_) => bail_spanned!(
            input,
            "derive(EnumTag) only works on `enum` types but found struct"
        ),
        syn::Data::Union(_) => bail_spanned!(
            input,
            "derive(EnumTag) only works on `enum` types but found union"
        ),
    };
    let tag_ident = format_ident!("{}Tag", ident);
    let variants = data.variants.iter().map(make_unit);
    let variant_idents = data.variants.iter().map(|variant| &variant.ident);
    let extra_derives = match input.attrs.iter().find(|a| {
        a.path()
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .eq(["enum_tag", "derive"])
    }) {
        Some(attr) if let Meta::List(meta_list) = &&attr.meta => &meta_list.tokens,
        Some(_attr) => {
            todo!()
        }
        None => &TokenStream2::new(),
    };

    Ok(quote! {
        const _: () = {
            #[derive(
                ::core::fmt::Debug,
                ::core::clone::Clone,
                ::core::marker::Copy,
                ::core::cmp::PartialEq,
                ::core::cmp::Eq,
                ::core::cmp::PartialOrd,
                ::core::cmp::Ord,
                ::core::hash::Hash,
                #extra_derives
            )]
            pub enum #tag_ident {
                #( #variants ),*
            }

            impl #impl_generics ::enum_tag::EnumTag for #ident #type_generics #where_clause {
                type Tag = #tag_ident;

                fn tag(&self) -> <Self as ::enum_tag::EnumTag>::Tag {
                    match self {
                        #(
                            Self::#variant_idents { .. } => <Self as ::enum_tag::EnumTag>::Tag::#variant_idents,
                        )*
                    }
                }
            }
        };
    })
}

/// Turns the given `variant` into a unit variant.
///
/// Everything else stays the same for the `variant`.
fn make_unit(variant: &syn::Variant) -> syn::Variant {
    syn::Variant {
        attrs: variant.attrs.clone(),
        ident: variant.ident.clone(),
        fields: syn::Fields::Unit,
        discriminant: variant.discriminant.clone(),
    }
}
