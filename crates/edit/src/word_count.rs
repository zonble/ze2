// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use stdext::unicode::Utf8Chars;

use crate::document::ReadableDocument;

/// Stores word-count statistics about a document.
pub struct WordCountStatistics {
    pub all_characters: usize,
    pub latin_words: usize,
    pub asian_characters: usize,
}

pub fn count_document(document: &dyn ReadableDocument, text_length: usize) -> WordCountStatistics {
    let mut stats = WordCountStatistics { all_characters: 0, latin_words: 0, asian_characters: 0 };
    let mut in_latin_word = false;
    let mut offset = 0;

    while offset < text_length {
        let chunk = document.read_forward(offset);
        let mut chars = Utf8Chars::new(chunk, 0);
        while let Some(ch) = chars.next() {
            stats.all_characters += 1;
            stats.asian_characters += usize::from(is_asian_character(ch));

            if is_latin_word_character(ch) {
                if !in_latin_word {
                    stats.latin_words += 1;
                    in_latin_word = true;
                }
            } else {
                in_latin_word = false;
            }
        }

        offset += chunk.len();
    }

    stats
}

fn is_latin_word_character(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || (ch.is_alphanumeric()
            && (('\u{00c0}'..='\u{024f}').contains(&ch) || ('\u{1e00}'..='\u{1eff}').contains(&ch)))
}

fn is_asian_character(ch: char) -> bool {
    ('\u{3040}'..='\u{30ff}').contains(&ch)
        || ('\u{3100}'..='\u{312f}').contains(&ch)
        || ('\u{31a0}'..='\u{31bf}').contains(&ch)
        || ('\u{31f0}'..='\u{31ff}').contains(&ch)
        || ('\u{3400}'..='\u{4dbf}').contains(&ch)
        || ('\u{4e00}'..='\u{9fff}').contains(&ch)
        || ('\u{ac00}'..='\u{d7af}').contains(&ch)
        || ('\u{f900}'..='\u{faff}').contains(&ch)
        || ('\u{20000}'..='\u{2ebef}').contains(&ch)
        || ('\u{30000}'..='\u{3134f}').contains(&ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(text: &str) -> WordCountStatistics {
        count_document(&text.as_bytes(), text.len())
    }

    #[test]
    fn counts_latin_words_as_runs() {
        let stats = count("Hello, world 123 cafe");
        assert_eq!(stats.all_characters, 21);
        assert_eq!(stats.latin_words, 4);
        assert_eq!(stats.asian_characters, 0);
    }

    #[test]
    fn counts_extended_latin_words() {
        let stats = count("café café");
        assert_eq!(stats.latin_words, 2);
    }

    #[test]
    fn counts_asian_characters() {
        let stats = count("Hello 世界かなカナ한글");
        assert_eq!(stats.latin_words, 1);
        assert_eq!(stats.asian_characters, 8);
    }
}
