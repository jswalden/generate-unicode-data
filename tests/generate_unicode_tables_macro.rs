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
    assert_eq!(charinfo_shift, 6);
    assert_eq!(charinfo_index1.len(), 1024);
    assert_eq!(charinfo_index2.len(), 11584);
}

#[test]
fn check_folding_tables() {
    assert_eq!(foldinfo.len(), 96);
    assert_eq!(folding_shift, 5);
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
                let mut v = vec![0; 8];
                v.fill(BAD);

                unsafe {
                    append_upper_case_special_casing(
                        $code as u16,
                        v.as_mut_ptr(),
                        OFFSET,
                    );
                }

                for i in 0..OFFSET {
                    assert_eq!(v[i], BAD);
                }

                let mut count = 1;
                assert_eq!(v[OFFSET], $replacement as u16);
                {
                    $(
                        count += 1;
                        assert_eq!(v[OFFSET + count - 1], $replacements as u16);
                    )+
                }

                for i in OFFSET + count..8 {
                    assert_eq!(v[i], BAD);
                }
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
