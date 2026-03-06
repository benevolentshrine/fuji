package main

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"github.com/lichi/fuji/internal/analyzer"
	"github.com/lichi/fuji/internal/models"
	"github.com/lichi/fuji/internal/output"
	"github.com/lichi/fuji/internal/tui"
)

const version = "0.1.0"

func main() {
	args := os.Args[1:]

	// Parse flags
	format := ""
	ciMode := false
	showVersion := false
	showHelp := false
	targetDir := "."

	for i := 0; i < len(args); i++ {
		switch args[i] {
		case "--format":
			if i+1 < len(args) {
				format = args[i+1]
				i++
			}
		case "--ci":
			ciMode = true
		case "--version", "-v":
			showVersion = true
		case "--help", "-h":
			showHelp = true
		default:
			if args[i][0] != '-' {
				targetDir = args[i]
			}
		}
	}

	if showVersion {
		fmt.Printf("fuji v%s\n", version)
		os.Exit(0)
	}

	if showHelp {
		printHelp()
		os.Exit(0)
	}

	// Non-interactive modes require valid target dir
	if format != "" || ciMode {
		validateDir(targetDir)

		result := runAnalysis(targetDir)
		if result == nil {
			os.Exit(1)
		}

		switch format {
		case "json":
			if err := output.WriteJSON(result); err != nil {
				fmt.Fprintf(os.Stderr, "Error writing JSON: %v\n", err)
				os.Exit(1)
			}
		case "md", "markdown":
			if err := output.WriteMarkdown(result); err != nil {
				fmt.Fprintf(os.Stderr, "Error writing Markdown: %v\n", err)
				os.Exit(1)
			}
		default:
			if ciMode {
				printCISummary(result)
				if result.Summary.SecurityIssues > 0 {
					os.Exit(2)
				}
				if result.Summary.TotalIssues > 0 {
					os.Exit(1)
				}
				os.Exit(0)
			}
		}
		return
	}

	// Interactive TUI mode
	// If user passed a path, go directly to analysis menu; otherwise show home
	tuiPath := ""
	var existingResult *models.AnalysisResult

	if targetDir != "." {
		// Optimization: if it's a JSON file, load it directly
		if strings.HasSuffix(strings.ToLower(targetDir), ".json") {
			res, err := loadJSON(targetDir)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Error loading JSON: %v\n", err)
				os.Exit(1)
			}
			existingResult = res
		} else {
			validateDir(targetDir)
			tuiPath = targetDir
		}
	}

	if err := tui.RunTUI(tuiPath, existingResult); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

func loadJSON(path string) (*models.AnalysisResult, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var res models.AnalysisResult
	if err := json.Unmarshal(data, &res); err != nil {
		return nil, err
	}
	return &res, nil
}

// validateDir checks that the given path exists and is a directory.
func validateDir(dir string) {
	info, err := os.Stat(dir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %s does not exist\n", dir)
		os.Exit(1)
	}
	if !info.IsDir() {
		fmt.Fprintf(os.Stderr, "Error: %s is not a directory\n", dir)
		os.Exit(1)
	}
}

func runAnalysis(dir string) *models.AnalysisResult {
	an := analyzer.NewAnalyzer(dir)

	// Drain progress channel
	go func() {
		for range an.Progress {
		}
	}()

	result, _, err := an.Run()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Analysis error: %v\n", err)
		return nil
	}
	return result
}

func printCISummary(result *models.AnalysisResult) {
	s := result.Summary
	fmt.Printf("🗻 fuji — CI Report\n")
	fmt.Printf("═══════════════════\n")
	fmt.Printf("Files analyzed:   %d\n", s.FilesAnalyzed)
	fmt.Printf("Files flagged:    %d\n", s.FilesFlagged)
	fmt.Printf("Total issues:     %d\n", s.TotalIssues)
	fmt.Printf("Security issues:  %d\n", s.SecurityIssues)
	fmt.Printf("AI-suspected:     %d\n", s.AISuspected)
	fmt.Printf("Avg complexity:   %.1f\n", s.AvgComplexity)
	fmt.Println()

	if s.SecurityIssues > 0 {
		fmt.Println("❌ FAIL — Security issues detected")
	} else if s.TotalIssues > 0 {
		fmt.Println("⚠️  WARN — Issues detected")
	} else {
		fmt.Println("✅ PASS — No issues found")
	}
}

func printHelp() {
	fmt.Println(`🗻 fuji — Codebase Intelligence Tool

Usage:
  fuji [flags] [directory]

Flags:
  --format <json|md>   Output format (non-interactive)
  --ci                 CI mode (exit codes: 0=clean, 1=issues, 2=security)
  -v, --version        Show version
  -h, --help           Show help

Examples:
  fuji .                    Interactive TUI (default)
  fuji report.json          View a pre-generated JSON report
  fuji --format json . > report.json  Generate JSON output
  fuji --format md .        Markdown report
  fuji --ci                 CI mode with exit codes

Keyboard shortcuts (TUI mode):
  j/k ↑/↓     Navigate
  h/l ←/→     Switch panes
  Enter       Open/toggle
  f           Filter files
  e           Open in $EDITOR
  ?           Help
  q/Esc       Quit`)
}
