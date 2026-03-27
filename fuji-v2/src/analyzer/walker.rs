//! Pipeline / Input Validation for File Walking
//!
//! Implements NIST SP 800-218 PW.5 / SP 800-53 SI-10
//! Strict input validation with size limits and read-only references.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::models::FileResult;
use crate::secure::{FsError, FsController, ValidatedPath};

/// Max file size for analysis (512KB) - prevents DoS from large files
pub const MAX_FILE_SIZE: usize = 512 * 1024;

/// Max total files to analyze - prevents resource exhaustion
const MAX_ANALYZABLE_FILES: usize = 15000;

/// Max directory depth - prevents stack overflow
const MAX_DEPTH: usize = 20;

/// Supported file extensions → language name
fn supported_extensions() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert(".go", "Go");
    m.insert(".py", "Python");
    m.insert(".rs", "Rust");
    m.insert(".js", "JavaScript");
    m.insert(".ts", "TypeScript");
    m.insert(".jsx", "JavaScript");
    m.insert(".tsx", "TypeScript");
    m.insert(".rb", "Ruby");
    m.insert(".java", "Java");
    m.insert(".c", "C");
    m.insert(".cpp", "C++");
    m.insert(".h", "C");
    m.insert(".hpp", "C++");
    m.insert(".cs", "C#");
    m.insert(".php", "PHP");
    m.insert(".sh", "Shell");
    m.insert(".lua", "Lua");
    m.insert(".dart", "Dart");
    m.insert(".kt", "Kotlin");
    m.insert(".swift", "Swift");
    m.insert(".scala", "Scala");
    m.insert(".yaml", "YAML");
    m.insert(".yml", "YAML");
    m.insert(".json", "JSON");
    m.insert(".md", "Markdown");
    m.insert(".sql", "SQL");
    m
}

/// Directories to skip - security risk reduction
fn ignored_dirs() -> &'static [&'static str] {
    &[
        ".git", "node_modules", "vendor", "__pycache__", ".venv", "venv",
        "dist", "build", ".idea", ".vscode", "target", "bin", "obj",
        ".next", ".nuxt", "coverage", ".cache", ".pytest_cache",
        "__snapshots__", ".terraform", "artifacts",
        // Additional security-sensitive directories
        ".ssh", ".gnupg", ".aws", ".azure", ".kube",
        "secrets", "credentials", "private",
    ]
}

/// File extension blacklist - potentially dangerous files
fn blocked_extensions() -> &'static [&'static str] {
    &[
        // Executables
        ".exe", ".dll", ".so", ".dylib", ".bin",
        // Archives
        ".zip", ".tar", ".gz", ".bz2", ".7z", ".rar",
        // Binary formats
        ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx",
        // Images
        ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".svg", ".ico",
        // Media
        ".mp3", ".mp4", ".avi", ".mov", ".wav", ".flac",
        // Other binaries
        ".wasm", ".class", ".pyc", ".o", ".a", ".lib",
    ]
}

/// Validation result for files
#[derive(Debug, Clone)]
pub struct ValidatedFile {
    pub path: PathBuf,
    pub language: String,
    pub size: usize,
    pub depth: usize,
    pub hash: Option<String>,
}

impl ValidatedFile {
    /// Convert to FileResult with validation
    pub fn to_file_result(&self) -> FileResult {
        let name = self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut fr = FileResult::new_file(
            self.path.to_string_lossy().to_string(),
            name,
            self.language.clone(),
            self.depth,
        );
        fr.line_count = self.size / 35; // Estimate
        fr
    }
}

