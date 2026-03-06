package tui

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/lipgloss"
)

// ─── Color Palette ───────────────────────────────────────────
// "Midnight Fuji" — A refined dark palette with cool accents

var (
	// Backgrounds & surfaces
	ColorBg           = lipgloss.Color("#0a0a0f")
	ColorSurface      = lipgloss.Color("#12121a")
	ColorSurfaceLight = lipgloss.Color("#1a1a2e")
	ColorBorder       = lipgloss.Color("#2a2a3d")
	ColorBorderFocus  = lipgloss.Color("#4a4a6a")

	// Text hierarchy
	ColorTextPrimary   = lipgloss.Color("#e2e2ef")
	ColorTextSecondary = lipgloss.Color("#8888a0")
	ColorTextDim       = lipgloss.Color("#4e4e6a")
	ColorTextMuted     = lipgloss.Color("#363650")
	ColorWhite         = lipgloss.Color("#ffffff")

	// Primary accents
	ColorAccent  = lipgloss.Color("#7c8aff") // Soft periwinkle blue
	ColorAccent2 = lipgloss.Color("#a78bfa") // Lavender
	ColorCyan    = lipgloss.Color("#67e8f9") // Light cyan
	ColorTeal    = lipgloss.Color("#2dd4bf") // Teal green

	// Semantic colors
	ColorSuccess = lipgloss.Color("#34d399") // Emerald
	ColorWarning = lipgloss.Color("#fbbf24") // Amber
	ColorError   = lipgloss.Color("#f87171") // Soft red
	ColorInfo    = lipgloss.Color("#60a5fa") // Sky blue

	// Title & feature colors
	ColorTitle   = lipgloss.Color("#c4b5fd") // Soft violet
	ColorTagline = lipgloss.Color("#6b7280") // Neutral gray
)

// ─── Severity Badges ─────────────────────────────────────────

func SeverityBadge(sev string) string {
	switch sev {
	case "critical":
		return lipgloss.NewStyle().
			Foreground(lipgloss.Color("#fecaca")).
			Background(lipgloss.Color("#991b1b")).
			Bold(true).Padding(0, 1).
			Render("CRIT")
	case "error":
		return lipgloss.NewStyle().
			Foreground(lipgloss.Color("#fed7aa")).
			Background(lipgloss.Color("#9a3412")).
			Padding(0, 1).
			Render("ERR ")
	case "warning":
		return lipgloss.NewStyle().
			Foreground(lipgloss.Color("#fef3c7")).
			Background(lipgloss.Color("#92400e")).
			Padding(0, 1).
			Render("WARN")
	default:
		return lipgloss.NewStyle().
			Foreground(lipgloss.Color("#dbeafe")).
			Background(lipgloss.Color("#1e3a5f")).
			Padding(0, 1).
			Render("INFO")
	}
}

// ─── Section Header ──────────────────────────────────────────

func SectionHeader(title string, width int) string {
	if width < 10 {
		width = 40
	}
	titleRendered := lipgloss.NewStyle().
		Foreground(ColorAccent).Bold(true).
		Render("  " + title + "  ")
	titleW := lipgloss.Width(titleRendered)
	remaining := width - titleW - 4
	if remaining < 2 {
		remaining = 2
	}
	left := lipgloss.NewStyle().Foreground(ColorBorder).Render("  ──")
	right := lipgloss.NewStyle().Foreground(ColorBorder).Render(strings.Repeat("─", remaining))
	return left + titleRendered + right
}

// ─── Divider ─────────────────────────────────────────────────

func Divider(width int) string {
	if width < 4 {
		width = 40
	}
	return lipgloss.NewStyle().Foreground(ColorTextMuted).
		Render("  " + strings.Repeat("─", width-4))
}

// ─── Thin Divider (dotted) ───────────────────────────────────

func ThinDivider(width int) string {
	if width < 4 {
		width = 40
	}
	return lipgloss.NewStyle().Foreground(ColorTextMuted).
		Render("  " + strings.Repeat("╌", width-4))
}

// ─── Progress Bar ────────────────────────────────────────────

