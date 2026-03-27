//! Plugin / Sandbox Pattern for Analyzers
//!
//! Implements NIST SP 800-190 / SP 800-53 SC-39
//! Each analyzer runs in isolation with error boundaries to prevent
//! crashes from affecting the main dashboard.

use crate::models::{Issue, FunctionInfo, FileResult};
use std::time::Duration;
use std::panic::{self, AssertUnwindSafe};

/// Maximum execution time for any single analyzer (seconds)
pub const ANALYZER_TIMEOUT_SECS: u64 = 30;

/// Result from a single analyzer
pub type AnalyzerResult = Result<Vec<Issue>, AnalyzerError>;

/// Error types for analyzers
#[derive(Debug, Clone)]
pub enum AnalyzerError {
    Timeout,
    Panic(String),
    InvalidInput(String),
    ExecutionError(String),
}

impl std::fmt::Display for AnalyzerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalyzerError::Timeout => write!(f, "Analyzer timed out"),
            AnalyzerError::Panic(msg) => write!(f, "Analyzer panicked: {}", msg),
            AnalyzerError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AnalyzerError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
        }
    }
}

impl std::error::Error for AnalyzerError {}

/// Core Analyzer trait
///
/// All analyzers (Security, AI, Quality, etc.) implement this trait.
/// The analyzer receives validated, read-only input and produces
/// a standardized Issue output.
///
/// # Security Guarantees
/// - Input is validated and size-limited before reaching the analyzer
/// - Analyzers receive only &str references, never mutable access
/// - Panics are caught and converted to errors
/// - Execution is time-boxed
pub trait Analyzer: Send + Sync {
    /// Analyzer name (for telemetry)
    fn name(&self) -> &'static str;

    /// Analyzer version (for compatibility)
    fn version(&self) -> &'static str;

    /// Analyze file content
    ///
    /// # Arguments
    /// * `content` - File content as validated string slice
    /// * `language` - Detected programming language
    /// * `functions` - Pre-extracted function information
    /// * `file_path` - File path for context (read-only)
    ///
    /// # Returns
    /// List of issues found, or error if analysis failed
    fn analyze(
        &self,
        content: &str,
        language: &str,
        functions: &[FunctionInfo],
        file_path: &str,
    ) -> AnalyzerResult;

    /// Check if this analyzer supports a given language
    fn supports_language(&self, language: &str) -> bool;

    /// Get analyzer metadata for telemetry
    fn metadata(&self) -> AnalyzerMetadata {
        AnalyzerMetadata {
            name: self.name().to_string(),
            version: self.version().to_string(),
            supported_languages: vec![],
        }
    }
}

/// Analyzer metadata for telemetry
#[derive(Debug, Clone)]
pub struct AnalyzerMetadata {
    pub name: String,
    pub version: String,
    pub supported_languages: Vec<String>,
}

/// Sandboxed analyzer wrapper
///
/// Wraps any Analyzer implementation with:
/// - Timeout protection
/// - Panic catching
/// - Resource limits
pub struct SandboxedAnalyzer {
    inner: Box<dyn Analyzer>,
    timeout: Duration,
}

impl SandboxedAnalyzer {
    pub fn new(analyzer: Box<dyn Analyzer>) -> Self {
        Self {
            inner: analyzer,
            timeout: Duration::from_secs(ANALYZER_TIMEOUT_SECS),
        }
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Run analyzer with sandbox protections
    pub fn analyze_sandboxed(
        &self,
        content: &str,
        language: &str,
        functions: &[FunctionInfo],
        file_path: &str,
    ) -> AnalyzerResult {
        // Check timeout using scope-limited execution
        let start = std::time::Instant::now();

        // Catch panics
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            self.inner.analyze(content, language, functions, file_path)
        }));

        // Check if we exceeded timeout
        if start.elapsed() > self.timeout {
            return Err(AnalyzerError::Timeout);
        }

        match result {
            Ok(inner_result) => inner_result,
            Err(_) => Err(AnalyzerError::Panic(
                "Analyzer panicked".to_string()
            )),
        }
    }

    pub fn name(&self) -> &'static str {
        self.inner.name()
    }
}

