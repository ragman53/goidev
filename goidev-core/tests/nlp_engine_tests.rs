use goidev_core::nlp_engine;

#[test]
fn test_get_base_form_single_word() {
    assert_eq!(nlp_engine::get_base_form("running"), "run");
    assert_eq!(nlp_engine::get_base_form("CATS"), "cat");
}

#[test]
fn test_get_base_form_phrase() {
    // Current desired behavior for phrases: preserve them lowercased, maybe no stemming?
    // The plan says: "base form" will simply be the lowercased phrase
    assert_eq!(
        nlp_engine::get_base_form("Machine Learning"),
        "machine learning"
    );
    assert_eq!(nlp_engine::get_base_form("hello world"), "hello world");
}

#[test]
fn test_sentence_for_word_simple() {
    let text = "Hello world. This is a test. Goodbye.";
    let s = nlp_engine::sentence_for_word(text, "test");
    assert_eq!(s, Some("This is a test.".to_string()));
}

#[test]
fn test_sentence_for_word_phrase() {
    let text = "We are studying machine learning today. It is fun.";
    // This is expected to fail with current implementation
    let s = nlp_engine::sentence_for_word(text, "machine learning");
    assert_eq!(
        s,
        Some("We are studying machine learning today.".to_string())
    );
}

#[test]
fn test_sentence_for_word_not_found() {
    let text = "One. Two. Three.";
    let s = nlp_engine::sentence_for_word(text, "Four");
    assert_eq!(s, None);
}
