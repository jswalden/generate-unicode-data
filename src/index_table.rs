//! Generate code for two index tables generated from a single input inde.

use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use unicode_info::types::NumericType;

fn generate_index_table(index: &Vec<u32>, elem_type: NumericType, index_name: &str) -> TokenStream {
    let index_name = Ident::new(index_name, Span::call_site());
    let n = index.len();

    let elems = match elem_type {
        NumericType::U8 => {
            let elems: Vec<u8> = index.iter().map(|v| *v as u8).collect();
            quote! { #( #elems ),* }
        }
        NumericType::U16 => {
            let elems: Vec<u16> = index.iter().map(|v| *v as u16).collect();
            quote! { #( #elems ),* }
        }
        NumericType::U32 => {
            let elems: Vec<u32> = index.iter().map(|v| *v as u32).collect();
            quote! { #( #elems ),* }
        }
    };

    quote! {const #index_name : [u8; #n] = [
        #elems
    ];}
}

pub fn generate_index_tables(
    index1: &Vec<u32>,
    index1_elem_type: NumericType,
    index1_name: &str,
    index2: &Vec<u32>,
    index2_elem_type: NumericType,
    index2_name: &str,
) -> TokenStream {
    let index1_code = generate_index_table(&index1, index1_elem_type, index1_name);
    let index2_code = generate_index_table(&index2, index2_elem_type, index2_name);

    quote! {
        #index1_code

        #index2_code
    }
}
