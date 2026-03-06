package analyzer

import (
	"math"
	"regexp"
	"strings"

	"github.com/lichi/fuji/internal/models"
)

// ─── Secret Detection ────────────────────────────────────────

type secretRule struct {
	Name     string
	Pattern  *regexp.Regexp
	Severity models.Severity
	Msg      string
	Fix      string
	Entropy  bool // require entropy check
}

var secretRules = []secretRule{
	// Cloud Provider Keys
	{"AWS Access Key", regexp.MustCompile(`AKIA[0-9A-Z]{16}`), models.SeverityCritical,
		"AWS Access Key ID found", "Use environment variables or AWS Secrets Manager", false},
	{"AWS Secret Key", regexp.MustCompile(`(?i)(aws_secret_access_key|aws_secret)\s*[:=]\s*["']([A-Za-z0-9/+=]{40})["']`), models.SeverityCritical,
		"AWS Secret Access Key found", "Use IAM roles or environment variables", true},
	{"GCP API Key", regexp.MustCompile(`AIza[0-9A-Za-z\-_]{35}`), models.SeverityCritical,
		"Google Cloud API key found", "Use service accounts instead of API keys", false},
	{"Azure Key", regexp.MustCompile(`(?i)(azure|subscription)[_-]?(key|secret|id)\s*[:=]\s*["']([^"']{8,})["']`), models.SeverityCritical,
		"Azure credential found", "Use Azure Key Vault or managed identities", true},

	// API Keys & Tokens
	{"Generic API Key", regexp.MustCompile(`(?i)(api[_-]?key|apikey)\s*[:=]\s*["']([^"']{8,})["']`), models.SeverityError,
		"Possible API key in string literal", "Move to environment variable or secrets manager", true},
	{"Generic Secret", regexp.MustCompile(`(?i)(secret|password|passwd|pwd|token|auth_token|access_token)\s*[:=]\s*["']([^"']{8,})["']`), models.SeverityCritical,
		"Possible hardcoded secret", "Use environment variables or a vault", true},
	{"Bearer Token", regexp.MustCompile(`(?i)(bearer\s+)[A-Za-z0-9\-_.~+/]{20,}`), models.SeverityCritical,
		"Hardcoded Bearer token", "Load tokens from secure storage at runtime", false},
	{"Basic Auth", regexp.MustCompile(`(?i)basic\s+[A-Za-z0-9+/=]{20,}`), models.SeverityError,
		"Hardcoded Basic Auth credentials", "Use secure credential storage", false},

	// Private Keys & Certs
	{"Private Key", regexp.MustCompile(`-----BEGIN\s+(RSA\s+|EC\s+|DSA\s+|OPENSSH\s+)?PRIVATE KEY-----`), models.SeverityCritical,
		"Private key embedded in source", "Store private keys in secure files outside source control", false},
	{"PGP Private Key", regexp.MustCompile(`-----BEGIN PGP PRIVATE KEY BLOCK-----`), models.SeverityCritical,
		"PGP private key found", "Never commit PGP keys to source", false},

	// Database Connection Strings
	{"Database URL", regexp.MustCompile(`(?i)(mongodb|postgres|mysql|redis|amqp|mssql)://[^\s"']+@[^\s"']+`), models.SeverityCritical,
		"Database connection string with credentials", "Use environment variables for connection strings", false},
	{"DSN with Password", regexp.MustCompile(`(?i)(user|username)[:=]\w+.*(pass|password)[:=]\w+`), models.SeverityError,
		"Connection string with inline credentials", "Use environment variables", true},

	// JWT
	{"JWT Token", regexp.MustCompile(`eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}`), models.SeverityError,
		"JWT token found in source", "Load JWT tokens dynamically, never hardcode", false},
	{"JWT Secret", regexp.MustCompile(`(?i)(jwt[_-]?secret|signing[_-]?key)\s*[:=]\s*["']([^"']{8,})["']`), models.SeverityCritical,
		"JWT signing secret hardcoded", "Use environment variable for JWT_SECRET", true},

	// Webhook & Slack
	{"Slack Webhook", regexp.MustCompile(`https://hooks\.slack\.com/services/T[A-Z0-9]+/B[A-Z0-9]+/[A-Za-z0-9]+`), models.SeverityError,
		"Slack webhook URL found", "Use environment variable for webhook URLs", false},
	{"Slack Token", regexp.MustCompile(`xox[bpars]-[0-9]{10,}-[0-9]{10,}-[a-zA-Z0-9]{20,}`), models.SeverityCritical,
		"Slack token found", "Use Slack app tokens from environment", false},

	// GitHub/GitLab
	{"GitHub Token", regexp.MustCompile(`gh[pousr]_[A-Za-z0-9_]{36,}`), models.SeverityCritical,
		"GitHub personal access token found", "Use GitHub Apps or environment variables", false},
	{"GitLab Token", regexp.MustCompile(`glpat-[A-Za-z0-9\-_]{20,}`), models.SeverityCritical,
		"GitLab personal access token found", "Use CI/CD variables", false},

	// Stripe
	{"Stripe Key", regexp.MustCompile(`sk_(live|test)_[A-Za-z0-9]{24,}`), models.SeverityCritical,
		"Stripe secret key found", "Use environment variable for Stripe keys", false},
	{"Stripe Publishable", regexp.MustCompile(`pk_(live|test)_[A-Za-z0-9]{24,}`), models.SeverityWarning,
		"Stripe publishable key in source (may be intentional)", "Consider using environment variables", false},

	// SendGrid / Twilio / Mailgun
	{"SendGrid Key", regexp.MustCompile(`SG\.[A-Za-z0-9\-_]{22,}\.[A-Za-z0-9\-_]{22,}`), models.SeverityCritical,
		"SendGrid API key found", "Use environment variable", false},
	{"Twilio Key", regexp.MustCompile(`SK[a-f0-9]{32}`), models.SeverityError,
		"Possible Twilio API key", "Use environment variable", false},
}

