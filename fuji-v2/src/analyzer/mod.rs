pub mod walker;
pub mod complexity;
pub mod ai;
pub mod security;
pub mod quality;
pub mod git;

use std::sync::mpsc;
use crate::models::*;
use rayon::prelude::*;

/// Analyzer orchestrates all analysis passes
pub struct Analyzer {
    pub root_dir: String,
    pub progress_tx: mpsc::Sender<ProgressUpdate>,
}

impl Analyzer {
    pub fn new(root_dir: String, progress_tx: mpsc::Sender<ProgressUpdate>) -> Self {
        let abs_dir = std::fs::canonicalize(&root_dir)
            .unwrap_or_else(|_| std::path::PathBuf::from(&root_dir))
            .to_string_lossy()
            .to_string();
        Analyzer {
            root_dir: abs_dir,
            progress_tx,
        }
    }

    /// Run executes the full analysis pipeline
    pub fn run(&self) -> Result<AnalysisResult, String> {
        // Phase 1: Walk directory
        self.send_progress("Parsing", 0.0, "Scanning directory tree...");
        let files = walker::walk_directory(&self.root_dir)
            .map_err(|e| format!("Walk error: {}", e))?;
        self.send_progress("Parsing", 1.0, &format!("Found {} files", files.len()));

        // Phase 2: Git analysis
        self.send_progress("Git", 0.0, "Analyzing git history...");
        let git_data = git::analyze_git(&self.root_dir);
        self.send_progress("Git", 1.0, "Git analysis complete");

        // Phase 3: Per-file analysis (parallel via rayon)
        self.send_progress("Patterns", 0.0, "Analyzing code patterns...");
        let _total = files.len();
        let analyzed: Vec<FileResult> = files
            .into_par_iter()
            .enumerate()
            .map(|(_idx, mut f)| {
                if f.language.is_empty() {
                    return f;
                }

                // Read file content
                let content = match std::fs::read_to_string(&f.path) {
                    Ok(c) => c,
                    Err(_) => return f,
                };

                // Check file size
                if content.len() > walker::MAX_FILE_SIZE {
                    return f;
                }

                // Actual line count
                f.line_count = content.lines().count();

                // Complexity analysis
                let (total_complexity, functions) =
                    complexity::analyze_complexity(&content, &f.language);
                f.complexity = total_complexity;
                f.functions = functions;

                // AI detection
                let (ai_score, ai_issues) =
                    ai::analyze_ai(&content, &f.language, &f.functions);
                f.ai_probability = ai_score;
                f.issues.extend(ai_issues);

                // Security scan
                let sec_issues = security::analyze_security(&content, &f.language);
                f.issues.extend(sec_issues);

                // Quality checks
                let qual_issues =
                    quality::analyze_quality(&content, &f.language, f.complexity, &f.functions);
                f.issues.extend(qual_issues);

                // Comment ratio
                f.comment_ratio = ai::comment_ratio(&content, &f.language);

                f
            })
            .collect();

        let files = analyzed;
        self.send_progress("Patterns", 1.0, "Pattern analysis complete");

        // Phase 4: Apply git info
        git::apply_git_info(&files, &git_data, &self.root_dir);

        // Build summary
        let summary = build_summary(&files);

        Ok(AnalysisResult {
            summary,
            files,
            root_dir: self.root_dir.clone(),
            tool_info: None,
            invocation: None,
            timestamp: None,
        })
    }

    fn send_progress(&self, phase: &str, progress: f64, message: &str) {
        let _ = self.progress_tx.send(ProgressUpdate {
            phase: phase.to_string(),
            progress,
            message: message.to_string(),
            timestamp: std::time::Instant::now(),
        });
    }
}

fn build_summary(files: &[FileResult]) -> AnalysisSummary {
    let mut summary = AnalysisSummary {
        files_analyzed: files.len(),
        ..Default::default()
    };

    let mut total_complexity: usize = 0;

    for f in files {
        total_complexity += f.complexity;

        if !f.issues.is_empty() {
            summary.files_flagged += 1;
        }
        summary.total_issues += f.issues.len();

        if f.ai_probability > 60.0 {
            summary.ai_suspected += 1;
        }

        for issue in &f.issues {
            if issue.category == Category::Security {
                summary.security_issues += 1;
            }
        }
    }

    if !files.is_empty() {
        summary.avg_complexity = total_complexity as f64 / files.len() as f64;
    }

    summary
}
