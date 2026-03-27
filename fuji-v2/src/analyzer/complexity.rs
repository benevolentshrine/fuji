use regex::Regex;
use std::collections::HashMap;
use crate::models::FunctionInfo;

/// Branch keywords per language for complexity counting
fn branch_keywords() -> HashMap<&'static str, Vec<&'static str>> {
    let mut m = HashMap::new();
    m.insert("Go", vec!["if ", "else if ", "for ", "switch ", "case ", "select ", "go ", "defer "]);
    m.insert("Python", vec!["if ", "elif ", "for ", "while ", "except ", "with "]);
    m.insert("Rust", vec!["if ", "else if ", "for ", "while ", "match ", "loop "]);
    m.insert("JavaScript", vec!["if ", "else if ", "for ", "while ", "switch ", "case ", "catch "]);
    m.insert("TypeScript", vec!["if ", "else if ", "for ", "while ", "switch ", "case ", "catch "]);
    m.insert("Java", vec!["if ", "else if ", "for ", "while ", "switch ", "case ", "catch "]);
    m.insert("C", vec!["if ", "else if ", "for ", "while ", "switch ", "case "]);
    m.insert("C++", vec!["if ", "else if ", "for ", "while ", "switch ", "case ", "catch "]);
    m.insert("Ruby", vec!["if ", "elsif ", "unless ", "while ", "until ", "for ", "rescue "]);
    m.insert("PHP", vec!["if ", "elseif ", "for ", "foreach ", "while ", "switch ", "case ", "catch "]);
    m.insert("Shell", vec!["if ", "elif ", "for ", "while ", "case "]);
    m.insert("Lua", vec!["if ", "elseif ", "for ", "while ", "repeat "]);
    m.insert("Dart", vec!["if ", "else if ", "for ", "while ", "switch ", "case ", "catch "]);
    m.insert("Kotlin", vec!["if ", "else if ", "for ", "while ", "when ", "catch "]);
    m.insert("Swift", vec!["if ", "else if ", "for ", "while ", "switch ", "case ", "catch "]);
    m
}

/// Function pattern regexes per language
fn func_patterns() -> HashMap<&'static str, Regex> {
    let mut m = HashMap::new();
    m.insert("Go", Regex::new(r"^func\s+(\(.*?\)\s+)?(\w+)\s*\(").unwrap());
    m.insert("Python", Regex::new(r"^\s*def\s+(\w+)\s*\(").unwrap());
    m.insert("Rust", Regex::new(r"^\s*(pub\s+)?fn\s+(\w+)\s*[<(]").unwrap());
    m.insert("JavaScript", Regex::new(r"^\s*(function\s+(\w+)|(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?(?:function|\())").unwrap());
    m.insert("TypeScript", Regex::new(r"^\s*(function\s+(\w+)|(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?(?:function|\())").unwrap());
    m.insert("Java", Regex::new(r"^\s*(?:public|private|protected|static|\s)*\s+\w+\s+(\w+)\s*\(").unwrap());
    m.insert("Ruby", Regex::new(r"^\s*def\s+(\w+)").unwrap());
    m.insert("PHP", Regex::new(r"^\s*(?:public|private|protected|static|\s)*\s*function\s+(\w+)\s*\(").unwrap());
    m.insert("Lua", Regex::new(r"(?:^|\s)(?:local\s+)?function\s+(\w+)\s*\(").unwrap());
    m.insert("Dart", Regex::new(r"^\s*(?:static\s+)?\w+\s+(\w+)\s*\(").unwrap());
    m.insert("Kotlin", Regex::new(r"^\s*(?:fun|suspend\s+fun)\s+(\w+)\s*\(").unwrap());
    m.insert("Swift", Regex::new(r"^\s*(?:func|static\s+func|class\s+func)\s+(\w+)\s*\(").unwrap());
    m
}

/// Analyze cyclomatic complexity and extract functions
pub fn analyze_complexity(content: &str, lang: &str) -> (usize, Vec<FunctionInfo>) {
    let lines: Vec<&str> = content.lines().collect();
    let kw_map = branch_keywords();
    let keywords = kw_map.get(lang).cloned().unwrap_or_else(|| {
        kw_map.get("Go").cloned().unwrap_or_default()
    });

    let fp_map = func_patterns();
    let func_pat = fp_map.get(lang);

    let mut functions: Vec<FunctionInfo> = Vec::new();
    let mut total_complexity: usize = 1; // base
    let mut current_func: Option<FunctionInfo> = None;
    let mut brace_depth: i32 = 0;
    let mut func_brace_start: i32 = 0;

    let logical_ops = ["&&", "||"];

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let line_num = i + 1;

        // Check for function start
        if let Some(pat) = func_pat {
            if let Some(caps) = pat.captures(trimmed) {
                // Save previous function
                if let Some(mut prev) = current_func.take() {
                    prev.end_line = line_num - 1;
                    prev.line_count = prev.end_line.saturating_sub(prev.start_line) + 1;
                    functions.push(prev);
                }

                let name = extract_func_name(&caps, lang);
                current_func = Some(FunctionInfo {
                    name,
                    start_line: line_num,
                    end_line: 0,
                    complexity: 1,
                    line_count: 0,
                    signature: None,
                });
                func_brace_start = brace_depth;
            }
        }

        // Count braces
        brace_depth += trimmed.matches('{').count() as i32;
        brace_depth -= trimmed.matches('}').count() as i32;

        // Count complexity keywords
        for kw in &keywords {
            if trimmed.contains(kw) {
                total_complexity += 1;
                if let Some(ref mut f) = current_func {
                    f.complexity += 1;
                }
            }
        }

        // Count logical operators
        for op in &logical_ops {
            let count = trimmed.matches(op).count();
            total_complexity += count;
            if let Some(ref mut f) = current_func {
                f.complexity += count;
            }
        }

        // Check if function ended
        if current_func.is_some() && brace_depth <= func_brace_start && i > 0 {
            if (lang == "Python" || lang == "Ruby")
                && !trimmed.is_empty()
                && !line.starts_with(' ')
                && !line.starts_with('\t')
            {
                if let Some(ref f) = current_func {
                    if line_num > f.start_line + 1 {
                        let mut f = current_func.take().unwrap();
                        f.end_line = line_num - 1;
                        f.line_count = f.end_line.saturating_sub(f.start_line) + 1;
                        functions.push(f);
                    }
                }
            }
            if lang == "Lua" && trimmed == "end" {
                if let Some(mut f) = current_func.take() {
                    f.end_line = line_num;
                    f.line_count = f.end_line.saturating_sub(f.start_line) + 1;
                    functions.push(f);
                }
            }
        }
    }

    // Close last function
    if let Some(mut f) = current_func.take() {
        f.end_line = lines.len();
        f.line_count = f.end_line.saturating_sub(f.start_line) + 1;
        functions.push(f);
    }

    (total_complexity, functions)
}

fn extract_func_name(caps: &regex::Captures, lang: &str) -> String {
    match lang {
        "Go" => {
            caps.get(2)
                .or_else(|| caps.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        }
        "Rust" => {
            caps.get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        }
        "JavaScript" | "TypeScript" => {
            for i in (1..caps.len()).rev() {
                if let Some(m) = caps.get(i) {
                    if !m.as_str().is_empty() {
                        return m.as_str().to_string();
                    }
                }
            }
            "unknown".to_string()
        }
        _ => {
            for i in 1..caps.len() {
                if let Some(m) = caps.get(i) {
                    if !m.as_str().is_empty() {
                        return m.as_str().to_string();
                    }
                }
            }
            "unknown".to_string()
        }
    }
}
