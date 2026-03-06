<p align="center">
  <pre align="center">
                ▲
               / \
              /   \
             /     \
            /       \
           /_________\
          /           \
         /             \
        /               \
  _____/_________________\_____
  </pre>
</p>

<h1 align="center">🗻 Fuji</h1>
<p align="center">
  <strong>Codebase Intelligence Engine</strong><br>
  <em>Heuristic security scanning, AI-generated code detection, and quality analysis — all from the terminal.</em>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.0-blue" alt="Version">
  <img src="https://img.shields.io/badge/go-1.25.7-00ADD8?logo=go" alt="Go Version">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
  <img src="https://img.shields.io/badge/languages-20+-orange" alt="Languages">
</p>

---

## Table of Contents

- [What is Fuji?](#what-is-fuji)
- [Features at a Glance](#features-at-a-glance)
- [Installation](#installation)
- [Usage](#usage)
  - [Interactive TUI Mode](#interactive-tui-mode)
  - [CLI / Non-Interactive Mode](#cli--non-interactive-mode)
  - [CI Mode](#ci-mode)
- [Analysis Pipeline](#analysis-pipeline)
  - [Phase 1: Directory Walking](#phase-1-directory-walking)
  - [Phase 2: Git History Analysis](#phase-2-git-history-analysis)
  - [Phase 3: Per-File Concurrent Analysis](#phase-3-per-file-concurrent-analysis)
  - [Phase 4: Summary & Propagation](#phase-4-summary--propagation)
- [Analysis Modes](#analysis-modes)
  - [🔒 Security Analysis](#-security-analysis)
  - [🤖 AI Detection](#-ai-detection)
  - [📊 Quality Analysis](#-quality-analysis)
- [Supported Languages](#supported-languages)
- [Architecture](#architecture)
  - [Project Structure](#project-structure)
  - [Package Breakdown](#package-breakdown)
  - [Data Models](#data-models)
- [Output Formats](#output-formats)
- [TUI Interface](#tui-interface)
  - [Screens](#screens)
  - [Keyboard Shortcuts](#keyboard-shortcuts)
  - [Theme & Colors](#theme--colors)
- [Configuration & Thresholds](#configuration--thresholds)
- [How It Works (Technical Deep Dive)](#how-it-works-technical-deep-dive)

---

## What is Fuji?

**Fuji** is a zero-dependency, static codebase intelligence tool written in Go. It analyzes source code repositories to surface three categories of findings:

1. **Security vulnerabilities** — hardcoded secrets, injection risks, crypto misuse, authentication bypasses, and information disclosure.
2. **AI-generated code patterns** — detects fingerprints of LLM-generated code using 12 weighted heuristic signals.
3. **Code quality issues** — cyclomatic complexity, dead code, duplication, deep nesting, magic numbers, naming inconsistencies, and more.

Fuji operates entirely offline with no API calls, no cloud dependencies, and no external services. Everything runs locally through pattern matching, heuristic scoring, and static analysis.

It provides three interfaces:
- An **interactive TUI** (Terminal User Interface) built with Bubble Tea and Lipgloss.
- A **CLI mode** that outputs structured JSON or Markdown reports.
- A **CI mode** with semantic exit codes for pipeline integration.

---

## Features at a Glance

| Feature | Details |
|---|---|
| **Security Scanner** | 25+ rules covering secrets, injections, crypto misuse, auth issues |
| **AI Code Detector** | 12-signal weighted scoring system (0–100%) |
| **Quality Analyzer** | 12 quality checks with configurable thresholds |
| **Language Support** | 20+ languages (Go, Python, Rust, JS, TS, Java, C, C++, Ruby, PHP, and more) |
| **Git Integration** | Commit churn, author tracking, last-modified dates via `go-git` |
| **Concurrent Analysis** | Semaphore-bounded goroutines (8 concurrent workers) |
| **Output Formats** | Interactive TUI, JSON, Markdown, CI summary |
| **Shannon Entropy** | Entropy-gated secret detection to reduce false positives |
| **Issue Severity** | 4 levels: Info, Warning, Error, Critical |
| **Issue Categories** | Security, Quality, AI Pattern, Performance |
| **Clipboard Support** | Copy results via `atotto/clipboard` + OSC52 terminal escape |
| **File Picker** | Native `zenity` file picker integration (Linux) |
| **Zero Config** | No configuration files needed — sensible defaults baked in |

---

## Installation

### From Source

```bash
git clone https://github.com/lichi/fuji.git
cd fuji
go build -o fuji .
```

### Run Directly

```bash
go run .
```

### Move to PATH

```bash
sudo mv fuji /usr/local/bin/
```

### Dependencies

Fuji has the following Go module dependencies:

| Dependency | Purpose |
|---|---|
| `charmbracelet/bubbletea` | Terminal UI framework (Elm-architecture) |
| `charmbracelet/lipgloss` | Terminal styling and layout |
| `alecthomas/chroma/v2` | Syntax highlighting engine |
| `go-git/go-git/v5` | Pure Go Git implementation for history analysis |
| `atotto/clipboard` | System clipboard access |
| `aymanbagabas/go-osc52/v2` | OSC52 terminal clipboard escape sequences |

---

## Usage

### Interactive TUI Mode

```bash
# Launch the home screen
fuji

# Go directly to the analysis menu for a specific folder
fuji /path/to/project

# Analyze current directory
fuji .
```

When launched without arguments, Fuji presents an interactive home screen where you can:
1. **Open** a folder (type a path or use the `Ctrl+B` native file picker)
2. **View Help** (full keyboard shortcut reference)
3. **View History** (placeholder — coming soon)

After selecting a folder, you choose an analysis mode:
- `1` — Security & Vulnerability Scan
- `2` — AI Code Detection
- `3` — Code Quality Analysis

Results are displayed in a scrollable view with severity badges, code context, and progress bars. Press `Enter` to copy results as Markdown to the clipboard.

### CLI / Non-Interactive Mode

```bash
# JSON output
fuji --format json /path/to/project

# Markdown report
fuji --format md /path/to/project
fuji --format markdown .
```

JSON output includes:
- Summary statistics (files analyzed, files flagged, average complexity, total issues)
- Per-file details (path, AI probability, complexity, issues with line numbers and types)
- Only files with issues or AI probability ≥30% are included

Markdown output includes:
- Summary table with all metrics
- Issues grouped by severity (🔴 Critical, 🟠 Error, 🟡 Warning, 🔵 Info)
- AI-suspected files table with visual probability bars
- Generated-by footer

### CI Mode

```bash
fuji --ci /path/to/project
```

CI mode outputs a plain-text summary and uses semantic exit codes:

| Exit Code | Meaning |
|-----------|---------|
| `0` | ✅ PASS — No issues found |
| `1` | ⚠️ WARN — Non-security issues detected |
| `2` | ❌ FAIL — Security issues detected |

Example CI output:
```
🗻 fuji — CI Report
═══════════════════
Files analyzed:   14
Files flagged:    3
Total issues:     12
Security issues:  2
AI-suspected:     1
Avg complexity:   4.3

❌ FAIL — Security issues detected
```

### Other Flags

```bash
fuji --version    # or -v — prints "fuji v0.1.0"
fuji --help       # or -h — prints usage and keyboard shortcuts
```

---

## Analysis Pipeline

Fuji runs a 4-phase analysis pipeline, coordinated by the `Analyzer` struct in `internal/analyzer/analyzer.go`.

### Phase 1: Directory Walking

**File:** `internal/analyzer/walker.go`

The `WalkDirectory` function recursively walks the target directory and builds a file tree:

- **Supported extensions** are mapped to language names (20+ extensions → 20+ languages).
- **Ignored directories** are automatically skipped: `.git`, `node_modules`, `vendor`, `__pycache__`, `.venv`, `venv`, `dist`, `build`, `.idea`, `.vscode`, `target`, `bin`, `obj`.
- **Hidden files/directories** (starting with `.`) are skipped.
- Files are organized into a tree structure (`FileResult` with `Children` and `Parent` pointers).
- Children are sorted alphabetically with directories first.
- First-level directories are auto-expanded in the TUI.
- Line counts are computed during the walk for each supported file.

### Phase 2: Git History Analysis

**File:** `internal/analyzer/git.go`

The `AnalyzeGit` function uses `go-git` to analyze the repository:

- Opens the repository at the root directory using `git.PlainOpen()`.
- Iterates through commit history starting from `HEAD` (up to **500 commits** for performance).
- For each commit, extracts `Stats()` (file-level diffs) to determine:
  - **File churn** — number of commits touching each file.
  - **Authors** — unique list of contributors per file.
  - **Last modified date** — most recent commit timestamp per file.
- Results are stored in a `GitAnalysis` struct with three maps keyed by relative file path.
- If the directory is not a Git repository, returns empty data gracefully (no error).

After the main analysis, `ApplyGitInfo` maps the git data back onto `FileResult` objects using `filepath.Rel` to match paths.

The `HighChurnFiles` utility returns the top N files sorted by commit count (useful for identifying hotspots).

### Phase 3: Per-File Concurrent Analysis

**File:** `internal/analyzer/analyzer.go`

Each file with a recognized language is analyzed concurrently:

- A **semaphore** limits concurrency to **8 goroutines** at a time.
- Progress updates are sent through a buffered channel (`cap=100`) for TUI display.
- For each file, four analysis passes run sequentially:
  1. **Complexity analysis** (`AnalyzeComplexity`)
  2. **AI detection** (`AnalyzeAI`)
  3. **Security scanning** (`AnalyzeSecurity`)
  4. **Quality checks** (`AnalyzeQuality`)
- File content is read once and passed to all analyzers.
- Comment ratio is computed separately via `CommentRatio`.

### Phase 4: Summary & Propagation

After all files are analyzed:

1. **Summary statistics** are computed:
   - Total files analyzed
   - Files flagged (files with any issues)
   - Average cyclomatic complexity
   - Total issue count
   - AI-suspected file count (AI score > 60%)
   - Security issue count

2. **Issue propagation** — issues from child files are propagated up to parent directories in the tree structure (for TUI navigation).

---

## Analysis Modes

### 🔒 Security Analysis

**File:** `internal/analyzer/security.go` (386 lines, 19KB)

The security analyzer scans every line of code against four categories of rules, with context-aware filtering to reduce false positives.

#### Secret Detection (18 rules)

Detects hardcoded secrets, credentials, and API keys:

| Secret Type | Pattern Example | Severity |
|---|---|---|
| AWS Access Key | `AKIA[0-9A-Z]{16}` | 🔴 Critical |
| AWS Secret Key | `aws_secret_access_key = "..."` | 🔴 Critical |
| GCP API Key | `AIza[0-9A-Za-z\-_]{35}` | 🔴 Critical |
| Azure Credential | `azure_key = "..."` | 🔴 Critical |
| Generic API Key | `api_key = "..."` | 🟠 Error |
| Generic Secret/Password | `password = "...", token = "..."` | 🔴 Critical |
| Bearer Token | `Bearer eyJ...` | 🔴 Critical |
| Basic Auth | `Basic dXNlcj...` | 🟠 Error |
| Private Key (RSA/EC/DSA/OpenSSH) | `-----BEGIN PRIVATE KEY-----` | 🔴 Critical |
| PGP Private Key | `-----BEGIN PGP PRIVATE KEY BLOCK-----` | 🔴 Critical |
| Database URL | `postgres://user:pass@host/db` | 🔴 Critical |
| JWT Token | `eyJ...` (3-part base64url) | 🟠 Error |
| JWT Secret | `jwt_secret = "..."` | 🔴 Critical |
| Slack Webhook/Token | `hooks.slack.com/services/...` | 🟠 Error / 🔴 Critical |
| GitHub Token | `ghp_...`, `gho_...` | 🔴 Critical |
| GitLab Token | `glpat-...` | 🔴 Critical |
| Stripe Key | `sk_live_...`, `sk_test_...` | 🔴 Critical |
| SendGrid / Twilio Key | `SG.xxx`, `SK[a-f0-9]{32}` | 🔴 Critical / 🟠 Error |

**Entropy gating:** Generic patterns (API keys, secrets, passwords) use **Shannon entropy** to filter out low-entropy placeholders like `"changeme"` or `"example"`. Only values with entropy ≥ 3.5 bits are flagged.

**Test file downgrade:** Secrets found in test files (`_test.go`, `test_*.py`, etc.) are automatically downgraded to `Info` severity.

#### Injection Detection (10 rules)

| Injection Type | Examples | Severity |
|---|---|---|
| SQL Injection | `fmt.Sprintf("SELECT ... %s")`, string concat in queries, `.raw(f"...")` | 🔴 Critical / 🟠 Error |
| Command Injection | `exec.Command(...+var)`, `os.system(f"...")`, `subprocess(shell=True)` | 🔴 Critical / 🟠 Error |
| XSS | `innerHTML = ... +`, `dangerouslySetInnerHTML`, `template.HTML()` | 🟡 Warning / 🟠 Error |
| Path Traversal | `os.Open(... + userInput)`, `../` sequences, `sendFile(... + var)` | 🟡 Warning / 🟠 Error |
| SSRF | `http.Get(... + var)`, `requests.get(f"...")`, `net.Dial(... + var)` | 🟠 Error |
| Deserialization | `pickle.loads()`, `yaml.load()`, `ObjectInputStream` | 🔴 Critical |

#### Crypto Misuse (10 rules)

| Issue | Detection | Severity |
|---|---|---|
| MD5 usage | `md5.New()`, `crypto/md5`, `hashlib.md5()` | 🟠 Error |
| SHA-1 usage | `sha1.New()` | 🟡 Warning |
| DES/3DES | `crypto/des`, `DES.new` | 🟠 Error |
| RC4 | `rc4.NewCipher` | 🟠 Error |
| Blowfish | `Blowfish...Encrypt` | 🟡 Warning |
| ECB mode | `NewECBEncrypter`, `MODE_ECB` | 🔴 Critical |
| Hardcoded IV/nonce/salt | `iv = []byte{...}` | 🟠 Error |
| Insecure random | `"math/rand"`, `random.randint()` | 🟡 Warning |
| Weak RSA key | `rsa.GenerateKey(..., 1024)` | 🟠 Error |

#### Auth & Info Disclosure (7 rules)

| Issue | Detection | Severity |
|---|---|---|
| Hardcoded admin check | `isAdmin = true` | 🟡 Warning |
| Security TODO | `// TODO: fix auth` | 🟡 Warning |
| CORS wildcard | `cors(...AllowAll...)` | 🟠 Error |
| Sensitive data logged | `fmt.Printf(password)` | 🟠 Error |
| Debug mode enabled | `DEBUG = True` | 🟡 Warning |
| Stack trace exposure | `printStackTrace`, `traceback` | 🟡 Warning |
| Sensitive file reference | `.env`, `.pem`, `.key` usage | 🔵 Info |

#### Context Filtering

The security scanner applies smart filtering:
- **Comment lines** are skipped (detected via `//`, `#`, `/*`, `*`, `--`, `;` prefixes).
- **Pattern definitions** are skipped — lines containing words like `Description`, `Usage`, `MustCompile`, `regexp`, `compile`, `message`, `placeholder`.
- **Test files** are detected via patterns like `_test.go`, `test_*.py`, `.test.`, `.spec.`, `__tests__/`.

---

### 🤖 AI Detection

**File:** `internal/analyzer/ai.go` (825 lines, 23KB)

The AI detector computes a probability score (0–100%) that a file was generated by an LLM, using a **weighted combination of 12 independent signals**:

| # | Signal | Weight | What It Measures |
|---|---|---|---|
| 1 | Token Diversity | 5% | Ratio of unique identifiers to total — low diversity → AI-like |
| 2 | Formatting Consistency | 5% | Indentation alignment + line length variance — high consistency → AI-like |
| 3 | Generic Naming | 8% | Usage of generic names like `data`, `result`, `handler`, `temp` — 26 built-in generic names |
| 4 | Comment Quality | 10% | Comment-to-code ratio — ratio > 50% is strongly AI-like |
| 5 | Function Uniformity | 5% | Variance of function sizes — low variance (< 10) → AI-like |
| 6 | Marker Phrases | 20% | LLM fingerprint phrases in comments (10 patterns) |
| 7 | Error Handling Boilerplate | 5% | Duplicate error handling blocks (language-specific patterns) |
| 8 | Naming Entropy | 5% | Naming convention consistency (camelCase vs snake_case) |
| 9 | Structural Fingerprint | 10% | Section headers, comment-before-code alternation patterns |
| 10 | Line Length Distribution | 5% | Gaussian distribution of line lengths — low skewness → AI-like |
| 11 | Readme-Style Comments | 12% | Comments that explain the obvious (9 patterns) |
| 12 | Repetitive Structure | 10% | Same API call pattern repeated 5+ times |

**Total signal weights: 100%**

#### Marker Phrase Patterns (Signal 6)

These detect LLM fingerprint phrases in comment text (language-agnostic — the comment marker is stripped first):

- `"This function/method/class handles/implements..."` — characteristic of LLM output
- `"Here we implement/create..."` — tutorial prose style
- `"The following code/function..."` — explanatory AI style
- `"Note that..." / "Please note..."` — overly formal tone
- `"We use/need/want this to..."` — tutorial-style narration
- `"For simplicity/clarity/readability..."` — AI hedging pattern
- `"First we... Then we..."` — sequential narration
- `"Make sure to..." / "Ensure that..."` — instructional advisory tone
- `"This is a helper/utility/wrapper..."` — self-describing comment
- `"Returns a... and/or..."` — verbose parameter documentation

#### Readme-Style Comment Patterns (Signal 11)

Comments that explain what the code already says:

- `"Import the necessary packages/modules"` 
- `"Define a new variable/constant/struct"`
- `"Check if the input/value is valid"`
- `"Return the result/response/error"`
- `"Loop through/over the items"`
- `"Handle the error/exception"`
- `"Open/Close/Read/Write the file/connection"`
- Numbered section headers (`"1. Something"`)
- Named utility comments (`"Quick function to..."`, `"Helper to..."`)

#### Repetitive Structure Detection (Signal 12)

Language-specific patterns for repetitive code:

| Language | Patterns Detected |
|---|---|
| Lua | `vim.keymap.set()`, `vim.api.nvim_create_autocmd()`, `vim.cmd()` |
| JavaScript | `addEventListener()`, `document.querySelector()` |
| Python | `self.xxx = ...` |
| Generic (all) | Any structural line pattern repeated 5+ times (strings normalized) |

#### Scoring Thresholds

- **> 60%** AI probability → file is counted as "AI-suspected" in the summary
- **> 50%** → file appears in the AI-suspected list in reports
- Files with fewer than 5 lines are not analyzed (returns 0%)

---

### 📊 Quality Analysis

**File:** `internal/analyzer/quality.go` (637 lines, 20KB)

The quality analyzer performs 12 distinct checks:

#### 1. Cyclomatic Complexity

**File:** `internal/analyzer/complexity.go`

Measures per-function cyclomatic complexity by counting branching constructs:

| Language | Keywords Counted |
|---|---|
| Go | `if`, `else if`, `for`, `switch`, `case`, `select`, `go`, `defer` |
| Python | `if`, `elif`, `for`, `while`, `except`, `with` |
| Rust | `if`, `else if`, `for`, `while`, `match`, `loop` |
| JavaScript / TypeScript | `if`, `else if`, `for`, `while`, `switch`, `case`, `catch` |
| Java | `if`, `else if`, `for`, `while`, `switch`, `case`, `catch` |
| Ruby | `if`, `elsif`, `unless`, `while`, `until`, `for`, `rescue` |
| PHP | `if`, `elseif`, `for`, `foreach`, `while`, `switch`, `case`, `catch` |
| Kotlin | `if`, `else if`, `for`, `while`, `when`, `catch` |
| Swift | `if`, `else if`, `for`, `while`, `switch`, `case`, `catch` |
| Shell | `if`, `elif`, `for`, `while`, `case` |
| Lua | `if`, `elseif`, `for`, `while`, `repeat` |
| Dart | `if`, `else if`, `for`, `while`, `switch`, `case`, `catch` |

Additionally, logical operators `&&` and `||` are counted per occurrence.

**Function detection** uses language-specific regex patterns to identify function declarations and track their start/end lines.

**Thresholds:**

| Threshold | Level |
|---|---|
| Complexity ≥ 10 | 🟡 Warning (`moderate_complexity`) |
| Complexity ≥ 20 | 🟠 Error (`high_complexity`) |

#### 2. Long Functions

| Threshold | Level |
|---|---|
| > 50 lines | 🟡 Warning (`long_function`) |
| > 100 lines AND complexity > 15 | 🟠 Error (`god_function`) |

#### 3. Long Files

| Threshold | Level |
|---|---|
| > 500 lines | 🔵 Info (`long_file`) |

#### 4. Deep Nesting

| Threshold | Level |
|---|---|
| Indentation > 4 levels (16+ spaces / 4+ tabs) | 🟡 Warning (`deep_nesting`) |

#### 5. Long Parameter Lists

| Threshold | Level |
|---|---|
| > 5 parameters per function | 🟡 Warning (`long_param_list`) |

#### 6. Dead Code Detection

Detects unreachable code after `return`, `break`, `continue`, `panic()`, `os.Exit()`, `sys.exit()`, `process.exit()`, and `throw`. Smart enough to:
- Skip closing braces after return statements
- Skip `case`/`default` labels in switch statements
- Detect and ignore returns inside closures/anonymous functions

#### 7. Empty Error Handlers

Detects:
- Empty `catch` blocks (JS/Java/C++)
- Empty `except` blocks (Python — just `pass`)
- Empty `if err != nil {}` blocks (Go)

Severity: 🟠 Error

#### 8. Error Swallowing (Go-specific)

Detects `_ = someFunc()` patterns where return values are discarded.

**Exceptions:** `fmt.Fprintf`, `io.Copy`, `copy()`, and `range` iterations are excluded.

Severity: 🟡 Warning

#### 9. Magic Numbers

Detects numeric literals with 3+ digits that aren't defined as constants.

**Exceptions:**
- HTTP status codes: 100, 200, 201, 204, 301, 302, 400, 401, 403, 404, 500
- Powers of 2: 1024, 2048, 4096, 8192
- Constants/variable definitions
- Comments and imports
- Hex color codes
- Numbers inside template literals

Severity: 🔵 Info

#### 10. Naming Consistency

Detects mixed naming conventions (camelCase vs snake_case) within a single file. Only flags when the minority convention exceeds 20% of identifiers.

**Skipped for:** Go (camelCase + PascalCase is idiomatic), C, C++.

Severity: 🔵 Info

#### 11. Unused Imports (Go-specific)

Detects imported packages whose short names don't appear in the code body (via `name.` pattern matching). Handles both single-line and block imports, as well as aliased imports.

**Exceptions:** `_` (blank) and `.` (dot) imports.

Severity: 🟡 Warning

#### 12. TODO/FIXME/HACK Markers

Detects `TODO:`, `FIXME:`, `HACK:`, `XXX:`, and `BUG:` markers in code.

Severity: 🔵 Info

#### 13. Duplicate Code Blocks

Uses SHA-256 hashing of 6-line sliding windows to detect copy-pasted code blocks. Only non-empty blocks are hashed, and each duplicate pair is reported once.

Severity: 🔵 Info

---

## Supported Languages

Fuji supports analysis of the following file types:

| Extension | Language |
|---|---|
| `.go` | Go |
| `.py` | Python |
| `.rs` | Rust |
| `.js`, `.jsx` | JavaScript |
| `.ts`, `.tsx` | TypeScript |
| `.rb` | Ruby |
| `.java` | Java |
| `.c`, `.h` | C |
| `.cpp`, `.hpp` | C++ |
| `.cs` | C# |
| `.php` | PHP |
| `.sh` | Shell |
| `.lua` | Lua |
| `.dart` | Dart |
| `.kt` | Kotlin |
| `.swift` | Swift |
| `.scala` | Scala |
| `.yaml`, `.yml` | YAML |
| `.json` | JSON |
| `.md` | Markdown |
| `.sql` | SQL |

---

## Architecture

### Project Structure

```
fuji/
├── main.go                          # Entry point, CLI flag parsing, mode routing
├── go.mod                           # Go module definition (github.com/lichi/fuji)
├── go.sum                           # Dependency checksums
├── cmd/                             # (reserved for future subcommands)
└── internal/
    ├── analyzer/
    │   ├── analyzer.go              # Orchestrator — 4-phase pipeline, summary builder
    │   ├── walker.go                # Directory tree walker, language detection
    │   ├── complexity.go            # Cyclomatic complexity per function, per language
    │   ├── security.go              # 45+ security rules: secrets, injections, crypto, auth
    │   ├── ai.go                    # 12-signal AI detection engine
    │   ├── quality.go               # 12 quality checks: duplication, dead code, naming...
    │   └── git.go                   # Git history analysis via go-git
    ├── models/
    │   └── models.go                # All data structures: Issue, FileResult, Summary, etc.
    ├── output/
    │   ├── json.go                  # JSON output formatter
    │   └── markdown.go              # Markdown report generator with tables and bars
    └── tui/
        ├── app.go                   # Bubble Tea model: state, Init, Update, View
        ├── screens.go               # Screen renderers: Home, Help, PathInput, Menu, Loading
        ├── results.go               # Results view: Security, AI, Quality result builders
        └── theme.go                 # Color palette (Noir Fuji), severity badges, progress bars
```

### Package Breakdown

#### `internal/analyzer` (7 files, ~79KB)

The core analysis engine. Each analyzer file exposes pure functions that accept file content (as a string) plus language name and return `[]models.Issue` or score values.

- **`analyzer.go`** — `Analyzer` struct with `Run()` method. Creates the pipeline: Walk → Git → analyzeFiles (concurrent) → buildSummary → propagateIssues. Uses a `chan models.ProgressUpdate` for real-time TUI updates.
- **`walker.go`** — `WalkDirectory()` builds a `*FileResult` tree. Maps 22 file extensions to language names. Ignores 12 common non-source directories. Sorts children (dirs first, then alphabetical).
- **`complexity.go`** — `AnalyzeComplexity()` returns total complexity and `[]FunctionInfo`. Tracks function boundaries using language-specific regex patterns and brace/indent depth.
- **`security.go`** — `AnalyzeSecurity()` / `AnalyzeSecurityWithPath()`. Four rule sets: `secretRules` (18), `injectionRules` (10), `cryptoRules` (10), `authRules` (7). Includes `shannonEntropy()` for entropy gating.
- **`ai.go`** — `AnalyzeAI()` returns a (score, issues) pair. 12 weighted signals combined into a 0–100% probability. Each signal is a pure function with its own scoring logic.
- **`quality.go`** — `AnalyzeQuality()` returns `[]Issue`. 12 independent checks with configurable threshold constants.
- **`git.go`** — `AnalyzeGit()` returns `*GitAnalysis`. `ApplyGitInfo()` maps git data → file results. `HighChurnFiles()` returns top-N most-changed files.

#### `internal/models` (1 file, ~3.6KB)

All shared data types:

- **`Severity`** — enum: Info (0), Warning (1), Error (2), Critical (3). Has `String()` and `Label()` methods.
- **`Category`** — enum: Security (0), Quality (1), AIPattern (2), Performance (3). Has `String()` method.
- **`Issue`** — finding with Line, Column, Type, Severity, Category, Message, Fix. JSON-serializable.
- **`FunctionInfo`** — per-function data: Name, StartLine, EndLine, Complexity, LineCount.
- **`GitInfo`** — per-file git data: CommitCount, LastAuthor, LastModified, Authors.
- **`FileResult`** — complete per-file result: Path, Language, LineCount, AIScore, Complexity, Functions, Issues, GitInfo, CommentRatio. Also contains tree navigation fields (IsDirectory, Children, Parent, Expanded, Depth).
- **`AnalysisSummary`** — aggregate stats: FilesAnalyzed, FilesFlagged, AvgComplexity, TotalIssues, AISuspected, SecurityIssues.
- **`AnalysisResult`** — top-level container: Summary, Files, RootDir.
- **`ProgressUpdate`** — real-time progress: Phase, Progress (0.0–1.0), Message.
- **`FileTreeNode`** — flattened tree node for TUI rendering.

#### `internal/output` (2 files, ~5.3KB)

- **`json.go`** — `WriteJSON()` outputs to stdout. Filters clean files (no issues, AI < 30%). Uses custom `JSONOutput` / `JSONFile` / `JSONIssue` structs for a clean API.
- **`markdown.go`** — `WriteMarkdown()` / `WriteMarkdownToString()` / `WriteMarkdownToWriter()`. Generates a formatted Markdown report with tables, severity grouping, AI probability bars, and a footer. Issues are grouped into Critical → Error → Warning → Info sections.

#### `internal/tui` (4 files, ~39KB)

The interactive terminal interface built on the Bubble Tea architecture:

- **`app.go`** — `App` struct (the Bubble Tea model). 6 screens: Home, Help, PathInput, AnalysisMenu, Results, Loading. Handles keyboard and mouse events. Includes `openFilePicker()` (zenity integration), `runAnalysis()` (background goroutine), and clipboard copy (atotto + OSC52).
- **`screens.go`** — Render functions for Home (ASCII art mountain + "FUJI" block title + buttons), Help (command index with key bindings), PathInput (text input with cursor + browse button), Menu (3 analysis options with descriptions), Loading (progress bar + message).
- **`results.go`** — 661 lines. Three result builders: `buildSecurityResults()` (per-file issue cards with code context), `buildAIResults()` (AI score cards with progress bars and explanations), `buildQualityResults()` (quality score, metrics table, issue breakdown chart, per-file details). Uses `getCodeLine()` to show actual source code at issue locations.
- **`theme.go`** — "Noir Fuji" color palette with 15 colors. `SeverityBadge()` renders `[CRIT]`, `[ERR ]`, `[WARN]`, `[INFO]` with color coding. `ProgressBar()` renders block-character bars.

### Data Models

```
AnalysisResult
├── Summary (AnalysisSummary)
│   ├── FilesAnalyzed: int
│   ├── FilesFlagged: int
│   ├── AvgComplexity: float64
│   ├── TotalIssues: int
│   ├── AISuspected: int
│   └── SecurityIssues: int
├── Files: []*FileResult
│   ├── Path, Language, LineCount
│   ├── AIScore (0-100)
│   ├── Complexity (cyclomatic)
│   ├── CommentRatio
│   ├── Functions: []FunctionInfo
│   │   ├── Name, StartLine, EndLine
│   │   ├── Complexity, LineCount
│   ├── Issues: []Issue
│   │   ├── Line, Column
│   │   ├── Type, Severity, Category
│   │   ├── Message, Fix
│   ├── GitInfo
│   │   ├── CommitCount, LastAuthor
│   │   ├── LastModified, Authors
│   └── Tree fields (IsDirectory, Children, Parent, Expanded, Depth)
└── RootDir: string
```

---

## Output Formats

### JSON

```json
{
  "summary": {
    "files_analyzed": 14,
    "files_flagged": 3,
    "avg_complexity": 4.3,
    "total_issues": 12
  },
  "files": [
    {
      "path": "/path/to/file.go",
      "ai_probability": 0.42,
      "complexity": 8,
      "issues": [
        {
          "line": 15,
          "type": "hardcoded_secret",
          "severity": "critical"
        }
      ]
    }
  ]
}
```

### Markdown

Generates a structured report with:
- Summary table
- Issues grouped by severity (🔴🟠🟡🔵)  
- AI-suspected files with visual progress bars (`████░░░░░░`)
- Generated-by footer with link

### CI Summary

Plain-text report with emoji indicators and semantic exit codes.

---

## TUI Interface

### Screens

| Screen | Description |
|---|---|
| **Home** | ASCII mountain art, "FUJI" block title, three buttons (Open, Help, History) |
| **Path Input** | Text input with cursor, Ctrl+B for native file picker (zenity), Enter to confirm |
| **Analysis Menu** | Three analysis modes with descriptions, keyboard/mouse selection |
| **Loading** | Animated progress bar with "INTELLIGENCE DATA ANALYSIS IN PROGRESS" message |
| **Results** | Scrollable results view with severity badges, code context lines, and progress bars |
| **Help** | Full keyboard shortcut reference organized by screen context |

### Keyboard Shortcuts

#### Home Screen
| Key | Action |
|---|---|
| `o` / `1` / `Enter` | Open folder for analysis |
| `h` / `2` | Show help screen |
| `i` / `3` | View scan history |
| `j` / `k` / `↑` / `↓` | Navigate buttons |
| `Tab` / `Shift+Tab` | Navigate buttons |
| `q` / `Esc` | Quit |

#### Path Input
| Key | Action |
|---|---|
| `Enter` | Confirm path (empty = current directory) |
| `Esc` | Cancel, go back to Home |
| `Backspace` | Delete last character |
| `Ctrl+U` | Clear entire input |
| `Ctrl+B` | Open native file picker (zenity) |

#### Analysis Menu
| Key | Action |
|---|---|
| `1` | Run Security & Vulnerability scan |
| `2` | Run AI Usage detection |
| `3` | Run Code Quality analysis |
| `Enter` | Run focused analysis mode |
| `j` / `k` / `↑` / `↓` | Navigate options |
| `b` / `Esc` | Back to Home |
| `q` | Quit |

#### Results View
| Key | Action |
|---|---|
| `j` / `k` / `↑` / `↓` | Scroll up/down (1 line) |
| `d` / `u` | Page down/up (10 lines) |
| `g` / `G` | Jump to top/bottom |
| Mouse wheel | Scroll up/down (3 lines) |
| `Enter` | Copy results as Markdown to clipboard |
| `b` / `Esc` | Back to Analysis Menu |
| `q` | Quit |

#### Global
| Key | Action |
|---|---|
| `Ctrl+C` | Force quit |
| Mouse click | Click any button |

### Theme & Colors

Fuji uses the **"Noir Fuji"** color palette — a dark, technical theme inspired by OpenCode:

| Color | Hex | Usage |
|---|---|---|
| Background | `#050505` | Deep technical black |
| Surface | `#121212` | Card/panel background |
| Surface Hover | `#1c1c1c` | Hover state |
| Border | `#333333` | Borders and dividers |
| Text Primary | `#e0e0e0` | Main text |
| Text Secondary | `#707070` | Muted descriptions |
| Text Dim | `#404040` | Hints, footers, disabled |
| Accent (Blue) | `#00aaff` | Primary accent, buttons |
| Cyan | `#00ffdd` | High-contrast highlights |
| Blue | `#58a6ff` | Info badges |
| Warning (Amber) | `#ffaa00` | Warning badges |
| Error (Red) | `#ff4444` | Error/Critical badges |
| Success (Green) | `#00ff77` | Pass indicators |
| White | `#ffffff` | Active/focused elements |
| Title | `#ffffff` | Screen titles |
| Tagline | `#808080` | Subtitle text |

---

## Configuration & Thresholds

All thresholds are defined as constants in `internal/analyzer/quality.go`:

| Constant | Value | Description |
|---|---|---|
| `maxFunctionLength` | 50 lines | Warning for long functions |
| `maxFileLength` | 500 lines | Info for long files |
| `complexityWarning` | 10 | Cyclomatic complexity warning threshold |
| `complexityCritical` | 20 | Cyclomatic complexity error threshold |
| `dupMinLines` | 6 | Minimum block size for duplication detection |
| `maxNestingDepth` | 4 | Maximum recommended nesting levels |
| `maxParamCount` | 5 | Maximum recommended parameters per function |
| `godFuncLines` | 100 | "God function" line count threshold |
| `godFuncComplexity` | 15 | "God function" complexity threshold |

AI detection threshold:
| Constant | Value | Location |
|---|---|---|
| AI-suspected threshold | 60% | `internal/analyzer/analyzer.go` (summary) |
| AI report threshold | 50% | `internal/output/markdown.go` (report) |
| Shannon entropy minimum | 3.5 bits | `internal/analyzer/security.go` (secrets) |
| Max git commits | 500 | `internal/analyzer/git.go` |
| Concurrency limit | 8 goroutines | `internal/analyzer/analyzer.go` |
| Progress channel buffer | 100 messages | `internal/analyzer/analyzer.go` |

---

## How It Works (Technical Deep Dive)

### Entry Point (`main.go`)

The `main()` function parses CLI flags by hand (no flag library):
1. Iterates through `os.Args[1:]` looking for `--format`, `--ci`, `--version`/`-v`, `--help`/`-h`.
2. Any non-flag argument is treated as the target directory (defaults to `.`).
3. Routes to one of three modes:
   - **`--format json|md`** or **`--ci`** — non-interactive: validates dir → runs analysis → outputs result.
   - **No flags with path** — opens TUI directly at the Analysis Menu screen.
   - **No flags, no path** — opens TUI at the Home screen.

### Concurrency Model

The analyzer uses a bounded semaphore pattern:
```go
sem := make(chan struct{}, 8) // max 8 goroutines
for _, f := range files {
    wg.Add(1)
    sem <- struct{}{} // blocks when 8 goroutines are in-flight
    go func(f *FileResult) {
        defer wg.Done()
        defer func() { <-sem }()
        // ... analyze file
    }(f)
}
wg.Wait()
```

### Shannon Entropy (Secret Detection)

For generic secret patterns, Fuji computes Shannon entropy:
```
H(s) = -Σ p(c) × log₂(p(c))
```
Where `p(c)` is the frequency of character `c` divided by string length. Values below 3.5 bits are considered low-entropy (likely placeholders like `"changeme"`, `"password123"`) and are not flagged.

### AI Score Computation

Each of the 12 signals produces a value between 0.0 (human-like) and 1.0 (AI-like). The final score:
```
score = Σ (signal_value × signal_weight) × 100
```
is then clamped to [0, 100].

### Duplicate Detection

Uses a sliding window of 6 lines, hashing each window with SHA-256 (first 8 bytes) and grouping by hash to find identical code blocks.

---

<p align="center">
  <strong>Built with Go + Bubble Tea + Lipgloss</strong><br>
  <em>🗻 Fuji — see your code from the summit.</em>
</p>
