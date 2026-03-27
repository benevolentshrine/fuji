# 🤝 Contributing to Vanguard V2

We're excited that you're interested in contributing to **Vanguard**! As a security-focused tool, we value contributions that improve our analysis heuristics, UI performance, and NIST compliance.

---

## 🛠️ Technical Requirements

Before you start, ensure you have the following installed:
- **Rust Toolchain**: [rustup.rs](https://rustup.rs/)
- **Git**: For version control.
- **Cargo-Edit**: (Optional) For managing dependencies.

---

## 🚀 Getting Started

1. **Fork the Repository**: Create your own copy of the Vanguard repository.
2. **Clone your Fork**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/fuji.git
   cd fuji/fuji-v2
   ```
3. **Install Dependencies**:
   ```bash
   cargo fetch
   ```
4. **Run Locally**:
   - Windows: `.\Launch-Vanguard.ps1`
   - Linux/macOS: `sh Launch-Vanguard.sh`

---

## 📂 Project Structure

- **`src/gui/`**: The `egui` frontend logic and custom vector theme engine.
- **`src/analyzer/`**: Core logic for Security, AI, Quality, and Complexity scanning.
- **`src/secure/`**: NIST-aligned Zero-Trust path validation and SARIF modeling.
- **`src/models.rs`**: Shared data structures and SARIF taxonomy.

---

## 📜 Contribution Guidelines

### 1. Branching Policy
Please create a feature branch for your changes:
```bash
git checkout -b feature/your-awesome-feature
```

### 2. Code Quality
- Vanguard aims for **zero warnings**. Run `cargo check` before committing.
- Ensure your code follows the standard Rust formatting (`cargo fmt`).

### 3. Testing
If you're adding a new analysis rule:
1. Add a test case in the relevant module in `src/analyzer/`.
2. Verify that the rule correctly identifies the target pattern without false positives.

### 4. Submitting a Pull Request
- Provide a clear description of the change.
- Link any related issues.
- Ensure the CI (if available) passes.

---

## 🛡️ Security Vulnerabilities
If you find a security vulnerability within Vanguard itself, please do **not** open a public issue. Instead, contact the maintainers directly.

---

*Thank you for helping make Vanguard the most robust code intelligence tool!*
