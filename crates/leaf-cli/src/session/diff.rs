use similar::{ChangeTag, TextDiff};

const MAX_NEW_FILE_LINES: usize = 100;

pub struct DiffPreview {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub truncated: bool,
    pub total_additions: usize,
    pub is_new_file: bool,
}

impl DiffPreview {
    pub fn new(path: &str, current: &str, proposed: &str) -> Self {
        let is_new_file = current.is_empty();
        let diff = TextDiff::from_lines(current, proposed);
        let mut additions = 0usize;
        let mut deletions = 0usize;

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Delete => deletions += 1,
                ChangeTag::Insert => additions += 1,
                ChangeTag::Equal => {}
            }
        }

        let truncated = is_new_file && additions > MAX_NEW_FILE_LINES;

        Self {
            path: path.to_string(),
            additions,
            deletions,
            truncated,
            total_additions: additions,
            is_new_file,
        }
    }

    pub fn render_inline(&self, current: &str, proposed: &str, line_offset: usize) {
        let diff = TextDiff::from_lines(current, proposed);
        let current_lines = current.lines().count().max(1);
        let proposed_lines = proposed.lines().count().max(1);
        let max_old = line_offset + current_lines;
        let max_new = line_offset + proposed_lines;
        let w = line_num_width(max_old.max(max_new));
        let mut old_line = line_offset;
        let mut new_line = line_offset;
        println!();
        for change in diff.iter_all_changes() {
            let content = change.value().trim_end_matches('\n');
            match change.tag() {
                ChangeTag::Delete => {
                    old_line += 1;
                    println!(
                        "\x1b[31m    {:>w$} {:>w$}| -{}\x1b[0m",
                        old_line,
                        "",
                        content,
                        w = w
                    );
                }
                ChangeTag::Insert => {
                    new_line += 1;
                    println!(
                        "\x1b[32m    {:>w$} {:>w$}| +{}\x1b[0m",
                        "",
                        new_line,
                        content,
                        w = w
                    );
                }
                ChangeTag::Equal => {
                    old_line += 1;
                    new_line += 1;
                    println!("    {:>w$} {:>w$}|  {}", old_line, new_line, content, w = w);
                }
            }
        }
        if self.truncated {
            println!(
                "\x1b[33m... {} more lines\x1b[0m",
                self.total_additions - MAX_NEW_FILE_LINES
            );
        }
    }

    pub fn summary(&self) -> String {
        if self.is_new_file {
            if self.truncated {
                format!(
                    "\n    New file: {} (+{} lines, {} total)",
                    self.path, self.additions, self.total_additions
                )
            } else {
                format!("\n    New file: {} (+{} lines)", self.path, self.additions)
            }
        } else if self.additions > 0 || self.deletions > 0 {
            format!(
                "\n    Edit: {} (+{} lines, -{} lines)",
                self.path, self.additions, self.deletions
            )
        } else {
            format!("\n    Edit: {} (no changes)", self.path)
        }
    }

    #[allow(clippy::string_slice)]
    pub fn find_line_offset(haystack: &str, needle: &str) -> usize {
        if needle.is_empty() {
            return 1;
        }
        haystack
            .find(needle)
            .map(|pos| haystack[..pos].lines().count())
            .unwrap_or(0)
    }
}

/// Calculate the width needed for line number display.
pub fn line_num_width(total_lines: usize) -> usize {
    if total_lines == 0 {
        return 1;
    }
    (total_lines as f64).log10().floor() as usize + 1
}

pub fn read_file_safe(path: &str) -> Option<(String, bool)> {
    match std::fs::read(path) {
        Ok(bytes) => {
            let is_binary = bytes.iter().take(8192).any(|&b| b == 0);
            if is_binary {
                Some((format!("Binary file, {} bytes", bytes.len()), true))
            } else {
                match String::from_utf8(bytes) {
                    Ok(content) => Some((content, false)),
                    Err(_) => Some((String::new(), false)),
                }
            }
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_preview_new_file() {
        let preview = DiffPreview::new("test.rs", "", "line1\nline2\n");
        assert!(preview.is_new_file);
        assert_eq!(preview.additions, 2);
        assert_eq!(preview.deletions, 0);
        assert!(!preview.truncated);
    }

    #[test]
    fn test_diff_preview_edit() {
        let preview = DiffPreview::new("test.rs", "old\n", "new\n");
        assert!(!preview.is_new_file);
        assert_eq!(preview.additions, 1);
        assert_eq!(preview.deletions, 1);
    }

    #[test]
    fn test_diff_preview_truncation() {
        let large_content: String = (0..150).map(|i| format!("line{}\n", i)).collect();
        let preview = DiffPreview::new("large.rs", "", &large_content);
        assert!(preview.truncated);
        assert_eq!(preview.total_additions, 150);
    }

    #[test]
    fn test_diff_preview_no_changes() {
        let preview = DiffPreview::new("same.rs", "content\n", "content\n");
        assert_eq!(preview.additions, 0);
        assert_eq!(preview.deletions, 0);
    }

    #[test]
    fn test_diff_counts() {
        let preview = DiffPreview::new(
            "test.rs",
            "line1\nline2\n",
            "line1\nline2 modified\nline3\n",
        );
        assert_eq!(preview.additions, 2);
        assert_eq!(preview.deletions, 1);
    }

    #[test]
    fn test_find_line_offset() {
        let file = "line1\nline2\nline3\nline4\n";
        assert_eq!(DiffPreview::find_line_offset(file, "line1"), 0);
        assert_eq!(DiffPreview::find_line_offset(file, "line2"), 1);
        assert_eq!(DiffPreview::find_line_offset(file, "line3"), 2);
        assert_eq!(DiffPreview::find_line_offset(file, "line4"), 3);
    }

    #[test]
    fn test_find_line_offset_empty_needle() {
        assert_eq!(DiffPreview::find_line_offset("content", ""), 1);
    }

    #[test]
    fn test_find_line_offset_not_found() {
        assert_eq!(DiffPreview::find_line_offset("content", "xyz"), 0);
    }
}
