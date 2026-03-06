package analyzer

import (
	"crypto/sha256"
	"encoding/hex"
	"regexp"
	"strconv"
	"strings"

	"github.com/lichi/fuji/internal/models"
)

// Quality thresholds
const (
	maxFunctionLength  = 50
	maxFileLength      = 500
	complexityWarning  = 10
	complexityCritical = 20
	dupMinLines        = 6
	maxNestingDepth    = 4
	maxParamCount      = 5
	godFuncLines       = 100
	godFuncComplexity  = 15
)

// AnalyzeQuality checks code quality issues — comprehensive
func AnalyzeQuality(content string, lang string, complexity int, functions []models.FunctionInfo) []models.Issue {
	var issues []models.Issue
	lines := strings.Split(content, "\n")

	// ── Complexity warnings ──
	for _, fn := range functions {
		if fn.Complexity >= complexityCritical {
			issues = append(issues, models.Issue{
				Line:     fn.StartLine,
				Type:     "high_complexity",
				Severity: models.SeverityError,
				Category: models.CategoryQuality,
				Message:  "Function '" + fn.Name + "' has cyclomatic complexity " + strconv.Itoa(fn.Complexity) + " (critical threshold: " + strconv.Itoa(complexityCritical) + ")",
				Fix:      "Break this function into smaller, focused functions. Extract conditional logic into helper functions.",
			})
		} else if fn.Complexity >= complexityWarning {
			issues = append(issues, models.Issue{
				Line:     fn.StartLine,
				Type:     "moderate_complexity",
				Severity: models.SeverityWarning,
				Category: models.CategoryQuality,
				Message:  "Function '" + fn.Name + "' has cyclomatic complexity " + strconv.Itoa(fn.Complexity) + " (warning threshold: " + strconv.Itoa(complexityWarning) + ")",
				Fix:      "Consider refactoring — extract branches into helper functions or use early returns.",
			})
		}
	}

	// ── Long function warnings ──
	for _, fn := range functions {
		if fn.LineCount > godFuncLines && fn.Complexity > godFuncComplexity {
			issues = append(issues, models.Issue{
				Line:     fn.StartLine,
				Type:     "god_function",
				Severity: models.SeverityError,
				Category: models.CategoryQuality,
				Message:  "God function '" + fn.Name + "' — " + strconv.Itoa(fn.LineCount) + " lines with complexity " + strconv.Itoa(fn.Complexity),
				Fix:      "This function does too much. Split into focused functions with single responsibilities.",
			})
		} else if fn.LineCount > maxFunctionLength {
			issues = append(issues, models.Issue{
				Line:     fn.StartLine,
				Type:     "long_function",
				Severity: models.SeverityWarning,
				Category: models.CategoryQuality,
				Message:  "Function '" + fn.Name + "' is " + strconv.Itoa(fn.LineCount) + " lines (recommended max: " + strconv.Itoa(maxFunctionLength) + ")",
				Fix:      "Break into smaller functions. Functions should ideally fit on one screen.",
			})
		}
	}

	// ── Long file warning ──
	if len(lines) > maxFileLength {
		issues = append(issues, models.Issue{
			Line:     1,
			Type:     "long_file",
			Severity: models.SeverityInfo,
			Category: models.CategoryQuality,
			Message:  "File is " + strconv.Itoa(len(lines)) + " lines (recommended max: " + strconv.Itoa(maxFileLength) + ")",
			Fix:      "Split into multiple files by responsibility. Group related functions together.",
		})
	}

	// ── Deep nesting ──
	issues = append(issues, checkDeepNesting(lines, lang)...)

	// ── Long parameter lists ──
	issues = append(issues, checkLongParams(lines, lang, functions)...)

	// ── Dead code detection ──
	issues = append(issues, checkDeadCode(lines, lang)...)

	// ── Empty error handlers ──
	issues = append(issues, checkEmptyErrorHandlers(lines, lang)...)

	// ── Error swallowing ──
	issues = append(issues, checkErrorSwallowing(lines, lang)...)

	// ── Magic numbers ──
	issues = append(issues, checkMagicNumbers(lines, lang)...)

	// ── Naming consistency ──
	issues = append(issues, checkNamingConsistency(lines, lang)...)

	// ── Unused imports (Go-specific) ──
	if lang == "Go" {
		issues = append(issues, checkUnusedImports(lines)...)
	}

	// ── TODO/FIXME/HACK markers ──
	issues = append(issues, checkTodoMarkers(lines)...)

	// ── Duplicate code blocks ──
	issues = append(issues, checkDuplication(lines)...)

	// ── Test coverage estimation ──
	// (This is done at file-set level in analyzer.go, not per-file)

	return issues
}