// ─── Injection Detection ─────────────────────────────────────

type injectionRule struct {
	Type    string
	Pattern *regexp.Regexp
	Msg     string
	Fix     string
	Sev     models.Severity
}

var injectionRules = []injectionRule{
	// SQL Injection
	{"sql_injection", regexp.MustCompile(`(?i)(?:fmt\.Sprintf|format|f"|%s|%v).*?(?:SELECT|INSERT|UPDATE|DELETE|DROP|ALTER|CREATE|TRUNCATE)\s`),
		"SQL injection — string formatting in SQL query", "Use parameterized queries / prepared statements", models.SeverityCritical},
	{"sql_injection", regexp.MustCompile(`(?i)["']\s*\+\s*\w+\s*\+\s*["'].*?(?:WHERE|AND|OR|SET|VALUES)\s`),
		"SQL injection — string concatenation in query", "Use parameterized queries / prepared statements", models.SeverityCritical},
	{"sql_injection", regexp.MustCompile(`(?i)(?:Query|Exec|Execute|prepare)\s*\(\s*(?:fmt\.Sprintf|"[^"]*"\s*\+)`),
		"SQL injection — dynamic query construction", "Use db.Query with $1/$2 placeholders", models.SeverityCritical},
	{"sql_injection", regexp.MustCompile(`(?i)\.raw\s*\(\s*f?["'].*\{`),
		"SQL injection — raw query with interpolation (ORM bypass)", "Use ORM query builders or parameterized queries", models.SeverityError},

	// Command Injection
	{"command_injection", regexp.MustCompile(`exec\.Command\([^)]*\+`),
		"Command injection — variable concatenated into exec.Command", "Validate/sanitize inputs, use execArgs not string concat", models.SeverityCritical},
	{"command_injection", regexp.MustCompile(`os\.system\([^)]*(?:\+|%|format|f")`),
		"Command injection — dynamic string in os.system()", "Use subprocess with argument list, never shell=True", models.SeverityCritical},
	{"command_injection", regexp.MustCompile(`subprocess\.(?:call|run|Popen)\([^)]*shell\s*=\s*True`),
		"Subprocess with shell=True — command injection risk", "Use shell=False and pass args as list", models.SeverityError},
	{"command_injection", regexp.MustCompile(`(?:child_process\.exec|execSync)\([^)]*\+`),
		"Node.js command injection — variable in exec()", "Use execFile() with argument array instead", models.SeverityCritical},
	{"command_injection", regexp.MustCompile(`\beval\s*\([^)]*(?:\+|format|f"|%s|\$\{)`),
		"eval() with dynamic input — code injection", "Avoid eval(). Use safe alternatives like JSON.parse()", models.SeverityCritical},
	{"command_injection", regexp.MustCompile(`(?i)Runtime\.getRuntime\(\)\.exec\([^)]*\+`),
		"Java command injection — variable in Runtime.exec()", "Use ProcessBuilder with argument list", models.SeverityCritical},

	// XSS
	{"xss", regexp.MustCompile(`(?i)(?:innerHTML|outerHTML|document\.write)\s*(?:=|\()\s*(?:[^"']*\+|\$\{|` + "`" + `)`),
		"XSS — dynamic content injected into DOM", "Use textContent instead of innerHTML, or sanitize input", models.SeverityError},
	{"xss", regexp.MustCompile(`(?i)\.html\(\s*(?:[^"']*\+|\$\{)`),
		"XSS — jQuery .html() with dynamic content", "Use .text() for untrusted content, or sanitize with DOMPurify", models.SeverityError},
	{"xss", regexp.MustCompile(`(?i)dangerouslySetInnerHTML`),
		"React dangerouslySetInnerHTML — XSS risk", "Sanitize with DOMPurify before setting inner HTML", models.SeverityWarning},
	{"xss", regexp.MustCompile(`(?i)template\.HTML\(`),
		"Go template.HTML() bypasses auto-escaping — XSS risk", "Only use template.HTML with trusted, pre-sanitized content", models.SeverityWarning},

	// Path Traversal
	{"path_traversal", regexp.MustCompile(`(?i)(?:os\.(?:Open|ReadFile|Create|Remove)|ioutil\.ReadFile|open)\([^)]*(?:\+|format|Sprintf|f"|%s|\$\{)`),
		"Path traversal — user input in file operation", "Validate path with filepath.Clean() and check it stays within allowed directory", models.SeverityError},
	{"path_traversal", regexp.MustCompile(`(?i)\.\.(?:/|\\\\)`),
		"Path traversal sequence '../' found", "Reject paths containing '..', use filepath.Clean()", models.SeverityWarning},
	{"path_traversal", regexp.MustCompile(`(?i)(?:sendFile|serveFile|send_file|serve_file)\([^)]*(?:\+|format|f")`),
		"Path traversal — dynamic path in file serving", "Whitelist allowed files or use filepath.Clean()", models.SeverityError},

	// SSRF
	{"ssrf", regexp.MustCompile(`(?i)(?:http\.Get|http\.Post|requests\.get|requests\.post|fetch|axios|urllib)\s*\([^)]*(?:\+|format|f"|%s|\$\{)`),
		"SSRF — user-controlled URL in HTTP request", "Validate URLs against allowlist, block internal IPs (10.x, 127.x, 169.254.x)", models.SeverityError},
	{"ssrf", regexp.MustCompile(`(?i)(?:net\.Dial|net\.DialTimeout|socket\.connect)\([^)]*(?:\+|format|f")`),
		"SSRF — dynamic address in network connection", "Validate and allowlist target addresses", models.SeverityError},

	// Deserialization
	{"deserialization", regexp.MustCompile(`(?i)(?:pickle\.loads?|yaml\.(?:load|unsafe_load)|marshal\.load|unserialize|ObjectInputStream)`),
		"Unsafe deserialization — remote code execution risk", "Use safe alternatives: yaml.safe_load(), json instead of pickle", models.SeverityCritical},
	{"deserialization", regexp.MustCompile(`(?i)json\.Unmarshal\([^,]+,\s*(?:interface\{\}|any)`),
		"JSON unmarshaling into interface{} — type confusion risk", "Unmarshal into typed structs for safer deserialization", models.SeverityWarning},
}