func ProgressBar(ratio float64, width int) string {
	if width < 1 {
		width = 1
	}
	filled := int(ratio * float64(width))
	if filled > width {
		filled = width
	}
	if filled < 0 {
		filled = 0
	}

	bar := lipgloss.NewStyle().Foreground(ColorAccent).Render(repeat('█', filled))
	if filled < width {
		bar += lipgloss.NewStyle().Foreground(ColorBorderFocus).Render(repeat('▓', 1))
		remaining := width - filled - 1
		if remaining > 0 {
			bar += lipgloss.NewStyle().Foreground(ColorTextMuted).Render(repeat('░', remaining))
		}
	}
	return bar
}

// ─── Gradient Progress Bar ───────────────────────────────────

func GradientBar(ratio float64, width int, startColor, endColor lipgloss.Color) string {
	if width < 1 {
		width = 1
	}
	filled := int(ratio * float64(width))
	if filled > width {
		filled = width
	}
	if filled < 0 {
		filled = 0
	}

	var parts []string
	for i := 0; i < filled; i++ {
		// Alternate between start and end for shimmer effect
		if i%2 == 0 {
			parts = append(parts, lipgloss.NewStyle().Foreground(startColor).Render("█"))
		} else {
			parts = append(parts, lipgloss.NewStyle().Foreground(endColor).Render("█"))
		}
	}
	empty := width - filled
	if empty > 0 {
		parts = append(parts, lipgloss.NewStyle().Foreground(ColorTextMuted).Render(repeat('░', empty)))
	}
	return strings.Join(parts, "")
}

// ─── Key Value Pair ──────────────────────────────────────────

func KeyValue(key string, value string, keyWidth int) string {
	k := lipgloss.NewStyle().Foreground(ColorTextSecondary).Width(keyWidth).Render(key)
	v := lipgloss.NewStyle().Foreground(ColorTextPrimary).Render(value)
	return "    " + k + v
}

// ─── Stat Box ────────────────────────────────────────────────
// Renders a small inline stat: label + value

func StatBox(label string, value string, color lipgloss.Color) string {
	l := lipgloss.NewStyle().Foreground(ColorTextSecondary).Render(label + " ")
	v := lipgloss.NewStyle().Foreground(color).Bold(true).Render(value)
	return l + v
}

// ─── Boxed Content ───────────────────────────────────────────

func RenderBox(lines []string, width int) []string {
	if width < 6 {
		width = 40
	}
	innerW := width - 6

	borderColor := lipgloss.NewStyle().Foreground(ColorBorder)
	var result []string

	top := borderColor.Render("  ╭" + strings.Repeat("─", innerW) + "╮")
	result = append(result, top)

	for _, line := range lines {
		lineW := lipgloss.Width(line)
		padding := innerW - lineW
		if padding < 0 {
			padding = 0
		}
		row := borderColor.Render("  │") + " " + line + strings.Repeat(" ", padding-1) + borderColor.Render("│")
		result = append(result, row)
	}

	bottom := borderColor.Render("  ╰" + strings.Repeat("─", innerW) + "╯")
	result = append(result, bottom)

	return result
}

// ─── Helpers ─────────────────────────────────────────────────

func repeat(ch rune, n int) string {
	if n <= 0 {
		return ""
	}
	r := make([]rune, n)
	for i := range r {
		r[i] = ch
	}
	return string(r)
}

func padRight(s string, width int) string {
	sw := lipgloss.Width(s)
	if sw >= width {
		return s
	}
	return s + strings.Repeat(" ", width-sw)
}

func truncateStr(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return "…" + s[len(s)-maxLen+1:]
}

// min returns the smaller of two ints.
func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// Compact stat line with separator
func StatsLine(stats []string) string {
	sep := lipgloss.NewStyle().Foreground(ColorTextMuted).Render(" │ ")
	return "    " + strings.Join(stats, sep)
}

// Tag renders a small colored tag
func Tag(label string, color lipgloss.Color) string {
	return lipgloss.NewStyle().
		Foreground(color).
		Render(fmt.Sprintf("[%s]", label))
}