// ─── Deep Nesting ───────────────────────────────────────────

func checkDeepNesting(lines []string, lang string) []models.Issue {
	var issues []models.Issue
	reported := make(map[int]bool)

	for i, line := range lines {
		indent := measureIndent(line)
		// Tab = 4 spaces
		nestLevel := indent / 4
		if lang == "Python" || lang == "Ruby" {
			nestLevel = indent / 4
		}

		if nestLevel > maxNestingDepth && !reported[nestLevel] {
			reported[nestLevel] = true
			issues = append(issues, models.Issue{
				Line:     i + 1,
				Type:     "deep_nesting",
				Severity: models.SeverityWarning,
				Category: models.CategoryQuality,
				Message:  "Code nested " + strconv.Itoa(nestLevel) + " levels deep (max recommended: " + strconv.Itoa(maxNestingDepth) + ")",
				Fix:      "Use early returns (guard clauses) to reduce nesting. Extract nested logic into helper functions.",
			})
		}
	}
	return issues
}

func measureIndent(line string) int {
	indent := 0
	for _, ch := range line {
		if ch == '\t' {
			indent += 4
		} else if ch == ' ' {
			indent++
		} else {
			break
		}
	}
	return indent
}

// ─── Long Parameter Lists ───────────────────────────────────

func checkLongParams(lines []string, lang string, functions []models.FunctionInfo) []models.Issue {
	var issues []models.Issue
	paramRe := regexp.MustCompile(`\(([^)]+)\)`)

	for _, fn := range functions {
		if fn.StartLine <= 0 || fn.StartLine > len(lines) {
			continue
		}
		funcLine := lines[fn.StartLine-1]
		matches := paramRe.FindStringSubmatch(funcLine)
		if len(matches) > 1 {
			params := strings.Split(matches[1], ",")
			count := 0
			for _, p := range params {
				trimmed := strings.TrimSpace(p)
				if len(trimmed) > 0 && trimmed != "" {
					count++
				}
			}
			if count > maxParamCount {
				issues = append(issues, models.Issue{
					Line:     fn.StartLine,
					Type:     "long_param_list",
					Severity: models.SeverityWarning,
					Category: models.CategoryQuality,
					Message:  "Function '" + fn.Name + "' has " + strconv.Itoa(count) + " parameters (max recommended: " + strconv.Itoa(maxParamCount) + ")",
					Fix:      "Group related parameters into a struct/object. Consider using options pattern or builder.",
				})
			}
		}
	}
	return issues
}

// ─── Dead Code Detection ────────────────────────────────────

func checkDeadCode(lines []string, lang string) []models.Issue {
	var issues []models.Issue

	// Detect unreachable code after return/break/continue/panic/os.Exit
	unreachableRe := regexp.MustCompile(`^\s*(return\b|break\b|continue\b|panic\(|os\.Exit\(|sys\.exit\(|process\.exit\(|throw\b)`)
	closeBraceRe := regexp.MustCompile(`^\s*[})\]]?\s*$`)
	closureRe := regexp.MustCompile(`func\s*\(`)

	for i := 0; i < len(lines)-1; i++ {
		trimmed := strings.TrimSpace(lines[i])
		if !unreachableRe.MatchString(trimmed) {
			continue
		}

		// Skip return statements inside closures/anonymous functions.
		// Look backwards for a func( on the same or preceding lines
		// without a matching closing brace.
		if isInsideClosure(lines, i, closureRe) {
			continue
		}

		// Check next non-empty line
		for j := i + 1; j < len(lines); j++ {
			nextTrimmed := strings.TrimSpace(lines[j])
			if len(nextTrimmed) == 0 {
				continue
			}
			if closeBraceRe.MatchString(nextTrimmed) {
				break // closing brace is fine
			}
			// Check if it's a case/default label
			if strings.HasPrefix(nextTrimmed, "case ") || strings.HasPrefix(nextTrimmed, "default:") {
				break
			}
			// It's unreachable code
			issues = append(issues, models.Issue{
				Line:     j + 1,
				Type:     "dead_code",
				Severity: models.SeverityWarning,
				Category: models.CategoryQuality,
				Message:  "Unreachable code after " + strings.TrimSpace(lines[i]),
				Fix:      "Remove this unreachable code or restructure the control flow.",
			})
			break
		}
	}

	return issues
}

