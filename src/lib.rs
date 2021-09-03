extern crate proc_macro;
use quote::quote;

mod generate_table;
mod index_table;

use unicode_info::case_folding;
use unicode_info::table;

fn generate_folding_tables(data: &case_folding::CaseFoldingData) -> proc_macro2::TokenStream {
    let table::TableSplit {
        index1,
        index1_elem_type,
        index2,
        index2_elem_type,
        shift,
    } = table::split_table(&data.bmp_folding_index);

    let folding_table = generate_table::generate_table(
        quote!(::unicode_info::case_folding::Delta),
        "foldinfo",
        &data.bmp_folding_table,
    );

    let folding_index_tables = index_table::generate_index_tables(
        &index1,
        index1_elem_type,
        "fold1",
        &index2,
        index2_elem_type,
        "fold2",
    );

    quote! {
        // The table of Deltas, into which the index tables index.
        #folding_table

        // The shift used in indexing into the two index tables.
        const folding_shift: u32 = #shift;

        // Index tables used to compute the index of the right Delta in the
        // folding table.
        #folding_index_tables
    }
}

#[proc_macro]
pub fn generate_unicode_tables(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Folding table plus two folding index tables.
    let folding_code = generate_folding_tables(&case_folding::process_case_folding());

    let code = quote! {
        #folding_code
    };

    proc_macro::TokenStream::from(code)
}
