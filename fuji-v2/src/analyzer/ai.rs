use regex::Regex;
use std::collections::{HashMap, HashSet};
use crate::models::{FunctionInfo, Issue, Severity, Category};

// ─── Generic Names ───────────────────────────────────────────

fn generic_names() -> HashSet<&'static str> {
    [
        "data", "result", "temp", "tmp", "handler", "process", "execute", "handle",
        "item", "element", "value", "input", "output", "response", "request", "obj",
        "val", "res", "req", "err", "ctx", "cfg", "info", "manager",
        "service", "helper", "utils", "util",
    ].into_iter().collect()
}

// ─── Comment Detection ───────────────────────────────────────

fn is_comment_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || trimmed.starts_with("--")
        || trimmed.starts_with("/*")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("'''")
        || trimmed.starts_with("\"\"\"")
}

fn extract_comment_text(line: &str) -> String {
    let trimmed = line.trim();
    for prefix in &["//", "--", "#"] {
        if trimmed.starts_with(prefix) {
            return trimmed[prefix.len()..].trim().to_string();
        }
    }
    trimmed.to_string()
}

// ─── AI Marker Phrases ──────────────────────────────────────

struct MarkerPattern {
    pattern: Regex,
    desc: &'static str,
}

fn marker_text_patterns() -> Vec<MarkerPattern> {
    vec![
        MarkerPattern {
            pattern: Regex::new(r"(?i)^This\s+(?:function|method|class|struct|module|package|script|file|block|section|component)\s+(?:is\s+(?:used|responsible|designed)|handles|implements|provides|creates|processes|validates|performs|manages|initializes|returns|takes|accepts|defines|sets\s+up|configures)").unwrap(),
            desc: "Comment starts with 'This function/module handles/implements...' — characteristic of LLM output",
        },
        MarkerPattern {
            pattern: Regex::new(r"(?i)^Here\s+we\s+(?:implement|define|create|handle|process|check|validate|perform|initialize|set\s+up|configure|load|import)").unwrap(),
            desc: "Comment starts with 'Here we implement/create...' — reading like tutorial prose",
        },
        MarkerPattern {
            pattern: Regex::new(r"(?i)^The\s+following\s+(?:function|code|method|block|section|config|configuration|module)\s+(?:is|will|does|handles|implements|configures|defines|sets)").unwrap(),
            desc: "Comment says 'The following code/function...' — explanatory style typical of AI",
        },
        MarkerPattern {
            pattern: Regex::new(r"(?i)^(?:Note|Please note|Important)\s*(?::|that)\s").unwrap(),
            desc: "Comment uses 'Note that...' / 'Please note...' — overly formal AI style",
        },
        MarkerPattern {
            pattern: Regex::new(r"(?i)^We\s+(?:use|need|want|can|should|must|have)\s+(?:to|this|a|an|the)\s").unwrap(),
            desc: "Comment uses 'We use this to...' / 'We need to...' — tutorial-style narration",
        },
        MarkerPattern {
            pattern: Regex::new(r"(?i)^(?:For|In)\s+(?:simplicity|brevity|clarity|readability|maintainability|safety|security|performance)\s*[,.]").unwrap(),
            desc: "Comment justifies with 'For simplicity/clarity...' — AI hedging pattern",
        },
        MarkerPattern {
            pattern: Regex::new(r"(?i)^(?:First|Next|Then|Finally|After that|Now)\s*[,.]?\s*(?:we|let's|I|you)").unwrap(),
            desc: "Sequential narration in comments — 'First we... Then we...' — AI tutorial style",
        },
        MarkerPattern {
            pattern: Regex::new(r"(?i)^(?:Make sure|Ensure|Don't forget|Remember)\s+(?:to|that)\s").unwrap(),
            desc: "Instructional comment — 'Make sure to...' — AI advisory tone",
        },
        MarkerPattern {
            pattern: Regex::new(r"(?i)^This\s+is\s+(?:a|an|the)\s+(?:helper|utility|wrapper|convenience|simple|basic|main|entry|default)\s").unwrap(),
            desc: "Self-describing comment — 'This is a helper/utility...'",
        },
        MarkerPattern {
            pattern: Regex::new(r"(?i)^(?:Configure|Setup|Initialize|Register|Load|Import|Define|Declare)\s+(?:the\s+|our\s+|all\s+)?(?:necessary|required|needed|essential|core|main|primary|default)").unwrap(),
            desc: "Setup narration — 'Configure the necessary...' — AI instructional voice",
        },
    ]
}

