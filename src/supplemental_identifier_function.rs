use crate::int_ranges;
use proc_macro2::{Ident, Span};
use quote::quote;
use unicode_info::types::CodePointSet;

pub fn generate_supplemental_identifer_function(
    name: &str,
    doc: &str,
    set: &CodePointSet,
) -> proc_macro2::TokenStream {
    let name = Ident::new(name, Span::call_site());

    let ranges: Vec<_> = int_ranges::int_ranges(set).collect();

    quote! {
        #[doc = #doc]
        pub fn #name(code: u32) -> bool {
            #( #ranges )*
            return false;
        }
    }
}
