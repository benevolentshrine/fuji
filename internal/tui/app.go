package tui

import (
	"fmt"
	"os/exec"
	"strings"
	"time"

	"github.com/atotto/clipboard"
	"github.com/aymanbagabas/go-osc52/v2"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/lichi/fuji/internal/analyzer"
	"github.com/lichi/fuji/internal/models"
	"github.com/lichi/fuji/internal/output"
)

// Screen identifiers
const (
	ScreenHome = iota
	ScreenHelp
	ScreenPathInput
	ScreenAnalysisMenu
	ScreenResults
	ScreenLoading
)

// Analysis modes
const (
	AnalysisSecurity = iota
	AnalysisAI
	AnalysisQuality
)

// Messages
type analysisCompleteMsg struct {
	result *models.AnalysisResult
	mode   int
}

type progressMsg struct {
	msg    string
	detail string
}

// Button represents a clickable button
type Button struct {
	Label   string
	Key     string
	X, Y    int
	Width   int
	Focused bool
}

// App is the root Bubble Tea model
type App struct {
	width  int
	height int
	screen int
	cursor int

	// Path input
	pathInput    string
	pathCursor   int
	selectedPath string

	// Analysis
	analysisMode int
	result       *models.AnalysisResult

	// Results scrolling
	scrollOffset int
	scrollMax    int

	// Buttons for current screen
	buttons []Button

	// Loading state
	loadingMsg    string
	loadingDetail string

	// Program reference for sending messages from goroutines
	program *tea.Program

	// Initial path from CLI
	initPath string

	// Status message (e.g. "Copied to clipboard")
	statusMsg string
}

type clearStatusMsg struct{}

func clearStatusCmd() tea.Cmd {
	return tea.Tick(2*time.Second, func(_ time.Time) tea.Msg {
		return clearStatusMsg{}
	})
}

// NewApp creates a new TUI app
func NewApp(rootDir string, existingResult *models.AnalysisResult) *App {
	a := &App{
		screen:   ScreenHome,
		initPath: rootDir,
		result:   existingResult,
	}
	return a
}

// Init starts the program
func (a *App) Init() tea.Cmd {
	// If we have an existing result, skip straight to results
	if a.result != nil {
		a.screen = ScreenResults
		a.analysisMode = AnalysisSecurity // Default mode
		a.selectedPath = a.result.RootDir
		return tea.DisableMouse
	}

	// If a path was given via CLI, skip to analysis menu
	if a.initPath != "" && a.initPath != "." {
		a.selectedPath = a.initPath
		a.screen = ScreenAnalysisMenu
	}
	// Explicitly disable mouse to ensure native terminal selection works
	return tea.DisableMouse
}

// Update handles messages
func (a *App) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		a.width = msg.Width
		a.height = msg.Height
		return a, nil

	case analysisCompleteMsg:
		a.result = msg.result
		a.analysisMode = msg.mode
		a.screen = ScreenResults
		a.scrollOffset = 0
		a.cursor = 0
		return a, nil

	case progressMsg:
		a.loadingMsg = msg.msg
		if msg.detail != "" {
			a.loadingDetail = msg.detail
		}
		return a, nil

	case fileSelectedMsg:
		if msg.path != "" {
			a.selectedPath = msg.path
			a.screen = ScreenAnalysisMenu
			a.cursor = 0
		}
		return a, nil

	case clearStatusMsg:
		a.statusMsg = ""
		return a, nil

	case tea.KeyMsg:
		return a.handleKey(msg)

	case tea.MouseMsg:
		return a.handleMouse(msg)
	}

	return a, nil
}

func (a *App) handleKey(msg tea.KeyMsg) (tea.Model, tea.Cmd) {
	key := msg.String()

	// Global
	switch key {
	case "ctrl+c":
		return a, tea.Quit
	}

	switch a.screen {
	case ScreenHome:
		return a.handleHomeKey(key)
	case ScreenHelp:
		return a.handleHelpKey(key)
	case ScreenPathInput:
		return a.handlePathInputKey(msg)
	case ScreenAnalysisMenu:
		return a.handleMenuKey(key)
	case ScreenResults:
		return a.handleResultsKey(key)
	}

	return a, nil
}

