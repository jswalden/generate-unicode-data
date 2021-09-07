//! Generate ASCII lookup tables.

use proc_macro2::Ident;
use proc_macro2::Span;
use quote::quote;

const ASCII: std::ops::RangeInclusive<u32> = 0x00..=0x7F;

/// Generate a `static` boolean ASCII lookup table with the given name,
/// populated using the given predicate function.
pub fn generate_ascii_table(
    table_name: &str,
    doc: &str,
    predicate: &dyn Fn(u32) -> bool,
) -> proc_macro2::TokenStream {
    let table_name = Ident::new(table_name, Span::call_site());

    let table_length = ASCII.count();

    let elems = ASCII.map(predicate);

    quote! {
        #[doc = #doc]
        static #table_name: [bool; #table_length] = [
            #( #elems ),*
        ];
    }
}
