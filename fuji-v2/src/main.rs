mod models;
mod analyzer;
mod output;
mod gui;
mod secure;

use std::sync::mpsc;

const VERSION: &str = "2.0.0";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut format = String::new();
    let mut ci_mode = false;
    let mut show_version = false;
    let mut show_help = false;
    let mut target_dir = String::from(".");

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                if i + 1 < args.len() {
                    format = args[i + 1].clone();
                    i += 1;
                }
            }
            "--ci" => ci_mode = true,
            "--version" | "-v" => show_version = true,
            "--help" | "-h" => show_help = true,
            arg => {
                if !arg.starts_with('-') {
                    target_dir = arg.to_string();
                }
            }
        }
        i += 1;
    }

    if show_version {
        println!("Vanguard v{}", VERSION);
        return;
    }

    if show_help {
        print_help();
        return;
    }

    // Non-interactive modes
    if !format.is_empty() || ci_mode {
        validate_dir(&target_dir);
        let result = run_analysis(&target_dir);

        match format.as_str() {
            "json" => {
                if let Err(e) = output::json::write_json(&result) {
                    eprintln!("Error writing JSON: {}", e);
                    std::process::exit(1);
                }
            }
            "md" | "markdown" => {
                if let Err(e) = output::markdown::write_markdown(&result) {
                    eprintln!("Error writing Markdown: {}", e);
                    std::process::exit(1);
                }
            }
            _ => {
                if ci_mode {
                    print_ci_summary(&result);
                    if result.summary.security_issues > 0 {
                        std::process::exit(2);
                    }
                    if result.summary.total_issues > 0 {
                        std::process::exit(1);
                    }
                }
            }
        }
        return;
    }

    // GUI mode
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("Vanguard — Codebase Intelligence"),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Vanguard",
        native_options,
        Box::new(|cc| Ok(Box::new(gui::FujiApp::new(cc)))),
    ) {
        eprintln!("GUI error: {}", e);
        std::process::exit(1);
    }
}

fn validate_dir(dir: &str) {
    let path = std::path::Path::new(dir);
    if !path.exists() {
        eprintln!("Error: {} does not exist", dir);
        std::process::exit(1);
    }
    if !path.is_dir() {
        eprintln!("Error: {} is not a directory", dir);
        std::process::exit(1);
    }
}

fn run_analysis(dir: &str) -> models::AnalysisResult {
    let (tx, rx) = mpsc::channel();

    // Drain progress channel in background
    std::thread::spawn(move || {
        for _ in rx {}
    });

    let an = analyzer::Analyzer::new(dir.to_string(), tx);
    match an.run() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Analysis error: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_ci_summary(result: &models::AnalysisResult) {
    let s = &result.summary;
    println!("🗻 Vanguard — CI Report");
    println!("═══════════════════");
    println!("Files analyzed:   {}", s.files_analyzed);
    println!("Files flagged:    {}", s.files_flagged);
    println!("Total issues:     {}", s.total_issues);
    println!("Security issues:  {}", s.security_issues);
    println!("AI-suspected:     {}", s.ai_suspected);
    println!("Avg complexity:   {:.1}", s.avg_complexity);
    println!();

    if s.security_issues > 0 {
        println!("❌ FAIL — Security issues detected");
    } else if s.total_issues > 0 {
        println!("⚠️  WARN — Issues detected");
    } else {
        println!("✅ PASS — No issues found");
    }
}

fn print_help() {
    println!(r#"🗻 Vanguard — Codebase Intelligence Tool

Usage:
  vanguard [flags] [directory]

Flags:
  --format <json|md>   Output format (non-interactive)
  --ci                 CI mode (exit codes: 0=clean, 1=issues, 2=security)
  -v, --version        Show version
  -h, --help           Show help

Examples:
  vanguard .                    Launch GUI (default)
  vanguard --format json .      Generate JSON output
  vanguard --format md .        Markdown report
  vanguard --ci .               CI mode with exit codes"#);
}