func (a *App) handleHomeKey(key string) (tea.Model, tea.Cmd) {
	switch key {
	case "o", "1", "enter":
		a.screen = ScreenPathInput
		a.pathInput = ""
		a.pathCursor = 0
		return a, nil
	case "h", "2":
		a.screen = ScreenHelp
		return a, nil
	case "i", "3":
		// History (placeholder for now)
		return a, nil
	case "q", "esc":
		return a, tea.Quit
	case "tab", "j", "down":
		a.cursor = (a.cursor + 1) % 3
		return a, nil
	case "shift+tab", "k", "up":
		a.cursor = (a.cursor + 2) % 3
		return a, nil
	}
	return a, nil
}

func (a *App) handleHelpKey(key string) (tea.Model, tea.Cmd) {
	switch key {
	case "b", "esc", "q", "backspace":
		a.screen = ScreenHome
		a.cursor = 0
		return a, nil
	}
	return a, nil
}

func (a *App) handlePathInputKey(msg tea.KeyMsg) (tea.Model, tea.Cmd) {
	key := msg.String()
	switch key {
	case "enter":
		path := strings.TrimSpace(a.pathInput)
		if path == "" {
			path = "."
		}
		a.selectedPath = path
		a.screen = ScreenAnalysisMenu
		a.cursor = 0
		return a, nil
	case "esc":
		a.screen = ScreenHome
		a.cursor = 0
		return a, nil
	case "backspace":
		if len(a.pathInput) > 0 {
			a.pathInput = a.pathInput[:len(a.pathInput)-1]
		}
		return a, nil
	case "ctrl+u":
		a.pathInput = ""
		return a, nil
	case "ctrl+v":
		// Paste from clipboard
		text, err := clipboard.ReadAll()
		if err == nil && text != "" {
			// Clean up pasted text (remove newlines, trim)
			text = strings.TrimSpace(strings.ReplaceAll(text, "\n", ""))
			a.pathInput += text
		}
		return a, nil
	case "ctrl+b":
		// Open native file picker
		return a, a.openFilePicker()
	default:
		if len(key) == 1 || key == " " {
			a.pathInput += key
		}
		return a, nil
	}
}

func (a *App) handleMenuKey(key string) (tea.Model, tea.Cmd) {
	switch key {
	case "1":
		return a, a.runAnalysis(AnalysisSecurity)
	case "2":
		return a, a.runAnalysis(AnalysisAI)
	case "3":
		return a, a.runAnalysis(AnalysisQuality)
	case "b", "esc":
		a.screen = ScreenHome
		a.cursor = 0
		return a, nil
	case "q":
		return a, tea.Quit
	case "tab", "j", "down":
		a.cursor = (a.cursor + 1) % 3
		return a, nil
	case "shift+tab", "k", "up":
		a.cursor = (a.cursor + 2) % 3
		return a, nil
	case "enter":
		return a, a.runAnalysis(a.cursor)
	}
	return a, nil
}

func (a *App) handleResultsKey(key string) (tea.Model, tea.Cmd) {
	switch key {
	case "j", "down":
		a.scrollOffset++
		if a.scrollOffset > a.scrollMax {
			a.scrollOffset = a.scrollMax
		}
		return a, nil
	case "k", "up":
		a.scrollOffset--
		if a.scrollOffset < 0 {
			a.scrollOffset = 0
		}
		return a, nil
	case "d":
		a.scrollOffset += 10
		if a.scrollOffset > a.scrollMax {
			a.scrollOffset = a.scrollMax
		}
		return a, nil
	case "u":
		a.scrollOffset -= 10
		if a.scrollOffset < 0 {
			a.scrollOffset = 0
		}
		return a, nil
	case "g":
		a.scrollOffset = 0
		return a, nil
	case "G":
		a.scrollOffset = a.scrollMax
		return a, nil
	case "b", "esc":
		a.screen = ScreenAnalysisMenu
		a.cursor = 0
		a.scrollOffset = 0
		return a, nil
	case "q":
		return a, tea.Quit
	case "enter", "ctrl+m":
		if a.result != nil {
			md, err := output.WriteMarkdownToString(a.result)
			if err == nil {
				// Try both atotto/clipboard (local) and OSC52 (terminal)
				clipboard.WriteAll(md)

				a.statusMsg = "result copied"
				return a, tea.Batch(
					clearStatusCmd(),
					func() tea.Msg {
						// Send OSC52 sequence to terminal
						fmt.Print(osc52.New(md).String())
						return nil
					},
				)
			}
			a.statusMsg = "Failed to copy: " + err.Error()
			return a, clearStatusCmd()
		}
		return a, nil
	}
	return a, nil
}

