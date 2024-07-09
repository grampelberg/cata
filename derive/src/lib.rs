mod container;

use proc_macro::TokenStream;

/// Derive the [`Container`] trait for structs and enums.
///
/// Looks for clap's `#[command]` in structs to generate a `next()` that can
/// dispatch to the next command which was initially parsed. Any structs without
/// `#[command]` will have a `next()` that returns `None`.
///
/// For commands with subcommands, the enum must also have
/// `#[derive(Container)]`.
///
/// # Examples
/// ```
/// use cata::Container;
/// use clap::{Parser, Subcommand};
///
/// #[derive(Parser, Container)]
/// pub struct Root {
///   #[command(subcommand)]
///   pub cmd: RootCmd,
/// }
///
/// #[derive(Subcommand, Container)]
/// pub enum RootCmd {
///   Subcommand(Subcommand)
/// }
///
/// #[derive(Parser, Container)]
/// pub struct Subcommand {}
/// ```
///
/// [`Container`]: cata::command::Container
#[proc_macro_derive(Container)]
pub fn derive_container(input: TokenStream) -> TokenStream {
    container::derive_container(syn::parse_macro_input!(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
