use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStatus {
    Unchanged,
    Added,
    Modified,
}

/// Per-line change state of the current buffer versus its committed (HEAD) version.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GitLineDiff {
    /// `status[i]` is the change state of current line `i`.
    pub status: Vec<LineStatus>,
    /// Indices of current lines that have one or more deleted lines immediately above them.
    pub deleted_before: HashSet<usize>,
}

#[derive(Debug, Clone, Copy)]
enum Op {
    Eq,
    Del,
    Ins,
}

/// Line-based LCS diff of `head` (committed) vs `current` (buffer) text.
pub fn diff_lines(head: &str, current: &str) -> GitLineDiff {
    let head_lines: Vec<&str> = head.lines().collect();
    let cur_lines: Vec<&str> = current.lines().collect();
    let n = head_lines.len();
    let m = cur_lines.len();

    // lcs[i][j] = length of LCS of head_lines[i..] and cur_lines[j..]
    let mut lcs = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if head_lines[i] == cur_lines[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }

    // Forward walk producing an op stream.
    let mut ops: Vec<Op> = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < n && j < m {
        if head_lines[i] == cur_lines[j] {
            ops.push(Op::Eq);
            i += 1;
            j += 1;
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            ops.push(Op::Del);
            i += 1;
        } else {
            ops.push(Op::Ins);
            j += 1;
        }
    }
    while i < n {
        ops.push(Op::Del);
        i += 1;
    }
    while j < m {
        ops.push(Op::Ins);
        j += 1;
    }

    // Group consecutive change hunks; classify inserts; record net deletions.
    let mut status = vec![LineStatus::Unchanged; m];
    let mut deleted_before: HashSet<usize> = HashSet::new();
    let mut cur_pos = 0usize;
    let mut k = 0;
    while k < ops.len() {
        match ops[k] {
            Op::Eq => {
                cur_pos += 1;
                k += 1;
            }
            _ => {
                let hunk_start = cur_pos;
                let mut dels = 0usize;
                let mut ins: Vec<usize> = Vec::new();
                loop {
                    match ops.get(k) {
                        Some(Op::Del) => {
                            dels += 1;
                            k += 1;
                        }
                        Some(Op::Ins) => {
                            ins.push(cur_pos);
                            cur_pos += 1;
                            k += 1;
                        }
                        _ => break,
                    }
                }
                for (rank, &idx) in ins.iter().enumerate() {
                    status[idx] = if rank < dels {
                        LineStatus::Modified
                    } else {
                        LineStatus::Added
                    };
                }
                if dels > ins.len() && m > 0 {
                    let anchor = (hunk_start + ins.len()).min(m - 1);
                    deleted_before.insert(anchor);
                }
            }
        }
    }

    GitLineDiff {
        status,
        deleted_before,
    }
}

/// Returns true if `working_dir` is inside a git work tree.
pub fn is_repo(working_dir: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(working_dir)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns the committed (HEAD) text of `file_path`, or `None` if not a repo,
/// no HEAD, or the file is untracked.
pub fn head_blob(working_dir: &Path, file_path: &Path) -> Option<String> {
    let root_out = Command::new("git")
        .arg("-C")
        .arg(working_dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !root_out.status.success() {
        return None;
    }
    let root = String::from_utf8(root_out.stdout).ok()?;
    let root_path = Path::new(root.trim());

    let abs = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        working_dir.join(file_path)
    };
    let rel = abs.strip_prefix(root_path).ok()?;
    let rel_str = rel.to_str()?;

    let show = Command::new("git")
        .arg("-C")
        .arg(root_path)
        .arg("show")
        .arg(format!("HEAD:{}", rel_str))
        .output()
        .ok()?;
    if !show.status.success() {
        return None;
    }
    String::from_utf8(show.stdout).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_is_all_unchanged() {
        let d = diff_lines("a\nb\nc\n", "a\nb\nc\n");
        assert_eq!(d.status, vec![LineStatus::Unchanged; 3]);
        assert!(d.deleted_before.is_empty());
    }

    #[test]
    fn test_added_line_in_middle() {
        let d = diff_lines("a\nc\n", "a\nb\nc\n");
        assert_eq!(d.status[0], LineStatus::Unchanged);
        assert_eq!(d.status[1], LineStatus::Added);
        assert_eq!(d.status[2], LineStatus::Unchanged);
    }

    #[test]
    fn test_modified_line() {
        let d = diff_lines("a\nb\nc\n", "a\nB\nc\n");
        assert_eq!(d.status[1], LineStatus::Modified);
    }

    #[test]
    fn test_removed_line_marks_following() {
        let d = diff_lines("a\nb\nc\n", "a\nc\n");
        assert_eq!(d.status, vec![LineStatus::Unchanged; 2]);
        assert!(d.deleted_before.contains(&1));
    }

    #[test]
    fn test_added_at_end() {
        let d = diff_lines("a\n", "a\nb\n");
        assert_eq!(d.status[1], LineStatus::Added);
    }

    #[test]
    fn test_removed_at_end_marks_last_line() {
        let d = diff_lines("a\nb\n", "a\n");
        assert!(d.deleted_before.contains(&0));
    }
}
