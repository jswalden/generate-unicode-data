use generate_unicode_data::generate_unicode_tables;

generate_unicode_tables!();

#[test]
fn check_generate_unicode_tables() {
    // CharacterInfo tables and indexes
    assert_eq!(charinfo.len(), 176);
    assert_eq!(charinfo_shift, 6);
    assert_eq!(charinfo_index1.len(), 1024);
    assert_eq!(charinfo_index2.len(), 11584);

    // Folding delta tables and indexes
    assert_eq!(foldinfo.len(), 96);
    assert_eq!(folding_shift, 5);
    assert_eq!(folding_index1.len(), 2048);
    assert_eq!(folding_index2.len(), 1856);

    // IsIdentifier{Start,Part}NonBMP functions

    // ChangesWhenUpperCasedSpecialCasing
    // LengthUpperCaseSpecialCasing
    // AppendUpperCaseSpecialCasing

    // ASCII lookup tables
    assert_eq!(isidstart.len(), 0x80);
    assert_eq!(isident.len(), 0x80);
    assert_eq!(isspace.len(), 0x80);

    // Latin-1 lookup tables
}
