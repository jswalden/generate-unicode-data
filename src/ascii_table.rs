use proc_macro2::Ident;
use proc_macro2::Span;
use quote::quote;

/// Generate a `const` ASCII lookup table with the given name, populated using
/// the given predicate function.
pub fn generate_ascii_table(
    table_name: &str,
    predicate: &dyn Fn(u32) -> bool,
) -> proc_macro2::TokenStream {
    let table_name = Ident::new(table_name, Span::call_site());

    let table_length = (0x00..=0x7Fu32).clone().count();

    let elems = (0x00..=0x7Fu32).map(predicate);

    quote! {
        const #table_name: [bool; #table_length] = [
            #( #elems ),*
        ];
    }
}
