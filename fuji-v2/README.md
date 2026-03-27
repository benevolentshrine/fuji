# 🛡️ Vanguard V2
### NIST-Aligned Code Intelligence & Security Architecture

**Vanguard** is a premium, cross-platform codebase analysis tool designed to provide deep architectural insights and security validation. Hardened against **NIST SP 800-series** standards, Vanguard is built for developers who demand high-performance, zero-trust security analysis.

---

## ✨ Key Features

### 🎨 Premium Vector UI
Vanguard features a custom-built GUI powered by `egui` and a proprietary vector rendering engine. 
- **No Font Dependencies**: All icons (Back, Copy, Folder, Analysis) are mathematically drawn programmatically, ensuring a pixel-perfect look on any OS without broken Unicode boxes.
- **Animated Cyber-Aesthetic**: Real-time matrix-rain backgrounds, pulse-glow buttons, and glassmorphism.

### 🔒 Enterprise-Grade Security
- **Zero-Trust Validation**: Implements **NIST SP 800-207** architectures via strict `FsController` path scoping.
- **SSDF Compliance**: Aligned with **NIST SP 800-218** for secure software development frameworks.
- **SARIF Native**: Export results in Industry-standard **SARIF 2.1.0** format (NIST SP 800-92) for integration with enterprise security dashboards.

### 🔍 5 High-Octane Analysis Modes
1. **🔒 Security & Vulnerability**: Detects hardcoded secrets, injection vectors, and cryptographic misuse.
2. **🤖 AI Code Detection**: Identifies AI-generated code patterns in your repository.
3. **📊 Code Quality**: Measures styling, dead code, and redundant logic.
4. **📐 Architecture Complexity**: Calculates cyclomatic complexity to identify unmaintainable monolithic functions.
5. **🌳 Git Churn Diagnostics**: Maps file stability through historical commit frequency to detect high-risk maintenance areas.

---

## 🚀 Getting Started

### Prerequisites
- [Rust Toolchain](https://rustup.rs/) (Cargo)

### Launching Vanguard
Vanguard includes optimized launch scripts for all major operating systems:

- **Windows**: Double-click `Launch-Vanguard.bat` or run `.\Launch-Vanguard.ps1` in PowerShell.
- **Linux/macOS**: Run `sh Launch-Vanguard.sh` in your terminal.

> [!NOTE]
> These launchers automatically isolate the build environment to prevent file-locking issues during self-analysis.

---

## 🛠️ Architecture
- **Language**: 100% Rust for memory safety and zero-cost abstractions.
- **Concurrency**: Parallelized scanning using `Rayon` for lightning-fast multi-core processing.
- **GUI**: Immediate-mode rendering via `eframe` / `egui`.

---
*Vanguard is the hardened successor to the Fuji project.*
