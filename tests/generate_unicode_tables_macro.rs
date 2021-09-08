use generate_unicode_data::generate_unicode_tables;
#[cfg(test)]
use unicode_info::constants::{
    COMBINING_ACUTE_ACCENT, COMBINING_DIAERESIS, COMBINING_MACRON_BELOW, GREEK_CAPITAL_LETTER_IOTA,
    GREEK_SMALL_LETTER_IOTA_WITH_DIALYTIKA_AND_TONOS, GREEK_SMALL_LETTER_UPSILON_WITH_PSILI,
    LATIN_CAPITAL_LETTER_H, LATIN_CAPITAL_LETTER_S, LATIN_SMALL_LETTER_H_WITH_LINE_BELOW,
    LATIN_SMALL_LETTER_SHARP_S,
};

generate_unicode_tables!();

#[test]
fn check_charinfo_tables() {
    assert_eq!(charinfo.len(), 176);
    assert_eq!(CHARINFO_SHIFT, 6);
    assert_eq!(charinfo_index1.len(), 1024);
    assert_eq!(charinfo_index2.len(), 11584);
}

#[test]
fn check_folding_tables() {
    assert_eq!(foldinfo.len(), 96);
    assert_eq!(FOLDING_SHIFT, 5);
    assert_eq!(folding_index1.len(), 2048);
    assert_eq!(folding_index2.len(), 1856);
}

#[test]
fn check_isidentifier_start_non_bmp() {
    // This code point constitutes a full range.
    const TIRHUTA_OM: u32 = 0x114C7;
    assert_eq!(is_identifier_start_non_bmp(TIRHUTA_OM), true);

    assert_eq!(is_identifier_start_non_bmp(TIRHUTA_OM - 1), false);
    assert_eq!(is_identifier_start_non_bmp(TIRHUTA_OM + 1), false);
}

#[test]
fn check_isidentifier_part_non_bmp() {
    // 0x11370..=0x11374 is a full range.
    const COMBINING_GRANTHA_LETTER_A: u32 = 0x11370;
    assert_eq!(
        is_identifier_part_non_bmp(COMBINING_GRANTHA_LETTER_A - 1),
        false
    );
    assert_eq!(
        is_identifier_part_non_bmp(COMBINING_GRANTHA_LETTER_A + 0),
        true
    );
    assert_eq!(
        is_identifier_part_non_bmp(COMBINING_GRANTHA_LETTER_A + 1),
        true
    );
    assert_eq!(
        is_identifier_part_non_bmp(COMBINING_GRANTHA_LETTER_A + 2),
        true
    );
    assert_eq!(
        is_identifier_part_non_bmp(COMBINING_GRANTHA_LETTER_A + 3),
        true
    );
    assert_eq!(
        is_identifier_part_non_bmp(COMBINING_GRANTHA_LETTER_A + 4),
        true
    );
    assert_eq!(
        is_identifier_part_non_bmp(COMBINING_GRANTHA_LETTER_A + 5),
        false
    );
}

#[test]
fn check_changes_when_upper_cased_special_casing() {
    assert_eq!(changes_when_upper_cased_special_casing('a' as u16), false);

    assert_eq!(
        changes_when_upper_cased_special_casing(LATIN_SMALL_LETTER_SHARP_S as u16),
        true
    );
    assert_eq!(
        changes_when_upper_cased_special_casing(
            GREEK_SMALL_LETTER_IOTA_WITH_DIALYTIKA_AND_TONOS as u16
        ),
        true
    );
    assert_eq!(
        changes_when_upper_cased_special_casing(GREEK_SMALL_LETTER_UPSILON_WITH_PSILI as u16),
        true
    );
}

