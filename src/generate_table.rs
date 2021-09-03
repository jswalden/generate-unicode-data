use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;

/// Generate a constant array, of element type `data_type` with name `name`, for
/// `table`.
pub fn generate_table<T>(data_type: TokenStream, name: &str, table: &Vec<T>) -> TokenStream
where
    T: quote::ToTokens + Copy,
{
    let name = Ident::new(name, Span::call_site());
    let n = table.len();

    quote! {const #name: [#data_type; #n] = [
        #( #table ),*
    ];}
}
