package tui

import (
	"fmt"
	"path/filepath"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/lichi/fuji/internal/analyzer"
	"github.com/lichi/fuji/internal/models"
)

// ─── Results Shell ───────────────────────────────────────────

func (a *App) renderResults() string {
	w := a.width
	h := a.height

	allLines := a.buildResultLines()
	a.scrollMax = len(allLines) - h + 6
	if a.scrollMax < 0 {
		a.scrollMax = 0
	}
	if a.scrollOffset > a.scrollMax {
		a.scrollOffset = a.scrollMax
	}

	// ── Header ──
	var header []string

	modeTitle := a.analysisModeTitle()
	header = append(header, SectionHeader(modeTitle, w))

	// Path info bar
	pathStr := truncateStr(a.selectedPath, 65)
	pathLine := lipgloss.NewStyle().Foreground(ColorTextDim).Render("  Folder: ") +
		lipgloss.NewStyle().Foreground(ColorTextSecondary).Render(pathStr)
	header = append(header, pathLine)
	header = append(header, Divider(w))

	// ── Footer ──
	var footer []string
	footer = append(footer, Divider(w))

	a.buttons = nil
	scrollInfo := ""
	if a.scrollMax > 0 {
		pct := float64(a.scrollOffset) / float64(a.scrollMax) * 100
		scrollInfo = lipgloss.NewStyle().Foreground(ColorTextDim).
			Render(fmt.Sprintf("%.0f%%", pct))
	}

	navHint := lipgloss.NewStyle().Foreground(ColorTextDim).
		Render("  [b] ← Back   [Enter] Copy   [q] Quit")
	if a.statusMsg != "" {
		navHint = lipgloss.NewStyle().Foreground(ColorSuccess).Bold(true).
			Render("  ✓ " + a.statusMsg)
	}
	footerLine := navHint +
		strings.Repeat(" ", max(1, w-lipgloss.Width(navHint)-lipgloss.Width(scrollInfo)-4)) +
		scrollInfo + "  "
	footer = append(footer, footerLine)

	bw := lipgloss.Width(navHint)
	a.buttons = append(a.buttons, Button{
		Label: "Back", Key: "b",
		X: 2, Y: h - 1, Width: bw,
	})

	// ── Viewport ──
	viewH := h - len(header) - len(footer)
	if viewH < 1 {
		viewH = 1
	}

	start := a.scrollOffset
	end := start + viewH
	if end > len(allLines) {
		end = len(allLines)
	}
	if start > len(allLines) {
		start = len(allLines)
	}

	visible := allLines[start:end]
	for len(visible) < viewH {
		visible = append(visible, "")
	}

	var output []string
	output = append(output, header...)
	output = append(output, visible...)
	output = append(output, footer...)

	return strings.Join(output, "\n")
}

func (a *App) analysisModeTitle() string {
	switch a.analysisMode {
	case AnalysisSecurity:
		return "SECURITY & VULNERABILITY REPORT"
	case AnalysisAI:
		return "AI CODE DETECTION REPORT"
	case AnalysisQuality:
		return "CODE QUALITY REPORT"
	default:
		return "ANALYSIS REPORT"
	}
}

func (a *App) buildResultLines() []string {
	if a.result == nil {
		return []string{"", "  No results available."}
	}

	switch a.analysisMode {
	case AnalysisSecurity:
		return a.buildSecurityResults()
	case AnalysisAI:
		return a.buildAIResults()
	case AnalysisQuality:
		return a.buildQualityResults()
	default:
		return []string{"", "  Unknown analysis mode."}
	}
}

// ─── Security Results ────────────────────────────────────────

