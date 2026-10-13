//! Mini line-diff for reviewing snapshots.
//!
//! Trims the common prefix/suffix and runs LCS on the (bounded) middle part:
//! enough to review a scene, without heavy data structures.

/// One line of the comparison.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiffLine {
    /// Unchanged.
    Same(String),
    /// Only in the old text.
    Del(String),
    /// Only in the new text.
    Add(String),
}

/// Line limit for the middle LCS (beyond: single deleted/added block).
const LCS_LIMIT: usize = 400;

/// Compares two texts line by line.
pub fn diff_lines(old: &str, new: &str) -> Vec<DiffLine> {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    // Common prefix.
    let mut pre = 0;
    while pre < a.len() && pre < b.len() && a[pre] == b[pre] {
        pre += 1;
    }
    // Common suffix.
    let mut suf = 0;
    while suf < a.len() - pre && suf < b.len() - pre && a[a.len() - 1 - suf] == b[b.len() - 1 - suf]
    {
        suf += 1;
    }
    let mut out = Vec::new();
    for line in &a[..pre] {
        out.push(DiffLine::Same((*line).to_string()));
    }
    let mid_a = &a[pre..a.len() - suf];
    let mid_b = &b[pre..b.len() - suf];
    if mid_a.len() <= LCS_LIMIT && mid_b.len() <= LCS_LIMIT {
        out.extend(lcs(mid_a, mid_b));
    } else {
        for line in mid_a {
            out.push(DiffLine::Del((*line).to_string()));
        }
        for line in mid_b {
            out.push(DiffLine::Add((*line).to_string()));
        }
    }
    for line in &a[a.len() - suf..] {
        out.push(DiffLine::Same((*line).to_string()));
    }
    out
}

fn lcs(a: &[&str], b: &[&str]) -> Vec<DiffLine> {
    let n = a.len();
    let m = b.len();
    let width = m + 1;
    let mut dp = vec![0u32; (n + 1) * width];
    for i in 1..=n {
        for j in 1..=m {
            dp[i * width + j] = if a[i - 1] == b[j - 1] {
                dp[(i - 1) * width + j - 1] + 1
            } else {
                dp[(i - 1) * width + j].max(dp[i * width + j - 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (n, m);
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            out.push(DiffLine::Same(a[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if dp[(i - 1) * width + j] >= dp[i * width + j - 1] {
            out.push(DiffLine::Del(a[i - 1].to_string()));
            i -= 1;
        } else {
            out.push(DiffLine::Add(b[j - 1].to_string()));
            j -= 1;
        }
    }
    while i > 0 {
        out.push(DiffLine::Del(a[i - 1].to_string()));
        i -= 1;
    }
    while j > 0 {
        out.push(DiffLine::Add(b[j - 1].to_string()));
        j -= 1;
    }
    out.reverse();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_change() {
        let diff = diff_lines("a\nb\nc", "a\nB\nc");
        assert!(diff.contains(&DiffLine::Same("a".to_string())));
        assert!(diff.contains(&DiffLine::Del("b".to_string())));
        assert!(diff.contains(&DiffLine::Add("B".to_string())));
        assert!(diff.contains(&DiffLine::Same("c".to_string())));
    }

    #[test]
    fn identical_is_all_same() {
        let diff = diff_lines("a\nb", "a\nb");
        assert!(diff.iter().all(|d| matches!(d, DiffLine::Same(_))));
    }
}
