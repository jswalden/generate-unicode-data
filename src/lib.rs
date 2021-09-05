extern crate proc_macro;
use quote::quote;

mod ascii_tables;
mod generate_table;
mod index_table;
mod latin1_tables;
mod supplemental_identifier_function;

use std::convert::TryFrom;
use unicode_info::bmp;
use unicode_info::bmp::CharacterInfo;
use unicode_info::case_folding;
use unicode_info::code_point_table;
use unicode_info::constants::{DOLLAR_SIGN, LOW_LINE, MAX_BMP};
use unicode_info::derived_core_properties;
use unicode_info::non_bmp;
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
        static charinfo_shift: u32 = #shift;

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
        static folding_shift: u32 = #shift;

        // Index tables used to compute the index of the right Delta in the
        // folding table.
        #folding_index_tables
    }
}

fn generate_isidentifier_start_part_functions(
    non_bmp: &non_bmp::NonBMPInfo,
) -> proc_macro2::TokenStream {
    let is_identifier_start_fn =
        supplemental_identifier_function::generate_supplemental_identifer_function(
            "is_identifier_start_non_bmp",
            &non_bmp.id_start_set,
        );

    let is_identifier_part_fn =
        supplemental_identifier_function::generate_supplemental_identifer_function(
            "is_identifier_part_non_bmp",
            &non_bmp.id_continue_set,
        );

    quote! {
        #is_identifier_start_fn

        #is_identifier_part_fn
    }
}

fn generate_ascii_lookup_tables(bmp: &bmp::BMPInfo) -> proc_macro2::TokenStream {
    let index = &bmp.index;
    let table = &bmp.table;

    let is_id_compat = |code: u32| [DOLLAR_SIGN, LOW_LINE].contains(&code);

    let is_id_start = |code: u32| {
        let CharacterInfo { flags, .. } = table[index[code as usize] as usize];
        flags.is_unicode_id_start() || is_id_compat(code)
    };

    let is_id_continue = |code: u32| {
        let CharacterInfo { flags, .. } = table[index[code as usize] as usize];
        flags.is_unicode_id_continue_only() || is_id_start(code)
    };

    let is_space = |code: u32| {
        let CharacterInfo { flags, .. } = table[index[code as usize] as usize];
        flags.is_space()
    };

    let isidstart_table = ascii_tables::generate_ascii_table("isidstart", &is_id_start);

    let isident_table = ascii_tables::generate_ascii_table("isident", &is_id_continue);

    let isspace_table = ascii_tables::generate_ascii_table("isspace", &is_space);

    quote! {
        #isidstart_table

        #isident_table

        #isspace_table
    }
}

fn generate_latin1_lookup_tables(bmp: &bmp::BMPInfo) -> proc_macro2::TokenStream {
    let index = &bmp.index;
    let table = &bmp.table;

    let to_lower_case = |code: u32| {
        assert!(code <= MAX_BMP);
        let cinfo = table[index[code as usize] as usize];
        u8::try_from(u16::wrapping_add(code as u16, cinfo.lower_delta.0))
            .expect("Latin-1 lowercases to Latin-1")
    };

    let latin1_to_lower_case_table =
        latin1_tables::generate_latin1_table("latin1_to_lower_case_table", &to_lower_case);

    quote! {
        #latin1_to_lower_case_table
    }
}

#[proc_macro]
pub fn generate_unicode_tables(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let cpt = code_point_table::generate_code_point_table();
    let dcp = derived_core_properties::process_derived_core_properties();
    let bmp = bmp::generate_bmp_info(&cpt, &dcp);
    let non_bmp = non_bmp::generate_non_bmp_info(&cpt);
    let cfd = case_folding::process_case_folding();

    // Character info table and two index tables.
    let charinfo_code = generate_charinfo_tables(&cpt, &dcp);

    // Folding table and two index tables.
    let folding_code = generate_folding_tables(&cfd);

    let isidentifier_start_part_code = generate_isidentifier_start_part_functions(&non_bmp);

    let ascii_lookup_code = generate_ascii_lookup_tables(&bmp);

    let latin1_lookup_code = generate_latin1_lookup_tables(&bmp);

    let code = quote! {
        #charinfo_code

        #folding_code

        #isidentifier_start_part_code

        // ChangesWhenUpperCasedSpecialCasing
        // LengthUpperCaseSpecialCasing
        // AppendUpperCaseSpecialCasing

        // ASCII lookup tables:
        // - isidstart
        // - isident
        // - isspace
        #ascii_lookup_code

        // Latin-1 lookup tables
        #latin1_lookup_code
    };

    code.into()
}
