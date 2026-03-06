package analyzer

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"

	"github.com/lichi/fuji/internal/models"
)

// Analyzer orchestrates all analysis passes
type Analyzer struct {
	RootDir  string
	Progress chan models.ProgressUpdate
}

// NewAnalyzer creates a new analyzer
func NewAnalyzer(rootDir string) *Analyzer {
	absDir, _ := filepath.Abs(rootDir)
	return &Analyzer{
		RootDir:  absDir,
		Progress: make(chan models.ProgressUpdate, 100),
	}
}

// Run executes the full analysis pipeline
func (a *Analyzer) Run() (*models.AnalysisResult, *models.FileResult, error) {
	// Phase 1: Walk directory
	a.sendProgress("Parsing", 0.0, "Scanning directory tree...")
	rootNode, files, err := WalkDirectory(a.RootDir)
	if err != nil {
		return nil, nil, err
	}
	a.sendProgress("Parsing", 1.0, "Found "+strconv.Itoa(len(files))+" files")

	// Phase 2: Git analysis
	a.sendProgress("Git", 0.0, "Analyzing git history...")
	gitData, _ := AnalyzeGit(a.RootDir)
	a.sendProgress("Git", 1.0, "Git analysis complete")

	// Phase 3: Per-file analysis (concurrent)
	a.sendProgress("Patterns", 0.0, "Analyzing code patterns...")
	a.analyzeFiles(files, gitData)
	a.sendProgress("Patterns", 1.0, "Pattern analysis complete")

	// Phase 4: Apply git info
	ApplyGitInfo(files, gitData, a.RootDir)

	// Build summary
	summary := buildSummary(files)

	// Propagate issues up to directories
	propagateIssues(rootNode)

	result := &models.AnalysisResult{
		Summary: summary,
		Files:   files,
		RootDir: a.RootDir,
	}

	close(a.Progress)
	return result, rootNode, nil
}

func (a *Analyzer) analyzeFiles(files []*models.FileResult, gitData *GitAnalysis) {
	var wg sync.WaitGroup
	sem := make(chan struct{}, 4) // limit concurrency to reduce memory pressure
	total := len(files)

	for idx, f := range files {
		if f.Language == "" {
			continue
		}

		wg.Add(1)
		sem <- struct{}{}

		go func(f *models.FileResult, idx int) {
			defer wg.Done()
			defer func() { <-sem }()

			// Double-check file size before reading
			info, err := os.Stat(f.Path)
			if err != nil || info.Size() > maxFileSize {
				return
			}

			content, err := ReadFileContent(f.Path)
			if err != nil {
				return
			}

			// Update line count with actual count now
			f.LineCount = strings.Count(content, "\n") + 1

			// Complexity analysis
			complexity, functions := AnalyzeComplexity(content, f.Language)
			f.Complexity = complexity
			f.Functions = functions

			// AI detection
			aiScore, aiIssues := AnalyzeAI(content, f.Language, functions)
			f.AIScore = aiScore
			f.Issues = append(f.Issues, aiIssues...)

			// Security scan
			secIssues := AnalyzeSecurity(content, f.Language)
			f.Issues = append(f.Issues, secIssues...)

			// Quality checks
			qualIssues := AnalyzeQuality(content, f.Language, complexity, functions)
			f.Issues = append(f.Issues, qualIssues...)

			// Comment ratio
			f.CommentRatio = CommentRatio(content, f.Language)

			// Progress update
			progress := float64(idx+1) / float64(total)
			a.sendProgress("Patterns", progress, fmt.Sprintf("Analyzing %d/%d: %s", idx+1, total, f.Name))
		}(f, idx)
	}

	wg.Wait()
}

func buildSummary(files []*models.FileResult) models.AnalysisSummary {
	summary := models.AnalysisSummary{
		FilesAnalyzed: len(files),
	}

	totalComplexity := 0
	for _, f := range files {
		totalComplexity += f.Complexity

		if len(f.Issues) > 0 {
			summary.FilesFlagged++
		}
		summary.TotalIssues += len(f.Issues)

		if f.AIScore > 60 {
			summary.AISuspected++
		}

		for _, issue := range f.Issues {
			if issue.Category == models.CategorySecurity {
				summary.SecurityIssues++
			}
		}
	}

	if len(files) > 0 {
		summary.AvgComplexity = float64(totalComplexity) / float64(len(files))
	}

	return summary
}

func propagateIssues(node *models.FileResult) {
	if node == nil {
		return
	}

	for _, child := range node.Children {
		if child.IsDirectory {
			propagateIssues(child)
		}
	}
}

func (a *Analyzer) sendProgress(phase string, progress float64, message string) {
	select {
	case a.Progress <- models.ProgressUpdate{
		Phase:    phase,
		Progress: progress,
		Message:  message,
	}:
	default:
	}
}
