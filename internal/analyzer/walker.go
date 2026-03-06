package analyzer

import (
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/lichi/fuji/internal/models"
)

// supportedExtensions maps extensions to language names
var supportedExtensions = map[string]string{
	".go":    "Go",
	".py":    "Python",
	".rs":    "Rust",
	".js":    "JavaScript",
	".ts":    "TypeScript",
	".jsx":   "JavaScript",
	".tsx":   "TypeScript",
	".rb":    "Ruby",
	".java":  "Java",
	".c":     "C",
	".cpp":   "C++",
	".h":     "C",
	".hpp":   "C++",
	".cs":    "C#",
	".php":   "PHP",
	".sh":    "Shell",
	".lua":   "Lua",
	".dart":  "Dart",
	".kt":    "Kotlin",
	".swift": "Swift",
	".scala": "Scala",
	".yaml":  "YAML",
	".yml":   "YAML",
	".json":  "JSON",
	".md":    "Markdown",
	".sql":   "SQL",
}

// Max file size for analysis (512KB) — skip minified/generated files
const maxFileSize = 512 * 1024

// Max files to analyze — safety net for extreme repos
const maxAnalyzableFiles = 15000

// ignoredDirs are directories to skip
var ignoredDirs = map[string]bool{
	".git":          true,
	"node_modules":  true,
	"vendor":        true,
	"__pycache__":   true,
	".venv":         true,
	"venv":          true,
	"dist":          true,
	"build":         true,
	".idea":         true,
	".vscode":       true,
	"target":        true,
	"bin":           true,
	"obj":           true,
	".next":         true,
	".nuxt":         true,
	"coverage":      true,
	".cache":        true,
	".pytest_cache": true,
	"__snapshots__": true,
	".terraform":    true,
	"artifacts":     true,
}

// WalkDirectory builds a file tree from the given root directory
func WalkDirectory(root string) (*models.FileResult, []*models.FileResult, error) {
	absRoot, err := filepath.Abs(root)
	if err != nil {
		return nil, nil, err
	}

	rootNode := &models.FileResult{
		Path:        absRoot,
		Name:        filepath.Base(absRoot),
		IsDirectory: true,
		Expanded:    true,
		Depth:       0,
	}

	var allFiles []*models.FileResult
	dirMap := map[string]*models.FileResult{absRoot: rootNode}

	err = filepath.Walk(absRoot, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return nil // skip errored paths
		}

		// Skip root itself
		if path == absRoot {
			return nil
		}

		// Skip ignored directories
		if info.IsDir() && ignoredDirs[info.Name()] {
			return filepath.SkipDir
		}

		// Skip hidden files/dirs
		if strings.HasPrefix(info.Name(), ".") && info.Name() != "." {
			if info.IsDir() {
				return filepath.SkipDir
			}
			return nil
		}

		rel, _ := filepath.Rel(absRoot, path)
		parentPath := filepath.Dir(path)
		parent := dirMap[parentPath]
		if parent == nil {
			parent = rootNode
		}

		depth := strings.Count(rel, string(os.PathSeparator))

		node := &models.FileResult{
			Path:        path,
			Name:        info.Name(),
			IsDirectory: info.IsDir(),
			Depth:       depth,
			Parent:      parent,
			Expanded:    false,
		}

		if info.IsDir() {
			node.Expanded = depth < 1 // auto-expand first level
			dirMap[path] = node
		} else {
			ext := filepath.Ext(info.Name())
			fileSize := info.Size()
			if lang, ok := supportedExtensions[ext]; ok {
				// Skip files that are too large (minified, generated, etc.)
				if fileSize > maxFileSize {
					node.Language = ""
				} else if len(allFiles) >= maxAnalyzableFiles {
					// Cap total files to prevent RAM exhaustion
					node.Language = ""
				} else {
					node.Language = lang
					// Estimate line count from file size (avoid reading content)
					node.LineCount = int(fileSize / 35) // ~35 bytes per line average
					allFiles = append(allFiles, node)
				}
			} else {
				node.Language = ""
			}
		}

		parent.Children = append(parent.Children, node)
		return nil
	})

	if err != nil {
		return nil, nil, err
	}

	// Sort children alphabetically, directories first
	sortChildren(rootNode)

	return rootNode, allFiles, nil
}

func sortChildren(node *models.FileResult) {
	if node.Children == nil {
		return
	}
	sort.Slice(node.Children, func(i, j int) bool {
		if node.Children[i].IsDirectory != node.Children[j].IsDirectory {
			return node.Children[i].IsDirectory
		}
		return strings.ToLower(node.Children[i].Name) < strings.ToLower(node.Children[j].Name)
	})
	for _, child := range node.Children {
		if child.IsDirectory {
			sortChildren(child)
		}
	}
}

func countLines(content []byte) int {
	if len(content) == 0 {
		return 0
	}
	count := 1
	for _, b := range content {
		if b == '\n' {
			count++
		}
	}
	return count
}

// LanguageForFile returns the language name for a file extension
func LanguageForFile(path string) string {
	ext := filepath.Ext(path)
	if lang, ok := supportedExtensions[ext]; ok {
		return lang
	}
	return ""
}

// ReadFileContent reads and returns file content as string
func ReadFileContent(path string) (string, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}
	return string(data), nil
}