// ─── Readme-style Patterns ──────────────────────────────────

fn readme_text_patterns() -> Vec<Regex> {
    vec![
        Regex::new(r"(?i)^(?:Import|Require)\s+(?:the\s+)?(?:necessary|required|needed)\s+(?:packages?|modules?|libraries?|dependencies)").unwrap(),
        Regex::new(r"(?i)^(?:Define|Declare|Create|Set|Initialize)\s+(?:a\s+|an\s+|the\s+)?(?:new\s+)?(?:variable|constant|struct|class|function|method|array|map|slice|list|object|table|config)\s").unwrap(),
        Regex::new(r"(?i)^(?:Check|Verify|Validate)\s+(?:if|that|whether)\s+(?:the\s+)?(?:input|value|data|result|error|user|param|file|path)").unwrap(),
        Regex::new(r"(?i)^(?:Return|Send|Print|Log|Output)\s+(?:the\s+)?(?:result|response|error|data|value|message|output)").unwrap(),
        Regex::new(r"(?i)^(?:Loop|Iterate)\s+(?:through|over|across)\s+(?:the\s+|all\s+|each\s+)?").unwrap(),
        Regex::new(r"(?i)^(?:Handle|Catch|Process)\s+(?:the\s+|any\s+)?(?:error|exception|panic|failure)").unwrap(),
        Regex::new(r"(?i)^(?:Open|Close|Read|Write)\s+(?:the\s+|a\s+)?(?:file|connection|database|stream|socket)").unwrap(),
        Regex::new(r"(?i)^\d+\.\s+[A-Z][a-z]+\s").unwrap(),
        Regex::new(r"(?i)^(?:Commands?\s+for|Quick\s+function\s+to|Helper\s+(?:function|to)|Utility\s+(?:function|to)|Wrapper\s+for)").unwrap(),
    ]
}

// ─── Error Handling Boilerplate ─────────────────────────────

fn error_boilerplate_patterns() -> HashMap<&'static str, Regex> {
    let mut m = HashMap::new();
    m.insert("Go", Regex::new(r"if\s+err\s*!=\s*nil\s*\{").unwrap());
    m.insert("Python", Regex::new(r"except\s+(?:Exception|BaseException|\w+Error)\s+as\s+\w+:").unwrap());
    m.insert("JavaScript", Regex::new(r"\.catch\s*\(\s*(?:err|error|e)\s*=>").unwrap());
    m.insert("Java", Regex::new(r"catch\s*\(\s*(?:Exception|Throwable|\w+Exception)\s+\w+\s*\)").unwrap());
    m
}

// ─── Main AI Analysis ───────────────────────────────────────

/// Compute AI probability score (0-100) and generate AI-pattern issues
pub fn analyze_ai(content: &str, lang: &str, functions: &[FunctionInfo]) -> (f64, Vec<Issue>) {
    let mut issues = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() < 5 {
        return (0.0, issues);
    }

    struct Signal {
        _name: &'static str,
        score: f64,
        weight: f64,
    }

    let mut signals = Vec::new();

    // 1. Token diversity
    let div_score = token_diversity(content);
    signals.push(Signal { _name: "token_diversity", score: div_score, weight: 0.05 });

    // 2. Formatting consistency
    let fmt_score = formatting_consistency(&lines);
    signals.push(Signal { _name: "formatting", score: fmt_score, weight: 0.05 });

    // 3. Generic naming
    let (gen_score, gen_issues) = generic_naming(content, &lines);
    issues.extend(gen_issues);
    signals.push(Signal { _name: "generic_naming", score: gen_score, weight: 0.08 });

    // 4. Comment quality
    let cmt_score = comment_quality(&lines);
    signals.push(Signal { _name: "comment_quality", score: cmt_score, weight: 0.10 });

    // 5. Function uniformity
    let uni_score = function_uniformity(functions);
    signals.push(Signal { _name: "function_uniformity", score: uni_score, weight: 0.05 });

    // 6. Marker phrases (strongest)
    let (mkr_score, mkr_issues) = marker_phrase_analysis(&lines);
    issues.extend(mkr_issues);
    signals.push(Signal { _name: "marker_phrases", score: mkr_score, weight: 0.20 });

    // 7. Error handling boilerplate
    let err_score = error_boilerplate_score(&lines, lang);
    signals.push(Signal { _name: "error_boilerplate", score: err_score, weight: 0.05 });

    // 8. Variable naming entropy
    let var_score = naming_entropy_score(content);
    signals.push(Signal { _name: "naming_entropy", score: var_score, weight: 0.05 });

    // 9. Structural fingerprint
    let str_score = structural_fingerprint(&lines);
    signals.push(Signal { _name: "structural_pattern", score: str_score, weight: 0.10 });

    // 10. Line length distribution
    let len_score = line_length_distribution(&lines);
    signals.push(Signal { _name: "line_length_dist", score: len_score, weight: 0.05 });

    // 11. Readme-style comments
    let (rdm_score, rdm_issues) = readme_comment_score(&lines);
    issues.extend(rdm_issues);
    signals.push(Signal { _name: "readme_comments", score: rdm_score, weight: 0.12 });

    // 12. Repetitive structure
    let rep_score = repetitive_structure_score(&lines);
    signals.push(Signal { _name: "repetitive_structure", score: rep_score, weight: 0.10 });

    // Weighted combination
    let mut ai_prob: f64 = signals.iter().map(|s| s.score * s.weight).sum::<f64>() * 100.0;
    ai_prob = ai_prob.clamp(0.0, 100.0);

    (ai_prob, issues)
}

