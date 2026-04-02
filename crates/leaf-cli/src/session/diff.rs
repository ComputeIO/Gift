use similar::{Change, ChangeTag, TextDiff};

const MAX_NEW_FILE_LINES: usize = 100;

pub struct DiffPreview {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub lines: Vec<String>,
    pub truncated: bool,
    pub total_additions: usize,
}

impl DiffPreview {
    /// Create a diff preview for file operations
    pub fn new(path: &str, current: &str, proposed: &str) -> Self {
        Self::compute(path, current, proposed)
    }

    /// Core diff computation with conditional truncation
    fn compute(path: &str, current: &str, proposed: &str) -> Self {
        let is_new_file = current.is_empty();
        let diff = TextDiff::from_lines(current, proposed);
        let mut additions = 0usize;
        let mut deletions = 0usize;
        let mut lines = Vec::new();

        for change in diff.iter_all_changes() {
            let change: Change<&str> = change;
            match change.tag() {
                ChangeTag::Delete => {
                    deletions += 1;
                    let line = change.value().trim_end_matches('\n');
                    lines.push(format!("\x1b[31m-{}\x1b[0m", line));
                }
                ChangeTag::Insert => {
                    additions += 1;
                    let value = change.value();
                    let line = if value.ends_with('\n') {
                        format!("\x1b[32m{}\x1b[0m", value)
                    } else {
                        format!("\x1b[32m{}\n\x1b[0m", value)
                    };
                    lines.push(line);
                }
                ChangeTag::Equal => {
                    let line = change.value().trim_end_matches('\n');
                    lines.push(format!(" {}\n", line));
                }
            }
        }

        // Truncate only for new files
        let truncated = if is_new_file && additions > MAX_NEW_FILE_LINES {
            lines.truncate(MAX_NEW_FILE_LINES);
            lines.push(format!(
                "\x1b[33m... {} more lines\x1b[0m",
                additions - MAX_NEW_FILE_LINES
            ));
            true
        } else {
            false
        };

        Self {
            path: path.to_string(),
            additions,
            deletions,
            lines,
            truncated,
            total_additions: additions,
        }
    }

    /// Render the diff to stdout
    pub fn render(&self) {
        for line in &self.lines {
            print!("{}", line);
        }
    }

    /// Generate summary line
    pub fn summary(&self) -> String {
        if self.truncated {
            format!(
                "\n  {}: {} (+{} lines, {} total)",
                console::style("New file").cyan(),
                self.path,
                self.additions,
                self.total_additions
            )
        } else if self.additions > 0 || self.deletions > 0 {
            format!(
                "\n  {}: {} (+{} lines, -{} lines)",
                console::style("Edit").cyan(),
                self.path,
                self.additions,
                self.deletions
            )
        } else {
            format!(
                "\n  {}: {} (no changes)",
                console::style("Edit").cyan(),
                self.path
            )
        }
    }
}

/// Check if a file appears to be binary
pub fn is_binary(content: &[u8]) -> bool {
    content.iter().take(8192).any(|&b| b == 0)
}

/// Get file content safely, returning None for binary files
pub fn read_file_safe(path: &str) -> Option<(String, bool)> {
    match std::fs::read(path) {
        Ok(bytes) => {
            if is_binary(&bytes) {
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
