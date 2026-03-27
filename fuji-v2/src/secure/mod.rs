//! Secure Filesystem Controller
//!
//! Implements Zero-Trust & Least Privilege (NIST SP 800-207 / SP 800-53 AC-6)
//! The GUI never has direct filesystem access. All filesystem operations go through
//! this validated controller with strict path sanitization and access control.

pub mod analyzer_trait;

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Maximum allowed file size (10MB) to prevent DoS
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum path length
pub const MAX_PATH_LEN: usize = 4096;

/// Allowed file extensions for analysis
pub const ALLOWED_EXTENSIONS: &[&str] = &[
    ".rs", ".py", ".js", ".ts", ".go", ".c", ".cpp", ".h", ".hpp",
    ".java", ".kt", ".scala", ".rb", ".php", ".cs", ".swift",
    ".md", ".txt", ".json", ".yaml", ".yml", ".toml", ".xml",
];

/// Filesystem access error types
#[derive(Debug, Clone, PartialEq)]
pub enum FsError {
    PathTooLong,
    InvalidPath,
    PathTraversalAttempt,
    NotInScope,
    FileTooLarge,
    InvalidExtension,
    IoError(String),
    PermissionDenied,
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsError::PathTooLong => write!(f, "Path exceeds maximum length"),
            FsError::InvalidPath => write!(f, "Invalid path format"),
            FsError::PathTraversalAttempt => write!(f, "Path traversal attempt detected"),
            FsError::NotInScope => write!(f, "Path not within allowed scope"),
            FsError::FileTooLarge => write!(f, "File exceeds size limit"),
            FsError::InvalidExtension => write!(f, "File extension not allowed"),
            FsError::IoError(e) => write!(f, "IO error: {}", e),
            FsError::PermissionDenied => write!(f, "Permission denied"),
        }
    }
}

impl std::error::Error for FsError {}

/// Validated path wrapper - ensures path passed security checks
#[derive(Debug, Clone)]
pub struct ValidatedPath {
    path: PathBuf,
    canonical: PathBuf,
}

impl ValidatedPath {
    pub fn as_path(&self) -> &Path {
        &self.path
    }

    pub fn as_canonical(&self) -> &Path {
        &self.canonical
    }

    pub fn to_string_lossy(&self) -> std::borrow::Cow<str> {
        self.path.to_string_lossy()
    }
}

/// Filesystem Controller - Zero-trust filesystem access
///
/// All filesystem access goes through this controller which validates:
/// - Path is within allowed scope (no directory traversal)
/// - Path length limits
/// - File size limits
/// - Allowed extensions only
pub struct FsController {
    /// Base directory that all operations must be within
    scope: PathBuf,
    /// Whether to allow file content reading
    allow_read: bool,
    /// Maximum file size allowed
    max_file_size: usize,
}

impl FsController {
    /// Create a new filesystem controller scoped to a directory
    pub fn new(scope: PathBuf) -> Self {
        let canonical_scope = std::fs::canonicalize(&scope).unwrap_or(scope);
        Self {
            scope: canonical_scope,
            allow_read: true,
            max_file_size: MAX_FILE_SIZE,
        }
    }

    /// Disable file reading (metadata only)
    pub fn read_only_meta(mut self) -> Self {
        self.allow_read = false;
        self
    }

    /// Set custom max file size
    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Validate and sanitize a path
    ///
    /// Returns a ValidatedPath only if:
    /// - Path is within the scope directory
    /// - No path traversal sequences
    /// - Path length is reasonable
    pub fn validate_path(&self, path: &str,
) -> Result<ValidatedPath, FsError> {
        // Check path length
        if path.len() > MAX_PATH_LEN {
            return Err(FsError::PathTooLong);
        }

        // Parse path
        let path = Path::new(path);

        // Reject absolute paths with different root
        if path.is_absolute() {
            // Will be checked against scope later
        }

        // Check for path traversal attempts
        let path_str = path.to_string_lossy();
        if path_str.contains("..") || path_str.contains("//") {
            return Err(FsError::PathTraversalAttempt);
        }

        // Canonicalize to resolve symlinks and get absolute path
        let canonical = std::fs::canonicalize(path)
            .map_err(|e| FsError::IoError(e.to_string()))?;

        // Ensure path is within scope
        if !canonical.starts_with(&self.scope) {
            return Err(FsError::NotInScope);
        }

        Ok(ValidatedPath {
            path: path.to_path_buf(),
            canonical,
        })
    }