// ─── Signal Implementations ─────────────────────────────────

fn extract_identifiers(content: &str) -> Vec<String> {
    let re = Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap();
    re.find_iter(content).map(|m| m.as_str().to_string()).collect()
}

fn token_diversity(content: &str) -> f64 {
    let words = extract_identifiers(content);
    if words.is_empty() { return 0.0; }
    let unique: HashSet<String> = words.iter().map(|w| w.to_lowercase()).collect();
    let ratio = unique.len() as f64 / words.len() as f64;
    if ratio > 0.6 { 0.0 }
    else if ratio < 0.2 { 1.0 }
    else { 1.0 - (ratio / 0.6) }
}

fn variance(nums: &[usize]) -> f64 {
    if nums.is_empty() { return 0.0; }
    let mean = nums.iter().sum::<usize>() as f64 / nums.len() as f64;
    let var_sum: f64 = nums.iter().map(|&n| { let d = n as f64 - mean; d * d }).sum();
    var_sum / nums.len() as f64
}

fn formatting_consistency(lines: &[&str]) -> f64 {
    if lines.len() < 10 { return 0.0; }

    let mut indents: HashMap<usize, usize> = HashMap::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        let mut indent = 0;
        for ch in line.chars() {
            if ch == '\t' { indent += 4; }
            else if ch == ' ' { indent += 1; }
            else { break; }
        }
        *indents.entry(indent).or_insert(0) += 1;
    }

    let mut tab_aligned = 0;
    let mut total = 0;
    for (&indent, &count) in &indents {
        total += count;
        if indent % 4 == 0 || indent % 2 == 0 {
            tab_aligned += count;
        }
    }
    if total == 0 { return 0.0; }

    let lengths: Vec<usize> = lines.iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len())
        .collect();
    let length_variance = variance(&lengths);
    let length_score = if length_variance < 100.0 { 1.0 - (length_variance / 100.0) } else { 0.0 };

    let align_ratio = tab_aligned as f64 / total as f64;
    let align_score = if align_ratio > 0.95 { 1.0 } else { 0.0 };

    align_score * 0.5 + length_score * 0.5
}

fn generic_naming(content: &str, lines: &[&str]) -> (f64, Vec<Issue>) {
    let mut issues = Vec::new();
    let identifiers = extract_identifiers(content);
    if identifiers.is_empty() { return (0.0, issues); }

    let generics = generic_names();
    let mut generic_count = 0;
    let mut seen: HashSet<String> = HashSet::new();

    for id in &identifiers {
        let lower = id.to_lowercase();
        if generics.contains(lower.as_str()) && !seen.contains(&lower) {
            generic_count += 1;
            seen.insert(lower.clone());
            for (i, line) in lines.iter().enumerate() {
                if line.contains(id.as_str()) {
                    issues.push(Issue {
                        line: i + 1,
                        column: None,
                        issue_type: "generic_naming".to_string(),
                        severity: Severity::Info,
                        category: Category::AIPattern,
                        message: format!("Generic identifier '{}' — common in AI-generated code", id),
                        fix: None,
                        ..Default::default()
                    });
                    break;
                }
            }
        }
    }

    let ratio = generic_count as f64 / (seen.len() as f64 + 1.0);
    let score = if ratio > 0.5 { 1.0 } else { ratio * 2.0 };
    (score, issues)
}

