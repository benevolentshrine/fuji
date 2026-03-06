package tui

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/lipgloss"
)

// ─── Home Screen ─────────────────────────────────────────────

func (a *App) renderHome() string {
	w := a.width
	h := a.height

	var lines []string

	// Big FUJI ASCII logo (7 rows tall)
	fujiLogo := []string{
		"███████ █    █  ████  █",
		"█       █    █ █    █ █",
		"█       █    █ █      █",
		"█████   █    █  ████  █",
		"█       █    █      █ █",
		"█       █    █ █    █ █",
		"█        ████   ████  ███████",
	}

	// Vertical centering — logo is 7 lines, tag 1, spacer 2, buttons 1, footer 2 = ~13 content lines
	contentHeight := 16
	topPad := (h - contentHeight) / 2
	if topPad < 1 {
		topPad = 1
	}
	for i := 0; i < topPad; i++ {
		lines = append(lines, "")
	}

	// Render logo
	logoStyle := lipgloss.NewStyle().Foreground(ColorTitle).Bold(true)
	for _, line := range fujiLogo {
		lines = append(lines, centerText(logoStyle.Render(line), w))
	}
	lines = append(lines, "")

	// Tagline
	tagline := lipgloss.NewStyle().Foreground(ColorTagline).Italic(true).
		Render("codebase intelligence engine")
	lines = append(lines, centerText(tagline, w))
	lines = append(lines, "")
	lines = append(lines, "")

	// Buttons
	a.buttons = nil
	btnDefs := []struct {
		key   string
		label string
		icon  string
	}{
		{"o", "Open", "◈"},
		{"h", "Help", "◇"},
		{"i", "History", "◆"},
	}

	var btnParts []string
	for i, bl := range btnDefs {
		var rendered string
		if i == a.cursor {
			rendered = lipgloss.NewStyle().
				Foreground(ColorAccent).
				Bold(true).
				Render(" ┌───────────┐ ")
			inner := lipgloss.NewStyle().
				Foreground(ColorAccent).
				Bold(true).
				Render(fmt.Sprintf(" │ %s %-7s │ ", bl.icon, bl.label))
			bottom := lipgloss.NewStyle().
				Foreground(ColorAccent).
				Bold(true).
				Render(" └───────────┘ ")
			_ = inner
			_ = bottom
			// Single-line focused button
			rendered = lipgloss.NewStyle().
				Foreground(ColorAccent).Bold(true).
				Render(fmt.Sprintf("  ▸ %s %s  ", bl.icon, bl.label))
		} else {
			rendered = lipgloss.NewStyle().
				Foreground(ColorTextSecondary).
				Render(fmt.Sprintf("    %s %s  ", bl.icon, bl.label))
		}
		btnParts = append(btnParts, rendered)
	}

	btnRow := strings.Join(btnParts, "  ")
	lines = append(lines, centerText(btnRow, w))

	// Register button positions
	btnStartX := (w - lipgloss.Width(btnRow)) / 2
	btnY := len(lines) - 1
	xOff := btnStartX
	for i, bl := range btnDefs {
		var rendered string
		if i == a.cursor {
			rendered = fmt.Sprintf("  ▸ %s %s  ", bl.icon, bl.label)
		} else {
			rendered = fmt.Sprintf("    %s %s  ", bl.icon, bl.label)
		}
		bw := lipgloss.Width(rendered) + 2
		a.buttons = append(a.buttons, Button{
			Label: bl.label, Key: bl.key,
			X: xOff, Y: btnY, Width: bw,
		})
		xOff += bw
	}

	lines = append(lines, "")
	lines = append(lines, "")

	// Footer separator + version
	footerSep := lipgloss.NewStyle().Foreground(ColorTextMuted).
		Render(strings.Repeat("·", min(40, w-20)))
	lines = append(lines, centerText(footerSep, w))

	footer := lipgloss.NewStyle().Foreground(ColorTextDim).
		Render("v0.1.0  ·  press q to quit")
	lines = append(lines, centerText(footer, w))

	return strings.Join(lines, "\n")
}

// ─── Help Screen ─────────────────────────────────────────────