func (a *App) buildSecurityResults() []string {
	var lines []string

	// Count by severity
	totalSec := 0
	critCount := 0
	errCount := 0
	warnCount := 0
	infoCount := 0
	for _, f := range a.result.Files {
		for _, issue := range f.Issues {
			if issue.Category == models.CategorySecurity {
				totalSec++
				switch issue.Severity {
				case models.SeverityCritical:
					critCount++
				case models.SeverityError:
					errCount++
				case models.SeverityWarning:
					warnCount++
				default:
					infoCount++
				}
			}
		}
	}

	lines = append(lines, "")

	if totalSec == 0 {
		lines = append(lines, "")
		check := lipgloss.NewStyle().Foreground(ColorSuccess).Bold(true).
			Render("  ✓  No security issues found")
		lines = append(lines, check)
		lines = append(lines, "")
		lines = append(lines, lipgloss.NewStyle().Foreground(ColorTextSecondary).
			Render("  Your codebase looks clean. No hardcoded secrets, injection"))
		lines = append(lines, lipgloss.NewStyle().Foreground(ColorTextSecondary).
			Render("  risks, or insecure patterns were detected."))
		return lines
	}

	// Summary stats
	var stats []string
	stats = append(stats, StatBox("Total", fmt.Sprintf("%d", totalSec), ColorTextPrimary))
	if critCount > 0 {
		stats = append(stats, StatBox("Critical", fmt.Sprintf("%d", critCount), ColorError))
	}
	if errCount > 0 {
		stats = append(stats, StatBox("Error", fmt.Sprintf("%d", errCount), ColorWarning))
	}
	if warnCount > 0 {
		stats = append(stats, StatBox("Warning", fmt.Sprintf("%d", warnCount), lipgloss.Color("#fbbf24")))
	}
	if infoCount > 0 {
		stats = append(stats, StatBox("Info", fmt.Sprintf("%d", infoCount), ColorInfo))
	}
	lines = append(lines, StatsLine(stats))
	lines = append(lines, "")

	// Per-file issues
	for _, f := range a.result.Files {
		var secIssues []models.Issue
		for _, issue := range f.Issues {
			if issue.Category == models.CategorySecurity {
				secIssues = append(secIssues, issue)
			}
		}
		if len(secIssues) == 0 {
			continue
		}

		relPath := relativePath(f.Path, a.result.RootDir)

		for _, issue := range secIssues {
			badge := SeverityBadge(issue.Severity.String())
			codeLine := getCodeLine(f.Path, issue.Line)

			// Issue header
			lines = append(lines, fmt.Sprintf("  %s  %s",
				badge,
				lipgloss.NewStyle().Foreground(ColorTextPrimary).Bold(true).
					Render(issue.Type)))

			// File location
			lines = append(lines, fmt.Sprintf("       %s %s:%d",
				lipgloss.NewStyle().Foreground(ColorTextDim).Render("at"),
				lipgloss.NewStyle().Foreground(ColorInfo).Render(relPath),
				issue.Line))

			// Code snippet
			if codeLine != "" {
				codeDisplay := strings.TrimSpace(codeLine)
				if len(codeDisplay) > 70 {
					codeDisplay = codeDisplay[:67] + "..."
				}
				lines = append(lines, fmt.Sprintf("       %s %s",
					lipgloss.NewStyle().Foreground(ColorTextMuted).Render("│"),
					lipgloss.NewStyle().Foreground(ColorWarning).Render(codeDisplay)))
			}

			// Message
			lines = append(lines, fmt.Sprintf("       %s",
				lipgloss.NewStyle().Foreground(ColorTextSecondary).Render(issue.Message)))

			// Fix suggestion
			if issue.Fix != "" {
				lines = append(lines, fmt.Sprintf("       %s %s",
					lipgloss.NewStyle().Foreground(ColorSuccess).Render("→"),
					lipgloss.NewStyle().Foreground(ColorSuccess).Render(issue.Fix)))
			}

			lines = append(lines, "")
		}
	}

	return lines
}

// ─── AI Detection Results ────────────────────────────────────

