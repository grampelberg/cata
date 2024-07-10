use proc_macro2::TokenStream;
use quote::quote;

/// Generate implementation of the `ValueParserFactory` trait for structs.
///
/// This relies on the `TypedValueParser` implementation for [`File`].
///
/// [`File`]: cata::file::File<T>
pub fn derive(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;

    quote! {
        #[automatically_derived]
        impl ::clap::builder::ValueParserFactory for #name {
            type Parser = ::cata::file::File<#name>;

            fn value_parser() -> Self::Parser {
                ::cata::file::File::default()
            }
        }
    }
}