// isInsideClosure checks if line i is inside a closure/anonymous function
// by looking for an unmatched func( in the preceding lines.
func isInsideClosure(lines []string, i int, closureRe *regexp.Regexp) bool {
	braces := 0
	for k := i; k >= 0; k-- {
		line := lines[k]
		braces += strings.Count(line, "}") - strings.Count(line, "{")
		// If we find a func( and the brace depth indicates we're inside it
		if closureRe.MatchString(line) && braces <= 0 {
			// Make sure it's not a top-level func declaration (those start at column 0)
			trimmed := strings.TrimSpace(line)
			if !strings.HasPrefix(trimmed, "func ") || strings.Contains(trimmed, "= func(") || strings.Contains(trimmed, "(func(") {
				return true
			}
		}
	}
	return false
}

// ─── Empty Error Handlers ───────────────────────────────────

func checkEmptyErrorHandlers(lines []string, lang string) []models.Issue {
	var issues []models.Issue

	for i := 0; i < len(lines)-1; i++ {
		trimmed := strings.TrimSpace(lines[i])

		isEmpty := false
		handlerType := ""

		switch {
		case strings.Contains(trimmed, "catch") && strings.HasSuffix(trimmed, "{"):
			// catch (err) { } or catch { }
			handlerType = "catch block"
			isEmpty = isBlockEmpty(lines, i+1)
		case strings.Contains(trimmed, "except") && strings.HasSuffix(trimmed, ":"):
			// except Exception as e:
			handlerType = "except block"
			isEmpty = isPythonBlockEmpty(lines, i+1)
		case lang == "Go" && strings.Contains(trimmed, "if err != nil") && strings.HasSuffix(trimmed, "{"):
			handlerType = "error handler"
			isEmpty = isBlockEmpty(lines, i+1)
		}

		if isEmpty && handlerType != "" {
			issues = append(issues, models.Issue{
				Line:     i + 1,
				Type:     "empty_error_handler",
				Severity: models.SeverityError,
				Category: models.CategoryQuality,
				Message:  "Empty " + handlerType + " — errors silently swallowed",
				Fix:      "At minimum, log the error. Consider returning it or wrapping with context.",
			})
		}
	}

	return issues
}

func isBlockEmpty(lines []string, startIdx int) bool {
	if startIdx >= len(lines) {
		return false
	}
	for i := startIdx; i < len(lines); i++ {
		trimmed := strings.TrimSpace(lines[i])
		if trimmed == "}" {
			return true // only closing brace means empty
		}
		if len(trimmed) > 0 {
			return false
		}
	}
	return false
}

func isPythonBlockEmpty(lines []string, startIdx int) bool {
	if startIdx >= len(lines) {
		return false
	}
	trimmed := strings.TrimSpace(lines[startIdx])
	return trimmed == "pass" || len(trimmed) == 0
}

// ─── Error Swallowing ───────────────────────────────────────

func checkErrorSwallowing(lines []string, lang string) []models.Issue {
	var issues []models.Issue

	if lang != "Go" {
		return issues
	}

	// Detect _ = someFunc() pattern (ignoring errors)
	swallowRe := regexp.MustCompile(`^\s*_\s*(?:,\s*_\s*)?=\s*\w+`)
	// But exclude _ = fmt.Fprintf (common and OK) and range iterations
	okSwallowRe := regexp.MustCompile(`(?i)_\s*=\s*(?:fmt\.|io\.Copy|copy\(|range\s)`)

	for i, line := range lines {
		if swallowRe.MatchString(line) && !okSwallowRe.MatchString(line) {
			issues = append(issues, models.Issue{
				Line:     i + 1,
				Type:     "error_swallowing",
				Severity: models.SeverityWarning,
				Category: models.CategoryQuality,
				Message:  "Return value discarded with _ — possible error swallowing",
				Fix:      "Handle the error: check it, wrap it, or explicitly document why it's safe to ignore.",
			})
		}
	}
	return issues
}