#[test]
fn check_length_upper_case_special_casing() {
    assert_eq!(
        length_upper_case_special_casing(LATIN_SMALL_LETTER_SHARP_S as u16),
        2
    );
    assert_eq!(
        length_upper_case_special_casing(GREEK_SMALL_LETTER_IOTA_WITH_DIALYTIKA_AND_TONOS as u16),
        3
    );
    assert_eq!(
        length_upper_case_special_casing(GREEK_SMALL_LETTER_UPSILON_WITH_PSILI as u16),
        2
    );
}

#[test]
fn check_append_upper_case_special_casing() {
    const BAD: u16 = 0xFFFF;
    const OFFSET: usize = 3;

    macro_rules! test_append {
        ( $code:ident, [ $replacement:ident $( , $replacements:ident )+ $(,)? ] $(,)? ) => {
            {
                let mut v = vec![BAD; 8];

                let number_written = {
                    let mut n = 1usize; // replacement
                    $(
                        let _ = &$replacements;
                        n += 1;
                    )+
                    n
                };

                let mut index = OFFSET;
                unsafe {
                    append_upper_case_special_casing(
                        $code as u16,
                        v.as_mut_ptr(),
                        &mut index as *mut usize,
                    );
                }

                assert_eq!(index, OFFSET + number_written);

                assert!(v[0..OFFSET].iter().all(|unit| *unit as u16 == BAD));
                assert_eq!(v[OFFSET..OFFSET + number_written],
                           [$replacement as u16, $( $replacements as u16 ),+ ],
                           "replacement units");
                assert!(v[OFFSET + number_written..8].iter().all(|unit| *unit as u16 == BAD));
            }
        }
    }

    test_append!(
        LATIN_SMALL_LETTER_SHARP_S,
        [LATIN_CAPITAL_LETTER_S, LATIN_CAPITAL_LETTER_S]
    );
    test_append!(
        GREEK_SMALL_LETTER_IOTA_WITH_DIALYTIKA_AND_TONOS,
        [
            GREEK_CAPITAL_LETTER_IOTA,
            COMBINING_DIAERESIS,
            COMBINING_ACUTE_ACCENT
        ]
    );
    test_append!(
        LATIN_SMALL_LETTER_H_WITH_LINE_BELOW,
        [LATIN_CAPITAL_LETTER_H, COMBINING_MACRON_BELOW]
    );
}

#[test]
fn check_ascii_lookup_tables() {
    assert_eq!(isidstart.len(), 0x80);
    assert_eq!(isident.len(), 0x80);
    assert_eq!(isspace.len(), 0x80);
}

#[test]
fn check_latin1_lookup_tables() {
    for (i, c) in latin1_to_lower_case_table.iter().enumerate() {
        assert!(i < 256);
        assert_eq!(
            (i as u8 as char)
                .to_string()
                .to_lowercase()
                .chars()
                .nth(0)
                .expect("lowercasing can't empty"),
            *c as char
        );
    }
}

#[test]
fn check_changes_when_upper_cased_non_bmp() {
    /*
    #define CHECK_RANGE(FROM, TO, LEAD, TRAIL_FROM, TRAIL_TO, DIFF) \
      if (lead == LEAD && trail >= TRAIL_FROM && trail <= TRAIL_TO) return true;

    inline bool ChangesWhenUpperCasedNonBMP(char16_t lead, char16_t trail) {
      FOR_EACH_NON_BMP_UPPERCASE(CHECK_RANGE)
      return false;
    }

    #undef CHECK_RANGE

        // U+10428 DESERET SMALL LETTER LONG I .. U+1044F DESERET SMALL LETTER EW
        // U+104D8 OSAGE SMALL LETTER A .. U+104FB OSAGE SMALL LETTER ZHA
        // U+10CC0 OLD HUNGARIAN SMALL LETTER A .. U+10CF2 OLD HUNGARIAN SMALL LETTER US
        // U+118C0 WARANG CITI SMALL LETTER NGAA .. U+118DF WARANG CITI SMALL LETTER VIYO
        // U+16E60 MEDEFAIDRIN SMALL LETTER M .. U+16E7F MEDEFAIDRIN SMALL LETTER Y
        // U+1E922 ADLAM SMALL LETTER ALIF .. U+1E943 ADLAM SMALL LETTER SHA
        #define FOR_EACH_NON_BMP_UPPERCASE(MACRO) \
            MACRO(0x10428, 0x1044f, 0xd801, 0xdc28, 0xdc4f, -40) \
            MACRO(0x104d8, 0x104fb, 0xd801, 0xdcd8, 0xdcfb, -40) \
            MACRO(0x10cc0, 0x10cf2, 0xd803, 0xdcc0, 0xdcf2, -64) \
            MACRO(0x118c0, 0x118df, 0xd806, 0xdcc0, 0xdcdf, -32) \
            MACRO(0x16e60, 0x16e7f, 0xd81b, 0xde60, 0xde7f, -32) \
            MACRO(0x1e922, 0x1e943, 0xd83a, 0xdd22, 0xdd43, -34)
    */
    assert!(changes_when_upper_cased_non_bmp(0xD801, 0xDC28));
}

