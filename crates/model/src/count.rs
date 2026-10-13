//! PlotVale word-count rules.
//!
//! A single definition, used everywhere (editor, goals, statistics, export):
//! a word is a whitespace-separated run containing at least one alphanumeric
//! character. So `don't`, numbers, and hyphenated words each count as one
//! word — as writers expect.

use serde::{Deserialize, Serialize};

/// Statistics of a text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordCounts {
    /// Words under the PlotVale rules.
    pub words: usize,
    /// Unicode characters (codepoints), spaces included.
    pub characters: usize,
    /// Characters without whitespace.
    pub characters_no_spaces: usize,
    /// Paragraphs (blocks separated by blank lines).
    pub paragraphs: usize,
}

impl std::ops::Add for WordCounts {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            words: self.words + rhs.words,
            characters: self.characters + rhs.characters,
            characters_no_spaces: self.characters_no_spaces + rhs.characters_no_spaces,
            paragraphs: self.paragraphs + rhs.paragraphs,
        }
    }
}

impl std::iter::Sum for WordCounts {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |a, b| a + b)
    }
}

/// Counts the words of a text under the PlotVale rules.
pub fn count_words(text: &str) -> WordCounts {
    let words = text
        .split_whitespace()
        .filter(|token| token.chars().any(|c| c.is_alphanumeric()))
        .count();
    let characters = text.chars().count();
    let characters_no_spaces = text.chars().filter(|c| !c.is_whitespace()).count();
    let paragraphs = text
        .split('\n')
        .collect::<Vec<_>>()
        .split(|line: &&str| line.trim().is_empty())
        .filter(|block| block.iter().any(|line| !line.trim().is_empty()))
        .count();
    WordCounts {
        words,
        characters,
        characters_no_spaces,
        paragraphs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn italian_rules() {
        // don't = 1 word; numbers and hyphens = 1 word; punctuation alone = 0.
        let c = count_words("l'ho detto — ben 3 volte — all'amico.");
        assert_eq!(c.words, 6);
        // A lone "—" is not a word.
        assert_eq!(count_words("—").words, 0);
        assert_eq!(count_words("").words, 0);
    }

    #[test]
    fn paragraphs_and_chars() {
        let c = count_words("Prima riga.\n\nSeconda riga.\nTerza riga.");
        assert_eq!(c.paragraphs, 2);
        assert_eq!(c.words, 6);
        assert!(c.characters > c.characters_no_spaces);
    }

    #[test]
    fn markdown_markers_do_not_inflate() {
        // Lone formatting markers are not words.
        assert_eq!(count_words("**grassetto** e *corsivo*").words, 3);
        assert_eq!(count_words("# Titolo").words, 1);
    }
}