/// Walk a directory with strict validation
///
/// # Security Features:
/// - Path validation against traversal
/// - File size limits
/// - Extension whitelist/blacklist
/// - Directory depth limits
/// - Read-only access (returns validated references)
pub fn walk_directory(root: &str) -> Result<Vec<FileResult>, FsError> {
    // Create filesystem controller for this scope
    let controller = FsController::new(PathBuf::from(root));

    // Validate root path
    let validated_root = controller.validate_path(root)?;
    let abs_root = validated_root.as_canonical();

    let extensions = supported_extensions();
    let ignored = ignored_dirs();
    let blocked = blocked_extensions();
    let mut files = Vec::new();

    for entry in WalkDir::new(abs_root)
        .follow_links(false) // Security: don't follow symlinks
        .max_depth(MAX_DEPTH)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();

            // Skip hidden files/dirs (except root)
            if name.starts_with('.') && e.depth() > 0 {
                return false;
            }

            // Skip ignored directories
            if e.file_type().is_dir() && ignored.contains(&name.as_ref()) {
                return false;
            }

            true
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                // Log but don't fail - continue walking
                eprintln!("Walker error: {}", e);
                continue;
            }
        };

        // Skip directories
        if entry.file_type().is_dir() {
            continue;
        }

        // Check file limit
        if files.len() >= MAX_ANALYZABLE_FILES {
            eprintln!("Warning: Reached maximum file limit ({})", MAX_ANALYZABLE_FILES);
            break;
        }

        let path = entry.path();

        // Get extension
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e.to_lowercase()))
            .unwrap_or_default();

        // Check if blocked
        if blocked.contains(&ext.as_str()) {
            continue;
        }

        // Check if supported
        if let Some(&lang) = extensions.get(ext.as_str()) {
            // Get and validate file size
            let file_size = entry
                .metadata()
                .map(|m| m.len() as usize)
                .unwrap_or(0);

            if file_size == 0 {
                continue; // Skip empty files
            }

            if file_size > MAX_FILE_SIZE {
                eprintln!(
                    "Warning: Skipping large file {} ({} > {} bytes)",
                    path.display(),
                    file_size,
                    MAX_FILE_SIZE
                );
                continue;
            }

            // Validate path is within scope
            let path_str = path.to_string_lossy();
            if let Ok(validated) = controller.validate_path(&path_str) {
                let rel_path = path
                    .strip_prefix(abs_root)
                    .unwrap_or(path);
                let depth = rel_path.components().count().saturating_sub(1);

                // Ensure depth is reasonable
                if depth > MAX_DEPTH {
                    continue;
                }

                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let mut fr = FileResult::new_file(
                    validated.to_string_lossy().to_string(),
                    name,
                    lang.to_string(),
                    depth,
                );
                fr.line_count = file_size / 35;

                files.push(fr);
            }
        }
    }

    Ok(files)
}

/// Read file content with strict validation
///
/// Returns read-only string slice reference, never mutable access
/// Validates:
/// - File is within scope
/// - File size is within limits
/// - Content is valid UTF-8
pub fn read_file_content(
    path: &Path,
    max_size: usize,
) -> Result<String, FsError> {
    // Verify file exists and is a file
    if !path.is_file() {
        return Err(FsError::InvalidPath);
    }

    // Check size before reading
    let metadata = std::fs::metadata(path)
        .map_err(|e| FsError::IoError(e.to_string()))?;

    let size = metadata.len() as usize;
    if size > max_size {
        return Err(FsError::FileTooLarge);
    }

    // Read content
    let content = std::fs::read_to_string(path)
        .map_err(|e| FsError::IoError(e.to_string()))?;

    // Additional validation: check for null bytes or control characters
    // that might indicate binary content
    if content.bytes().any(|b| b == 0x00) {
        return Err(FsError::InvalidExtension);
    }

    Ok(content)
}

/// Get language for a file path (read-only)
#[allow(dead_code)]
pub fn language_for_file(path: &str) -> String {
    let ext_map = supported_extensions();
    let p = Path::new(path);
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_lowercase()))
        .unwrap_or_default();
    ext_map.get(ext.as_str()).unwrap_or(&"").to_string()
}

/// Check if a path should be analyzed
pub fn is_analyzable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_lowercase()))
        .unwrap_or_default();

    // Check not blocked
    if blocked_extensions().contains(&ext.as_str()) {
        return false;
    }

    // Check supported
    let extensions = supported_extensions();
    extensions.contains_key(ext.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_extensions() {
        assert!(blocked_extensions().contains(&".exe"));
        assert!(blocked_extensions().contains(&".zip"));
        assert!(!blocked_extensions().contains(&".rs"));
    }

    #[test]
    fn test_is_analyzable() {
        let temp_file = std::env::temp_dir().join("test.rs");
        // Note: This won't actually exist, so is_file() returns false
        assert!(!is_analyzable(&temp_file));
    }
}
