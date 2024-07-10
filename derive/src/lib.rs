/// Procedural macros to derive traits for CLI construction.
mod container;
mod file;

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
///
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
///   Child(Child)
/// }
///
/// #[derive(Parser, Container)]
/// pub struct Child {}
/// ```
///
/// [`Container`]: cata::command::Container
#[proc_macro_derive(Container)]
pub fn derive_container(input: TokenStream) -> TokenStream {
    container::derive(syn::parse_macro_input!(input))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Implement value parsing for arbitrary structs deserialized from files.
///
/// Implements the [`ValueParserFactory`] trait for struct which uses the
/// [`File<T>`] value parser to deserialize a path passed in as an argument into
/// the provided struct.
///
/// # Examples
///
/// ```
/// use cata::File;
///
/// #[derive(Clone, Debug, Deserialize)]
/// struct Thing {
///   single: String,
/// }
///
/// #[derive(clap::Parser)]
/// struct Cmd {
///   input: Thing,
/// }
/// ```
///
/// [`Container`]: cata::command::Container
/// [`ValueParserFactory`]: clap::builder::ValueParserFactory
#[proc_macro_derive(File)]
pub fn derive_file(input: TokenStream) -> TokenStream {
    file::derive(&syn::parse_macro_input!(input)).into()
}
