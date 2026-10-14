// PlotVale word-count rules, ported 1:1 from crates/model/src/count.rs.
// A word is a whitespace-separated run containing at least one alphanumeric
// (Unicode-aware) character: `don't`, numbers, hyphenated words = 1 word.
// Counts run locally on every keystroke (no IPC): the editor, status bar,
// and goals stay instant even on 150k-word scenes.

import type { WordCounts } from "./types";

const ALNUM = /[\p{L}\p{N}]/u;

export function countWords(text: string): WordCounts {
  let words = 0;
  for (const token of text.split(/\s+/)) {
    if (token !== "" && ALNUM.test(token)) words++;
  }
  const chars = [...text];
  let noSpaces = 0;
  for (const ch of chars) {
    if (!/\s/.test(ch)) noSpaces++;
  }
  // Paragraphs: blocks separated by blank lines.
  let paragraphs = 0;
  let inBlock = false;
  for (const line of text.split("\n")) {
    if (line.trim() === "") {
      inBlock = false;
    } else if (!inBlock) {
      inBlock = true;
      paragraphs++;
    }
  }
  return { words, characters: chars.length, characters_no_spaces: noSpaces, paragraphs };
}

/** Today's date as YYYY-MM-DD in local time (daily-goal accounting). */
export function todayLocal(): string {
  const d = new Date();
  const m = `${d.getMonth() + 1}`.padStart(2, "0");
  const day = `${d.getDate()}`.padStart(2, "0");
  return `${d.getFullYear()}-${m}-${day}`;
}
