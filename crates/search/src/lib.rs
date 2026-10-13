//! `plotvale-search` — ricerca in tutto il progetto.
//!
//! Un [`Index`] leggero ricostruito all'apertura (e aggiornato alla modifica):
//! ricerca semplice o con espressioni regolari, con estratti di contesto.
//! Le [`SavedQuery`] sono ricerche nominate che si rivalutano da sole
//! (le "collezioni" di PlotVale).

use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use plotvale_model::{NodeId, Project};

/// Errori di ricerca.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SearchError {
    /// Espressione regolare non valida.
    #[error("espressione regolare non valida: {0}")]
    BadRegex(String),
}

/// Opzioni di una ricerca.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SearchOptions {
    /// Distingue maiuscole/minuscole.
    pub case_sensitive: bool,
    /// Interpreta il testo come regex.
    pub regex: bool,
}

/// Un'occorrenza trovata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    /// Documento contenente l'occorrenza.
    pub doc_id: NodeId,
    /// Riga (da 1).
    pub line_no: usize,
    /// Colonna in caratteri (da 1).
    pub col: usize,
    /// Estratto della riga con contesto.
    pub excerpt: String,
}

/// Ricerca salvata (collezione dinamica).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedQuery {
    /// Nome mostrato nella lista.
    pub name: String,
    /// Testo o regex da cercare.
    pub pattern: String,
    /// Se `true`, è un'espressione regolare.
    #[serde(default)]
    pub regex: bool,
    /// Se `true`, distingue maiuscole/minuscole.
    #[serde(default)]
    pub case_sensitive: bool,
}

impl SavedQuery {
    /// Valuta la ricerca sul progetto corrente.
    pub fn evaluate(&self, index: &Index) -> Result<Vec<Hit>, SearchError> {
        index.search(
            &self.pattern,
            SearchOptions {
                case_sensitive: self.case_sensitive,
                regex: self.regex,
            },
        )
    }
}

/// Indice di ricerca: righe dei documenti fuori dal cestino.
#[derive(Clone, Debug, Default)]
pub struct Index {
    docs: Vec<IndexedDoc>,
}

#[derive(Clone, Debug)]
struct IndexedDoc {
    id: NodeId,
    title: String,
    lines: Vec<String>,
}

impl Index {
    /// Costruisce l'indice dai documenti del progetto (cestino escluso).
    pub fn build(project: &Project) -> Self {
        let mut docs = Vec::new();
        for id in project.tree.ids() {
            if project.tree.is_in_trash(id) {
                continue;
            }
            let text = project.doc_text(id);
            if text.trim().is_empty() {
                continue;
            }
            let title = project
                .tree
                .get(id)
                .map(|n| n.title.clone())
                .unwrap_or_default();
            docs.push(IndexedDoc {
                id,
                title,
                lines: text.lines().map(str::to_string).collect(),
            });
        }
        Self { docs }
    }

    /// Numero di documenti indicizzati.
    pub fn len(&self) -> usize {
        self.docs.len()
    }

    /// `true` se non c'è nulla da cercare.
    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }

    /// Cerca il pattern. Al massimo 5000 risultati (protegge l'interfaccia).
    pub fn search(&self, pattern: &str, opts: SearchOptions) -> Result<Vec<Hit>, SearchError> {
        if pattern.is_empty() {
            return Ok(Vec::new());
        }
        let matcher = Matcher::compile(pattern, opts)?;
        let mut hits = Vec::new();
        for doc in &self.docs {
            // Cerca anche nel titolo del documento.
            for (line_no, line) in std::iter::once(&doc.title)
                .chain(doc.lines.iter())
                .enumerate()
            {
                for (start, end) in matcher.find_all(line) {
                    hits.push(Hit {
                        doc_id: doc.id,
                        line_no: line_no + 1,
                        col: line.chars().take(start).count() + 1,
                        excerpt: excerpt(line, start, end),
                    });
                    if hits.len() >= 5000 {
                        return Ok(hits);
                    }
                }
            }
        }
        Ok(hits)
    }
}

