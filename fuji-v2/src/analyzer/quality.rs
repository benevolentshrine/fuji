use regex::Regex;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use crate::models::{Issue, Severity, Category, FunctionInfo};

// Quality thresholds
const MAX_FUNCTION_LENGTH: usize = 50;
const MAX_FILE_LENGTH: usize = 500;
const COMPLEXITY_WARNING: usize = 10;
const COMPLEXITY_CRITICAL: usize = 20;
const DUP_MIN_LINES: usize = 6;
const MAX_NESTING_DEPTH: usize = 4;
const MAX_PARAM_COUNT: usize = 5;
const GOD_FUNC_LINES: usize = 100;
const GOD_FUNC_COMPLEXITY: usize = 15;

pub fn analyze_quality(content: &str, lang: &str, _complexity: usize, functions: &[FunctionInfo]) -> Vec<Issue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Complexity warnings
    for f in functions {
        if f.complexity >= COMPLEXITY_CRITICAL {
            issues.push(Issue {
                line: f.start_line, column: None,
                issue_type: "high_complexity".to_string(),
                severity: Severity::Error, category: Category::Quality,
                message: format!("Function '{}' has cyclomatic complexity {} (critical threshold: {})", f.name, f.complexity, COMPLEXITY_CRITICAL),
                fix: Some("Break this function into smaller, focused functions. Extract conditional logic into helper functions.".to_string()),
                ..Default::default()
            });
        } else if f.complexity >= COMPLEXITY_WARNING {
            issues.push(Issue {
                line: f.start_line, column: None,
                issue_type: "moderate_complexity".to_string(),
                severity: Severity::Warning, category: Category::Quality,
                message: format!("Function '{}' has cyclomatic complexity {} (warning threshold: {})", f.name, f.complexity, COMPLEXITY_WARNING),
                fix: Some("Consider refactoring — extract branches into helper functions or use early returns.".to_string()),
                ..Default::default()
            });
        }
    }

    // Long function / god function warnings
    for f in functions {
        if f.line_count > GOD_FUNC_LINES && f.complexity > GOD_FUNC_COMPLEXITY {
            issues.push(Issue {
                line: f.start_line, column: None,
                issue_type: "god_function".to_string(),
                severity: Severity::Error, category: Category::Quality,
                message: format!("God function '{}' — {} lines with complexity {}", f.name, f.line_count, f.complexity),
                fix: Some("This function does too much. Split into focused functions with single responsibilities.".to_string()),
                ..Default::default()
            });
        } else if f.line_count > MAX_FUNCTION_LENGTH {
            issues.push(Issue {
                line: f.start_line, column: None,
                issue_type: "long_function".to_string(),
                severity: Severity::Warning, category: Category::Quality,
                message: format!("Function '{}' is {} lines (recommended max: {})", f.name, f.line_count, MAX_FUNCTION_LENGTH),
                fix: Some("Break into smaller functions. Functions should ideally fit on one screen.".to_string()),
                ..Default::default()
            });
        }
    }

    // Long file
    if lines.len() > MAX_FILE_LENGTH {
        issues.push(Issue {
            line: 1, column: None,
            issue_type: "long_file".to_string(),
            severity: Severity::Info, category: Category::Quality,
            message: format!("File is {} lines (recommended max: {})", lines.len(), MAX_FILE_LENGTH),
            fix: Some("Split into multiple files by responsibility.".to_string()),
            ..Default::default()
        });
    }

    // Deep nesting
    issues.extend(check_deep_nesting(&lines));

    // Long parameter lists
    issues.extend(check_long_params(&lines, functions));

    // Dead code
    issues.extend(check_dead_code(&lines));

    // Empty error handlers
    issues.extend(check_empty_error_handlers(&lines, lang));

    // Error swallowing (Go-specific)
    if lang == "Go" {
        issues.extend(check_error_swallowing(&lines));
    }

    // Magic numbers
    issues.extend(check_magic_numbers(&lines));

    // TODO/FIXME markers
    issues.extend(check_todo_markers(&lines));

    // Duplicate code blocks
    issues.extend(check_duplication(&lines));

    issues
}

