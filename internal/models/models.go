package models

// Severity levels for issues
type Severity int

const (
	SeverityInfo Severity = iota
	SeverityWarning
	SeverityError
	SeverityCritical
)

func (s Severity) String() string {
	switch s {
	case SeverityInfo:
		return "info"
	case SeverityWarning:
		return "warning"
	case SeverityError:
		return "error"
	case SeverityCritical:
		return "critical"
	default:
		return "unknown"
	}
}

func (s Severity) Label() string {
	switch s {
	case SeverityInfo:
		return "INFO"
	case SeverityWarning:
		return "WARN"
	case SeverityError:
		return "ERROR"
	case SeverityCritical:
		return "CRIT"
	default:
		return "???"
	}
}

// Category of issue
type Category int

const (
	CategorySecurity Category = iota
	CategoryQuality
	CategoryAIPattern
	CategoryPerformance
)

func (c Category) String() string {
	switch c {
	case CategorySecurity:
		return "Security"
	case CategoryQuality:
		return "Quality"
	case CategoryAIPattern:
		return "AI Pattern"
	case CategoryPerformance:
		return "Performance"
	default:
		return "Unknown"
	}
}

// Issue represents a single finding in a file
type Issue struct {
	Line     int      `json:"line"`
	Column   int      `json:"column,omitempty"`
	Type     string   `json:"type"`
	Severity Severity `json:"severity"`
	Category Category `json:"category"`
	Message  string   `json:"message"`
	Fix      string   `json:"fix,omitempty"`
}

// FunctionInfo holds per-function analysis
type FunctionInfo struct {
	Name       string `json:"name"`
	StartLine  int    `json:"start_line"`
	EndLine    int    `json:"end_line"`
	Complexity int    `json:"complexity"`
	LineCount  int    `json:"line_count"`
}

// GitInfo holds git-related data for a file
type GitInfo struct {
	CommitCount  int      `json:"commit_count"`
	LastAuthor   string   `json:"last_author"`
	LastModified string   `json:"last_modified"`
	Authors      []string `json:"authors,omitempty"`
}

// FileResult holds all analysis data for a single file
type FileResult struct {
	Path          string         `json:"path"`
	Language      string         `json:"language"`
	LineCount     int            `json:"line_count"`
	AIScore       float64        `json:"ai_probability"`
	Complexity    int            `json:"complexity"`
	Functions     []FunctionInfo `json:"functions,omitempty"`
	Issues        []Issue        `json:"issues"`
	GitInfo       *GitInfo       `json:"git_info,omitempty"`
	CommentRatio  float64        `json:"comment_ratio"`
	IsDirectory   bool           `json:"-"`
	Name          string         `json:"-"`
	Children      []*FileResult  `json:"-"`
	Parent        *FileResult    `json:"-"`
	Expanded      bool           `json:"-"`
	Depth         int            `json:"-"`
}

// AnalysisSummary holds aggregate stats
type AnalysisSummary struct {
	FilesAnalyzed  int     `json:"files_analyzed"`
	FilesFlagged   int     `json:"files_flagged"`
	AvgComplexity  float64 `json:"avg_complexity"`
	TotalIssues    int     `json:"total_issues"`
	AISuspected    int     `json:"ai_suspected"`
	SecurityIssues int     `json:"security_issues"`
}

// AnalysisResult is the top-level result container
type AnalysisResult struct {
	Summary AnalysisSummary `json:"summary"`
	Files   []*FileResult   `json:"files"`
	RootDir string          `json:"root_dir"`
}

// ProgressUpdate is sent during analysis to update UI
type ProgressUpdate struct {
	Phase    string  // "Parsing", "Git", "Patterns", "Security"
	Progress float64 // 0.0 - 1.0
	Message  string
}

// FileTreeNode is a flattened representation for the TUI
type FileTreeNode struct {
	File     *FileResult
	Depth    int
	IsLast   bool
	Visible  bool
}
