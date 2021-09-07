extern crate proc_macro;
use quote::quote;

mod ascii_tables;
mod generate_table;
mod index_table;
mod int_ranges;
mod latin1_tables;
mod special_casing_functions;
mod supplemental_identifier_function;

use crate::special_casing_functions::generate_special_casing_functions;
use std::convert::TryFrom;
use unicode_info::bmp;
use unicode_info::bmp::CharacterInfo;
use unicode_info::case_folding;
use unicode_info::code_point_table;
use unicode_info::constants::{DOLLAR_SIGN, LOW_LINE, MAX_BMP};
use unicode_info::derived_core_properties;
use unicode_info::non_bmp;
use unicode_info::special_casing;
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
        r#"
A table of `CharacterInfo`s.  Every BMP code point is associated with one
such `CharacterInfo`, at index determined using the code point,
`CHARINFO_SHIFT`, and `charinfo_index1` and `charinfo_index2`.  Specifically,

```text
let mask = (1usize << CHARINFO_SHIFT) - 1;
for code_point in 0..=0xFFFFu16 {
    let index1_entry = charinfo_index1[code_point >> CHARINFO_SHIFT];
    let index1_index_component = index1_entry << CHARINFO_SHIFT;
    let mask_component = code_point & mask;
    // ...and the `CharacterInfo` pertinent to `code_point` is therefore:
    let cinfo = charinfo[index2[index1_index_component + mask_component]];
}
```
"#
        .trim(),
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

        /// The shift used in indexing into the two index tables.
        const CHARINFO_SHIFT: u32 = #shift;

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
        r#"
A table of `Delta`s, each a value that can be added (with wrapping) to some BMP
code point to determine the code point to which it case-folds.  The precise
`Delta` that applies to a given code point is determined using the code point,
`FOLDING_SHIFT`, and `folding_index1` and `folding_index2`.  Specifically,

```text
let mask = (1usize << FOLDING_SHIFT) - 1;
for code_point in 0..=0xFFFFu16 {
    let index1_entry = folding_index1[code_point >> FOLDING_SHIFT];
    let index1_index_component = index1_entry << FOLDING_SHIFT;
    let mask_component = code_point & mask;
    // ...and the `Delta` pertinent to `code_point` is therefore:
    let delta = foldinfo[index2[index1_index_component + mask_component]];
}
```
"#
        .trim(),
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
        const FOLDING_SHIFT: u32 = #shift;

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
            r#"
Return true iff the provided _non-BMP_ code point may validly appear as the
first character in an identifier.

It is an error to call this function with a BMP code point, i.e. one whose value
is 0xFFFF or lower.
            "#
            .trim(),
            &non_bmp.id_start_set,
        );

    let is_identifier_part_fn =
        supplemental_identifier_function::generate_supplemental_identifer_function(
            "is_identifier_part_non_bmp",
            r#"
Return true iff the provided _non-BMP_ code point may validly appear within an
identifier after its first character.

It is an error to call this function with a BMP code point, i.e. one whose value
is 0xFFFF or lower.
                        "#
            .trim(),
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

    let isidstart_table = ascii_tables::generate_ascii_table(
        "isidstart",
        r#"
A lookup table storing at index `i` whether the ASCII code point with value `i`
is matched by the ECMAScript IdentifierStart production, allowing it to appear
at the start of an identifier.
    "#
        .trim(),
        &is_id_start,
    );

    let isident_table = ascii_tables::generate_ascii_table(
        "isident",
        r#"
A lookup table storing at index `i` whether the ASCII code point with value `i`
is matched by the ECMAScript IdentifierPart production, allowing it to appear
within an identifier after its starting character.  (This is the same as
`idstart` except that numbers are permitted.)
        "#
        .trim(),
        &is_id_continue,
    );

    let isspace_table = ascii_tables::generate_ascii_table(
        "isspace",
        r#"
A lookup table storing at index `i` whether the ASCII code point with value `i`
matches either of the ECMAScript WhiteSpace or LineTerminator productions.
        "#
        .trim(),
        &is_space,
    );

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

    let latin1_to_lower_case_table = latin1_tables::generate_latin1_table(
        "latin1_to_lower_case_table",
        r#"
A lookup table storing at index `i` the value of the lowercase form of the code
point with value `i`.
        "#,
        &to_lower_case,
    );

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
    let scd = special_casing::process_special_casing(&bmp);

    // Character info table and two index tables.
    let charinfo_code = generate_charinfo_tables(&cpt, &dcp);

    // Folding table and two index tables.
    let folding_code = generate_folding_tables(&cfd);

    let isidentifier_start_part_code = generate_isidentifier_start_part_functions(&non_bmp);

    let special_casing_code = generate_special_casing_functions(&scd);

    let ascii_lookup_code = generate_ascii_lookup_tables(&bmp);

    let latin1_lookup_code = generate_latin1_lookup_tables(&bmp);

    let code = quote! {
        /* Generated by the generate_unicode_tables! macro, DO NOT MODIFY */

        #charinfo_code

        #folding_code

        #isidentifier_start_part_code

        #special_casing_code

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