func (a *App) buildAIResults() []string {
	var lines []string

	type aiFile struct {
		path      string
		score     float64
		issues    []models.Issue
		functions []models.FunctionInfo
	}

	var highFiles, medFiles, lowFiles []aiFile

	for _, f := range a.result.Files {
		if f.Language == "" {
			continue
		}
		var aiIssues []models.Issue
		for _, issue := range f.Issues {
			if issue.Category == models.CategoryAIPattern {
				aiIssues = append(aiIssues, issue)
			}
		}

		af := aiFile{
			path:      f.Path,
			score:     f.AIScore,
			issues:    aiIssues,
			functions: f.Functions,
		}

		if f.AIScore > 35 {
			highFiles = append(highFiles, af)
		} else if f.AIScore > 20 {
			medFiles = append(medFiles, af)
		} else if len(aiIssues) > 0 {
			lowFiles = append(lowFiles, af)
		}
	}

	lines = append(lines, "")

	if len(highFiles) == 0 && len(medFiles) == 0 {
		lines = append(lines, "")
		check := lipgloss.NewStyle().Foreground(ColorSuccess).Bold(true).
			Render("  ✓  No significant AI-generated code detected")
		lines = append(lines, check)
		lines = append(lines, "")
		lines = append(lines, lipgloss.NewStyle().Foreground(ColorTextSecondary).
			Render("  All files show human-like coding patterns."))
		if len(lowFiles) > 0 {
			lines = append(lines, "")
			lines = append(lines, lipgloss.NewStyle().Foreground(ColorTextDim).
				Render(fmt.Sprintf("  %d file(s) have minor AI-like patterns (score < 20%%)", len(lowFiles))))
		}
		return lines
	}

	// Summary
	var stats []string
	stats = append(stats, StatBox("Files", fmt.Sprintf("%d", len(a.result.Files)), ColorTextPrimary))
	if len(highFiles) > 0 {
		stats = append(stats, StatBox("High", fmt.Sprintf("%d", len(highFiles)), ColorError))
	}
	if len(medFiles) > 0 {
		stats = append(stats, StatBox("Moderate", fmt.Sprintf("%d", len(medFiles)), ColorWarning))
	}
	if len(lowFiles) > 0 {
		stats = append(stats, StatBox("Low", fmt.Sprintf("%d", len(lowFiles)), ColorTextDim))
	}
	lines = append(lines, StatsLine(stats))
	lines = append(lines, "")

	// High probability files
	if len(highFiles) > 0 {
		for _, af := range highFiles {
			lines = append(lines, a.renderAIFileCard(af, "HIGH", ColorError)...)
		}
	}

	// Medium files
	if len(medFiles) > 0 {
		for _, af := range medFiles {
			lines = append(lines, a.renderAIFileCard(af, "MODERATE", ColorWarning)...)
		}
	}

	return lines
}

func (a *App) renderAIFileCard(af struct {
	path      string
	score     float64
	issues    []models.Issue
	functions []models.FunctionInfo
}, level string, color lipgloss.Color) []string {
	var lines []string
	relPath := relativePath(af.path, a.result.RootDir)

	// File header with score bar
	tag := Tag(level, color)
	scoreBar := GradientBar(af.score/100, 20, ColorAccent, color)
	scorePct := lipgloss.NewStyle().Foreground(color).Bold(true).
		Render(fmt.Sprintf("%.0f%%", af.score))

	lines = append(lines, fmt.Sprintf("  %s  %s  %s %s",
		tag,
		lipgloss.NewStyle().Foreground(ColorTextPrimary).Bold(true).Render(relPath),
		scoreBar,
		scorePct))

	// Reasons
	if len(af.issues) > 0 {
		for _, issue := range af.issues {
			lines = append(lines, fmt.Sprintf("       %s %s",
				lipgloss.NewStyle().Foreground(ColorTextMuted).Render("·"),
				lipgloss.NewStyle().Foreground(ColorTextSecondary).Render(issue.Message)))
		}
	}

	// Function uniformity
	if len(af.functions) > 3 {
		sizes := make([]int, len(af.functions))
		totalSize := 0
		for i, fn := range af.functions {
			sizes[i] = fn.LineCount
			totalSize += fn.LineCount
		}
		avgSize := totalSize / len(af.functions)
		uniform := true
		for _, s := range sizes {
			diff := s - avgSize
			if diff < 0 {
				diff = -diff
			}
			if diff > avgSize/2 {
				uniform = false
				break
			}
		}
		if uniform {
			lines = append(lines,
				lipgloss.NewStyle().Foreground(ColorTextDim).Render(
					fmt.Sprintf("       · Functions uniformly sized (~%d lines) — typical AI pattern", avgSize)))
		}
	}

	lines = append(lines, "")
	return lines
}

// ─── Quality Results ─────────────────────────────────────────

type qualityMetrics struct {
	totalFiles         int
	totalLines         int
	totalFunctions     int
	totalComplexity    int
	totalQualityIssues int
	filesWithIssues    int
	longFunctions      int
	complexFunctions   int
	duplications       int
	unusedImports      int
	todos              int
	deadCode           int
	deepNesting        int
	errorSwallow       int
	avgComplexity      float64
	avgCommentRatio    float64
	score              float64
	scoreColor         lipgloss.Color
	scoreLabel         string
}

