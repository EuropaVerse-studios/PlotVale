//! `plotvale-search` — project-wide search.
//!
//! A light [`Index`] rebuilt on open (and updated on edit): plain or
//! regular-expression search, with context snippets. [`SavedQuery`] items are
//! named searches that re-evaluate on their own (PlotVale "collections").

use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use plotvale_model::{NodeId, Project};

/// Search errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SearchError {
    /// Invalid regular expression.
    #[error("invalid regular expression: {0}")]
    BadRegex(String),
}

/// Search options.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SearchOptions {
    /// Case sensitive.
    pub case_sensitive: bool,
    /// Interprets the text as regex.
    pub regex: bool,
}

/// One match found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    /// Document holding the match.
    pub doc_id: NodeId,
    /// Line (from 1).
    pub line_no: usize,
    /// Column in characters (from 1).
    pub col: usize,
    /// Line excerpt with context.
    pub excerpt: String,
}

/// Saved search (dynamic collection).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedQuery {
    /// Name shown in the list.
    pub name: String,
    /// Text or regex to search.
    pub pattern: String,
    /// If `true`, a regular expression.
    #[serde(default)]
    pub regex: bool,
    /// If `true`, case sensitive.
    #[serde(default)]
    pub case_sensitive: bool,
}

impl SavedQuery {
    /// Evaluates the search on the current project.
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

/// Search index: lines of documents outside trash.
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
    /// Builds the index from project documents (trash excluded).
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

    /// Number of indexed documents.
    pub fn len(&self) -> usize {
        self.docs.len()
    }

    /// `true` if there is nothing to search.
    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }

    /// Searches the pattern. At most 5000 results (protects the interface).
    pub fn search(&self, pattern: &str, opts: SearchOptions) -> Result<Vec<Hit>, SearchError> {
        if pattern.is_empty() {
            return Ok(Vec::new());
        }
        let matcher = Matcher::compile(pattern, opts)?;
        let mut hits = Vec::new();
        for doc in &self.docs {
            // Also searches the document title.
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

    /// Matches as (start_byte, end_byte) on the line.
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

/// ~80-character excerpt around the match.
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
