package analyzer

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

// IsGitURL detects whether the input string is a git repository URL.
func IsGitURL(input string) bool {
	input = strings.TrimSpace(input)
	if input == "" {
		return false
	}

	// SSH format: git@github.com:user/repo.git
	if strings.HasPrefix(input, "git@") {
		return true
	}

	// git:// protocol
	if strings.HasPrefix(input, "git://") {
		return true
	}

	// HTTPS URLs to known git hosts
	if strings.HasPrefix(input, "https://") || strings.HasPrefix(input, "http://") {
		lower := strings.ToLower(input)
		knownHosts := []string{
			"github.com", "gitlab.com", "bitbucket.org",
			"codeberg.org", "sr.ht", "gitea.com",
		}
		for _, host := range knownHosts {
			if strings.Contains(lower, host) {
				return true
			}
		}
		if strings.HasSuffix(lower, ".git") {
			return true
		}
	}

	return false
}

// RepoNameFromURL extracts a short display name from the git URL.
func RepoNameFromURL(url string) string {
	url = strings.TrimSpace(url)
	url = strings.TrimRight(url, "/")
	url = strings.TrimSuffix(url, ".git")

	parts := strings.Split(url, "/")
	if len(parts) >= 2 {
		return parts[len(parts)-2] + "/" + parts[len(parts)-1]
	}
	if len(parts) >= 1 {
		return parts[len(parts)-1]
	}
	return url
}

// CloneProgressFn is called with progress messages during clone
type CloneProgressFn func(msg string)

// CloneRepo clones a git repository to a temporary directory using git CLI.
// Uses --depth=1 --single-branch for speed.
func CloneRepo(url string, onProgress CloneProgressFn) (string, func(), error) {
	// Use ~/.fuji/tmp instead of /tmp to avoid tmpfs size limits on large repos
	home, err := os.UserHomeDir()
	if err != nil {
		return "", nil, fmt.Errorf("failed to get home directory: %w", err)
	}
	fujiTmp := filepath.Join(home, ".fuji", "tmp")
	if err := os.MkdirAll(fujiTmp, 0755); err != nil {
		return "", nil, fmt.Errorf("failed to create fuji temp dir: %w", err)
	}

	tmpDir, err := os.MkdirTemp(fujiTmp, "clone-*")
	if err != nil {
		return "", nil, fmt.Errorf("failed to create temp directory: %w", err)
	}

	cleanup := func() {
		os.RemoveAll(tmpDir)
	}

	if onProgress != nil {
		onProgress("Initializing clone...")
	}

	// Use git CLI for reliable cloning with progress
	gitPath, err := exec.LookPath("git")
	if err != nil {
		cleanup()
		return "", nil, fmt.Errorf("git is not installed — required for cloning repos")
	}

	cmd := exec.Command(gitPath, "clone",
		"--depth=1",
		"--single-branch",
		"--progress",
		url,
		tmpDir,
	)

	// Capture stderr for BOTH progress and error reporting
	stderr, err := cmd.StderrPipe()
	if err != nil {
		cleanup()
		return "", nil, fmt.Errorf("failed to capture git output: %w", err)
	}

	if err := cmd.Start(); err != nil {
		cleanup()
		return "", nil, fmt.Errorf("failed to start git clone: %w", err)
	}

	// Read progress from stderr line by line and keep track of last message for errors
	var lastErrLine string
	scanner := bufio.NewScanner(stderr)
	scanner.Split(scanGitProgress)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line != "" {
			if strings.Contains(strings.ToLower(line), "error") || strings.Contains(strings.ToLower(line), "fatal") {
				lastErrLine = line
			}
			if onProgress != nil {
				onProgress(parseGitProgress(line))
			}
		}
	}

	if err := cmd.Wait(); err != nil {
		cleanup()
		if lastErrLine != "" {
			return "", nil, fmt.Errorf("git clone failed: %s", lastErrLine)
		}
		return "", nil, fmt.Errorf("clone failed: %w", err)
	}

	if onProgress != nil {
		onProgress("Clone complete ✓")
	}

	return tmpDir, cleanup, nil
}

// parseGitProgress cleans up git's stderr output into a readable message
func parseGitProgress(line string) string {
	// Git outputs lines like:
	// "Receiving objects:  45% (123/274)"
	// "Resolving deltas:  100% (42/42)"
	// "Cloning into '/tmp/fuji-clone-xxx'..."
	line = strings.TrimSpace(line)

	if strings.Contains(line, "Receiving objects") {
		return "📥 " + line
	}
	if strings.Contains(line, "Resolving deltas") {
		return "🔗 " + line
	}
	if strings.Contains(line, "Counting objects") || strings.Contains(line, "Enumerating objects") {
		return "📦 " + line
	}
	if strings.Contains(line, "Compressing objects") {
		return "🗜 " + line
	}
	if strings.Contains(line, "Cloning into") {
		return "Starting clone..."
	}
	return line
}

// scanGitProgress is a bufio.SplitFunc that splits on \r or \n.
// Git progress uses \r for in-place updates.
func scanGitProgress(data []byte, atEOF bool) (advance int, token []byte, err error) {
	if atEOF && len(data) == 0 {
		return 0, nil, nil
	}
	// Find \r or \n
	for i, b := range data {
		if b == '\r' || b == '\n' {
			return i + 1, data[:i], nil
		}
	}
	if atEOF {
		return len(data), data, nil
	}
	return 0, nil, nil
}