func (a *App) collectQualityMetrics() qualityMetrics {
	m := qualityMetrics{totalFiles: len(a.result.Files)}
	commentRatioSum := 0.0

	for _, f := range a.result.Files {
		m.totalComplexity += f.Complexity
		m.totalFunctions += len(f.Functions)
		m.totalLines += f.LineCount
		commentRatioSum += f.CommentRatio

		hasQual := false
		for _, issue := range f.Issues {
			switch issue.Type {
			case "dead_code":
				m.deadCode++
			case "deep_nesting":
				m.deepNesting++
			case "empty_error_handler", "error_swallowing":
				m.errorSwallow++
			}
			if issue.Category != models.CategoryQuality {
				continue
			}
			m.totalQualityIssues++
			hasQual = true
			switch issue.Type {
			case "long_function":
				m.longFunctions++
			case "high_complexity", "moderate_complexity":
				m.complexFunctions++
			case "code_duplication":
				m.duplications++
			case "unused_import":
				m.unusedImports++
			case "todo_marker":
				m.todos++
			}
		}
		if hasQual {
			m.filesWithIssues++
		}
	}

	if m.totalFiles > 0 {
		m.avgComplexity = float64(m.totalComplexity) / float64(m.totalFiles)
		m.avgCommentRatio = commentRatioSum / float64(m.totalFiles) * 100
	}

	m.score = 100.0
	m.score -= float64(m.longFunctions) * 3
	m.score -= float64(m.complexFunctions) * 4
	m.score -= float64(m.duplications) * 2
	m.score -= float64(m.unusedImports) * 1
	if m.avgComplexity > 15 {
		m.score -= (m.avgComplexity - 15) * 2
	}
	if m.score < 0 {
		m.score = 0
	}

	m.scoreColor, m.scoreLabel = ColorSuccess, "Excellent"
	if m.score < 50 {
		m.scoreColor, m.scoreLabel = ColorError, "Needs Work"
	} else if m.score < 70 {
		m.scoreColor, m.scoreLabel = ColorWarning, "Fair"
	} else if m.score < 85 {
		m.scoreColor, m.scoreLabel = ColorInfo, "Good"
	}

	return m
}

// renderQualityScore renders the score display with a gradient bar.
func renderQualityScore(m qualityMetrics, w int) []string {
	var lines []string

	scoreStr := lipgloss.NewStyle().Foreground(m.scoreColor).Bold(true).
		Render(fmt.Sprintf("%.0f", m.score))
	outOf := lipgloss.NewStyle().Foreground(ColorTextDim).Render("/100")
	label := lipgloss.NewStyle().Foreground(m.scoreColor).Render(m.scoreLabel)

	lines = append(lines, fmt.Sprintf("  %s%s  %s  %s",
		scoreStr, outOf,
		GradientBar(m.score/100, 30, ColorAccent, m.scoreColor),
		label))
	lines = append(lines, "")

	return lines
}

// renderMetricsTable renders the metrics in a clean grid.
func renderMetricsTable(m qualityMetrics, w int) []string {
	var lines []string
	lines = append(lines, SectionHeader("METRICS", w))
	lines = append(lines, "")

	for _, row := range [][2]string{
		{"Files Analyzed", fmt.Sprintf("%d", m.totalFiles)},
		{"Total Lines", fmt.Sprintf("%d", m.totalLines)},
		{"Functions", fmt.Sprintf("%d", m.totalFunctions)},
		{"Avg Complexity", fmt.Sprintf("%.1f per file", m.avgComplexity)},
		{"Comment Ratio", fmt.Sprintf("%.1f%%", m.avgCommentRatio)},
		{"Quality Issues", fmt.Sprintf("%d", m.totalQualityIssues)},
	} {
		lines = append(lines, KeyValue(row[0], row[1], 22))
	}
	lines = append(lines, "")
	return lines
}