func (a *App) renderHelp() string {
	w := a.width
	h := a.height

	var lines []string

	topPad := (h - 34) / 2
	if topPad < 1 {
		topPad = 1
	}
	for i := 0; i < topPad; i++ {
		lines = append(lines, "")
	}

	lines = append(lines, SectionHeader("COMMAND INDEX", w))
	lines = append(lines, "")

	sections := []struct {
		header string
		items  [][2]string
	}{
		{
			"Navigation",
			[][2]string{
				{"o  /  1", "Open a folder or paste git URL"},
				{"h  /  2", "Show this help screen"},
				{"i  /  3", "View scan history"},
				{"j  /  k", "Navigate up / down"},
				{"Enter", "Select focused item"},
			},
		},
		{
			"Analysis",
			[][2]string{
				{"1", "Security & vulnerability scan"},
				{"2", "AI code detection"},
				{"3", "Code quality analysis"},
				{"b  /  Esc", "Back to previous screen"},
			},
		},
		{
			"Results",
			[][2]string{
				{"j / k / ↑ / ↓", "Scroll line by line"},
				{"d  /  u", "Page down / up (10 lines)"},
				{"g  /  G", "Jump to top / bottom"},
				{"Enter", "Copy results to clipboard"},
				{"Mouse scroll", "Scroll results"},
			},
		},
		{
			"Global",
			[][2]string{
				{"q", "Quit application"},
				{"Ctrl+C", "Force quit"},
			},
		},
	}

	keyStyle := lipgloss.NewStyle().Foreground(ColorAccent).Width(20)
	descStyle := lipgloss.NewStyle().Foreground(ColorTextSecondary)
	headerStyle := lipgloss.NewStyle().Foreground(ColorAccent2).Bold(true)

	for _, sec := range sections {
		lines = append(lines, "    "+headerStyle.Render(sec.header))
		lines = append(lines, "")
		for _, item := range sec.items {
			lines = append(lines, "      "+keyStyle.Render(item[0])+descStyle.Render(item[1]))
		}
		lines = append(lines, "")
	}

	// Back hint
	back := lipgloss.NewStyle().Foreground(ColorTextDim).
		Render("  press b or Esc to go back")
	lines = append(lines, back)

	return strings.Join(lines, "\n")
}

// ─── Path Input Screen ───────────────────────────────────────

func (a *App) renderPathInput() string {
	w := a.width
	h := a.height

	var lines []string

	topPad := (h - 16) / 2
	if topPad < 1 {
		topPad = 1
	}
	for i := 0; i < topPad; i++ {
		lines = append(lines, "")
	}

	lines = append(lines, SectionHeader("OPEN PROJECT", w))
	lines = append(lines, "")

	prompt := lipgloss.NewStyle().Foreground(ColorTextSecondary).
		Render("  Enter folder path or git repo URL:")
	lines = append(lines, prompt)
	lines = append(lines, "")

	// Input field with box
	inputW := 60
	if inputW > w-10 {
		inputW = w - 10
	}
	displayInput := a.pathInput
	if len(displayInput) > inputW-6 {
		displayInput = "…" + displayInput[len(displayInput)-inputW+7:]
	}

	borderStyle := lipgloss.NewStyle().Foreground(ColorBorderFocus)
	inputTop := borderStyle.Render("  ╭" + strings.Repeat("─", inputW) + "╮")
	inputContent := borderStyle.Render("  │ ") +
		lipgloss.NewStyle().Foreground(ColorAccent).Render("❯ ") +
		lipgloss.NewStyle().Foreground(ColorTextPrimary).Render(displayInput) +
		lipgloss.NewStyle().Foreground(ColorAccent).Render("▏") +
		strings.Repeat(" ", max(0, inputW-lipgloss.Width(displayInput)-4)) +
		borderStyle.Render("│")
	inputBottom := borderStyle.Render("  ╰" + strings.Repeat("─", inputW) + "╯")

	lines = append(lines, centerText(inputTop, w))
	lines = append(lines, centerText(inputContent, w))
	lines = append(lines, centerText(inputBottom, w))
	lines = append(lines, "")

	// Browse button
	a.buttons = nil
	browseText := lipgloss.NewStyle().Foreground(ColorAccent2).
		Render("  [Ctrl+B] Browse with file picker")
	lines = append(lines, centerText(browseText, w))

	bw := lipgloss.Width(browseText)
	bx := (w - bw) / 2
	a.buttons = append(a.buttons, Button{
		Label: "Browse", Key: "ctrl+b",
		X: bx, Y: topPad + 8, Width: bw,
	})

	lines = append(lines, "")

	// Status message
	if a.statusMsg != "" {
		status := lipgloss.NewStyle().Foreground(ColorError).
			Render("  ⚠ " + a.statusMsg)
		lines = append(lines, centerText(status, w))
		lines = append(lines, "")
	}

	hint := lipgloss.NewStyle().Foreground(ColorTextDim).
		Render("Enter to confirm  ·  Esc to go back  ·  Empty = current dir")
	lines = append(lines, centerText(hint, w))

	return strings.Join(lines, "\n")
}