fn comment_quality(lines: &[&str]) -> f64 {
    let mut comment_lines = 0;
    let mut code_lines = 0;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        if is_comment_line(line) { comment_lines += 1; }
        else { code_lines += 1; }
    }
    if code_lines == 0 { return 0.0; }
    let ratio = comment_lines as f64 / code_lines as f64;
    if ratio > 0.5 { 0.9 }
    else if ratio > 0.3 { 0.6 }
    else if ratio > 0.2 { 0.3 }
    else { 0.0 }
}

fn function_uniformity(functions: &[FunctionInfo]) -> f64 {
    if functions.len() < 3 { return 0.0; }
    let sizes: Vec<usize> = functions.iter().map(|f| f.line_count).collect();
    let v = variance(&sizes);
    if v < 10.0 { 0.8 }
    else if v < 25.0 { 0.4 }
    else { 0.0 }
}

fn marker_phrase_analysis(lines: &[&str]) -> (f64, Vec<Issue>) {
    let mut issues = Vec::new();
    let mut hit_count = 0;
    let mut comment_count = 0;
    let patterns = marker_text_patterns();

    for (i, line) in lines.iter().enumerate() {
        if is_comment_line(line) {
            comment_count += 1;
            let text = extract_comment_text(line);
            if text.len() < 5 { continue; }
            for mp in &patterns {
                if mp.pattern.is_match(&text) {
                    hit_count += 1;
                    issues.push(Issue {
                        line: i + 1, column: None,
                        issue_type: "ai_marker_phrase".to_string(),
                        severity: Severity::Info, category: Category::AIPattern,
                        message: mp.desc.to_string(), fix: None,
                        ..Default::default()
                    });
                    break;
                }
            }
        }
    }

    if comment_count < 3 { return (0.0, issues); }
    let ratio = hit_count as f64 / comment_count as f64;
    let score = if ratio > 0.15 { 1.0 }
        else if ratio > 0.08 { 0.7 }
        else if ratio > 0.03 { 0.4 }
        else if hit_count > 0 { 0.2 }
        else { 0.0 };
    (score, issues)
}

fn error_boilerplate_score(lines: &[&str], lang: &str) -> f64 {
    let patterns = error_boilerplate_patterns();
    let pat = match patterns.get(lang) {
        Some(p) => p,
        None => return 0.0,
    };

    let mut error_blocks: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if pat.is_match(line) {
            let mut block = line.to_string();
            for j in 1..=3 {
                if i + j < lines.len() {
                    block.push('\n');
                    block.push_str(lines[i + j].trim());
                }
            }
            error_blocks.push(block);
        }
    }

    if error_blocks.len() < 3 { return 0.0; }

    let mut block_counts: HashMap<String, usize> = HashMap::new();
    for b in &error_blocks {
        *block_counts.entry(b.clone()).or_insert(0) += 1;
    }

    let max_dup = block_counts.values().copied().max().unwrap_or(0);
    let dup_ratio = max_dup as f64 / error_blocks.len() as f64;
    if dup_ratio > 0.7 { 0.9 }
    else if dup_ratio > 0.5 { 0.5 }
    else { 0.0 }
}

fn naming_entropy_score(content: &str) -> f64 {
    let identifiers = extract_identifiers(content);
    if identifiers.len() < 10 { return 0.0; }

    let mut camel_count = 0;
    let mut snake_count = 0;
    for id in &identifiers {
        if id.len() < 3 { continue; }
        let has_underscore = id.contains('_');
        let has_upper = id.chars().any(|c| c.is_uppercase());
        let has_lower = id.chars().any(|c| c.is_lowercase());

        if has_underscore && has_lower { snake_count += 1; }
        else if has_upper && has_lower && !has_underscore { camel_count += 1; }
    }

    let total = camel_count + snake_count;
    if total < 5 { return 0.0; }

    let dominant = camel_count.max(snake_count);
    let consistency = dominant as f64 / total as f64;
    if consistency > 0.95 { 0.6 }
    else if consistency > 0.85 { 0.3 }
    else { 0.0 }
}

