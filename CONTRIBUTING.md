# Contributing to Fuji

Thanks for your interest in contributing!

## Getting Started
```bash
git clone https://github.com/benevolentshrine/fuji.git
cd fuji
go build -o fuji .
```

## Running Fuji
```bash
# Analyze current directory
./fuji .

# JSON output
./fuji --format json /path/to/project

# CI mode
./fuji --ci /path/to/project
```

## How to Contribute

1. Fork the repo
2. Create a branch: `git checkout -b feat/your-feature`
3. Make your changes
4. Commit with a clear message: `git commit -m "feat: add X"`
5. Push and open a PR

## Guidelines

- Keep PRs focused and small
- Write clear commit messages
- If adding a new analyzer, follow the existing pattern in `internal/analyzer/`
- Test your changes before submitting

## Project Structure
```
internal/
├── analyzer/   # Core analysis engine
├── models/     # Shared data types  
├── output/     # JSON and Markdown formatters
└── tui/        # Terminal UI
```

## Questions?

Open an issue or reach out on GitHub.
