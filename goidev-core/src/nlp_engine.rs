use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};
use unicode_segmentation::UnicodeSegmentation;

/// Extract sentences from a block of text using a pragmatic regex-based splitter.
pub fn extract_sentences(text: &str) -> Vec<String> {
    let re = Regex::new(r"(?s).*?[.!?](?:\s+|$)").unwrap();
    let mut sentences = Vec::new();
    for cap in re.find_iter(text) {
        let s = cap.as_str().trim();
        if !s.is_empty() {
            sentences.push(s.to_string());
        }
    }
    // Fallback: if nothing matched, return the whole trimmed text as one sentence
    if sentences.is_empty() {
        let t = text.trim();
        if !t.is_empty() {
            sentences.push(t.to_string());
        }
    }
    sentences
}

/// Tokenize a sentence or block into word tokens using unicode word boundaries.
pub fn tokenize_words(text: &str) -> Vec<String> {
    UnicodeSegmentation::unicode_words(text)
        .map(|w| w.to_string())
        .collect()
}

/// Return the stem/base form of an English word using Snowball stemmer.
pub fn get_base_form(word: &str) -> String {
    let stemmer = Stemmer::create(Algorithm::English);
    stemmer.stem(word).to_string()
}

/// Find the sentence that contains `word` (case-insensitive). Returns the first match.
pub fn sentence_for_word(block_text: &str, word: &str) -> Option<String> {
    let sentences = extract_sentences(block_text);
    let word_lower = word.to_lowercase();
    for s in sentences {
        // Tokenize and lower-case tokens for comparison
        for tok in UnicodeSegmentation::unicode_words(&s) {
            if tok.to_lowercase() == word_lower {
                return Some(s);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sentences_simple() {
        let text = "Hello world. This is a test! Is it working?";
        let s = extract_sentences(text);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0], "Hello world.");
        assert_eq!(s[1], "This is a test!");
        assert_eq!(s[2], "Is it working?");
    }

    #[test]
    fn test_tokenize_and_stem() {
        let text = "Running runs ran";
        let tokens = tokenize_words(text);
        assert_eq!(tokens.len(), 3);
        assert_eq!(get_base_form("running"), "run");
        assert_eq!(get_base_form("runs"), "run");
    }

    #[test]
    fn test_sentence_for_word() {
        let block = "Dr. Smith went home. The quick brown fox jumps over the lazy dog.";
        let sent = sentence_for_word(block, "fox");
        assert!(sent.is_some());
        assert_eq!(sent.unwrap(), "The quick brown fox jumps over the lazy dog.");
    }
}
