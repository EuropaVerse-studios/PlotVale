//! Regole di conteggio parole di PlotVale.
//!
//! Una sola definizione, usata ovunque (editor, obiettivi, statistiche, export):
//! una parola è una sequenza separata da spazi che contiene almeno un carattere
//! alfanumerico. Così `l'ho`, `dell'arte`, i numeri e le parole con trattino contano
//! come una parola sola — come si aspetta uno scrittore italiano.

use serde::{Deserialize, Serialize};

/// Statistiche di un testo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordCounts {
    /// Parole secondo le regole di PlotVale.
    pub words: usize,
    /// Caratteri Unicode (codepoint), spazi inclusi.
    pub characters: usize,
    /// Caratteri senza spazi bianchi.
    pub characters_no_spaces: usize,
    /// Paragrafi (blocchi separati da righe vuote).
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

/// Conta le parole di un testo secondo le regole di PlotVale.
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
        // l'ho = 1 parola; numeri e trattini = 1 parola; punteggiatura sola = 0.
        let c = count_words("l'ho detto — ben 3 volte — all'amico.");
        assert_eq!(c.words, 6);
        // "—" da solo non è una parola.
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
        // I marker di formattazione isolati non sono parole.
        assert_eq!(count_words("**grassetto** e *corsivo*").words, 3);
        assert_eq!(count_words("# Titolo").words, 1);
    }
}
