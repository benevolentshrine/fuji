package analyzer

import (
	"math"
	"regexp"
	"strings"
	"unicode"

	"github.com/lichi/fuji/internal/models"
)

// ─── Generic Names (expanded) ────────────────────────────────

var genericNames = map[string]bool{
	"data": true, "result": true, "temp": true, "tmp": true,
	"handler": true, "process": true, "execute": true, "handle": true,
	"item": true, "element": true, "value": true, "input": true,
	"output": true, "response": true, "request": true, "obj": true,
	"val": true, "res": true, "req": true, "err": true,
	"ctx": true, "cfg": true, "info": true, "manager": true,
	"service": true, "helper": true, "utils": true, "util": true,
}

// ─── Comment Detection (language-aware) ──────────────────────

// isCommentLineAI detects if a line is a comment in ANY supported language
func isCommentLineAI(line string) bool {
	trimmed := strings.TrimSpace(line)
	return strings.HasPrefix(trimmed, "//") ||
		strings.HasPrefix(trimmed, "#") ||
		strings.HasPrefix(trimmed, "--") || // Lua, SQL, Haskell
		strings.HasPrefix(trimmed, "/*") ||
		strings.HasPrefix(trimmed, "* ") ||
		strings.HasPrefix(trimmed, "'''") ||
		strings.HasPrefix(trimmed, `"""`)
}

// extractCommentText extracts the text after the comment marker
func extractCommentText(line string) string {
	trimmed := strings.TrimSpace(line)
	// Try each prefix in order of specificity
	for _, prefix := range []string{"//", "--", "#"} {
		if strings.HasPrefix(trimmed, prefix) {
			return strings.TrimSpace(trimmed[len(prefix):])
		}
	}
	return trimmed
}

// ─── AI Marker Phrases (language-agnostic) ───────────────────

// These detect LLM fingerprint phrases in comments regardless of comment syntax.
// We match on the comment TEXT, not the comment marker.
var markerTextPatterns = []struct {
	Pattern *regexp.Regexp
	Desc    string
}{
	{regexp.MustCompile(`(?i)^This\s+(?:function|method|class|struct|module|package|script|file|block|section|component)\s+(?:is\s+(?:used|responsible|designed)|handles|implements|provides|creates|processes|validates|performs|manages|initializes|returns|takes|accepts|defines|sets\s+up|configures)`),
		"Comment starts with 'This function/module handles/implements...' — characteristic of LLM output"},
	{regexp.MustCompile(`(?i)^Here\s+we\s+(?:implement|define|create|handle|process|check|validate|perform|initialize|set\s+up|configure|load|import)`),
		"Comment starts with 'Here we implement/create...' — reading like tutorial prose"},
	{regexp.MustCompile(`(?i)^The\s+following\s+(?:function|code|method|block|section|config|configuration|module)\s+(?:is|will|does|handles|implements|configures|defines|sets)`),
		"Comment says 'The following code/function...' — explanatory style typical of AI"},
	{regexp.MustCompile(`(?i)^(?:Note|Please note|Important)\s*(?::|that)\s`),
		"Comment uses 'Note that...' / 'Please note...' — overly formal AI style"},
	{regexp.MustCompile(`(?i)^We\s+(?:use|need|want|can|should|must|have)\s+(?:to|this|a|an|the)\s`),
		"Comment uses 'We use this to...' / 'We need to...' — tutorial-style narration"},
	{regexp.MustCompile(`(?i)^(?:For|In)\s+(?:simplicity|brevity|clarity|readability|maintainability|safety|security|performance)\s*[,.]`),
		"Comment justifies with 'For simplicity/clarity...' — AI hedging pattern"},
	{regexp.MustCompile(`(?i)^(?:First|Next|Then|Finally|After that|Now)\s*[,.]?\s*(?:we|let's|I|you)`),
		"Sequential narration in comments — 'First we... Then we...' — AI tutorial style"},
	{regexp.MustCompile(`(?i)^(?:Make sure|Ensure|Don't forget|Remember)\s+(?:to|that)\s`),
		"Instructional comment — 'Make sure to...' — AI advisory tone"},
	{regexp.MustCompile(`(?i)^This\s+is\s+(?:a|an|the)\s+(?:helper|utility|wrapper|convenience|simple|basic|main|entry|default)\s`),
		"Self-describing comment — 'This is a helper/utility...'"},
	{regexp.MustCompile(`(?i)^(?:Returns?|Accepts?|Takes?|Receives?|Expects?)\s+(?:a|an|the)\s+\w+\s+(?:and|or|that|which|containing)`),
		"Verbose parameter documentation in line comment"},
	{regexp.MustCompile(`(?i)^(?:Configure|Setup|Initialize|Register|Load|Import|Define|Declare)\s+(?:the\s+|our\s+|all\s+)?(?:necessary|required|needed|essential|core|main|primary|default)`),
		"Setup narration — 'Configure the necessary...' — AI instructional voice"},
}

