extern crate proc_macro;
use proc_macro::{TokenStream};
use quote::{quote};

use unicode_info::derived_core_properties;
use unicode_info::code_point_table;

#[proc_macro]
pub fn generate_unicode_tables(_input: TokenStream) -> TokenStream {
    let dcp = derived_core_properties::process_derived_core_properties();

    let starts = dcp.id_start;
    let starts_len = starts.len();

    let continues = dcp.id_continue;
    let continues_len = continues.len();

    let code = quote!(
        static id_start_count: usize = #starts_len;
        static id_continue_count: usize = #continues_len;
    );
    
    code_point_table::generate_code_point_table();
    
    TokenStream::from(code)
}