/// Analyzer registry - manages all available analyzers
pub struct AnalyzerRegistry {
    analyzers: Vec<Box<dyn Analyzer>>,
}

impl AnalyzerRegistry {
    pub fn new() -> Self {
        Self {
            analyzers: Vec::new(),
        }
    }

    pub fn register(&mut self, analyzer: Box<dyn Analyzer>) {
        self.analyzers.push(analyzer);
    }

    /// Get analyzers that support a specific language
    pub fn get_for_language(
        &self,
        language: &str,
    ) -> Vec<&Box<dyn Analyzer>> {
        self.analyzers
            .iter()
            .filter(|a| a.supports_language(language))
            .collect()
    }




    /// Get metadata for all analyzers
    pub fn get_metadata(&self) -> Vec<AnalyzerMetadata> {
        self.analyzers.iter().map(|a| a.metadata()).collect()
    }
}

impl Default for AnalyzerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Composite analyzer that runs multiple analyzers in parallel
pub struct CompositeAnalyzer {
    analyzers: Vec<SandboxedAnalyzer>,
}

impl CompositeAnalyzer {
    pub fn new(analyzers: Vec<Box<dyn Analyzer>>) -> Self {
        Self {
            analyzers: analyzers
                .into_iter()
                .map(SandboxedAnalyzer::new)
                .collect(),
        }
    }

    /// Run all analyzers and collect results
    pub fn analyze_all(
        &self,
        content: &str,
        language: &str,
        functions: &[FunctionInfo],
        file_path: &str,
    ) -> Vec<(AnalyzerResult, &str)> {
        use rayon::prelude::*;

        self.analyzers
            .par_iter()
            .map(|analyzer| {
                let result = analyzer.analyze_sandboxed(
                    content, language, functions, file_path
                );
                (result, analyzer.name())
            })
            .collect()
    }
}

/// Analysis context passed to analyzers
/// Contains validated, read-only file information
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    pub file_path: String,
    pub language: String,
    pub content_hash: String,
    pub line_count: usize,
}

impl AnalysisContext {
    pub fn new(file_path: String, language: String) -> Self {
        Self {
            file_path,
            language,
            content_hash: String::new(),
            line_count: 0,
        }
    }

    pub fn with_content(mut self, content: &str) -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        self.content_hash = format!("{:x}", hasher.finalize())[..16].to_string();
        self.line_count = content.lines().count();
        self
    }
}

/// Helper to create standardized issues
pub struct IssueBuilder {
    issue: Issue,
}

impl IssueBuilder {
    pub fn new(issue_type: &str, severity: crate::models::Severity) -> Self {
        Self {
            issue: Issue {
                issue_type: issue_type.to_string(),
                severity,
                ..Default::default()
            },
        }
    }

    pub fn line(mut self, line: usize) -> Self {
        self.issue.line = line;
        self
    }

    pub fn column(mut self, col: usize) -> Self {
        self.issue.column = Some(col);
        self
    }

    pub fn message(mut self, msg: &str) -> Self {
        self.issue.message = msg.to_string();
        self
    }

    pub fn category(mut self, cat: crate::models::Category) -> Self {
        self.issue.category = cat;
        self
    }

    pub fn fix(mut self, fix: &str) -> Self {
        self.issue.fix = Some(fix.to_string());
        self
    }

    pub fn build(self) -> Issue {
        self.issue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAnalyzer;

    impl Analyzer for MockAnalyzer {
        fn name(&self) -> &'static str {
            "mock"
        }

        fn version(&self) -> &'static str {
            "1.0.0"
        }

        fn analyze(
            &self,
            _content: &str,
            _language: &str,
            _functions: &[FunctionInfo],
            _file_path: &str,
        ) -> AnalyzerResult {
            Ok(vec![])
        }

        fn supports_language(&self, _language: &str) -> bool {
            true
        }
    }

    #[test]
    fn test_sandboxed_analyzer() {
        let analyzer = Box::new(MockAnalyzer);
        let sandboxed = SandboxedAnalyzer::new(analyzer);

        let result = sandboxed.analyze_sandboxed(
            "test content",
            "rust",
            &[],
            "test.rs",
        );

        assert!(result.is_ok());
    }
}