// ─── Readme-style Comments (language-agnostic) ──────────────

// These detect comments that explain the obvious
var readmeTextPatterns = []*regexp.Regexp{
	regexp.MustCompile(`(?i)^(?:Import|Require)\s+(?:the\s+)?(?:necessary|required|needed)\s+(?:packages?|modules?|libraries?|dependencies)`),
	regexp.MustCompile(`(?i)^(?:Define|Declare|Create|Set|Initialize)\s+(?:a\s+|an\s+|the\s+)?(?:new\s+)?(?:variable|constant|struct|class|function|method|array|map|slice|list|object|table|config)\s`),
	regexp.MustCompile(`(?i)^(?:Check|Verify|Validate)\s+(?:if|that|whether)\s+(?:the\s+)?(?:input|value|data|result|error|user|param|file|path)`),
	regexp.MustCompile(`(?i)^(?:Return|Send|Print|Log|Output)\s+(?:the\s+)?(?:result|response|error|data|value|message|output)`),
	regexp.MustCompile(`(?i)^(?:Loop|Iterate)\s+(?:through|over|across)\s+(?:the\s+|all\s+|each\s+)?`),
	regexp.MustCompile(`(?i)^(?:Handle|Catch|Process)\s+(?:the\s+|any\s+)?(?:error|exception|panic|failure)`),
	regexp.MustCompile(`(?i)^(?:Open|Close|Read|Write)\s+(?:the\s+|a\s+)?(?:file|connection|database|stream|socket)`),
	// Numbered section headers — very AI
	regexp.MustCompile(`(?i)^\d+\.\s+[A-Z][a-z]+\s`),
	// Labeled sections — "Commands for running", "Quick function to", etc.
	regexp.MustCompile(`(?i)^(?:Commands?\s+for|Quick\s+function\s+to|Helper\s+(?:function|to)|Utility\s+(?:function|to)|Wrapper\s+for)`),
}

// ─── Comment Pattern Matching ────────────────────────────────

var commentPatterns = map[string]*regexp.Regexp{
	"Go":         regexp.MustCompile(`^\s*(//|/\*)`),
	"Python":     regexp.MustCompile(`^\s*(#|"""|''')`),
	"Rust":       regexp.MustCompile(`^\s*(//|/\*)`),
	"JavaScript": regexp.MustCompile(`^\s*(//|/\*)`),
	"TypeScript": regexp.MustCompile(`^\s*(//|/\*)`),
	"Java":       regexp.MustCompile(`^\s*(//|/\*)`),
	"Ruby":       regexp.MustCompile(`^\s*#`),
	"PHP":        regexp.MustCompile(`^\s*(//|/\*|#)`),
	"Shell":      regexp.MustCompile(`^\s*#`),
	"Lua":        regexp.MustCompile(`^\s*--`),
	"Dart":       regexp.MustCompile(`^\s*(//|/\*)`),
	"Kotlin":     regexp.MustCompile(`^\s*(//|/\*)`),
	"Swift":      regexp.MustCompile(`^\s*(//|/\*)`),
	"SQL":        regexp.MustCompile(`^\s*--`),
	"Markdown":   regexp.MustCompile(`^\s*$`), // no real "comments" in markdown
}

var docCommentPattern = regexp.MustCompile(`^\s*(//\s+\w+|#\s+\w+|--\s+\w+|/\*\*|\s+\*\s+@|"""|'''|///\s+)`)

// ─── Error Handling Boilerplate ──────────────────────────────

var errorBoilerplatePatterns = map[string]*regexp.Regexp{
	"Go":         regexp.MustCompile(`if\s+err\s*!=\s*nil\s*\{`),
	"Python":     regexp.MustCompile(`except\s+(?:Exception|BaseException|\w+Error)\s+as\s+\w+:`),
	"JavaScript": regexp.MustCompile(`\.catch\s*\(\s*(?:err|error|e)\s*=>`),
	"Java":       regexp.MustCompile(`catch\s*\(\s*(?:Exception|Throwable|\w+Exception)\s+\w+\s*\)`),
}