func (a *App) handleMouse(msg tea.MouseMsg) (tea.Model, tea.Cmd) {
	switch msg.Button {
	case tea.MouseButtonWheelUp:
		if a.screen == ScreenResults {
			a.scrollOffset -= 3
			if a.scrollOffset < 0 {
				a.scrollOffset = 0
			}
		}
		return a, nil
	case tea.MouseButtonWheelDown:
		if a.screen == ScreenResults {
			a.scrollOffset += 3
			if a.scrollOffset > a.scrollMax {
				a.scrollOffset = a.scrollMax
			}
		}
		return a, nil
	case tea.MouseButtonLeft:
		return a.handleClick(msg.X, msg.Y)
	}
	return a, nil
}

func (a *App) handleClick(x, y int) (tea.Model, tea.Cmd) {
	for i, btn := range a.buttons {
		if x >= btn.X && x < btn.X+btn.Width && y == btn.Y {
			switch a.screen {
			case ScreenHome:
				switch i {
				case 0:
					a.screen = ScreenPathInput
					a.pathInput = ""
					return a, nil
				case 1:
					a.screen = ScreenHelp
					return a, nil
				case 2:
					return a, nil
				}
			case ScreenPathInput:
				if i == 0 {
					// Browse button
					return a, a.openFilePicker()
				}
			case ScreenAnalysisMenu:
				if i < 3 {
					return a, a.runAnalysis(i)
				}
				if i == 3 {
					a.screen = ScreenHome
					a.cursor = 0
					return a, nil
				}
			case ScreenResults:
				if i == 0 {
					a.screen = ScreenAnalysisMenu
					a.cursor = 0
					a.scrollOffset = 0
					return a, nil
				}
			}
		}
	}
	return a, nil
}

// openFilePicker launches native file picker
func (a *App) openFilePicker() tea.Cmd {
	return func() tea.Msg {
		cmd := exec.Command("zenity", "--file-selection", "--directory", "--title=Select folder to analyze")
		out, err := cmd.Output()
		if err != nil {
			return nil
		}
		path := strings.TrimSpace(string(out))
		if path != "" {
			return fileSelectedMsg{path: path}
		}
		return nil
	}
}

type fileSelectedMsg struct {
	path string
}

// runAnalysis starts the analysis in background
func (a *App) runAnalysis(mode int) tea.Cmd {
	a.screen = ScreenLoading
	a.loadingMsg = "Preparing analysis..."
	a.loadingDetail = ""
	path := a.selectedPath
	prog := a.program
	return func() tea.Msg {
		an := analyzer.NewAnalyzer(path)

		// Forward progress updates to TUI
		go func() {
			for update := range an.Progress {
				if prog != nil {
					detail := ""
					if update.Progress > 0 && update.Progress < 1 {
						detail = fmt.Sprintf("%.0f%%", update.Progress*100)
					}
					prog.Send(progressMsg{
						msg:    update.Message,
						detail: detail,
					})
				}
			}
		}()

		result, _, _ := an.Run()
		if result == nil {
			result = &models.AnalysisResult{}
		}
		return analysisCompleteMsg{result: result, mode: mode}
	}
}

// View renders the current screen
func (a *App) View() string {
	if a.width == 0 || a.height == 0 {
		return ""
	}

	var content string

	switch a.screen {
	case ScreenHome:
		content = a.renderHome()
	case ScreenHelp:
		content = a.renderHelp()
	case ScreenPathInput:
		content = a.renderPathInput()
	case ScreenAnalysisMenu:
		content = a.renderMenu()
	case ScreenResults:
		content = a.renderResults()
	case ScreenLoading:
		content = a.renderLoading()
	}

	return content
}

// RunTUI starts the Bubble Tea program
func RunTUI(rootDir string, existingResult *models.AnalysisResult) error {
	app := NewApp(rootDir, existingResult)
	p := tea.NewProgram(app,
		tea.WithAltScreen(),
	)
	app.program = p
	_, err := p.Run()
	return err
}