// ─── Crypto Misuse ───────────────────────────────────────────

type cryptoRule struct {
	Pattern *regexp.Regexp
	Msg     string
	Fix     string
	Sev     models.Severity
}

var cryptoRules = []cryptoRule{
	// Weak hashing
	{regexp.MustCompile(`(?i)\b(?:md5|MD5)\.(?:New|Sum|Hash|Create|digest)\b`),
		"MD5 is cryptographically broken", "Use SHA-256 or SHA-3", models.SeverityError},
	{regexp.MustCompile(`(?i)\b(?:sha1|SHA1)\.(?:New|Sum|Hash|Create|digest)\b`),
		"SHA-1 is deprecated and vulnerable to collision attacks", "Use SHA-256 or SHA-3", models.SeverityWarning},
	{regexp.MustCompile(`(?i)"crypto/md5"`),
		"MD5 package imported — MD5 is broken for security", "Use crypto/sha256 instead", models.SeverityError},
	{regexp.MustCompile(`(?i)hashlib\.md5\(`),
		"Python MD5 usage", "Use hashlib.sha256() or hashlib.sha3_256()", models.SeverityError},

	// Weak ciphers
	{regexp.MustCompile(`(?i)\b(?:crypto/des|des\.NewCipher|DES\.new|DES_EDE)\b`),
		"DES/3DES is weak encryption", "Use AES-256-GCM", models.SeverityError},
	{regexp.MustCompile(`(?i)\b(?:crypto/rc4|rc4\.NewCipher|ARC4|RC4)\b`),
		"RC4 is broken", "Use AES-256-GCM or ChaCha20-Poly1305", models.SeverityError},
	{regexp.MustCompile(`(?i)\bBlowfish\b.*(?:New|Cipher|Encrypt)`),
		"Blowfish is outdated", "Use AES-256-GCM", models.SeverityWarning},

	// ECB mode
	{regexp.MustCompile(`(?i)(?:NewECBEncrypter|NewECBDecrypter|ECB|MODE_ECB|mode=ECB)`),
		"ECB mode does not provide semantic security — identical blocks produce identical ciphertext",
		"Use CBC with random IV, or better: GCM (authenticated encryption)", models.SeverityCritical},

	// Hardcoded IV/nonce
	{regexp.MustCompile(`(?i)(?:iv|nonce|salt)\s*[:=]\s*(?:\[\]byte\{|bytes\(|b["'])[^})"']*(?:\}|["']\))`),
		"Hardcoded IV/nonce/salt — breaks encryption security", "Generate random IV/nonce using crypto/rand for each operation", models.SeverityError},

	// Insecure random
	{regexp.MustCompile(`"math/rand"`),
		"math/rand is not cryptographically secure", "Use crypto/rand for security-sensitive randomness", models.SeverityWarning},
	{regexp.MustCompile(`(?i)\brandom\.(?:randint|random|choice|shuffle)\b`),
		"Python random module is not secure for crypto", "Use secrets module for security-sensitive randomness", models.SeverityWarning},

	// Weak key size
	{regexp.MustCompile(`(?i)(?:rsa\.GenerateKey|RSA\.generate)\s*\([^,]*,?\s*(?:1024|512)\s*\)`),
		"RSA key size too small (< 2048 bits)", "Use at least 2048-bit RSA keys, prefer 4096", models.SeverityError},
}