// renderIssueBreakdown renders a proportional bar chart of issue types.
func renderIssueBreakdown(m qualityMetrics, w int) []string {
	if m.totalQualityIssues == 0 {
		return nil
	}

	var lines []string
	lines = append(lines, SectionHeader("ISSUE BREAKDOWN", w))
	lines = append(lines, "")

	type issueGroup struct {
		label string
		count int
		color lipgloss.Color
	}
	groups := []issueGroup{
		{"Complex Functions", m.complexFunctions, ColorWarning},
		{"Long Functions", m.longFunctions, ColorWarning},
		{"Code Duplication", m.duplications, ColorInfo},
		{"Unused Imports", m.unusedImports, ColorTextSecondary},
		{"TODO Markers", m.todos, ColorTextDim},
	}
	if m.deadCode > 0 {
		groups = append(groups, issueGroup{"Dead Code", m.deadCode, ColorError})
	}
	if m.deepNesting > 0 {
		groups = append(groups, issueGroup{"Deep Nesting", m.deepNesting, ColorWarning})
	}
	if m.errorSwallow > 0 {
		groups = append(groups, issueGroup{"Error Handling", m.errorSwallow, ColorError})
	}

	for _, g := range groups {
		if g.count == 0 {
			continue
		}
		ratio := float64(g.count) / float64(m.totalQualityIssues)
		bar := GradientBar(ratio, 20, g.color, g.color)
		lines = append(lines, fmt.Sprintf("    %s %s %s",
			lipgloss.NewStyle().Foreground(g.color).Width(22).Render(g.label),
			bar,
			lipgloss.NewStyle().Foreground(ColorTextDim).Render(fmt.Sprintf("%d", g.count))))
	}
	lines = append(lines, "")
	return lines
}

// renderFilesWithIssues renders per-file quality details.
func (a *App) renderFilesWithIssues(w int) []string {
	var lines []string
	lines = append(lines, SectionHeader("FILES WITH ISSUES", w))
	lines = append(lines, "")

	for _, f := range a.result.Files {
		var qualIssues []models.Issue
		for _, issue := range f.Issues {
			if issue.Category == models.CategoryQuality {
				qualIssues = append(qualIssues, issue)
			}
		}
		if len(qualIssues) == 0 {
			continue
		}

		relPath := relativePath(f.Path, a.result.RootDir)
		lines = append(lines, fmt.Sprintf("  %s  %s",
			lipgloss.NewStyle().Foreground(ColorInfo).Render(relPath),
			lipgloss.NewStyle().Foreground(ColorTextDim).
				Render(fmt.Sprintf("(%d)", len(qualIssues)))))

		for _, issue := range qualIssues {
			badge := SeverityBadge(issue.Severity.String())
			lines = append(lines, fmt.Sprintf("    %s  L%-4d %s",
				badge, issue.Line,
				lipgloss.NewStyle().Foreground(ColorTextSecondary).Render(issue.Message)))
			if issue.Fix != "" {
				lines = append(lines, fmt.Sprintf("             %s %s",
					lipgloss.NewStyle().Foreground(ColorSuccess).Render("→"),
					lipgloss.NewStyle().Foreground(ColorSuccess).Render(issue.Fix)))
			}
		}
		lines = append(lines, "")
	}
	return lines
}

// buildQualityResults orchestrates all quality result sections.
func (a *App) buildQualityResults() []string {
	if len(a.result.Files) == 0 {
		return []string{
			"",
			lipgloss.NewStyle().Foreground(ColorTextSecondary).
				Render("  No files to analyze."),
		}
	}

	m := a.collectQualityMetrics()
	w := a.width

	var lines []string
	lines = append(lines, "")
	lines = append(lines, renderQualityScore(m, w)...)
	lines = append(lines, renderMetricsTable(m, w)...)
	lines = append(lines, renderIssueBreakdown(m, w)...)
	if m.filesWithIssues > 0 {
		lines = append(lines, a.renderFilesWithIssues(w)...)
	}
	return lines
}

// ─── Helpers ─────────────────────────────────────────────────

func relativePath(fullPath, rootDir string) string {
	rel, err := filepath.Rel(rootDir, fullPath)
	if err != nil {
		return fullPath
	}
	return rel
}

func getCodeLine(filePath string, lineNum int) string {
	content, err := analyzer.ReadFileContent(filePath)
	if err != nil {
		return ""
	}
	fileLines := strings.Split(content, "\n")
	if lineNum > 0 && lineNum <= len(fileLines) {
		line := fileLines[lineNum-1]
		if len(line) > 80 {
			line = line[:77] + "..."
		}
		return line
	}
	return ""
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}
