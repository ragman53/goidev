use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};
use std::sync::OnceLock;
use unicode_segmentation::UnicodeSegmentation;

// 正規表現をコンパイル済みの状態で保持する（パフォーマンス改善）
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
    // Stemmingの前に、記号などを除去してクリーンな単語にする
    let clean_word = clean_token(word);
    let stemmer = Stemmer::create(Algorithm::English);
    stemmer.stem(&clean_word).to_string()
}

/// 文末の記号などを除去するヘルパー
fn clean_token(token: &str) -> String {
    token.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Find the sentence that contains `word` (case-insensitive).
pub fn sentence_for_word(block_text: &str, word: &str) -> Option<String> {
    let sentences = extract_sentences(block_text);
    // 検索対象の単語もクリーンアップする（例: "running." -> "running"）
    let target_clean = clean_token(word).to_lowercase();
    
    if target_clean.is_empty() {
        return None;
    }

    for s in sentences {
        for tok in UnicodeSegmentation::unicode_words(s.as_str()) {
            // 文中の単語と、クリーンアップ済みの選択単語を比較
            if tok.to_lowercase() == target_clean {
                return Some(s);
            }
        }
    }
    None
}