// ─── Auth & Info Disclosure ──────────────────────────────────

type authRule struct {
	Type    string
	Pattern *regexp.Regexp
	Msg     string
	Fix     string
	Sev     models.Severity
}

var authRules = []authRule{
	// Auth bypass
	{"auth_bypass", regexp.MustCompile(`(?i)(?:admin|isAdmin|is_admin|role)\s*[:=]=?\s*(?:true|True|1|"admin")`),
		"Hardcoded admin/role check — potential auth bypass", "Implement proper RBAC with database-backed roles", models.SeverityWarning},
	{"auth_bypass", regexp.MustCompile(`(?i)(?://|#)\s*(?:TODO|FIXME|HACK)\s*.*(?:auth|login|password|security|csrf|cors)`),
		"Security-related TODO — incomplete security implementation", "Resolve before deployment", models.SeverityWarning},
	{"auth_bypass", regexp.MustCompile(`(?i)(?:cors|CORS).*(?:\*|AllowAll|allow_all|allowOrigin.*\*)`),
		"CORS allows all origins — security risk", "Restrict CORS to specific trusted origins", models.SeverityError},

	// Information disclosure
	{"info_disclosure", regexp.MustCompile(`(?i)(?:fmt\.Printf|print|console\.log|log\.Print)\s*\(.*(?:password|secret|token|key|credential)`),
		"Sensitive data logged to output", "Never log credentials. Use masked logging", models.SeverityError},
	{"info_disclosure", regexp.MustCompile(`(?i)(?:debug\s*[:=]\s*(?:true|True|1)|DEBUG\s*=\s*(?:True|1|true))`),
		"Debug mode enabled — may expose sensitive information", "Disable debug mode in production", models.SeverityWarning},
	{"info_disclosure", regexp.MustCompile(`(?i)(?:stacktrace|stack_trace|traceback|printStackTrace)\b`),
		"Stack trace exposure — may leak internal paths and info", "Catch exceptions and return generic error messages", models.SeverityWarning},
	{"info_disclosure", regexp.MustCompile(`(?i)\.(?:env|pem|key|pfx|p12|jks)\b.*(?:=|open|read|load)`),
		"Sensitive file reference in source", "Ensure sensitive files are in .gitignore", models.SeverityInfo},
}

