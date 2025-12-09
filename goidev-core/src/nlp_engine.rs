use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};
use std::sync::OnceLock;
use unicode_segmentation::UnicodeSegmentation;

// Keep compiled regex in a static for performance.
static SENTENCE_REGEX: OnceLock<Regex> = OnceLock::new();

/// Extract sentences from a block of text using a pragmatic regex-based splitter.
pub fn extract_sentences(text: &str) -> Vec<String> {
    let re = SENTENCE_REGEX.get_or_init(|| Regex::new(r"(?s).*?[.!?](?:\s+|$)").unwrap());
    let mut sentences = Vec::new();
    for cap in re.find_iter(text) {
        let s = cap.as_str().trim();
        if !s.is_empty() {
            sentences.push(s.to_string());
        }
    }
    // Fallback
    if sentences.is_empty() {
        let t = text.trim();
        if !t.is_empty() {
            sentences.push(t.to_string());
        }
    }
    sentences
}

/// Tokenize a sentence or block into word tokens.
pub fn tokenize_words(text: &str) -> Vec<String> {
    UnicodeSegmentation::unicode_words(text)
        .map(|w| w.to_string())
        .collect()
}

/// Return the stem/base form of an English word using Snowball stemmer.
pub fn get_base_form(word: &str) -> String {
    // Check if it's a phrase (contains spaces after trim)
    if word.trim().contains(char::is_whitespace) {
        return clean_phrase(word).to_lowercase();
    }

    // Clean token of punctuation before stemming
    let clean_word = clean_token(word);
    let stemmer = Stemmer::create(Algorithm::English);
    // Stemmer usually expects lowercase
    stemmer.stem(&clean_word.to_lowercase()).to_string()
}

/// Helper to remove punctuation from a single word
fn clean_token(token: &str) -> String {
    token.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Helper for phrases: preserve spaces, remove other punctuation
fn clean_phrase(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .trim()
        .to_string()
}

/// Find the sentence that contains `word` (case-insensitive).
pub fn sentence_for_word(block_text: &str, word: &str) -> Option<String> {
    let sentences = extract_sentences(block_text);
    let target_clean = clean_phrase(word).to_lowercase();

    if target_clean.is_empty() {
        return None;
    }

    // 1) Return sentence that contains the target string (handles phrases)
    for s in &sentences {
        if s.to_lowercase().contains(&target_clean) {
            return Some(s.clone());
        }
    }

    // 2) Return a sentence with any token match (word-boundary aware)
    let target_tokens: Vec<String> = tokenize_words(&target_clean)
        .into_iter()
        .map(|t| t.to_lowercase())
        .collect();

    if !target_tokens.is_empty() {
        for s in &sentences {
            let s_tokens: Vec<String> = tokenize_words(&s.to_lowercase());
            if s_tokens.iter().any(|tok| target_tokens.iter().any(|t| t == tok)) {
                return Some(s.clone());
            }
        }
    }

    // 3) If not found, return the first sentence (avoid returning empty)
    sentences.into_iter().next()
}
