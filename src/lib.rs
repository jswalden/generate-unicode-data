extern crate proc_macro;
use quote::quote;

mod generate_table;
mod index_table;

use unicode_info::bmp;
use unicode_info::case_folding;
use unicode_info::code_point_table;
use unicode_info::derived_core_properties;
use unicode_info::table;

fn generate_charinfo_tables(
    table: &code_point_table::CodePointTable,
    derived_properties: &derived_core_properties::DerivedCorePropertyData,
) -> proc_macro2::TokenStream {
    let bmp::BMPInfo { index, table } = bmp::generate_bmp_info(table, derived_properties);

    let table::TableSplit {
        index1,
        index1_elem_type,
        index2,
        index2_elem_type,
        shift,
    } = table::split_table(&index);

    let info_table = generate_table::generate_table(
        quote!(::unicode_info::bmp::CharacterInfo),
        "charinfo",
        &table,
    );

    let info_index_tables = index_table::generate_index_tables(
        &index1,
        index1_elem_type,
        "charinfo_index1",
        &index2,
        index2_elem_type,
        "charinfo_index2",
    );

    quote! {
        // The table of CharacterInfos, into which the index tables index.
        #info_table

        // The shift used in indexing into the two index tables.
        const charinfo_shift: u32 = #shift;

        // Index tables used to compute the index of the right CharacterInfo in
        // the info table.
        #info_index_tables
    }
}

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
        "folding_index1",
        &index2,
        index2_elem_type,
        "folding_index2",
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
    let cpt = code_point_table::generate_code_point_table();
    let dcp = derived_core_properties::process_derived_core_properties();
    let cfd = case_folding::process_case_folding();

    // Character info table and two index tables.
    let charinfo_code = generate_charinfo_tables(&cpt, &dcp);

    // Folding table and two index tables.
    let folding_code = generate_folding_tables(&cfd);

    let code = quote! {
        #charinfo_code

        #folding_code
    };

    code.into()
}