// ─── Context Helpers ─────────────────────────────────────────

var (
	commentLineRe = regexp.MustCompile(`^\s*(?://|#|/\*|\*|--|;)`)
	testFileRe    = regexp.MustCompile(`(?i)(?:_test\.go|test_|_test\.py|\.test\.|\.spec\.|__tests__|tests?/)`)
	stringDefRe   = regexp.MustCompile(`(?i)(?:Description|Usage|Help|Example|Pattern|Regex|MustCompile|regexp|compile|message|msg|label|placeholder|hint)`)
)

func isCommentLine(line string) bool {
	return commentLineRe.MatchString(line)
}

func isTestFile(path string) bool {
	return testFileRe.MatchString(path)
}

func isPatternDefinition(line string) bool {
	return stringDefRe.MatchString(line)
}

// ─── Main Analysis Function ─────────────────────────────────

// AnalyzeSecurity scans content for security issues
func AnalyzeSecurity(content string, lang string) []models.Issue {
	return AnalyzeSecurityWithPath(content, lang, "")
}

// AnalyzeSecurityWithPath scans content with filepath context for smarter filtering
func AnalyzeSecurityWithPath(content string, lang string, filePath string) []models.Issue {
	var issues []models.Issue
	lines := strings.Split(content, "\n")
	inTestFile := isTestFile(filePath)

	for i, line := range lines {
		lineNum := i + 1
		trimmed := strings.TrimSpace(line)

		// Skip empty lines and comments
		if len(trimmed) == 0 || isCommentLine(line) {
			continue
		}

		// Skip lines that are pattern/regex definitions (meta — defining rules, not vulnerable code)
		if isPatternDefinition(line) {
			continue
		}

		// === Secret Detection ===
		for _, rule := range secretRules {
			if rule.Pattern.MatchString(line) {
				// Entropy gate for generic patterns
				if rule.Entropy {
					matches := rule.Pattern.FindStringSubmatch(line)
					secretVal := extractSecretValue(matches)
					if secretVal != "" && shannonEntropy(secretVal) < 3.5 {
						continue // Low entropy — probably placeholder like "changeme"
					}
				}

				sev := rule.Severity
				if inTestFile {
					sev = models.SeverityInfo // downgrade in test files
				}

				issues = append(issues, models.Issue{
					Line:     lineNum,
					Type:     "hardcoded_secret",
					Severity: sev,
					Category: models.CategorySecurity,
					Message:  rule.Msg,
					Fix:      rule.Fix,
				})
			}
		}

		// === Injection Detection ===
		for _, rule := range injectionRules {
			if rule.Pattern.MatchString(line) {
				issues = append(issues, models.Issue{
					Line:     lineNum,
					Type:     rule.Type,
					Severity: rule.Sev,
					Category: models.CategorySecurity,
					Message:  rule.Msg,
					Fix:      rule.Fix,
				})
			}
		}

		// === Crypto Misuse ===
		for _, rule := range cryptoRules {
			if rule.Pattern.MatchString(line) {
				issues = append(issues, models.Issue{
					Line:     lineNum,
					Type:     "insecure_crypto",
					Severity: rule.Sev,
					Category: models.CategorySecurity,
					Message:  rule.Msg,
					Fix:      rule.Fix,
				})
			}
		}

		// === Auth & Info Disclosure ===
		for _, rule := range authRules {
			if rule.Pattern.MatchString(line) {
				issues = append(issues, models.Issue{
					Line:     lineNum,
					Type:     rule.Type,
					Severity: rule.Sev,
					Category: models.CategorySecurity,
					Message:  rule.Msg,
					Fix:      rule.Fix,
				})
			}
		}
	}

	return issues
}

// extractSecretValue tries to pull the actual secret from regex matches
func extractSecretValue(matches []string) string {
	if len(matches) == 0 {
		return ""
	}
	// Return the last capture group (usually the secret value)
	for i := len(matches) - 1; i >= 1; i-- {
		if len(matches[i]) > 4 {
			return matches[i]
		}
	}
	return matches[0]
}

// shannonEntropy calculates the Shannon entropy of a string
func shannonEntropy(s string) float64 {
	if len(s) == 0 {
		return 0
	}

	freq := make(map[rune]float64)
	for _, c := range s {
		freq[c]++
	}

	entropy := 0.0
	length := float64(len(s))
	for _, count := range freq {
		p := count / length
		if p > 0 {
			entropy -= p * math.Log2(p)
		}
	}
	return entropy
}