// ─── Magic Numbers ──────────────────────────────────────────

func checkMagicNumbers(lines []string, lang string) []models.Issue {
	var issues []models.Issue
	magicRe := regexp.MustCompile(`\b(\d{3,})\b`)
	// Exceptions: common numbers, array indexes, etc.
	okNumbers := map[string]bool{
		"100": true, "200": true, "201": true, "204": true,
		"301": true, "302": true, "400": true, "401": true,
		"403": true, "404": true, "500": true, // HTTP status codes
		"1000": true, "1024": true, "2048": true, "4096": true, "8192": true, // Powers
		"1000000": true, // Million
	}
	constLineRe := regexp.MustCompile(`(?i)^\s*(?:const|var|let|final|static|#define|=\s*\d)`)
	hexColorRe := regexp.MustCompile(`#[0-9a-fA-F]{3,8}`)

	reported := make(map[string]bool)

	for i, line := range lines {
		trimmed := strings.TrimSpace(line)

		// Skip constant definitions, comments, imports, and hex color strings
		if constLineRe.MatchString(trimmed) ||
			strings.HasPrefix(trimmed, "//") ||
			strings.HasPrefix(trimmed, "#") ||
			strings.HasPrefix(trimmed, "import") ||
			hexColorRe.MatchString(line) {
			continue
		}

		// Skip lines that are inside string literals containing digits
		// (e.g., regex patterns, format strings)
		if strings.Contains(line, "`") && strings.Count(line, "`")%2 == 0 {
			continue
		}

		matches := magicRe.FindAllString(line, -1)
		for _, m := range matches {
			if okNumbers[m] || reported[m] {
				continue
			}
			reported[m] = true
			issues = append(issues, models.Issue{
				Line:     i + 1,
				Type:     "magic_number",
				Severity: models.SeverityInfo,
				Category: models.CategoryQuality,
				Message:  "Magic number " + m + " — consider defining as a named constant",
				Fix:      "Extract to a named constant for readability: const someName = " + m,
			})
		}
	}
	return issues
}

// ─── Naming Consistency ─────────────────────────────────────

func checkNamingConsistency(lines []string, lang string) []models.Issue {
	var issues []models.Issue

	// Don't check languages where mixed conventions are normal
	if lang == "Go" || lang == "C" || lang == "C++" {
		// Go uses camelCase for unexported, PascalCase for exported — both are fine
		return issues
	}

	identRe := regexp.MustCompile(`\b([a-z][a-zA-Z0-9]+_[a-zA-Z0-9]+)\b`)   // camel_Snake
	camelRe := regexp.MustCompile(`\b([a-z][a-z0-9]*[A-Z][a-zA-Z0-9]*)\b`) // camelCase
	snakeRe := regexp.MustCompile(`\b([a-z][a-z0-9]*_[a-z0-9_]+)\b`)       // snake_case

	camelCount := 0
	snakeCount := 0

	for _, line := range lines {
		camelCount += len(camelRe.FindAllString(line, -1))
		snakeCount += len(snakeRe.FindAllString(line, -1))
	}

	total := camelCount + snakeCount
	if total < 10 {
		return issues // not enough data
	}

	minor := camelCount
	minorStyle := "camelCase"
	majorStyle := "snake_case"
	if snakeCount < camelCount {
		minor = snakeCount
		minorStyle = "snake_case"
		majorStyle = "camelCase"
	}

	if float64(minor)/float64(total) > 0.2 {
		// Mixed convention
		// Find first occurrence of minority style
		var searchRe *regexp.Regexp
		if minorStyle == "camelCase" {
			searchRe = camelRe
		} else {
			searchRe = identRe
		}

		for i, line := range lines {
			if searchRe.MatchString(line) {
				issues = append(issues, models.Issue{
					Line:     i + 1,
					Type:     "naming_inconsistency",
					Severity: models.SeverityInfo,
					Category: models.CategoryQuality,
					Message:  "Mixed naming conventions — file uses mostly " + majorStyle + " but also contains " + minorStyle,
					Fix:      "Pick one convention and use it consistently throughout the file.",
				})
				break
			}
		}
	}

	return issues
}

// ─── Unused Imports ─────────────────────────────────────────

