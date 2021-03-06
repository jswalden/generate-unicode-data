//! Generate Latin-1 lookup tables.

use proc_macro2::Ident;
use proc_macro2::Span;
use quote::quote;

const LATIN1: std::ops::RangeInclusive<u32> = 0x00..=0xFFu32;

/// Generate a `static` boolean ASCII lookup table with the given name,
/// populated using the given predicate function.
pub fn generate_latin1_table(
    table_name: &str,
    doc: &str,
    predicate: &dyn Fn(u32) -> u8,
) -> proc_macro2::TokenStream {
    let table_name = Ident::new(table_name, Span::call_site());

    let table_length = LATIN1.count();

    let elems = LATIN1.map(predicate);

    quote! {
        #[no_mangle]
        #[doc = #doc]
        static #table_name: [u8; #table_length] = [
            #( #elems ),*
        ];
    }
}