enum Matcher {
    Plain {
        needle: String,
        case_sensitive: bool,
    },
    Regex(Regex),
}

impl Matcher {
    fn compile(pattern: &str, opts: SearchOptions) -> Result<Self, SearchError> {
        if opts.regex {
            let re = RegexBuilder::new(pattern)
                .case_insensitive(!opts.case_sensitive)
                .build()
                .map_err(|e| SearchError::BadRegex(e.to_string()))?;
            Ok(Self::Regex(re))
        } else if opts.case_sensitive {
            Ok(Self::Plain {
                needle: pattern.to_string(),
                case_sensitive: true,
            })
        } else {
            Ok(Self::Plain {
                needle: pattern.to_lowercase(),
                case_sensitive: false,
            })
        }
    }

    /// Occorrenze come (inizio_byte, fine_byte) sulla riga.
    fn find_all(&self, line: &str) -> Vec<(usize, usize)> {
        match self {
            Self::Regex(re) => re.find_iter(line).map(|m| (m.start(), m.end())).collect(),
            Self::Plain {
                needle,
                case_sensitive,
            } => {
                if needle.is_empty() {
                    return Vec::new();
                }
                let haystack;
                let hay: &str = if *case_sensitive {
                    line
                } else {
                    haystack = line.to_lowercase();
                    &haystack
                };
                let mut out = Vec::new();
                let mut from = 0;
                while let Some(pos) = hay[from..].find(needle.as_str()) {
                    let start = from + pos;
                    out.push((start, start + needle.len()));
                    from = start + needle.len().max(1);
                }
                out
            }
        }
    }
}

/// Estratto di ~80 caratteri attorno all'occorrenza.
fn excerpt(line: &str, start: usize, end: usize) -> String {
    let chars: Vec<char> = line.chars().collect();
    let byte_to_char = |byte: usize| line[..byte.min(line.len())].chars().count();
    let s = byte_to_char(start);
    let e = byte_to_char(end).max(s + 1);
    let from = s.saturating_sub(40);
    let to = (e + 40).min(chars.len());
    let mut out = String::new();
    if from > 0 {
        out.push('…');
    }
    out.extend(chars[from..to].iter());
    if to < chars.len() {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotvale_model::Project;

    fn sample() -> Project {
        let mut project = Project::new("Prova", "Autore", "0.1.0");
        let cap = project.tree.children_of(project.tree.draft_root).unwrap()[0];
        let scene = project.tree.children_of(cap).unwrap()[0];
        project.set_doc_text(scene, "Elena guardò il mare.\nIl MARE era calmo.\n");
        project
    }

    #[test]
    fn plain_search_is_case_insensitive_by_default() {
        let project = sample();
        let index = Index::build(&project);
        let hits = index.search("mare", SearchOptions::default()).unwrap();
        assert_eq!(hits.len(), 2);
        let hits = index
            .search(
                "mare",
                SearchOptions {
                    case_sensitive: true,
                    regex: false,
                },
            )
            .unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn regex_search_and_bad_regex() {
        let project = sample();
        let index = Index::build(&project);
        let hits = index
            .search(
                "m.re",
                SearchOptions {
                    case_sensitive: false,
                    regex: true,
                },
            )
            .unwrap();
        assert_eq!(hits.len(), 2);
        assert!(matches!(
            index.search(
                "([",
                SearchOptions {
                    case_sensitive: false,
                    regex: true
                }
            ),
            Err(SearchError::BadRegex(_))
        ));
    }

    #[test]
    fn trash_is_not_indexed() {
        let mut project = sample();
        let cap = project.tree.children_of(project.tree.draft_root).unwrap()[0];
        project.tree.move_to_trash(cap).unwrap();
        let index = Index::build(&project);
        assert!(index.is_empty());
    }

    #[test]
    fn saved_query_evaluates() {
        let project = sample();
        let index = Index::build(&project);
        let q = SavedQuery {
            name: "Il mare".to_string(),
            pattern: "mare".to_string(),
            regex: false,
            case_sensitive: false,
        };
        assert_eq!(q.evaluate(&index).unwrap().len(), 2);
    }
}