func checkUnusedImports(lines []string) []models.Issue {
	var issues []models.Issue
	inImport := false
	importStart := 0

	importRe := regexp.MustCompile(`^\s*"([^"]+)"`)
	aliasRe := regexp.MustCompile(`^\s*(\w+)\s+"([^"]+)"`)

	for i, line := range lines {
		trimmed := strings.TrimSpace(line)

		if trimmed == "import (" {
			inImport = true
			importStart = i
			continue
		}
		if inImport && trimmed == ")" {
			inImport = false
			continue
		}
		if strings.HasPrefix(trimmed, "import \"") {
			matches := regexp.MustCompile(`import\s+"([^"]+)"`).FindStringSubmatch(trimmed)
			if len(matches) > 1 {
				pkg := matches[1]
				parts := strings.Split(pkg, "/")
				name := parts[len(parts)-1]
				body := strings.Join(lines[i+1:], "\n")
				if !strings.Contains(body, name+".") && !strings.HasPrefix(name, "_") {
					issues = append(issues, models.Issue{
						Line:     i + 1,
						Type:     "unused_import",
						Severity: models.SeverityWarning,
						Category: models.CategoryQuality,
						Message:  "Import '" + pkg + "' may be unused",
						Fix:      "Remove unused imports to keep code clean.",
					})
				}
			}
			continue
		}

		if inImport {
			var pkg, name string
			if m := aliasRe.FindStringSubmatch(line); len(m) > 2 {
				name = m[1]
				pkg = m[2]
			} else if m := importRe.FindStringSubmatch(line); len(m) > 1 {
				pkg = m[1]
				parts := strings.Split(pkg, "/")
				name = parts[len(parts)-1]
			} else {
				continue
			}

			if name == "_" || name == "." {
				continue
			}

			afterImports := strings.Join(lines[importStart+10:], "\n")
			if !strings.Contains(afterImports, name+".") {
				issues = append(issues, models.Issue{
					Line:     i + 1,
					Type:     "unused_import",
					Severity: models.SeverityWarning,
					Category: models.CategoryQuality,
					Message:  "Import '" + pkg + "' may be unused",
					Fix:      "Remove unused imports to keep code clean.",
				})
			}
		}
	}

	return issues
}

// ─── TODO/FIXME/HACK Markers ────────────────────────────────

func checkTodoMarkers(lines []string) []models.Issue {
	var issues []models.Issue
	todoRe := regexp.MustCompile(`(?i)\b(TODO|FIXME|HACK|XXX|BUG)\b[:\s](.*)`)

	for i, line := range lines {
		if m := todoRe.FindStringSubmatch(line); len(m) > 2 {
			issues = append(issues, models.Issue{
				Line:     i + 1,
				Type:     "todo_marker",
				Severity: models.SeverityInfo,
				Category: models.CategoryQuality,
				Message:  strings.ToUpper(m[1]) + ": " + strings.TrimSpace(m[2]),
				Fix:      "Resolve this TODO before merging to main branch.",
			})
		}
	}
	return issues
}

// ─── Duplicate Code Blocks ──────────────────────────────────

func checkDuplication(lines []string) []models.Issue {
	var issues []models.Issue
	if len(lines) < dupMinLines*2 {
		return issues
	}

	hashes := make(map[string]int)
	reportedLines := make(map[int]bool)

	for i := 0; i <= len(lines)-dupMinLines; i++ {
		block := make([]string, dupMinLines)
		empty := true
		for j := 0; j < dupMinLines; j++ {
			block[j] = strings.TrimSpace(lines[i+j])
			if len(block[j]) > 0 {
				empty = false
			}
		}
		if empty {
			continue
		}

		combined := strings.Join(block, "\n")
		h := sha256.Sum256([]byte(combined))
		hash := hex.EncodeToString(h[:8])

		if firstLine, exists := hashes[hash]; exists {
			if !reportedLines[i+1] && !reportedLines[firstLine] {
				issues = append(issues, models.Issue{
					Line:     i + 1,
					Type:     "code_duplication",
					Severity: models.SeverityInfo,
					Category: models.CategoryQuality,
					Message:  "Duplicate code block (also at line " + strconv.Itoa(firstLine) + ")",
					Fix:      "Extract duplicated code into a shared function.",
				})
				reportedLines[i+1] = true
			}
		} else {
			hashes[hash] = i + 1
		}
	}

	return issues
}

// itoa removed — use strconv.Itoa from stdlib instead.
