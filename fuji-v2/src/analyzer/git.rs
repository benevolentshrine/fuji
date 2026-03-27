use std::collections::HashMap;
use crate::models::GitInfo;

/// Git analysis data
pub struct GitAnalysis {
    pub file_churn: HashMap<String, usize>,
    pub file_authors: HashMap<String, Vec<String>>,
    pub last_modified: HashMap<String, String>,
}

impl Default for GitAnalysis {
    fn default() -> Self {
        GitAnalysis {
            file_churn: HashMap::new(),
            file_authors: HashMap::new(),
            last_modified: HashMap::new(),
        }
    }
}

/// Analyze git history of a repository (offline, local only)
pub fn analyze_git(root_dir: &str) -> GitAnalysis {
    let mut result = GitAnalysis::default();

    let repo = match git2::Repository::discover(root_dir) {
        Ok(r) => r,
        Err(_) => return result, // Not a git repo
    };

    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return result,
    };

    let oid = match head.target() {
        Some(o) => o,
        None => return result,
    };

    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(_) => return result,
    };

    if revwalk.push(oid).is_err() {
        return result;
    }

    let max_commits = 100;
    let mut commit_count = 0;

    for oid_result in revwalk {
        if commit_count >= max_commits {
            break;
        }
        commit_count += 1;

        let oid = match oid_result {
            Ok(o) => o,
            Err(_) => continue,
        };

        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let tree = match commit.tree() {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Get parent tree for diff
        let parent_tree = commit.parent(0)
            .ok()
            .and_then(|p| p.tree().ok());

        let diff = match repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&tree),
            None,
        ) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let author_name = commit.author().name().unwrap_or("unknown").to_string();
        let time = commit.time();
        let timestamp = time.seconds();
        let date = format_timestamp(timestamp);

        // Iterate over changed files in this commit
        let _ = diff.foreach(
            &mut |delta, _progress| {
                if let Some(path) = delta.new_file().path() {
                    let rel_path = path.to_string_lossy().to_string();

                    // Churn
                    *result.file_churn.entry(rel_path.clone()).or_insert(0) += 1;

                    // Authors
                    let authors = result.file_authors.entry(rel_path.clone()).or_insert_with(Vec::new);
                    if !authors.contains(&author_name) {
                        authors.push(author_name.clone());
                    }

                    // Last modified
                    result.last_modified.entry(rel_path).or_insert_with(|| date.clone());
                }
                true
            },
            None, None, None,
        );
    }

    result
}

/// Apply git info to file results
pub fn apply_git_info(files: &[crate::models::FileResult], git_data: &GitAnalysis, root_dir: &str) {
    let root = std::path::Path::new(root_dir);

    for f in files {
        let file_path = std::path::Path::new(&f.path);
        let rel_path = file_path
            .strip_prefix(root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .replace('\\', "/"); // normalize for cross-platform

        let churn = git_data.file_churn.get(&rel_path).copied().unwrap_or(0);
        let authors = git_data.file_authors.get(&rel_path).cloned().unwrap_or_default();
        let last_mod = git_data.last_modified.get(&rel_path).cloned().unwrap_or_default();

        if churn > 0 || !authors.is_empty() {
            let last_author = authors.first().cloned().unwrap_or_default();
            // Note: we can't mutate through shared reference, this will be handled at the orchestrator level
            let _ = GitInfo {
                commit_count: churn,
                last_author,
                last_modified: last_mod,
                authors,
                commit_hash: None,
            };
        }
    }
}

fn format_timestamp(seconds: i64) -> String {
    // Simple date formatting without pulling in chrono
    let days_since_epoch = seconds / 86400;
    let mut y = 1970i32;
    let mut remaining_days = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }

    let months = if is_leap_year(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 1;
    for &days_in_month in &months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        m += 1;
    }
    let d = remaining_days + 1;

    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn is_leap_year(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
