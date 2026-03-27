use regex::Regex;
use crate::models::{Issue, Severity, Category};

// ─── Secret Detection ────────────────────────────────────────

struct SecretRule {
    _name: &'static str,
    pattern: Regex,
    severity: Severity,
    msg: &'static str,
    fix: &'static str,
    entropy: bool,
}

fn secret_rules() -> Vec<SecretRule> {
    vec![
        // Cloud Provider Keys
        SecretRule { _name: "AWS Access Key", pattern: Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(), severity: Severity::Critical, msg: "AWS Access Key ID found", fix: "Use environment variables or AWS Secrets Manager", entropy: false },
        SecretRule { _name: "AWS Secret Key", pattern: Regex::new(r#"(?i)(aws_secret_access_key|aws_secret)\s*[:=]\s*["']([A-Za-z0-9/+=]{40})["']"#).unwrap(), severity: Severity::Critical, msg: "AWS Secret Access Key found", fix: "Use IAM roles or environment variables", entropy: true },
        SecretRule { _name: "GCP API Key", pattern: Regex::new(r"AIza[0-9A-Za-z\-_]{35}").unwrap(), severity: Severity::Critical, msg: "Google Cloud API key found", fix: "Use service accounts instead of API keys", entropy: false },
        // API Keys & Tokens
        SecretRule { _name: "Generic API Key", pattern: Regex::new(r#"(?i)(api[_-]?key|apikey)\s*[:=]\s*["']([^"']{8,})["']"#).unwrap(), severity: Severity::Error, msg: "Possible API key in string literal", fix: "Move to environment variable or secrets manager", entropy: true },
        SecretRule { _name: "Generic Secret", pattern: Regex::new(r#"(?i)(secret|password|passwd|pwd|token|auth_token|access_token)\s*[:=]\s*["']([^"']{8,})["']"#).unwrap(), severity: Severity::Critical, msg: "Possible hardcoded secret", fix: "Use environment variables or a vault", entropy: true },
        SecretRule { _name: "Bearer Token", pattern: Regex::new(r"(?i)(bearer\s+)[A-Za-z0-9\-_.~+/]{20,}").unwrap(), severity: Severity::Critical, msg: "Hardcoded Bearer token", fix: "Load tokens from secure storage at runtime", entropy: false },
        // Private Keys
        SecretRule { _name: "Private Key", pattern: Regex::new(r"-----BEGIN\s+(RSA\s+|EC\s+|DSA\s+|OPENSSH\s+)?PRIVATE KEY-----").unwrap(), severity: Severity::Critical, msg: "Private key embedded in source", fix: "Store private keys in secure files outside source control", entropy: false },
        // Database
        SecretRule { _name: "Database URL", pattern: Regex::new(r#"(?i)(mongodb|postgres|mysql|redis|amqp|mssql)://[^\s"']+@[^\s"']+"#).unwrap(), severity: Severity::Critical, msg: "Database connection string with credentials", fix: "Use environment variables for connection strings", entropy: false },
        // JWT
        SecretRule { _name: "JWT Token", pattern: Regex::new(r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}").unwrap(), severity: Severity::Error, msg: "JWT token found in source", fix: "Load JWT tokens dynamically, never hardcode", entropy: false },
        // GitHub/GitLab
        SecretRule { _name: "GitHub Token", pattern: Regex::new(r"gh[pousr]_[A-Za-z0-9_]{36,}").unwrap(), severity: Severity::Critical, msg: "GitHub personal access token found", fix: "Use GitHub Apps or environment variables", entropy: false },
        // Stripe
        SecretRule { _name: "Stripe Key", pattern: Regex::new(r"sk_(live|test)_[A-Za-z0-9]{24,}").unwrap(), severity: Severity::Critical, msg: "Stripe secret key found", fix: "Use environment variable for Stripe keys", entropy: false },
        // Slack
        SecretRule { _name: "Slack Webhook", pattern: Regex::new(r"https://hooks\.slack\.com/services/T[A-Z0-9]+/B[A-Z0-9]+/[A-Za-z0-9]+").unwrap(), severity: Severity::Error, msg: "Slack webhook URL found", fix: "Use environment variable for webhook URLs", entropy: false },
    ]
}

// ─── Injection Detection ─────────────────────────────────────

struct InjectionRule {
    issue_type: &'static str,
    pattern: Regex,
    msg: &'static str,
    fix: &'static str,
    severity: Severity,
}

fn injection_rules() -> Vec<InjectionRule> {
    vec![
        // SQL Injection
        InjectionRule { issue_type: "sql_injection", pattern: Regex::new(r"(?i)(?:fmt\.Sprintf|format|f\x22|%s|%v).*?(?:SELECT|INSERT|UPDATE|DELETE|DROP|ALTER|CREATE|TRUNCATE)\s").unwrap(), msg: "SQL injection — string formatting in SQL query", fix: "Use parameterized queries / prepared statements", severity: Severity::Critical },
        InjectionRule { issue_type: "sql_injection", pattern: Regex::new(r#"(?i)["\x27]\s*\+\s*\w+\s*\+\s*["\x27].*?(?:WHERE|AND|OR|SET|VALUES)\s"#).unwrap(), msg: "SQL injection — string concatenation in query", fix: "Use parameterized queries / prepared statements", severity: Severity::Critical },
        // Command Injection
        InjectionRule { issue_type: "command_injection", pattern: Regex::new(r"exec\.Command\([^)]*\+").unwrap(), msg: "Command injection — variable concatenated into exec.Command", fix: "Validate/sanitize inputs, use execArgs not string concat", severity: Severity::Critical },
        InjectionRule { issue_type: "command_injection", pattern: Regex::new(r"os\.system\([^)]*(?:\+|%|format|f\x22)").unwrap(), msg: "Command injection — dynamic string in os.system()", fix: "Use subprocess with argument list, never shell=True", severity: Severity::Critical },
        InjectionRule { issue_type: "command_injection", pattern: Regex::new(r"subprocess\.(?:call|run|Popen)\([^)]*shell\s*=\s*True").unwrap(), msg: "Subprocess with shell=True — command injection risk", fix: "Use shell=False and pass args as list", severity: Severity::Error },
        InjectionRule { issue_type: "command_injection", pattern: Regex::new(r"\beval\s*\([^)]*(?:\+|format|f\x22|%s|\$\{)").unwrap(), msg: "eval() with dynamic input — code injection", fix: "Avoid eval(). Use safe alternatives like JSON.parse()", severity: Severity::Critical },
        // XSS
        InjectionRule { issue_type: "xss", pattern: Regex::new(r"(?i)dangerouslySetInnerHTML").unwrap(), msg: "React dangerouslySetInnerHTML — XSS risk", fix: "Sanitize with DOMPurify before setting inner HTML", severity: Severity::Warning },
        // Path Traversal
        InjectionRule { issue_type: "path_traversal", pattern: Regex::new(r"(?i)\.\.(?:/|\\\\)").unwrap(), msg: "Path traversal sequence '../' found", fix: "Reject paths containing '..', use filepath.Clean()", severity: Severity::Warning },
        // SSRF
        InjectionRule { issue_type: "ssrf", pattern: Regex::new(r"(?i)(?:http\.Get|http\.Post|requests\.get|requests\.post|fetch|axios|urllib)\s*\([^)]*(?:\+|format|f\x22|%s|\$\{)").unwrap(), msg: "SSRF — user-controlled URL in HTTP request", fix: "Validate URLs against allowlist, block internal IPs", severity: Severity::Error },
        // Deserialization
        InjectionRule { issue_type: "deserialization", pattern: Regex::new(r"(?i)(?:pickle\.loads?|yaml\.(?:load|unsafe_load)|marshal\.load|unserialize|ObjectInputStream)").unwrap(), msg: "Unsafe deserialization — remote code execution risk", fix: "Use safe alternatives: yaml.safe_load(), json instead of pickle", severity: Severity::Critical },
    ]
}

// ─── Crypto Misuse ───────────────────────────────────────────

struct CryptoRule {
    pattern: Regex,
    msg: &'static str,
    fix: &'static str,
    severity: Severity,
}

fn crypto_rules() -> Vec<CryptoRule> {
    vec![
        CryptoRule { pattern: Regex::new(r"(?i)\b(?:md5|MD5)\.(?:New|Sum|Hash|Create|digest)\b").unwrap(), msg: "MD5 is cryptographically broken", fix: "Use SHA-256 or SHA-3", severity: Severity::Error },
        CryptoRule { pattern: Regex::new(r"(?i)\b(?:sha1|SHA1)\.(?:New|Sum|Hash|Create|digest)\b").unwrap(), msg: "SHA-1 is deprecated and vulnerable to collision attacks", fix: "Use SHA-256 or SHA-3", severity: Severity::Warning },
        CryptoRule { pattern: Regex::new(r#"(?i)"crypto/md5""#).unwrap(), msg: "MD5 package imported — MD5 is broken for security", fix: "Use crypto/sha256 instead", severity: Severity::Error },
        CryptoRule { pattern: Regex::new(r"(?i)hashlib\.md5\(").unwrap(), msg: "Python MD5 usage", fix: "Use hashlib.sha256() or hashlib.sha3_256()", severity: Severity::Error },
        CryptoRule { pattern: Regex::new(r"(?i)\b(?:crypto/des|des\.NewCipher|DES\.new|DES_EDE)\b").unwrap(), msg: "DES/3DES is weak encryption", fix: "Use AES-256-GCM", severity: Severity::Error },
        CryptoRule { pattern: Regex::new(r"(?i)(?:NewECBEncrypter|NewECBDecrypter|ECB|MODE_ECB|mode=ECB)").unwrap(), msg: "ECB mode does not provide semantic security", fix: "Use CBC with random IV, or better: GCM (authenticated encryption)", severity: Severity::Critical },
        CryptoRule { pattern: Regex::new(r#""math/rand""#).unwrap(), msg: "math/rand is not cryptographically secure", fix: "Use crypto/rand for security-sensitive randomness", severity: Severity::Warning },
    ]
}

// ─── Auth & Info Disclosure ──────────────────────────────────

struct AuthRule {
    issue_type: &'static str,
    pattern: Regex,
    msg: &'static str,
    fix: &'static str,
    severity: Severity,
}

fn auth_rules() -> Vec<AuthRule> {
    vec![
        AuthRule { issue_type: "auth_bypass", pattern: Regex::new(r#"(?i)(?:admin|isAdmin|is_admin|role)\s*[:=]=?\s*(?:true|True|1|"admin")"#).unwrap(), msg: "Hardcoded admin/role check — potential auth bypass", fix: "Implement proper RBAC with database-backed roles", severity: Severity::Warning },
        AuthRule { issue_type: "auth_bypass", pattern: Regex::new(r"(?i)(?:cors|CORS).*(?:\*|AllowAll|allow_all|allowOrigin.*\*)").unwrap(), msg: "CORS allows all origins — security risk", fix: "Restrict CORS to specific trusted origins", severity: Severity::Error },
        AuthRule { issue_type: "info_disclosure", pattern: Regex::new(r"(?i)(?:fmt\.Printf|print|console\.log|log\.Print)\s*\(.*(?:password|secret|token|key|credential)").unwrap(), msg: "Sensitive data logged to output", fix: "Never log credentials. Use masked logging", severity: Severity::Error },
        AuthRule { issue_type: "info_disclosure", pattern: Regex::new(r"(?i)(?:debug\s*[:=]\s*(?:true|True|1)|DEBUG\s*=\s*(?:True|1|true))").unwrap(), msg: "Debug mode enabled — may expose sensitive information", fix: "Disable debug mode in production", severity: Severity::Warning },
    ]
}

// ─── Context Helpers ─────────────────────────────────────────

fn is_comment_line(line: &str) -> bool {
    let re = Regex::new(r"^\s*(?://|#|/\*|\*|--|;)").unwrap();
    re.is_match(line)
}

fn is_pattern_definition(line: &str) -> bool {
    let re = Regex::new(r"(?i)(?:Description|Usage|Help|Example|Pattern|Regex|MustCompile|regexp|compile|message|msg|label|placeholder|hint)").unwrap();
    re.is_match(line)
}

// ─── Shannon Entropy ─────────────────────────────────────────

fn shannon_entropy(s: &str) -> f64 {
    if s.is_empty() { return 0.0; }
    let mut freq = std::collections::HashMap::new();
    for c in s.chars() {
        *freq.entry(c).or_insert(0.0f64) += 1.0;
    }
    let length = s.len() as f64;
    let mut entropy = 0.0;
    for &count in freq.values() {
        let p = count / length;
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }
    entropy
}

fn extract_secret_value(caps: &regex::Captures) -> String {
    for i in (1..caps.len()).rev() {
        if let Some(m) = caps.get(i) {
            if m.as_str().len() > 4 {
                return m.as_str().to_string();
            }
        }
    }
    caps.get(0).map(|m| m.as_str().to_string()).unwrap_or_default()
}

// ─── Main Security Analysis ─────────────────────────────────

pub fn analyze_security(content: &str, _lang: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let secrets = secret_rules();
    let injections = injection_rules();
    let cryptos = crypto_rules();
    let auths = auth_rules();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || is_comment_line(line) { continue; }
        if is_pattern_definition(line) { continue; }

        // Secret Detection
        for rule in &secrets {
            if rule.pattern.is_match(line) {
                if rule.entropy {
                    if let Some(caps) = rule.pattern.captures(line) {
                        let secret_val = extract_secret_value(&caps);
                        if !secret_val.is_empty() && shannon_entropy(&secret_val) < 3.5 {
                            continue;
                        }
                    }
                }
                issues.push(Issue {
                    line: line_num, column: None,
                    issue_type: "hardcoded_secret".to_string(),
                    severity: rule.severity, category: Category::Security,
                    message: rule.msg.to_string(),
                    fix: Some(rule.fix.to_string()),
                    ..Default::default()
                });
            }
        }

        // Injection Detection
        for rule in &injections {
            if rule.pattern.is_match(line) {
                issues.push(Issue {
                    line: line_num, column: None,
                    issue_type: rule.issue_type.to_string(),
                    severity: rule.severity, category: Category::Security,
                    message: rule.msg.to_string(),
                    fix: Some(rule.fix.to_string()),
                    ..Default::default()
                });
            }
        }

        // Crypto Misuse
        for rule in &cryptos {
            if rule.pattern.is_match(line) {
                issues.push(Issue {
                    line: line_num, column: None,
                    issue_type: "insecure_crypto".to_string(),
                    severity: rule.severity, category: Category::Security,
                    message: rule.msg.to_string(),
                    fix: Some(rule.fix.to_string()),
                    ..Default::default()
                });
            }
        }

        // Auth & Info Disclosure
        for rule in &auths {
            if rule.pattern.is_match(line) {
                issues.push(Issue {
                    line: line_num, column: None,
                    issue_type: rule.issue_type.to_string(),
                    severity: rule.severity, category: Category::Security,
                    message: rule.msg.to_string(),
                    fix: Some(rule.fix.to_string()),
                    ..Default::default()
                });
            }
        }
    }

    issues
}
