use generate_unicode_data::generate_unicode_tables;

generate_unicode_tables!();

#[test]
fn check_generate_unicode_tables() {
    // These constants were derived not from testing and finding out, but from
    // comments at end of the respective sections in DerivedCoreProperties.txt.
    assert_eq!(id_start_count, 131_482);
    assert_eq!(id_continue_count, 134_434);

    assert_eq!(folding_shift, 5, "at present");
}
