package output

import (
	"encoding/json"
	"os"

	"github.com/lichi/fuji/internal/models"
)

// JSONOutput represents the JSON output format
type JSONOutput struct {
	Summary JSONSummary `json:"summary"`
	Files   []JSONFile `json:"files"`
}

type JSONSummary struct {
	FilesAnalyzed int     `json:"files_analyzed"`
	FilesFlagged  int     `json:"files_flagged"`
	AvgComplexity float64 `json:"avg_complexity"`
	TotalIssues   int     `json:"total_issues"`
}

type JSONFile struct {
	Path          string      `json:"path"`
	AIProb        float64     `json:"ai_probability"`
	Complexity    int         `json:"complexity"`
	Issues        []JSONIssue `json:"issues"`
}

type JSONIssue struct {
	Line     int    `json:"line"`
	Type     string `json:"type"`
	Severity string `json:"severity"`
}

// WriteJSON outputs analysis results as JSON
func WriteJSON(result *models.AnalysisResult) error {
	output := JSONOutput{
		Summary: JSONSummary{
			FilesAnalyzed: result.Summary.FilesAnalyzed,
			FilesFlagged:  result.Summary.FilesFlagged,
			AvgComplexity: result.Summary.AvgComplexity,
			TotalIssues:   result.Summary.TotalIssues,
		},
	}

	for _, f := range result.Files {
		if len(f.Issues) == 0 && f.AIScore < 30 {
			continue // skip clean files in output
		}

		jf := JSONFile{
			Path:       f.Path,
			AIProb:     f.AIScore / 100,
			Complexity: f.Complexity,
		}

		for _, issue := range f.Issues {
			jf.Issues = append(jf.Issues, JSONIssue{
				Line:     issue.Line,
				Type:     issue.Type,
				Severity: issue.Severity.String(),
			})
		}

		output.Files = append(output.Files, jf)
	}

	enc := json.NewEncoder(os.Stdout)
	enc.SetIndent("", "  ")
	return enc.Encode(output)
}