// ─── Repetitive Structure Patterns ──────────────────────────

// These detect highly repetitive code structures that AI produces
var repetitiveStructures = map[string][]*regexp.Regexp{
	"Lua": {
		regexp.MustCompile(`vim\.keymap\.set\(`),        // Repetitive keymap definitions
		regexp.MustCompile(`vim\.api\.nvim_create_autocmd\(`), // Repetitive autocmds
		regexp.MustCompile(`vim\.cmd\(`),                // Repetitive vim commands
	},
	"JavaScript": {
		regexp.MustCompile(`addEventListener\(`),
		regexp.MustCompile(`document\.querySelector\(`),
	},
	"Python": {
		regexp.MustCompile(`self\.\w+\s*=\s*`),
	},
}

// ─── Main AI Analysis ───────────────────────────────────────

// AnalyzeAI computes an AI probability score (0-100) and generates issues
func AnalyzeAI(content string, lang string, functions []models.FunctionInfo) (float64, []models.Issue) {
	var issues []models.Issue
	lines := strings.Split(content, "\n")

	if len(lines) < 5 {
		return 0, issues // too small to analyze
	}

	// Signal scores (0.0 = human, 1.0 = AI-like)
	type signal struct {
		name   string
		score  float64
		weight float64
	}

	var signals []signal

	// 1. Token diversity (low = AI-like)
	divScore := tokenDiversity(content)
	signals = append(signals, signal{"token_diversity", divScore, 0.05})

	// 2. Formatting consistency (high = AI-like)
	fmtScore := formattingConsistency(lines)
	signals = append(signals, signal{"formatting", fmtScore, 0.05})

	// 3. Generic naming
	genScore, genIssues := genericNaming(content, lang)
	issues = append(issues, genIssues...)
	signals = append(signals, signal{"generic_naming", genScore, 0.08})

	// 4. Comment quality (over-commenting = AI)
	cmtScore := commentQuality(lines, lang, functions)
	signals = append(signals, signal{"comment_quality", cmtScore, 0.10})

	// 5. Function uniformity
	uniScore := functionUniformity(functions)
	signals = append(signals, signal{"function_uniformity", uniScore, 0.05})

	// 6. Marker phrases (strongest signal)
	mkrScore, mkrIssues := markerPhraseAnalysis(lines)
	issues = append(issues, mkrIssues...)
	signals = append(signals, signal{"marker_phrases", mkrScore, 0.20})

	// 7. Error handling boilerplate
	errScore := errorBoilerplateScore(lines, lang)
	signals = append(signals, signal{"error_boilerplate", errScore, 0.05})

	// 8. Variable naming entropy (consistent = AI)
	varScore := namingEntropyScore(content)
	signals = append(signals, signal{"naming_entropy", varScore, 0.05})

	// 9. Structural fingerprint (function-comment alternation OR section headers)
	strScore := structuralFingerprint(lines, lang)
	signals = append(signals, signal{"structural_pattern", strScore, 0.10})

	// 10. Line length distribution (Gaussian = AI)
	lenScore := lineLengthDistribution(lines)
	signals = append(signals, signal{"line_length_dist", lenScore, 0.05})

	// 11. Readme-style comments (explaining the obvious)
	rdmScore, rdmIssues := readmeCommentScore(lines)
	issues = append(issues, rdmIssues...)
	signals = append(signals, signal{"readme_comments", rdmScore, 0.12})

	// 12. Repetitive structure (same API call pattern repeated)
	repScore := repetitiveStructureScore(lines, lang)
	signals = append(signals, signal{"repetitive_structure", repScore, 0.10})

	// Weighted combination
	aiProb := 0.0
	for _, s := range signals {
		aiProb += s.score * s.weight
	}
	aiProb *= 100

	if aiProb > 100 {
		aiProb = 100
	}
	if aiProb < 0 {
		aiProb = 0
	}

	return aiProb, issues
}

// ─── Signal Implementations ─────────────────────────────────

