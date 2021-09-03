use generate_unicode_data::generate_unicode_tables;

generate_unicode_tables!();

#[test]
fn check_generate_unicode_tables() {
    // Check for expected current values.
    assert_eq!(foldinfo.len(), 96);
    assert_eq!(folding_shift, 5);
    assert_eq!(fold1.len(), 2048);
    assert_eq!(fold2.len(), 1856);
}