// ─── Analysis Menu Screen ────────────────────────────────────

func (a *App) renderMenu() string {
	w := a.width
	h := a.height

	var lines []string

	topPad := (h - 24) / 2
	if topPad < 1 {
		topPad = 1
	}
	for i := 0; i < topPad; i++ {
		lines = append(lines, "")
	}

	lines = append(lines, SectionHeader("SELECT ANALYSIS", w))
	lines = append(lines, "")

	// Path info bar
	pathStr := truncateStr(a.selectedPath, 60)
	pathLine := lipgloss.NewStyle().Foreground(ColorTextSecondary).Render("  ◈ Folder: ") +
		lipgloss.NewStyle().Foreground(ColorTextPrimary).Render(pathStr)
	lines = append(lines, pathLine)
	lines = append(lines, "")
	lines = append(lines, Divider(w))
	lines = append(lines, "")

	// Menu items
	a.buttons = nil
	menuItems := []struct {
		key   string
		icon  string
		label string
		desc  string
		color lipgloss.Color
	}{
		{"1", "🔒", "Security & Vulnerability", "Secrets, injections, crypto misuse, auth issues", ColorError},
		{"2", "🤖", "AI Code Detection", "Detect AI-generated code patterns & fingerprints", ColorAccent},
		{"3", "📊", "Code Quality", "Complexity, duplication, dead code, naming", ColorSuccess},
	}

	for i, item := range menuItems {
		var rendered string
		if i == a.cursor {
			// Focused item with accent indicator
			nameStyle := lipgloss.NewStyle().Foreground(item.color).Bold(true)
			rendered = lipgloss.NewStyle().Foreground(item.color).Render("  ▸ ") +
				item.icon + " " +
				nameStyle.Render(item.label)
		} else {
			nameStyle := lipgloss.NewStyle().Foreground(ColorTextPrimary)
			rendered = "    " + item.icon + " " + nameStyle.Render(item.label)
		}
		lines = append(lines, rendered)

		descStyle := lipgloss.NewStyle().Foreground(ColorTextDim)
		lines = append(lines, descStyle.Render("        "+item.desc))
		lines = append(lines, "")

		// Register button
		bw := lipgloss.Width(rendered)
		a.buttons = append(a.buttons, Button{
			Label: item.label, Key: item.key,
			X: 2, Y: topPad + 6 + i*3, Width: bw + 4,
		})
	}

	lines = append(lines, "")

	// Navigation
	nav := lipgloss.NewStyle().Foreground(ColorTextDim).
		Render("  [b] ← Back    [q] Quit")
	lines = append(lines, nav)

	bw := lipgloss.Width(nav)
	a.buttons = append(a.buttons, Button{
		Label: "Back", Key: "b",
		X: 2, Y: topPad + 6 + 9, Width: bw,
	})

	return strings.Join(lines, "\n")
}

// ─── Loading Screen ──────────────────────────────────────────

func (a *App) renderLoading() string {
	w := a.width
	h := a.height

	var lines []string

	topPad := (h - 12) / 2
	if topPad < 1 {
		topPad = 1
	}
	for i := 0; i < topPad; i++ {
		lines = append(lines, "")
	}

	// Mountain silhouette
	mountain := lipgloss.NewStyle().Foreground(ColorTextMuted).Render("▲")
	lines = append(lines, centerText(mountain, w))
	lines = append(lines, "")

	// Main message
	msg := lipgloss.NewStyle().Foreground(ColorAccent).Bold(true).
		Render(a.loadingMsg)
	lines = append(lines, centerText(msg, w))
	lines = append(lines, "")

	// Progress detail (live updating line)
	if a.loadingDetail != "" {
		detail := lipgloss.NewStyle().Foreground(ColorTextSecondary).
			Render(a.loadingDetail)
		lines = append(lines, centerText(detail, w))
	} else {
		lines = append(lines, "")
	}
	lines = append(lines, "")

	// Animated bar
	bar := GradientBar(0.5, 36, ColorAccent, ColorAccent2)
	lines = append(lines, centerText(bar, w))
	lines = append(lines, "")

	hint := lipgloss.NewStyle().Foreground(ColorTextDim).
		Render("this may take a moment for large repositories")
	lines = append(lines, centerText(hint, w))

	return strings.Join(lines, "\n")
}

// ─── Helpers ─────────────────────────────────────────────────

func centerText(s string, width int) string {
	sw := lipgloss.Width(s)
	if sw >= width {
		return s
	}
	pad := (width - sw) / 2
	return strings.Repeat(" ", pad) + s
}