fn check_deep_nesting(lines: &[&str]) -> Vec<Issue> {
    let mut issues = Vec::new();
    let mut reported: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (i, line) in lines.iter().enumerate() {
        let mut indent = 0;
        for ch in line.chars() {
            if ch == '\t' { indent += 4; }
            else if ch == ' ' { indent += 1; }
            else { break; }
        }
        let nest_level = indent / 4;
        if nest_level > MAX_NESTING_DEPTH && !reported.contains(&nest_level) {
            reported.insert(nest_level);
            issues.push(Issue {
                line: i + 1, column: None,
                issue_type: "deep_nesting".to_string(),
                severity: Severity::Warning, category: Category::Quality,
                message: format!("Code nested {} levels deep (max recommended: {})", nest_level, MAX_NESTING_DEPTH),
                fix: Some("Use early returns (guard clauses) to reduce nesting.".to_string()),
                ..Default::default()
            });
        }
    }
    issues
}

fn check_long_params(lines: &[&str], functions: &[FunctionInfo]) -> Vec<Issue> {
    let mut issues = Vec::new();
    let param_re = Regex::new(r"\(([^)]+)\)").unwrap();

    for f in functions {
        if f.start_line == 0 || f.start_line > lines.len() { continue; }
        let func_line = lines[f.start_line - 1];
        if let Some(caps) = param_re.captures(func_line) {
            if let Some(params_str) = caps.get(1) {
                let count = params_str.as_str().split(',')
                    .filter(|p| !p.trim().is_empty())
                    .count();
                if count > MAX_PARAM_COUNT {
                    issues.push(Issue {
                        line: f.start_line, column: None,
                        issue_type: "long_param_list".to_string(),
                        severity: Severity::Warning, category: Category::Quality,
                        message: format!("Function '{}' has {} parameters (max recommended: {})", f.name, count, MAX_PARAM_COUNT),
                        fix: Some("Group related parameters into a struct/object.".to_string()),
                    ..Default::default()
                    });
                }
            }
        }
    }
    issues
}

fn check_dead_code(lines: &[&str]) -> Vec<Issue> {
    let mut issues = Vec::new();
    let unreachable_re = Regex::new(r"^\s*(return\b|break\b|continue\b|panic\(|os\.Exit\(|sys\.exit\(|process\.exit\(|throw\b)").unwrap();
    let close_brace_re = Regex::new(r"^\s*[})\]]?\s*$").unwrap();

    for i in 0..lines.len().saturating_sub(1) {
        let trimmed = lines[i].trim();
        if !unreachable_re.is_match(trimmed) { continue; }

        for j in (i + 1)..lines.len() {
            let next_trimmed = lines[j].trim();
            if next_trimmed.is_empty() { continue; }
            if close_brace_re.is_match(next_trimmed) { break; }
            if next_trimmed.starts_with("case ") || next_trimmed.starts_with("default:") { break; }
            issues.push(Issue {
                line: j + 1, column: None,
                issue_type: "dead_code".to_string(),
                severity: Severity::Warning, category: Category::Quality,
                message: format!("Unreachable code after {}", lines[i].trim()),
                fix: Some("Remove this unreachable code or restructure the control flow.".to_string()),
                ..Default::default()
            });
            break;
        }
    }
    issues
}

fn check_empty_error_handlers(lines: &[&str], lang: &str) -> Vec<Issue> {
    let mut issues = Vec::new();

    for i in 0..lines.len().saturating_sub(1) {
        let trimmed = lines[i].trim();
        let mut is_empty = false;
        let mut handler_type = "";

        if trimmed.contains("catch") && trimmed.ends_with('{') {
            handler_type = "catch block";
            is_empty = is_block_empty(lines, i + 1);
        } else if trimmed.contains("except") && trimmed.ends_with(':') {
            handler_type = "except block";
            is_empty = is_python_block_empty(lines, i + 1);
        } else if lang == "Go" && trimmed.contains("if err != nil") && trimmed.ends_with('{') {
            handler_type = "error handler";
            is_empty = is_block_empty(lines, i + 1);
        }

        if is_empty && !handler_type.is_empty() {
            issues.push(Issue {
                line: i + 1, column: None,
                issue_type: "empty_error_handler".to_string(),
                severity: Severity::Error, category: Category::Quality,
                message: format!("Empty {} — errors silently swallowed", handler_type),
                fix: Some("At minimum, log the error. Consider returning it or wrapping with context.".to_string()),
                ..Default::default()
            });
        }
    }
    issues
}

fn is_block_empty(lines: &[&str], start: usize) -> bool {
    if start >= lines.len() { return false; }
    for i in start..lines.len() {
        let trimmed = lines[i].trim();
        if trimmed == "}" { return true; }
        if !trimmed.is_empty() { return false; }
    }
    false
}

fn is_python_block_empty(lines: &[&str], start: usize) -> bool {
    if start >= lines.len() { return false; }
    let trimmed = lines[start].trim();
    trimmed == "pass" || trimmed.is_empty()
}

