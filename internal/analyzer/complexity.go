package analyzer

import (
	"regexp"
	"strings"

	"github.com/lichi/fuji/internal/models"
)

// branchKeywords maps language to branching keywords for complexity counting
var branchKeywords = map[string][]string{
	"Go":         {"if ", "else if ", "for ", "switch ", "case ", "select ", "go ", "defer "},
	"Python":     {"if ", "elif ", "for ", "while ", "except ", "with "},
	"Rust":       {"if ", "else if ", "for ", "while ", "match ", "loop "},
	"JavaScript": {"if ", "else if ", "for ", "while ", "switch ", "case ", "catch "},
	"TypeScript": {"if ", "else if ", "for ", "while ", "switch ", "case ", "catch "},
	"Java":       {"if ", "else if ", "for ", "while ", "switch ", "case ", "catch "},
	"C":          {"if ", "else if ", "for ", "while ", "switch ", "case "},
	"C++":        {"if ", "else if ", "for ", "while ", "switch ", "case ", "catch "},
	"Ruby":       {"if ", "elsif ", "unless ", "while ", "until ", "for ", "rescue "},
	"PHP":        {"if ", "elseif ", "for ", "foreach ", "while ", "switch ", "case ", "catch "},
	"Shell":      {"if ", "elif ", "for ", "while ", "case "},
	"Lua":        {"if ", "elseif ", "for ", "while ", "repeat "},
	"Dart":       {"if ", "else if ", "for ", "while ", "switch ", "case ", "catch "},
	"Kotlin":     {"if ", "else if ", "for ", "while ", "when ", "catch "},
	"Swift":      {"if ", "else if ", "for ", "while ", "switch ", "case ", "catch "},
}

var logicalOps = []string{"&&", "||"}

// funcPatterns matches function declarations per language
var funcPatterns = map[string]*regexp.Regexp{
	"Go":         regexp.MustCompile(`^func\s+(\(.*?\)\s+)?(\w+)\s*\(`),
	"Python":     regexp.MustCompile(`^\s*def\s+(\w+)\s*\(`),
	"Rust":       regexp.MustCompile(`^\s*(pub\s+)?fn\s+(\w+)\s*[<(]`),
	"JavaScript": regexp.MustCompile(`^\s*(function\s+(\w+)|(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?(?:function|\())`),
	"TypeScript": regexp.MustCompile(`^\s*(function\s+(\w+)|(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?(?:function|\())`),
	"Java":       regexp.MustCompile(`^\s*(?:public|private|protected|static|\s)*\s+\w+\s+(\w+)\s*\(`),
	"Ruby":       regexp.MustCompile(`^\s*def\s+(\w+)`),
	"PHP":        regexp.MustCompile(`^\s*(?:public|private|protected|static|\s)*\s*function\s+(\w+)\s*\(`),
	"Lua":        regexp.MustCompile(`(?:^|\s)(?:local\s+)?function\s+(\w+)\s*\(`),
	"Dart":       regexp.MustCompile(`^\s*(?:static\s+)?\w+\s+(\w+)\s*\(`),
	"Kotlin":     regexp.MustCompile(`^\s*(?:fun|suspend\s+fun)\s+(\w+)\s*\(`),
	"Swift":      regexp.MustCompile(`^\s*(?:func|static\s+func|class\s+func)\s+(\w+)\s*\(`),
}

// AnalyzeComplexity computes cyclomatic complexity per function
func AnalyzeComplexity(content string, lang string) (int, []models.FunctionInfo) {
	lines := strings.Split(content, "\n")
	keywords := branchKeywords[lang]
	if keywords == nil {
		// Fallback
		keywords = branchKeywords["Go"]
	}

	funcPat := funcPatterns[lang]
	var functions []models.FunctionInfo
	totalComplexity := 1 // base complexity

	var currentFunc *models.FunctionInfo
	braceDepth := 0
	funcBraceStart := 0

	for i, line := range lines {
		trimmed := strings.TrimSpace(line)
		lineNum := i + 1

		// Check for function start
		if funcPat != nil {
			matches := funcPat.FindStringSubmatch(trimmed)
			if len(matches) > 0 {
				// Save previous function
				if currentFunc != nil {
					currentFunc.EndLine = lineNum - 1
					currentFunc.LineCount = currentFunc.EndLine - currentFunc.StartLine + 1
					functions = append(functions, *currentFunc)
				}

				name := extractFuncName(matches, lang)
				currentFunc = &models.FunctionInfo{
					Name:       name,
					StartLine:  lineNum,
					Complexity: 1,
				}
				funcBraceStart = braceDepth
			}
		}

		// Count braces for scope tracking
		braceDepth += strings.Count(trimmed, "{") - strings.Count(trimmed, "}")

		// Count complexity
		for _, kw := range keywords {
			if strings.Contains(trimmed, kw) {
				totalComplexity++
				if currentFunc != nil {
					currentFunc.Complexity++
				}
			}
		}

		for _, op := range logicalOps {
			count := strings.Count(trimmed, op)
			totalComplexity += count
			if currentFunc != nil {
				currentFunc.Complexity += count
			}
		}

		// Check if function ended (brace depth returned)
		if currentFunc != nil && braceDepth <= funcBraceStart && i > 0 {
			// For Python/Ruby, use indentation heuristic
			if lang == "Python" || lang == "Ruby" {
				if len(trimmed) > 0 && !strings.HasPrefix(line, " ") && !strings.HasPrefix(line, "\t") && lineNum > currentFunc.StartLine+1 {
					currentFunc.EndLine = lineNum - 1
					currentFunc.LineCount = currentFunc.EndLine - currentFunc.StartLine + 1
					functions = append(functions, *currentFunc)
					currentFunc = nil
				}
			}
			// For Lua, use 'end' keyword
			if lang == "Lua" && trimmed == "end" {
				currentFunc.EndLine = lineNum
				currentFunc.LineCount = currentFunc.EndLine - currentFunc.StartLine + 1
				functions = append(functions, *currentFunc)
				currentFunc = nil
			}
		}
	}

	// Close last function
	if currentFunc != nil {
		currentFunc.EndLine = len(lines)
		currentFunc.LineCount = currentFunc.EndLine - currentFunc.StartLine + 1
		functions = append(functions, *currentFunc)
	}

	_ = funcBraceStart
	return totalComplexity, functions
}

func extractFuncName(matches []string, lang string) string {
	switch lang {
	case "Go":
		if len(matches) > 2 && matches[2] != "" {
			return matches[2]
		}
		if len(matches) > 1 {
			return matches[1]
		}
	case "Rust":
		if len(matches) > 2 && matches[2] != "" {
			return matches[2]
		}
	case "JavaScript", "TypeScript":
		for i := len(matches) - 1; i >= 1; i-- {
			if matches[i] != "" {
				return matches[i]
			}
		}
	default:
		for i := 1; i < len(matches); i++ {
			if matches[i] != "" {
				return matches[i]
			}
		}
	}
	return "unknown"
}
