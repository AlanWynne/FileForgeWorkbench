//! Integration tests for case folding.

use ff_encoding::*;

#[test]
fn case_fold_for_search_comparison() {
    // Validates: Requirement 10.1, 10.7
    let converter: Box<dyn ICaseConverter> = Box::new(CaseFolder::new());

    // Case-insensitive search: fold both search term and text
    let text = "Hello World";
    let search = "HELLO WORLD";

    let folded_text = converter.case_convert_string(text, CaseMode::Fold);
    let folded_search = converter.case_convert_string(search, CaseMode::Fold);

    assert_eq!(folded_text, folded_search);
}

#[test]
fn case_fold_german_eszett() {
    // Validates: Requirement 10.4, 10.6
    let folder = CaseFolder::new();

    // ß folds to "ss"
    let result = folder.case_convert_string("Straße", CaseMode::Fold);
    assert_eq!(result, "straße".replace('ß', "ss"));
    // Also: "strasse"
    assert_eq!(result, "strasse");
}

#[test]
fn case_fold_ligatures() {
    // Validates: Requirement 10.4, 10.6
    let folder = CaseFolder::new();

    assert_eq!(folder.case_convert_string("\u{FB01}", CaseMode::Fold), "fi");
    assert_eq!(folder.case_convert_string("\u{FB02}", CaseMode::Fold), "fl");
    assert_eq!(
        folder.case_convert_string("\u{FB03}", CaseMode::Fold),
        "ffi"
    );
}

#[test]
fn case_upper_lower_roundtrip_ascii() {
    // Validates: Requirement 10.1
    let folder = CaseFolder::new();

    let text = "hello world";
    let upper = folder.case_convert_string(text, CaseMode::Upper);
    let lower = folder.case_convert_string(&upper, CaseMode::Lower);

    assert_eq!(upper, "HELLO WORLD");
    assert_eq!(lower, text);
}

#[test]
fn icase_converter_trait_is_send_sync() {
    // Validates: Requirement 10.7
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CaseFolder>();
}