fn structural_fingerprint(lines: &[&str]) -> f64 {
    let section_re = Regex::new(r"^\s*(?://|--|#)\s*\d+\.\s+[A-Z]").unwrap();
    let mut section_count = 0;
    let mut transitions = 0;
    let mut last_was_comment = false;
    let mut code_block_count = 0;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        if section_re.is_match(line) { section_count += 1; }
        let is_cmt = is_comment_line(line);
        if !is_cmt && last_was_comment { transitions += 1; }
        if !is_cmt { code_block_count += 1; }
        last_was_comment = is_cmt;
    }

    let mut score = 0.0;
    if section_count >= 3 { score = 0.9; }
    else if section_count >= 2 { score = 0.6; }

    if code_block_count > 5 {
        let ratio = transitions as f64 / code_block_count as f64;
        if ratio > 0.4 {
            let comment_score = ratio.min(1.0);
            if comment_score > score { score = comment_score; }
        }
    }
    score
}

fn line_length_distribution(lines: &[&str]) -> f64 {
    let lengths: Vec<usize> = lines.iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.len())
        .collect();
    if lengths.len() < 20 { return 0.0; }

    let v = variance(&lengths);
    let mean = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;
    let stddev = v.sqrt();
    if stddev == 0.0 { return 0.8; }

    let skew_sum: f64 = lengths.iter()
        .map(|&l| ((l as f64 - mean) / stddev).powi(3))
        .sum();
    let skewness = (skew_sum / lengths.len() as f64).abs();

    if skewness < 0.3 { 0.7 }
    else if skewness < 0.7 { 0.3 }
    else { 0.0 }
}

fn readme_comment_score(lines: &[&str]) -> (f64, Vec<Issue>) {
    let mut issues = Vec::new();
    let mut hit_count = 0;
    let mut comment_count = 0;
    let patterns = readme_text_patterns();

    for (i, line) in lines.iter().enumerate() {
        if is_comment_line(line) {
            comment_count += 1;
            let text = extract_comment_text(line);
            if text.len() < 5 { continue; }
            for pat in &patterns {
                if pat.is_match(&text) {
                    hit_count += 1;
                    issues.push(Issue {
                        line: i + 1, column: None,
                        issue_type: "readme_comment".to_string(),
                        severity: Severity::Info, category: Category::AIPattern,
                        message: "Comment explains obvious code — typical of AI-generated output".to_string(),
                        fix: None,
                        ..Default::default()
                    });
                    break;
                }
            }
        }
    }

    if comment_count < 2 { return (0.0, issues); }
    let ratio = hit_count as f64 / comment_count as f64;
    let score = if ratio > 0.20 { 1.0 }
        else if ratio > 0.10 { 0.6 }
        else if hit_count > 0 { 0.3 }
        else { 0.0 };
    (score, issues)
}

fn repetitive_structure_score(lines: &[&str]) -> f64 {
    // Generic: detect any normalized line pattern repeated 5+ times
    let re_dq = Regex::new(r#""[^"]*""#).unwrap();
    let re_sq = Regex::new(r"'[^']*'").unwrap();

    let mut normalized: HashMap<String, usize> = HashMap::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.len() < 15 { continue; }
        let s = re_dq.replace_all(trimmed, r#""""#).to_string();
        let s = re_sq.replace_all(&s, "''").to_string();
        *normalized.entry(s).or_insert(0) += 1;
    }

    let high_repeat: usize = normalized.values().filter(|&&c| c >= 5).sum();
    if high_repeat == 0 { return 0.0; }

    let ratio = high_repeat as f64 / lines.len() as f64;
    if ratio > 0.15 { 0.8 }
    else if ratio > 0.08 { 0.5 }
    else { 0.2 }
}

// ─── CommentRatio (used by quality) ─────────────────────────

pub fn comment_ratio(content: &str, _lang: &str) -> f64 {
    let lines: Vec<&str> = content.lines().collect();
    let mut comment_lines = 0;
    let mut code_lines = 0;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        if is_comment_line(line) { comment_lines += 1; }
        else { code_lines += 1; }
    }
    if code_lines == 0 { return 0.0; }
    comment_lines as f64 / code_lines as f64
}