func tokenDiversity(content string) float64 {
	words := extractIdentifiers(content)
	if len(words) == 0 {
		return 0
	}
	unique := make(map[string]bool)
	for _, w := range words {
		unique[strings.ToLower(w)] = true
	}
	ratio := float64(len(unique)) / float64(len(words))
	if ratio > 0.6 {
		return 0.0
	}
	if ratio < 0.2 {
		return 1.0
	}
	return 1.0 - (ratio / 0.6)
}

func extractIdentifiers(content string) []string {
	re := regexp.MustCompile(`\b[a-zA-Z_][a-zA-Z0-9_]*\b`)
	return re.FindAllString(content, -1)
}

func formattingConsistency(lines []string) float64 {
	if len(lines) < 10 {
		return 0
	}
	indents := make(map[int]int)
	for _, line := range lines {
		if len(strings.TrimSpace(line)) == 0 {
			continue
		}
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
		indents[indent]++
	}

	tabAligned := 0
	total := 0
	for indent, count := range indents {
		total += count
		if indent%4 == 0 || indent%2 == 0 {
			tabAligned += count
		}
	}
	if total == 0 {
		return 0
	}

	lengths := make([]int, 0)
	for _, line := range lines {
		if len(strings.TrimSpace(line)) > 0 {
			lengths = append(lengths, len(line))
		}
	}
	lengthVariance := variance(lengths)
	lengthScore := 0.0
	if lengthVariance < 100 {
		lengthScore = 1.0 - (lengthVariance / 100)
	}

	alignScore := float64(tabAligned) / float64(total)
	if alignScore > 0.95 {
		alignScore = 1.0
	} else {
		alignScore = 0.0
	}

	return (alignScore*0.5 + lengthScore*0.5)
}

func variance(nums []int) float64 {
	if len(nums) == 0 {
		return 0
	}
	sum := 0.0
	for _, n := range nums {
		sum += float64(n)
	}
	mean := sum / float64(len(nums))
	varSum := 0.0
	for _, n := range nums {
		diff := float64(n) - mean
		varSum += diff * diff
	}
	return varSum / float64(len(nums))
}

func genericNaming(content string, lang string) (float64, []models.Issue) {
	var issues []models.Issue
	identifiers := extractIdentifiers(content)
	if len(identifiers) == 0 {
		return 0, issues
	}
	genericCount := 0
	seen := make(map[string]bool)
	lines := strings.Split(content, "\n")

	for _, id := range identifiers {
		lower := strings.ToLower(id)
		if genericNames[lower] && !seen[lower] {
			genericCount++
			seen[lower] = true
			for i, line := range lines {
				if strings.Contains(line, id) {
					issues = append(issues, models.Issue{
						Line:     i + 1,
						Type:     "generic_naming",
						Severity: models.SeverityInfo,
						Category: models.CategoryAIPattern,
						Message:  "Generic identifier '" + id + "' — common in AI-generated code",
					})
					break
				}
			}
		}
	}

	ratio := float64(genericCount) / float64(len(seen)+1)
	if ratio > 0.5 {
		return 1.0, issues
	}
	return ratio * 2, issues
}

func commentQuality(lines []string, lang string, functions []models.FunctionInfo) float64 {
	commentLines := 0
	codeLines := 0

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if len(trimmed) == 0 {
			continue
		}
		if isCommentLineAI(line) {
			commentLines++
		} else {
			codeLines++
		}
	}
	if codeLines == 0 {
		return 0
	}

	commentRatio := float64(commentLines) / float64(codeLines)

	// High comment ratio + many comments = AI-like
	if commentRatio > 0.5 {
		return 0.9
	}
	if commentRatio > 0.3 {
		return 0.6
	}
	if commentRatio > 0.2 {
		return 0.3
	}
	return 0.0
}

func functionUniformity(functions []models.FunctionInfo) float64 {
	if len(functions) < 3 {
		return 0
	}
	sizes := make([]int, len(functions))
	for i, f := range functions {
		sizes[i] = f.LineCount
	}
	v := variance(sizes)
	if v < 10 {
		return 0.8
	}
	if v < 25 {
		return 0.4
	}
	return 0.0
}

// ─── Marker Phrases (language-agnostic) ─────────────────────

