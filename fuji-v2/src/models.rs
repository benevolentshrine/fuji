//! Standardized Telemetry / SARIF-Compatible Models
//!
//! Implements NIST SP 800-92 / SP 800-137
//! All analyzer outputs conform to standardized structures
//! with the goal of SARIF (Static Analysis Results Interchange Format) compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// ═══ SARIF-COMPATIBLE CORE STRUCTURES ════════════════════════════
// ═══════════════════════════════════════════════════════════════

/// SARIF-compatible severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Severity {
    None,
    /// Info level (backward compatibility)
    Info,
    Note,
    Warning,
    Error,
    Critical,
}

impl Severity {
    /// SARIF level mapping
    pub fn to_sarif_level(&self) -> &'static str {
        match self {
            Severity::None => "none",
            Severity::Info => "note",
            Severity::Note => "note",
            Severity::Warning => "warning",
            Severity::Error => "error",
            Severity::Critical => "error", // SARIF uses error for critical
        }
    }

    /// Get label for display
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            Severity::None => "NONE",
            Severity::Info => "INFO",
            Severity::Note => "NOTE",
            Severity::Warning => "WARN",
            Severity::Error => "ERROR",
            Severity::Critical => "CRITICAL",
        }
    }

    /// Convert from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(Severity::None),
            "info" => Some(Severity::Info),
            "note" => Some(Severity::Note),
            "warning" | "warn" => Some(Severity::Warning),
            "error" => Some(Severity::Error),
            "critical" => Some(Severity::Critical),
            _ => None,
        }
    }
}

impl Default for Severity {
    fn default() -> Self {
        Severity::None
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// SARIF taxonomy category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Category {
    Security,
    Quality,
    AIPattern,
    Performance,
    Maintainability,
    Reliability,
    Complexity,
    Git,
}

impl Category {
    /// SARIF taxonomic category
    pub fn to_sarif_taxonomy(&self) -> &'static str {
        match self {
            Category::Security => "security",
            Category::Quality => "quality",
            Category::AIPattern => "ai-pattern",
            Category::Performance => "performance",
            Category::Maintainability => "maintainability",
            Category::Reliability => "reliability",
            Category::Complexity => "complexity",
            Category::Git => "git",
        }
    }
}

impl Default for Category {
    fn default() -> Self {
        Category::Quality
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Security => write!(f, "Security"),
            Category::Quality => write!(f, "Quality"),
            Category::AIPattern => write!(f, "AI Pattern"),
            Category::Performance => write!(f, "Performance"),
            Category::Maintainability => write!(f, "Maintainability"),
            Category::Reliability => write!(f, "Reliability"),
            Category::Complexity => write!(f, "Complexity"),
            Category::Git => write!(f, "Git"),
        }
    }
}

/// SARIF-compatible Issue (Result)
///
/// Represents a single finding with SARIF-compatible fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Line number (1-based per SARIF spec)
    pub line: usize,

    /// Column number (optional, 1-based)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,

    /// End line for multi-line issues
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,

    /// End column
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<usize>,

    /// SARIF rule ID / Issue type identifier
    #[serde(rename = "ruleId")]
    pub issue_type: String,

    /// SARIF severity level
    pub severity: Severity,

    /// Taxonomic category
    pub category: Category,

    /// Human-readable message
    pub message: String,

    /// Markdown-formatted message (SARIF)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "messageMarkdown")]
    pub message_markdown: Option<String>,

    /// Suggested fix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,

    /// Code snippet (SARIF physicalLocation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<CodeSnippet>,

    /// SARIF rule index reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_index: Option<usize>,

    /// Additional properties for telemetry
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl Default for Issue {
    fn default() -> Self {
        Self {
            line: 0,
            column: None,
            end_line: None,
            end_column: None,
            issue_type: String::new(),
            severity: Severity::default(),
            category: Category::default(),
            message: String::new(),
            message_markdown: None,
            fix: None,
            snippet: None,
            rule_index: None,
            properties: HashMap::new(),
        }
    }
}

/// SARIF-compatible code snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// Function information for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub complexity: usize,
    pub line_count: usize,

    /// SARIF: Function signature for context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// Git information (provenance tracking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub commit_count: usize,
    pub last_author: String,
    pub last_modified: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub authors: Vec<String>,

    /// SARIF: Provenance information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,
}

/// SARIF-compatible file result
///
/// Represents analysis results for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    /// Absolute file path (SARIF: artifactLocation)
    pub path: String,

    /// Programming language
    pub language: String,

    /// Total lines of code
    pub line_count: usize,

    /// AI-generated code probability (0-100)
    pub ai_probability: f64,

    /// Cyclomatic complexity
    pub complexity: usize,

    /// Functions in this file
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub functions: Vec<FunctionInfo>,

    /// Issues found in this file
    pub issues: Vec<Issue>,

    /// Git provenance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_info: Option<GitInfo>,

    /// Comment ratio (0-1)
    pub comment_ratio: f64,

    // Internal fields — not serialized
    #[serde(skip)]
    #[allow(dead_code)]
    pub name: String,

    #[serde(skip)]
    #[allow(dead_code)]
    pub is_directory: bool,

    #[serde(skip)]
    #[allow(dead_code)]
    pub depth: usize,

    /// File hash for deduplication (SARIF)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

impl FileResult {
    pub fn new_file(path: String, name: String, language: String, depth: usize) -> Self {
        Self {
            path,
            language,
            line_count: 0,
            ai_probability: 0.0,
            complexity: 0,
            functions: Vec::new(),
            issues: Vec::new(),
            git_info: None,
            comment_ratio: 0.0,
            name,
            is_directory: false,
            depth,
            content_hash: None,
        }
    }

