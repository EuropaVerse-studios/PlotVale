// Minimal line diff for snapshot compare (LCS on lines, capped output).
// Old lines removed, new lines added; unchanged context lines kept.

export type DiffLine = { type: "same" | "del" | "add"; text: string };

const MAX_LINES = 600;

export function diffLines(oldText: string, newText: string): { lines: DiffLine[]; truncated: boolean } {
  const a = oldText.split("\n");
  const b = newText.split("\n");
  // Cap input size: huge scenes diff the head only (the dialog says "…").
  const ac = a.slice(0, MAX_LINES);
  const bc = b.slice(0, MAX_LINES);
  const truncated = a.length > MAX_LINES || b.length > MAX_LINES;
  const n = ac.length;
  const m = bc.length;
  // LCS table (capped inputs keep this small).
  const dp: Uint16Array[] = Array.from({ length: n + 1 }, () => new Uint16Array(m + 1));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i][j] =
        ac[i] === bc[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
    }
  }
  const lines: DiffLine[] = [];
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (ac[i] === bc[j]) {
      lines.push({ type: "same", text: ac[i] });
      i++;
      j++;
    } else if (dp[i + 1][j] >= dp[i][j + 1]) {
      lines.push({ type: "del", text: ac[i] });
      i++;
    } else {
      lines.push({ type: "add", text: bc[j] });
      j++;
    }
  }
  while (i < n) lines.push({ type: "del", text: ac[i++] });
  while (j < m) lines.push({ type: "add", text: bc[j++] });
  return { lines, truncated };
}