func markerPhraseAnalysis(lines []string) (float64, []models.Issue) {
	var issues []models.Issue
	hitCount := 0
	commentCount := 0

	for i, line := range lines {
		if isCommentLineAI(line) {
			commentCount++
			commentText := extractCommentText(line)
			if len(commentText) < 5 {
				continue
			}
			for _, mp := range markerTextPatterns {
				if mp.Pattern.MatchString(commentText) {
					hitCount++
					issues = append(issues, models.Issue{
						Line:     i + 1,
						Type:     "ai_marker_phrase",
						Severity: models.SeverityInfo,
						Category: models.CategoryAIPattern,
						Message:  mp.Desc,
					})
					break // only one marker per line
				}
			}
		}
	}

	if commentCount < 3 {
		return 0, issues
	}

	// If >10% of comments contain marker phrases, very AI-like
	ratio := float64(hitCount) / float64(commentCount)
	if ratio > 0.15 {
		return 1.0, issues
	}
	if ratio > 0.08 {
		return 0.7, issues
	}
	if ratio > 0.03 {
		return 0.4, issues
	}
	if hitCount > 0 {
		return 0.2, issues
	}
	return 0.0, issues
}

// ─── Error Handling Boilerplate ──────────────────────────────

func errorBoilerplateScore(lines []string, lang string) float64 {
	pat := errorBoilerplatePatterns[lang]
	if pat == nil {
		return 0
	}

	var errorBlocks []string
	for i, line := range lines {
		if pat.MatchString(line) {
			block := line
			for j := 1; j <= 3 && i+j < len(lines); j++ {
				block += "\n" + strings.TrimSpace(lines[i+j])
			}
			errorBlocks = append(errorBlocks, block)
		}
	}

	if len(errorBlocks) < 3 {
		return 0
	}

	blockCounts := make(map[string]int)
	for _, b := range errorBlocks {
		blockCounts[b]++
	}

	maxDup := 0
	for _, count := range blockCounts {
		if count > maxDup {
			maxDup = count
		}
	}

	dupRatio := float64(maxDup) / float64(len(errorBlocks))
	if dupRatio > 0.7 {
		return 0.9
	}
	if dupRatio > 0.5 {
		return 0.5
	}
	return 0.0
}

// ─── Naming Entropy ─────────────────────────────────────────

func namingEntropyScore(content string) float64 {
	identifiers := extractIdentifiers(content)
	if len(identifiers) < 10 {
		return 0
	}

	camelCount := 0
	snakeCount := 0
	for _, id := range identifiers {
		if len(id) < 3 {
			continue
		}
		hasUnderscore := strings.Contains(id, "_")
		hasUpper := false
		hasLower := false
		for _, r := range id {
			if unicode.IsUpper(r) {
				hasUpper = true
			}
			if unicode.IsLower(r) {
				hasLower = true
			}
		}

		if hasUnderscore && hasLower {
			snakeCount++
		} else if hasUpper && hasLower && !hasUnderscore {
			camelCount++
		}
	}

	total := camelCount + snakeCount
	if total < 5 {
		return 0
	}

	dominant := camelCount
	if snakeCount > dominant {
		dominant = snakeCount
	}

	consistency := float64(dominant) / float64(total)
	if consistency > 0.95 {
		return 0.6
	}
	if consistency > 0.85 {
		return 0.3
	}
	return 0.0
}

// ─── Structural Fingerprint ─────────────────────────────────

func structuralFingerprint(lines []string, lang string) float64 {
	// Count section headers (numbered: "-- 1. Something", "// 2. Something")
	sectionHeaderRe := regexp.MustCompile(`^\s*(?://|--|#)\s*\d+\.\s+[A-Z]`)
	sectionCount := 0

	// Count comment-before-code transitions
	transitions := 0
	lastWasComment := false
	codeBlockCount := 0

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if len(trimmed) == 0 {
			continue
		}

		if sectionHeaderRe.MatchString(line) {
			sectionCount++
		}

		isComment := isCommentLineAI(line)
		isCode := !isComment

		if isCode && lastWasComment {
			transitions++
		}
		if isCode {
			codeBlockCount++
		}

		lastWasComment = isComment
	}

	score := 0.0

	// Numbered sections (very AI-like)
	if sectionCount >= 3 {
		score = 0.9
	} else if sectionCount >= 2 {
		score = 0.6
	}

	// High comment-before-code ratio
	if codeBlockCount > 5 {
		ratio := float64(transitions) / float64(codeBlockCount)
		if ratio > 0.4 {
			commentScore := math.Min(1.0, ratio)
			if commentScore > score {
				score = commentScore
			}
		}
	}

	return score
}

// ─── Line Length Distribution ────────────────────────────────