#[test]
fn check_changes_when_lower_cased_non_bmp() {
    /*
    #define CHECK_RANGE(FROM, TO, LEAD, TRAIL_FROM, TRAIL_TO, DIFF) \
      if (lead == LEAD && trail >= TRAIL_FROM && trail <= TRAIL_TO) return true;

    inline bool ChangesWhenLowerCasedNonBMP(char16_t lead, char16_t trail) {
      FOR_EACH_NON_BMP_LOWERCASE(CHECK_RANGE)
      return false;
    }

    #undef CHECK_RANGE

    // U+10400 DESERET CAPITAL LETTER LONG I .. U+10427 DESERET CAPITAL LETTER EW
        // U+104B0 OSAGE CAPITAL LETTER A .. U+104D3 OSAGE CAPITAL LETTER ZHA
        // U+10C80 OLD HUNGARIAN CAPITAL LETTER A .. U+10CB2 OLD HUNGARIAN CAPITAL LETTER US
        // U+118A0 WARANG CITI CAPITAL LETTER NGAA .. U+118BF WARANG CITI CAPITAL LETTER VIYO
        // U+16E40 MEDEFAIDRIN CAPITAL LETTER M .. U+16E5F MEDEFAIDRIN CAPITAL LETTER Y
        // U+1E900 ADLAM CAPITAL LETTER ALIF .. U+1E921 ADLAM CAPITAL LETTER SHA
        #define FOR_EACH_NON_BMP_LOWERCASE(MACRO) \
            MACRO(0x10400, 0x10427, 0xd801, 0xdc00, 0xdc27, 40) \
            MACRO(0x104b0, 0x104d3, 0xd801, 0xdcb0, 0xdcd3, 40) \
            MACRO(0x10c80, 0x10cb2, 0xd803, 0xdc80, 0xdcb2, 64) \
            MACRO(0x118a0, 0x118bf, 0xd806, 0xdca0, 0xdcbf, 32) \
            MACRO(0x16e40, 0x16e5f, 0xd81b, 0xde40, 0xde5f, 32) \
            MACRO(0x1e900, 0x1e921, 0xd83a, 0xdd00, 0xdd21, 34)
    */
    assert!(changes_when_lower_cased_non_bmp(0xD801, 0xDC00));
}