    /// Count issues by severity
    pub fn severity_counts(&self) -> HashMap<Severity, usize> {
        let mut counts = HashMap::new();
        for issue in &self.issues {
            *counts.entry(issue.severity).or_insert(0) += 1;
        }
        counts
    }

    /// Get security issues only
    pub fn security_issues(&self) -> Vec<&Issue> {
        self.issues
            .iter()
            .filter(|i| i.category == Category::Security)
            .collect()
    }
}

/// SARIF-compatible analysis summary
///
/// Aggregated metrics across all analyzed files
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisSummary {
    /// Total files analyzed
    pub files_analyzed: usize,

    /// Files with issues
    pub files_flagged: usize,

    /// Average complexity
    pub avg_complexity: f64,

    /// Total issues found
    pub total_issues: usize,

    /// Files suspected of AI-generated code
    pub ai_suspected: usize,

    /// Security-specific issues
    pub security_issues: usize,

    /// Issues by severity (SARIF metrics)
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub severity_breakdown: HashMap<String, usize>,

    /// Analysis duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
}

/// Complete analysis result (SARIF-compatible)
///
/// Top-level structure matching SARIF Run object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Summary metrics
    pub summary: AnalysisSummary,

    /// Per-file results
    pub files: Vec<FileResult>,

    /// Root directory analyzed
    pub root_dir: String,

    /// SARIF: Tool information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_info: Option<ToolInfo>,

    /// SARIF: Invocation details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invocation: Option<InvocationInfo>,

    /// Timestamp of analysis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// SARIF Tool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub information_uri: String,
}

/// SARIF Invocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationInfo {
    pub command_line: String,
    pub execution_successful: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
}

/// Progress update for telemetry
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    /// Analysis phase
    #[allow(dead_code)]
    pub phase: String,

    /// Progress (0.0 - 1.0)
    pub progress: f64,

    /// Human-readable message
    pub message: String,

    /// Timestamp
    pub timestamp: std::time::Instant,
}

impl ProgressUpdate {
    pub fn new(phase: &str, progress: f64, message: &str) -> Self {
        Self {
            phase: phase.to_string(),
            progress,
            message: message.to_string(),
            timestamp: std::time::Instant::now(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// ═══ SARIF EXPORT HELPERS ═══════════════════════════════════════
// ═══════════════════════════════════════════════════════════════

impl AnalysisResult {
    /// Convert to SARIF format
    pub fn to_sarif(&self) -> SarifLog {
        let mut rules = Vec::new();
        let mut results = Vec::new();

        // Collect unique rules
        let mut rule_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

        for file in &self.files {
            for issue in &file.issues {
                if rule_ids.insert(issue.issue_type.clone()) {
                    rules.push(SarifRule {
                        id: issue.issue_type.clone(),
                        name: issue.issue_type.clone(),
                        short_description: issue.message.clone(),
                        full_description: None,
                        help_uri: None,
                        properties: None,
                    });
                }

                results.push(SarifResult {
                    rule_id: issue.issue_type.clone(),
                    level: issue.severity.to_sarif_level().to_string(),
                    message: SarifMessage {
                        text: issue.message.clone(),
                        markdown: issue.message_markdown.clone(),
                    },
                    locations: vec![SarifLocation {
                        physical_location: SarifPhysicalLocation {
                            artifact_location: SarifArtifactLocation {
                                uri: file.path.clone(),
                                uri_base_id: Some("PROJECT_ROOT".to_string()),
                            },
                            region: Some(SarifRegion {
                                start_line: issue.line as i64,
                                start_column: issue.column.map(|c| c as i64),
                                end_line: issue.end_line.map(|l| l as i64),
                                end_column: issue.end_column.map(|c| c as i64),
                            }),
                        },
                    }],
                });
            }
        }

        SarifLog {
            version: "2.1.0".to_string(),
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifToolComponent {
                        name: "fuji".to_string(),
                        version: Some("2.0.0".to_string()),
                        information_uri: Some("https://github.com/user/fuji".to_string()),
                        rules,
                    },
                },
                results,
            }],
        }
    }
}

/// SARIF Log root structure
#[derive(Debug, Serialize)]
pub struct SarifLog {
    pub version: String,
    pub runs: Vec<SarifRun>,
}

#[derive(Debug, Serialize)]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
}

#[derive(Debug, Serialize)]
pub struct SarifTool {
    pub driver: SarifToolComponent,
}

#[derive(Debug, Serialize)]
pub struct SarifToolComponent {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub information_uri: Option<String>,
    pub rules: Vec<SarifRule>,
}

#[derive(Debug, Serialize)]
pub struct SarifRule {
    pub id: String,
    pub name: String,
    pub short_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SarifResult {
    pub rule_id: String,
    pub level: String,
    pub message: SarifMessage,
    pub locations: Vec<SarifLocation>,
}

#[derive(Debug, Serialize)]
pub struct SarifMessage {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SarifLocation {
    pub physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Serialize)]
pub struct SarifPhysicalLocation {
    pub artifact_location: SarifArtifactLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<SarifRegion>,
}

#[derive(Debug, Serialize)]
pub struct SarifArtifactLocation {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri_base_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SarifRegion {
    pub start_line: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_column: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_from_str() {
        assert_eq!(Severity::from_str("error"), Some(Severity::Error));
        assert_eq!(Severity::from_str("CRITICAL"), Some(Severity::Critical));
        assert_eq!(Severity::from_str("unknown"), None);
    }

    #[test]
    fn test_sarif_conversion() {
        let result = AnalysisResult {
            summary: Default::default(),
            files: vec![],
            root_dir: "/test".to_string(),
            tool_info: None,
            invocation: None,
            timestamp: None,
        };

        let sarif = result.to_sarif();
        assert_eq!(sarif.version, "2.1.0");
    }
}