fn check_error_swallowing(lines: &[&str]) -> Vec<Issue> {
    let mut issues = Vec::new();
    let swallow_re = Regex::new(r"^\s*_\s*(?:,\s*_\s*)?=\s*\w+").unwrap();
    let ok_swallow_re = Regex::new(r"(?i)_\s*=\s*(?:fmt\.|io\.Copy|copy\(|range\s)").unwrap();

    for (i, line) in lines.iter().enumerate() {
        if swallow_re.is_match(line) && !ok_swallow_re.is_match(line) {
            issues.push(Issue {
                line: i + 1, column: None,
                issue_type: "error_swallowing".to_string(),
                severity: Severity::Warning, category: Category::Quality,
                message: "Return value discarded with _ — possible error swallowing".to_string(),
                fix: Some("Handle the error: check it, wrap it, or explicitly document why it's safe to ignore.".to_string()),
                ..Default::default()
            });
        }
    }
    issues
}

fn check_magic_numbers(lines: &[&str]) -> Vec<Issue> {
    let mut issues = Vec::new();
    let magic_re = Regex::new(r"\b(\d{3,})\b").unwrap();
    let ok_numbers: std::collections::HashSet<&str> = [
        "100", "200", "201", "204", "301", "302", "400", "401",
        "403", "404", "500", "1000", "1024", "2048", "4096", "8192", "1000000",
    ].into_iter().collect();
    let const_line_re = Regex::new(r"(?i)^\s*(?:const|var|let|final|static|#define|=\s*\d)").unwrap();
    let mut reported: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if const_line_re.is_match(trimmed) || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("import") {
            continue;
        }
        for caps in magic_re.captures_iter(line) {
            if let Some(m) = caps.get(1) {
                let num = m.as_str();
                if ok_numbers.contains(num) || reported.contains(num) { continue; }
                reported.insert(num.to_string());
                issues.push(Issue {
                    line: i + 1, column: None,
                    issue_type: "magic_number".to_string(),
                    severity: Severity::Info, category: Category::Quality,
                    message: format!("Magic number {} — consider defining as a named constant", num),
                    fix: Some(format!("Extract to a named constant for readability: const SOME_NAME: usize = {};", num)),
                    ..Default::default()
                });
            }
        }
    }
    issues
}

fn check_todo_markers(lines: &[&str]) -> Vec<Issue> {
    let mut issues = Vec::new();
    let todo_re = Regex::new(r"(?i)\b(TODO|FIXME|HACK|XXX|BUG)\b[:\s](.*)").unwrap();

    for (i, line) in lines.iter().enumerate() {
        if let Some(caps) = todo_re.captures(line) {
            let tag = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let desc = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            issues.push(Issue {
                line: i + 1, column: None,
                issue_type: "todo_marker".to_string(),
                severity: Severity::Info, category: Category::Quality,
                message: format!("{}: {}", tag.to_uppercase(), desc),
                fix: Some("Resolve this TODO before merging to main branch.".to_string()),
                ..Default::default()
            });
        }
    }
    issues
}

fn check_duplication(lines: &[&str]) -> Vec<Issue> {
    let mut issues = Vec::new();
    if lines.len() < DUP_MIN_LINES * 2 { return issues; }

    let mut hashes: HashMap<String, usize> = HashMap::new();
    let mut reported: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for i in 0..=lines.len().saturating_sub(DUP_MIN_LINES) {
        let mut block = Vec::new();
        let mut empty = true;
        for j in 0..DUP_MIN_LINES {
            let trimmed = lines[i + j].trim();
            block.push(trimmed);
            if !trimmed.is_empty() { empty = false; }
        }
        if empty { continue; }

        let combined = block.join("\n");
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let result = hasher.finalize();
        let hash = format!("{:x}", &result[..8].iter().fold(0u64, |acc, &b| (acc << 8) | b as u64));

        if let Some(&first_line) = hashes.get(&hash) {
            if !reported.contains(&(i + 1)) && !reported.contains(&first_line) {
                issues.push(Issue {
                    line: i + 1, column: None,
                    issue_type: "code_duplication".to_string(),
                    severity: Severity::Info, category: Category::Quality,
                    message: format!("Duplicate code block (also at line {})", first_line),
                    fix: Some("Extract duplicated code into a shared function.".to_string()),
                    ..Default::default()
                });
                reported.insert(i + 1);
            }
        } else {
            hashes.insert(hash, i + 1);
        }
    }
    issues
}