#[test]
fn check_to_upper_case_non_bmp_trail() {
    /*
    inline char16_t ToUpperCaseNonBMPTrail(char16_t lead, char16_t trail) {
    #define CALC_TRAIL(FROM, TO, LEAD, TRAIL_FROM, TRAIL_TO, DIFF)  \
      if (lead == LEAD && trail >= TRAIL_FROM && trail <= TRAIL_TO) \
        return trail + DIFF;
      FOR_EACH_NON_BMP_UPPERCASE(CALC_TRAIL)
    #undef CALL_TRAIL

      return trail;
    }

    // U+10428 DESERET SMALL LETTER LONG I .. U+1044F DESERET SMALL LETTER EW
        // U+104D8 OSAGE SMALL LETTER A .. U+104FB OSAGE SMALL LETTER ZHA
        // U+10CC0 OLD HUNGARIAN SMALL LETTER A .. U+10CF2 OLD HUNGARIAN SMALL LETTER US
        // U+118C0 WARANG CITI SMALL LETTER NGAA .. U+118DF WARANG CITI SMALL LETTER VIYO
        // U+16E60 MEDEFAIDRIN SMALL LETTER M .. U+16E7F MEDEFAIDRIN SMALL LETTER Y
        // U+1E922 ADLAM SMALL LETTER ALIF .. U+1E943 ADLAM SMALL LETTER SHA
        #define FOR_EACH_NON_BMP_UPPERCASE(MACRO) \
            MACRO(0x10428, 0x1044f, 0xd801, 0xdc28, 0xdc4f, -40) \
            MACRO(0x104d8, 0x104fb, 0xd801, 0xdcd8, 0xdcfb, -40) \
            MACRO(0x10cc0, 0x10cf2, 0xd803, 0xdcc0, 0xdcf2, -64) \
            MACRO(0x118c0, 0x118df, 0xd806, 0xdcc0, 0xdcdf, -32) \
            MACRO(0x16e60, 0x16e7f, 0xd81b, 0xde60, 0xde7f, -32) \
            MACRO(0x1e922, 0x1e943, 0xd83a, 0xdd22, 0xdd43, -34)
    */
    assert!(to_upper_case_non_bmp_trail(0xD801, 0xDC28) == 0xDC00);
}

#[test]
fn check_to_lower_case_non_bmp_trail() {
    /*
    inline char16_t ToLowerCaseNonBMPTrail(char16_t lead, char16_t trail) {
    #define CALC_TRAIL(FROM, TO, LEAD, TRAIL_FROM, TRAIL_TO, DIFF)  \
      if (lead == LEAD && trail >= TRAIL_FROM && trail <= TRAIL_TO) \
        return trail + DIFF;
      FOR_EACH_NON_BMP_LOWERCASE(CALC_TRAIL)
    #undef CALL_TRAIL

      return trail;
    }

    // U+10400 DESERET CAPITAL LETTER LONG I .. U+10427 DESERET CAPITAL LETTER EW
        // U+104B0 OSAGE CAPITAL LETTER A .. U+104D3 OSAGE CAPITAL LETTER ZHA
        // U+10C80 OLD HUNGARIAN CAPITAL LETTER A .. U+10CB2 OLD HUNGARIAN CAPITAL LETTER US
        // U+118A0 WARANG CITI CAPITAL LETTER NGAA .. U+118BF WARANG CITI CAPITAL LETTER VIYO
        // U+16E40 MEDEFAIDRIN CAPITAL LETTER M .. U+16E5F MEDEFAIDRIN CAPITAL LETTER Y
        // U+1E900 ADLAM CAPITAL LETTER ALIF .. U+1E921 ADLAM CAPITAL LETTER SHA
        #define FOR_EACH_NON_BMP_LOWERCASE(MACRO) \
            MACRO(0x10400, 0x10427, 0xd801, 0xdc00, 0xdc27, 40) \
            MACRO(0x104b0, 0x104d3, 0xd801, 0xdcb0, 0xdcd3, 40) \
            MACRO(0x10c80, 0x10cb2, 0xd803, 0xdc80, 0xdcb2, 64) \
            MACRO(0x118a0, 0x118bf, 0xd806, 0xdca0, 0xdcbf, 32) \
            MACRO(0x16e40, 0x16e5f, 0xd81b, 0xde40, 0xde5f, 32) \
            MACRO(0x1e900, 0x1e921, 0xd83a, 0xdd00, 0xdd21, 34)
    */
    assert!(to_lower_case_non_bmp_trail(0xD801, 0xDC00) == 0xDC28);
}
