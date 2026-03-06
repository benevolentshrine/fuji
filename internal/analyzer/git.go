package analyzer

import (
	"errors"
	"path/filepath"
	"sort"
	"time"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing/object"
	"github.com/lichi/fuji/internal/models"
)

var errStopIteration = errors.New("stop")

// GitAnalysis holds aggregate git data for the project
type GitAnalysis struct {
	FileChurn    map[string]int      // path -> commit count
	FileAuthors  map[string][]string // path -> authors
	LastModified map[string]time.Time
}

// AnalyzeGit performs git analysis on the repository
func AnalyzeGit(rootDir string) (*GitAnalysis, error) {
	repo, err := git.PlainOpen(rootDir)
	if err != nil {
		// Not a git repo or can't open — return empty
		return &GitAnalysis{
			FileChurn:    make(map[string]int),
			FileAuthors:  make(map[string][]string),
			LastModified: make(map[string]time.Time),
		}, nil
	}

	result := &GitAnalysis{
		FileChurn:    make(map[string]int),
		FileAuthors:  make(map[string][]string),
		LastModified: make(map[string]time.Time),
	}

	// Iterate commits
	ref, err := repo.Head()
	if err != nil {
		return result, nil
	}

	commitIter, err := repo.Log(&git.LogOptions{From: ref.Hash()})
	if err != nil {
		return result, nil
	}

	// Limit commits — for shallow clones (depth=1) this is already 1.
	// For full repos, cap at 100 to avoid memory explosion from Stats().
	maxCommits := 100
	commitCount := 0

	_ = commitIter.ForEach(func(c *object.Commit) error {
		if commitCount >= maxCommits {
			return errStopIteration
		}
		commitCount++

		// Stats() is expensive — it diffs the entire tree. Recover from panics.
		var stats object.FileStats
		func() {
			defer func() { recover() }()
			stats, _ = c.Stats()
		}()
		if stats == nil {
			return nil
		}

		for _, stat := range stats {
			relPath := stat.Name
			result.FileChurn[relPath]++

			author := c.Author.Name
			authors := result.FileAuthors[relPath]
			found := false
			for _, a := range authors {
				if a == author {
					found = true
					break
				}
			}
			if !found {
				result.FileAuthors[relPath] = append(authors, author)
			}

			if existing, ok := result.LastModified[relPath]; !ok || c.Author.When.After(existing) {
				result.LastModified[relPath] = c.Author.When
			}
		}

		return nil
	})

	return result, nil
}

// ApplyGitInfo applies git analysis data to file results
func ApplyGitInfo(files []*models.FileResult, gitData *GitAnalysis, rootDir string) {
	if gitData == nil {
		return
	}

	for _, f := range files {
		relPath, err := filepath.Rel(rootDir, f.Path)
		if err != nil {
			continue
		}

		churn := gitData.FileChurn[relPath]
		authors := gitData.FileAuthors[relPath]
		lastMod := gitData.LastModified[relPath]

		if churn > 0 || len(authors) > 0 {
			lastAuthor := ""
			if len(authors) > 0 {
				lastAuthor = authors[0]
			}
			lastModStr := ""
			if !lastMod.IsZero() {
				lastModStr = lastMod.Format("2006-01-02")
			}

			f.GitInfo = &models.GitInfo{
				CommitCount:  churn,
				LastAuthor:   lastAuthor,
				LastModified: lastModStr,
				Authors:      authors,
			}
		}
	}
}

// HighChurnFiles returns files sorted by commit count
func HighChurnFiles(gitData *GitAnalysis, limit int) []string {
	type kv struct {
		Key   string
		Value int
	}
	var sorted []kv
	for k, v := range gitData.FileChurn {
		sorted = append(sorted, kv{k, v})
	}
	sort.Slice(sorted, func(i, j int) bool {
		return sorted[i].Value > sorted[j].Value
	})

	var result []string
	for i, s := range sorted {
		if i >= limit {
			break
		}
		result = append(result, s.Key)
	}
	return result
}
