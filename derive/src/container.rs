use eyre::Result;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::visit::{self, Visit};

/// Find the command field in a struct.
///
/// Looks for the identifier of the command field in a struct. The first field
/// with a `#[command]` attribute is returned. It does not support a struct
/// having multiple commands and is naive about whether that is a subcommand or
/// not currently.
fn get_field_name(data: &syn::DataStruct, attr_name: &str) -> Option<Ident> {
    for field in &data.fields {
        for attr in &field.attrs {
            if attr.path().is_ident(attr_name) {
                return field.ident.clone();
            }
        }
    }

    None
}

/// Dispatch to the enum if it exists or return None.
///
/// Looks for `#[command]` in a struct and on the first hit generates an
/// implementation that calls into the `next()` field of that enum.
fn struct_impl(name: &Ident, data: &syn::DataStruct) -> TokenStream {
    #[allow(clippy::single_match_else)]
    let next_impl = match get_field_name(data, "command") {
        Some(field_name) => quote! {
            fn next(&self) -> Option<&dyn ::cata::command::Command> {
                self.#field_name.next()
            }
        },
        None => quote! {
             fn next(&self) -> Option<&dyn ::cata::command::Command> {
                 None
             }
        },
    };

    quote! {
        #[automatically_derived]
        impl ::cata::command::Container for #name {
            #next_impl
        }
    }
}

/// Accumulate all the variants in an enum.
///
/// This is used to generate the match statement in the `next()` function.
#[derive(Default)]
struct UnnamedTypes<'ast> {
    commands: Vec<&'ast syn::Ident>,
}

impl<'ast> Visit<'ast> for UnnamedTypes<'ast> {
    fn visit_variant(&mut self, i: &'ast syn::Variant) {
        self.commands.push(&i.ident);

        visit::visit_variant(self, i);
    }
}

/// Dispatch to the correct command in the enum via variants.
fn enum_impl(name: &Ident, data: &syn::DataEnum) -> TokenStream {
    let mut visitor = UnnamedTypes::default();
    visitor.visit_data_enum(data);

    let commands = visitor.commands;

    quote! {
        #[automatically_derived]
        impl ::cata::command::Container for #name {
            fn next(&self) -> Option<&dyn ::cata::command::Command> {
                match self {
                    #(Self::#commands(cmd) => Some(cmd),)*
                }
            }
        }
    }
}

/// Generate implementation of the `Container` trait for either a struct or an
/// enum.
///
/// The implementation of struct and enum are linked. This is because the AST
/// for the struct does not contain enough information to dispatch to the
/// correct command in the enum. Instead, a simple implementation in the struct
/// is generated pointing to `next()` on the enum which ends up doing the heavy
/// lifting.
pub fn derive(input: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &input.ident;

    match input.data {
        syn::Data::Struct(ref data) => Ok(struct_impl(name, data)),
        syn::Data::Enum(ref data) => Ok(enum_impl(name, data)),
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "Command can only be derived for structs or enums",
        )),
    }
}
