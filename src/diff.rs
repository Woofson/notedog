#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLine {
    Unchanged(String),
    Added(String),
    Deleted(String),
}

pub fn compute_line_diff(old_text: &str, new_text: &str) -> Vec<DiffLine> {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let n = old_lines.len();
    let m = new_lines.len();

    let mut dp = vec![vec![0; m + 1]; n + 1];

    for i in 0..n {
        for j in 0..m {
            if old_lines[i] == new_lines[j] {
                dp[i + 1][j + 1] = dp[i][j] + 1;
            } else {
                dp[i + 1][j + 1] = dp[i + 1][j].max(dp[i][j + 1]);
            }
        }
    }

    let mut diff = Vec::new();
    let mut i = n;
    let mut j = m;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
            diff.push(DiffLine::Unchanged(old_lines[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            diff.push(DiffLine::Added(new_lines[j - 1].to_string()));
            j -= 1;
        } else if i > 0 && (j == 0 || dp[i][j - 1] < dp[i - 1][j]) {
            diff.push(DiffLine::Deleted(old_lines[i - 1].to_string()));
            i -= 1;
        }
    }

    diff.reverse();
    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_line_diff() {
        let old_text = "Line 1\nLine 2\nLine 3";
        let new_text = "Line 1\nLine 2 Modified\nLine 3\nLine 4 Added";

        let diff = compute_line_diff(old_text, new_text);

        assert!(diff.contains(&DiffLine::Unchanged("Line 1".to_string())));
        assert!(diff.contains(&DiffLine::Deleted("Line 2".to_string())));
        assert!(diff.contains(&DiffLine::Added("Line 2 Modified".to_string())));
        assert!(diff.contains(&DiffLine::Added("Line 4 Added".to_string())));
    }
}