    /// Read directory contents (validated)
    pub fn read_dir(&self, path: &ValidatedPath) -> Result<Vec<DirEntry>, FsError> {
        let entries = std::fs::read_dir(path.as_canonical())
            .map_err(|e| FsError::IoError(e.to_string()))?;

        let mut result = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| FsError::IoError(e.to_string()))?;
            let path = entry.path();

            // Skip if not in scope
            if !path.starts_with(&self.scope) {
                continue;
            }

            // Check extension
            if let Some(ext) = path.extension() {
                let ext_str = format!(".{}", ext.to_string_lossy());
                if !ALLOWED_EXTENSIONS.contains(&ext_str.as_str()) {
                    continue;
                }
            }

            result.push(DirEntry {
                path,
                metadata: entry.metadata().ok(),
            });
        }

        Ok(result)
    }

    /// Read file content (validated)
    pub fn read_file(&self, path: &ValidatedPath,
) -> Result<String, FsError> {
        if !self.allow_read {
            return Err(FsError::PermissionDenied);
        }

        let canonical = path.as_canonical();

        // Check file size before reading
        let metadata = std::fs::metadata(canonical)
            .map_err(|e| FsError::IoError(e.to_string()))?;

        if metadata.len() > self.max_file_size as u64 {
            return Err(FsError::FileTooLarge);
        }

        // Validate extension
        if let Some(ext) = canonical.extension() {
            let ext_str = format!(".{}", ext.to_string_lossy());
            if !ALLOWED_EXTENSIONS.contains(&ext_str.as_str()) {
                return Err(FsError::InvalidExtension);
            }
        }

        // Read content
        std::fs::read_to_string(canonical)
            .map_err(|e| FsError::IoError(e.to_string()))
    }

    /// Check if path is a valid file within scope
    pub fn is_valid_file(&self, path: &ValidatedPath) -> bool {
        let canonical = path.as_canonical();

        if !canonical.exists() {
            return false;
        }

        if !canonical.is_file() {
            return false;
        }

        // Check extension
        if let Some(ext) = canonical.extension() {
            let ext_str = format!(".{}", ext.to_string_lossy());
            return ALLOWED_EXTENSIONS.contains(&ext_str.as_str());
        }

        false
    }

    /// Get scope directory
    pub fn scope(&self) -> &Path {
        &self.scope
    }
}

/// Directory entry with metadata
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    pub metadata: Option<std::fs::Metadata>,
}

/// Configuration for analysis request
/// GUI passes this to the secure controller
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Target path (must be validated)
    pub target_path: String,
    /// Analysis modes to run
    pub modes: Vec<AnalysisMode>,
    /// Maximum file size to scan
    pub max_file_size: usize,
    /// Exclude patterns
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnalysisMode {
    Security,
    AI,
    Quality,
    All,
}

/// Secure analysis request builder
///
/// GUI uses this to construct a validated analysis request
/// without directly accessing the filesystem
pub struct AnalysisRequestBuilder {
    config: AnalysisConfig,
}

impl AnalysisRequestBuilder {
    pub fn new(target_path: String) -> Self {
        Self {
            config: AnalysisConfig {
                target_path,
                modes: vec![AnalysisMode::All],
                max_file_size: MAX_FILE_SIZE,
                exclude_patterns: vec![
                    ".git".to_string(),
                    "node_modules".to_string(),
                    "target".to_string(),
                    "__pycache__".to_string(),
                ],
            },
        }
    }

    pub fn with_modes(mut self, modes: Vec<AnalysisMode>) -> Self {
        self.config.modes = modes;
        self
    }

    pub fn with_max_size(mut self, size: usize) -> Self {
        self.config.max_file_size = size;
        self
    }

    pub fn with_excludes(mut self, patterns: Vec<String>) -> Self {
        self.config.exclude_patterns = patterns;
        self
    }

    /// Build and validate the request
    pub fn build(self) -> Result<AnalysisConfig, FsError> {
        // Validate target path exists and is accessible
        let path = Path::new(&self.config.target_path);

        if !path.exists() {
            return Err(FsError::InvalidPath);
        }

        // Additional validation could go here

        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_traversal_detection() {
        let controller = FsController::new(PathBuf::from("/allowed"));

        assert!(matches!(
            controller.validate_path("/../etc/passwd"),
            Err(FsError::PathTraversalAttempt)
        ));

        assert!(matches!(
            controller.validate_path("/allowed/../../etc/passwd"),
            Err(FsError::NotInScope)
        ));
    }

    #[test]
    fn test_path_too_long() {
        let controller = FsController::new(PathBuf::from("/allowed"));
        let long_path = "a".repeat(MAX_PATH_LEN + 1);

        assert!(matches!(
            controller.validate_path(&long_path),
            Err(FsError::PathTooLong)
        ));
    }
}
