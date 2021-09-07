//! Generate a constant table of elements.

use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;

/// Generate a constant array, of element type `data_type` with name `name`, for
/// `table`.
pub fn generate_table<T>(
    data_type: TokenStream,
    name: &str,
    doc: &str,
    table: &Vec<T>,
) -> proc_macro2::TokenStream
where
    T: quote::ToTokens,
{
    let name = Ident::new(name, Span::call_site());
    let n = table.len();

    quote! {
        #[doc = #doc]
        static #name: [#data_type; #n] = [
        #( #table ),*
    ];}
}