func lineLengthDistribution(lines []string) float64 {
	lengths := make([]int, 0)
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if len(trimmed) > 0 {
			lengths = append(lengths, len(trimmed))
		}
	}
	if len(lengths) < 20 {
		return 0
	}

	v := variance(lengths)
	mean := 0.0
	for _, l := range lengths {
		mean += float64(l)
	}
	mean /= float64(len(lengths))

	stddev := math.Sqrt(v)
	if stddev == 0 {
		return 0.8
	}

	skewSum := 0.0
	for _, l := range lengths {
		skewSum += math.Pow((float64(l)-mean)/stddev, 3)
	}
	skewness := math.Abs(skewSum / float64(len(lengths)))

	if skewness < 0.3 {
		return 0.7
	}
	if skewness < 0.7 {
		return 0.3
	}
	return 0.0
}

// ─── Readme-Style Comments (language-agnostic) ──────────────

func readmeCommentScore(lines []string) (float64, []models.Issue) {
	var issues []models.Issue
	hitCount := 0
	commentCount := 0

	for i, line := range lines {
		if isCommentLineAI(line) {
			commentCount++
			commentText := extractCommentText(line)
			if len(commentText) < 5 {
				continue
			}
			for _, pat := range readmeTextPatterns {
				if pat.MatchString(commentText) {
					hitCount++
					issues = append(issues, models.Issue{
						Line:     i + 1,
						Type:     "readme_comment",
						Severity: models.SeverityInfo,
						Category: models.CategoryAIPattern,
						Message:  "Comment explains obvious code — typical of AI-generated output",
					})
					break
				}
			}
		}
	}

	if commentCount < 2 {
		return 0, issues
	}

	ratio := float64(hitCount) / float64(commentCount)
	if ratio > 0.20 {
		return 1.0, issues
	}
	if ratio > 0.10 {
		return 0.6, issues
	}
	if hitCount > 0 {
		return 0.3, issues
	}
	return 0.0, issues
}

// ─── Repetitive Structure Score ─────────────────────────────

func repetitiveStructureScore(lines []string, lang string) float64 {
	patterns := repetitiveStructures[lang]
	if len(patterns) == 0 {
		// Generic: detect any line pattern repeated 5+ times
		return genericRepetitionScore(lines)
	}

	totalMatches := 0
	for _, pat := range patterns {
		count := 0
		for _, line := range lines {
			if pat.MatchString(line) {
				count++
			}
		}
		if count >= 5 {
			totalMatches += count
		}
	}

	if totalMatches == 0 {
		return genericRepetitionScore(lines)
	}

	ratio := float64(totalMatches) / float64(len(lines))
	if ratio > 0.15 {
		return 1.0
	}
	if ratio > 0.08 {
		return 0.7
	}
	if ratio > 0.04 {
		return 0.4
	}
	return 0.0
}

func genericRepetitionScore(lines []string) float64 {
	// Normalize lines and count occurrences
	normalized := make(map[string]int)
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if len(trimmed) < 15 {
			continue
		}
		// Strip variable names — keep structure
		structure := regexp.MustCompile(`"[^"]*"`).ReplaceAllString(trimmed, `""`)
		structure = regexp.MustCompile(`'[^']*'`).ReplaceAllString(structure, `''`)
		normalized[structure]++
	}

	// Check if any structural pattern repeated 5+ times
	highRepeat := 0
	for _, count := range normalized {
		if count >= 5 {
			highRepeat += count
		}
	}

	if highRepeat == 0 {
		return 0
	}

	ratio := float64(highRepeat) / float64(len(lines))
	if ratio > 0.15 {
		return 0.8
	}
	if ratio > 0.08 {
		return 0.5
	}
	return 0.2
}

// ─── CommentRatio (used by quality) ─────────────────────────

func CommentRatio(content string, lang string) float64 {
	lines := strings.Split(content, "\n")
	commentLines := 0
	codeLines := 0
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if len(trimmed) == 0 {
			continue
		}
		if isCommentLineAI(line) {
			commentLines++
		} else {
			codeLines++
		}
	}
	if codeLines == 0 {
		return 0
	}
	return float64(commentLines) / float64(codeLines+commentLines)
}

func isUpper(s string) bool {
	for _, r := range s {
		if !unicode.IsUpper(r) && unicode.IsLetter(r) {
			return false
		}
	}
	return true
}
