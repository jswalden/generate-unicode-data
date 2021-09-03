extern crate proc_macro;
use quote::quote;

mod generate_table;

use unicode_info::case_folding;
use unicode_info::derived_core_properties;
use unicode_info::table;

fn generate_dummy_code() -> proc_macro2::TokenStream {
    let dcp = derived_core_properties::process_derived_core_properties();

    let starts = dcp.id_start;
    let starts_len = starts.len();

    let continues = dcp.id_continue;
    let continues_len = continues.len();

    let code = quote! {
        static id_start_count: usize = #starts_len;
        static id_continue_count: usize = #continues_len;
    };

    code
}

fn generate_folding_tables(data: &case_folding::CaseFoldingData) -> proc_macro2::TokenStream {
    let table::TableSplit {
        t1,
        t1_elem_type,
        t2,
        t2_elem_type,
        shift,
    } = table::split_table(&data.bmp_folding_index);
    let folding_tables = generate_table::generate_index_tables(
        &t1,
        t1_elem_type,
        "fold1",
        &t2,
        t2_elem_type,
        "fold2",
    );
    quote! {
        const folding_shift: u32 = #shift;

        #folding_tables
    }
}

#[proc_macro]
pub fn generate_unicode_tables(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let dummy_code = generate_dummy_code();

    let folding_info_table_code = quote!();

    let folding_code = generate_folding_tables(&case_folding::process_case_folding());

    let code = quote! {
        #dummy_code

        #folding_info_table_code

        #folding_code
    };

    proc_macro::TokenStream::from(code)
}
