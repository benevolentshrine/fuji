pub mod theme;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use eframe::egui;
use crate::models::{AnalysisResult, ProgressUpdate};
use crate::analyzer::Analyzer;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Home,
    AnalysisMenu,
    Loading,
    Results,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnalysisMode {
    Security,
    AI,
    Quality,
    Complexity,
    Git,
}

pub struct FujiApp {
    screen: Screen,
    selected_path: Option<String>,
    analysis_mode: AnalysisMode,
    result: Option<AnalysisResult>,
    loading_msg: String,
    loading_detail: String,
    progress_rx: Option<mpsc::Receiver<ProgressUpdate>>,
    result_holder: Arc<Mutex<Option<AnalysisResult>>>,
    is_analyzing: bool,
    scroll_offset: f32,
    status_msg: String,
    #[allow(dead_code)]
    menu_cursor: usize,
}

impl FujiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        FujiApp {
            screen: Screen::Home,
            selected_path: None,
            analysis_mode: AnalysisMode::Security,
            result: None,
            loading_msg: String::new(),
            loading_detail: String::new(),
            progress_rx: None,
            result_holder: Arc::new(Mutex::new(None)),
            is_analyzing: false,
            scroll_offset: 0.0,
            status_msg: String::new(),
            menu_cursor: 0,
        }
    }

    fn start_analysis(&mut self, mode: AnalysisMode, ctx: &egui::Context) {
        self.screen = Screen::Loading;
        self.analysis_mode = mode;
        self.loading_msg = "Preparing analysis...".to_string();
        self.loading_detail = String::new();
        self.is_analyzing = true;

        let (tx, rx) = mpsc::channel();
        self.progress_rx = Some(rx);

        let path = self.selected_path.clone().unwrap_or_else(|| ".".to_string());

        // Zero-Trust boundary: validate path through FsController (NIST SP 800-207)
        let path_buf = std::path::PathBuf::from(&path);
        let controller = crate::secure::FsController::new(path_buf);
        if let Err(e) = controller.validate_path(&path) {
            eprintln!("Path validation failed: {}", e);
            self.loading_msg = format!("❌ Invalid path: {}", e);
            self.is_analyzing = false;
            self.screen = Screen::Home;
            return;
        }

        let holder = self.result_holder.clone();
        let ctx = ctx.clone();

        std::thread::spawn(move || {
            let analyzer = Analyzer::new(path, tx);
            match analyzer.run() {
                Ok(result) => {
                    *holder.lock().unwrap() = Some(result);
                }
                Err(e) => {
                    eprintln!("Analysis error: {}", e);
                    *holder.lock().unwrap() = Some(AnalysisResult {
                        summary: Default::default(),
                        files: Vec::new(),
                        root_dir: String::new(),
                        tool_info: None,
                        invocation: None,
                        timestamp: None,
                    });
                }
            }
            ctx.request_repaint();
        });
    }

    fn render_home(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();

        ui.vertical_centered(|ui| {
            ui.add_space(available.y * 0.2);

            // Massive fuji Logo
            ui.label(
                egui::RichText::new("Vanguard")
                    .size(80.0)
                    .color(theme::COLOR_TEXT_PRIMARY)
                    .strong()
            );
            ui.add_space(32.0);

            let btn_width = 240.0;
            let btn_height = 48.0;

            if let Some(path) = self.selected_path.clone() {
                // STATE 2: Target Selected
                
                // Simulated "Target Selected" card
                ui.horizontal(|ui| {
                    ui.add_space((available.x - 300.0) / 2.0);
                    
                    egui::Frame::new()
                        .fill(theme::COLOR_SURFACE_LIGHT)
                        .stroke(egui::Stroke::new(1.0, theme::COLOR_ACCENT.linear_multiply(0.3)))
                        .corner_radius(8)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.set_width(300.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("📂").size(24.0));
                                ui.add_space(8.0);
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new("TARGET SELECTED").size(10.0).color(theme::COLOR_ACCENT).strong());
                                    
                                    // Extract folder name for display
                                    let folder_name = std::path::Path::new(&path)
                                        .file_name()
                                        .unwrap_or_else(|| std::ffi::OsStr::new(&path))
                                        .to_string_lossy()
                                        .into_owned();
                                    
                                    ui.label(egui::RichText::new(folder_name).size(16.0).color(theme::COLOR_TEXT_PRIMARY));
                                });
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Clear").clicked() {
                                        self.selected_path = None; // Reset
                                    }
                                });
                            });
                        });
                });
                
                ui.add_space(24.0);
                
                if theme::premium_button(ui, "RUN ANALYSIS", theme::COLOR_ACCENT, egui::vec2(300.0, 56.0)).clicked() {
                    self.screen = Screen::AnalysisMenu;
                }
                
            } else {
                // STATE 1: Empty / Upload
                if theme::premium_button(ui, "Open Local Folder", theme::COLOR_ACCENT, egui::vec2(btn_width, btn_height)).clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        self.selected_path = Some(folder.to_string_lossy().to_string());
                    }
                }

                ui.add_space(16.0);
                
                // Drag and drop zone mock
                egui::Frame::new()
                    .fill(theme::COLOR_BG)
                    .stroke(egui::Stroke::new(1.0, theme::COLOR_BORDER))
                    .corner_radius(8)
                    .inner_margin(32.0)
                    .show(ui, |ui| {
                        ui.set_width(btn_width);
                        ui.vertical_centered(|ui| {
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Select Target Directory").size(14.0).strong().color(theme::COLOR_TEXT_PRIMARY));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("(Drag and drop supported)").size(12.0).color(theme::COLOR_TEXT_DIM));
                        });
                    });
            }

            ui.add_space(60.0);

            // Engine specs
            ui.horizontal(|ui| {
                ui.add_space((available.x - 200.0) / 2.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("ENGINE").size(10.0).color(theme::COLOR_TEXT_MUTED));
                    ui.label(egui::RichText::new("RAYON").size(12.0).color(theme::COLOR_SECONDARY).strong());
                });
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("LANGUAGE").size(10.0).color(theme::COLOR_TEXT_MUTED));
                    ui.label(egui::RichText::new("RUST").size(12.0).color(theme::COLOR_ACCENT).strong());
                });
            });

            // Security Protocol Badges
            egui::Area::new(egui::Id::new("home_footer")).anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(16.0, -16.0)).show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("NIST SP 800-207 Zero-Trust").size(10.0).color(theme::COLOR_ACCENT));
                    ui.label(egui::RichText::new(" | ").size(10.0).color(theme::COLOR_TEXT_MUTED));
                    ui.label(egui::RichText::new("NIST SP 800-218 Pipeline Validation").size(10.0).color(theme::COLOR_ACCENT));
                    ui.label(egui::RichText::new(" | ").size(10.0).color(theme::COLOR_TEXT_MUTED));
                    ui.label(egui::RichText::new("SARIF 2.1.0 Telemetry Active").size(10.0).color(theme::COLOR_ACCENT));
                });
            });
            
            egui::Area::new(egui::Id::new("home_version")).anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -16.0)).show(ui.ctx(), |ui| {
                ui.label(egui::RichText::new("v2.0.0").size(10.0).color(theme::COLOR_TEXT_MUTED).strong());
            });

            egui::Area::new(egui::Id::new("home_controls")).anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0)).show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui.button(egui::RichText::new("Help").color(theme::COLOR_TEXT_DIM)).clicked() {
                         self.screen = Screen::Help;
                    }
                });
            });
        });
    }

    fn render_analysis_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let available = ui.available_size();

        ui.vertical_centered(|ui| {
            ui.add_space(available.y * 0.15);

            ui.label(
                egui::RichText::new("SELECT ANALYSIS")
                    .size(20.0)
                    .color(theme::COLOR_ACCENT)
                    .strong()
            );
            ui.add_space(8.0);

            // Path info
            let p = self.selected_path.clone().unwrap_or_else(|| ".".to_string());
            let folder_name = std::path::Path::new(&p)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new(&p))
                .to_string_lossy()
                .into_owned();
            ui.label(
                egui::RichText::new(format!("◈ {}", folder_name))
                    .size(13.0)
                    .color(theme::COLOR_TEXT_SECONDARY)
            );

            ui.add_space(30.0);

            let card_width = 360.0;
            let card_height = 70.0;

            struct MenuItem {
                label: &'static str,
                desc: &'static str,
                color: egui::Color32,
                mode: AnalysisMode,
                draw_icon: fn(&mut egui::Ui, egui::Rect, egui::Color32),
            }

            let items = vec![
                MenuItem { 
                    label: "Security & Vulnerability", desc: "Secrets, injections, crypto misuse", 
                    color: theme::COLOR_ERROR, mode: AnalysisMode::Security, 
                    draw_icon: theme::draw_icon_check
                },
                MenuItem { 
                    label: "AI Code Detection", desc: "Detect AI-generated code patterns", 
                    color: theme::COLOR_PRIMARY, mode: AnalysisMode::AI, 
                    draw_icon: theme::draw_icon_folder
                },
                MenuItem { 
                    label: "Code Quality", desc: "Complexity, styling, dead code", 
                    color: theme::COLOR_SUCCESS, mode: AnalysisMode::Quality, 
                    draw_icon: theme::draw_icon_copy
                },
                MenuItem { 
                    label: "Architecture Complexity", desc: "Cyclomatic monolith identification", 
                    color: theme::COLOR_WARNING, mode: AnalysisMode::Complexity, 
                    draw_icon: theme::draw_icon_folder
                },
                MenuItem { 
                    label: "Git Churn Diagnostics", desc: "Map highly unstable architectural paths", 
                    color: theme::COLOR_INFO, mode: AnalysisMode::Git, 
                    draw_icon: theme::draw_icon_copy
                },
            ];

            for item in items {
                if theme::premium_analysis_card(ui, item.label, item.desc, item.color, false, item.draw_icon).clicked() {
                    self.start_analysis(item.mode, ctx);
                }

                ui.add_space(12.0);
            }

            ui.add_space(20.0);

            if ui.add(
                egui::Button::new(
                    egui::RichText::new("Back").size(14.0).color(theme::COLOR_TEXT_DIM)
                ).fill(egui::Color32::TRANSPARENT)
            ).clicked() {
                self.screen = Screen::Home;
            }
        });
    }

    fn render_loading(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Check for progress updates
        if let Some(ref rx) = self.progress_rx {
            while let Ok(update) = rx.try_recv() {
                self.loading_msg = update.message;
                if update.progress > 0.0 && update.progress < 1.0 {
                    self.loading_detail = format!("{:.0}%", update.progress * 100.0);
                }
            }
        }

        // Check if analysis is done
        if let Ok(mut holder) = self.result_holder.try_lock() {
            if let Some(result) = holder.take() {
                self.result = Some(result);
                self.screen = Screen::Results;
                self.is_analyzing = false;
                self.scroll_offset = 0.0;
                return;
            }
        }

        // Request repaint while loading
        ctx.request_repaint();

        let available = ui.available_size();
        ui.vertical_centered(|ui| {
            ui.add_space(available.y * 0.35);

            ui.label(
                egui::RichText::new("Wait")
                    .size(24.0)
                    .color(theme::COLOR_TEXT_MUTED)
            );
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new(&self.loading_msg)
                    .size(16.0)
                    .color(theme::COLOR_ACCENT)
                    .strong()
            );
            ui.add_space(8.0);
            if !self.loading_detail.is_empty() {
                ui.label(
                    egui::RichText::new(&self.loading_detail)
                        .size(13.0)
                        .color(theme::COLOR_TEXT_SECONDARY)
                );
            }
            ui.add_space(16.0);

            // Spinner
            ui.spinner();

            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("this may take a moment for large repositories")
                    .size(11.0)
                    .color(theme::COLOR_TEXT_DIM)
            );
        });
    }

    fn render_results(&mut self, ui: &mut egui::Ui) {
        let result = match &self.result {
            Some(r) => r.clone(),
            None => return,
        };

        // Header bar
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Vanguard")
                    .size(24.0)
                    .color(theme::COLOR_PRIMARY)
                    .strong()
            );
            ui.add_space(16.0);
            
            // Breadcrumbs
            let p = self.selected_path.clone().unwrap_or_else(|| ".".to_string());
            let folder_name = std::path::Path::new(&p)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new(&p))
                .to_string_lossy()
                .into_owned();
            ui.label(
                egui::RichText::new(folder_name)
                    .size(14.0)
                    .color(theme::COLOR_TEXT_DIM)
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if theme::premium_icon_button(ui, "Back", theme::draw_icon_back, theme::COLOR_TEXT_DIM, egui::vec2(100.0, 32.0)).clicked() {
                    self.screen = Screen::AnalysisMenu;
                }

                if theme::premium_icon_button(ui, "Copy Report", theme::draw_icon_copy, theme::COLOR_ACCENT, egui::vec2(140.0, 32.0)).clicked() {
                    let md = crate::output::markdown::render_markdown(&result);
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&md);
                        self.status_msg = "Copied to clipboard ✓".to_string();
                    }
                }

                if theme::premium_button(ui, "JSON", theme::COLOR_SURFACE_LIGHT, egui::vec2(100.0, 32.0)).clicked() {
                    // Try to serialize, or fallback to markdown if json isn't available
                    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Error generating JSON".to_string());
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&json);
                        self.status_msg = "JSON Copied ✓".to_string();
                    }
                }

                if !self.status_msg.is_empty() {
                    ui.label(
                        egui::RichText::new(&self.status_msg)
                            .size(12.0)
                            .color(theme::COLOR_SUCCESS)
                    );
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        
        let available = ui.available_size();
        
        // Split layout: 25% File Tree, 75% Details
        ui.columns(2, |cols| {
            cols[0].set_width(available.x * 0.25);
            
            // LEFT COLUMN: Overview & File Tree
            cols[0].vertical(|ui| {
                ui.add_space(8.0);
                
                // Overview Board
                let s = &result.summary;
                egui::Frame::new()
                    .fill(theme::COLOR_SURFACE_LIGHT)
                    .corner_radius(6)
                    .inner_margin(12.0)
                    .stroke(egui::Stroke::new(1.0, theme::COLOR_BORDER))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("FILES").size(10.0).color(theme::COLOR_TEXT_MUTED));
                                ui.label(egui::RichText::new(s.files_analyzed.to_string()).size(20.0).color(theme::COLOR_TEXT_PRIMARY).strong());
                            });
                            ui.add_space(12.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("ISSUES").size(10.0).color(theme::COLOR_TEXT_MUTED));
                                ui.label(egui::RichText::new(s.total_issues.to_string()).size(20.0).color(if s.total_issues > 0 { theme::COLOR_WARNING } else { theme::COLOR_SUCCESS }).strong());
                            });
                        });
                        
                        ui.add_space(16.0);
                        
                        let health = (100.0 - (s.total_issues as f32 * 2.0)).max(0.0) as i32;
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Health Index:").size(12.0).color(theme::COLOR_TEXT_DIM));
                            ui.label(egui::RichText::new(format!("{}", health)).size(14.0).color(if health > 80 { theme::COLOR_SUCCESS } else { theme::COLOR_WARNING }).strong());
                        });
                    });

                ui.add_space(16.0);
                ui.label(egui::RichText::new("FILES WITH ISSUES").size(10.0).color(theme::COLOR_TEXT_MUTED).strong());
                ui.add_space(8.0);

                // Simple list of files with issues for the current mode
                egui::ScrollArea::vertical().id_salt("file_tree").show(ui, |ui| {
                    let mut has_files = false;
                    for f in &result.files {
                        let has_relevant_issues = f.issues.iter().any(|issue| {
                            match self.analysis_mode {
                                AnalysisMode::Security => issue.category == crate::models::Category::Security,
                                AnalysisMode::AI => issue.category == crate::models::Category::AIPattern,
                                AnalysisMode::Quality => issue.category == crate::models::Category::Quality,
                                AnalysisMode::Complexity => issue.category == crate::models::Category::Complexity,
                                AnalysisMode::Git => issue.category == crate::models::Category::Git,
                            }
                        });

                        if !has_relevant_issues { continue; }
                        has_files = true;

                        let root = std::path::Path::new(&result.root_dir);
                        let file_path = std::path::Path::new(&f.path);
                        let rel_path = file_path.strip_prefix(root).unwrap_or(file_path).to_string_lossy();

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📄").size(12.0));
                            ui.label(egui::RichText::new(rel_path).size(12.0).color(theme::COLOR_TEXT_SECONDARY));
                        });
                        ui.add_space(4.0);
                    }
                    
                    if !has_files {
                        ui.label(egui::RichText::new("No files found.").size(12.0).color(theme::COLOR_TEXT_DIM));
                    }
                });
            });
            
            // RIGHT COLUMN: Issue Cards
            cols[1].vertical(|ui| {
                ui.add_space(8.0);
                let mode_label = match self.analysis_mode {
                    AnalysisMode::Security => "Security Issues",
                    AnalysisMode::AI => "AI Detection Results",
                    AnalysisMode::Quality => "Quality Issues",
                    AnalysisMode::Complexity => "Architecture Complexity",
                    AnalysisMode::Git => "Git Churn Diagnostics",
                };
                ui.label(
                    egui::RichText::new(mode_label)
                        .size(18.0)
                        .color(theme::COLOR_ACCENT)
                        .strong()
                );
                ui.add_space(16.0);
                
                egui::ScrollArea::vertical().id_salt("issue_list").show(ui, |ui| {
                    let mut has_any = false;
                    for f in &result.files {
                        let relevant_issues: Vec<_> = f.issues.iter().filter(|issue| {
                            match self.analysis_mode {
                                AnalysisMode::Security => issue.category == crate::models::Category::Security,
                                AnalysisMode::AI => issue.category == crate::models::Category::AIPattern,
                                AnalysisMode::Quality => issue.category == crate::models::Category::Quality,
                                AnalysisMode::Complexity => issue.category == crate::models::Category::Complexity,
                                AnalysisMode::Git => issue.category == crate::models::Category::Git,
                            }
                        }).collect();

                        if relevant_issues.is_empty() { continue; }
                        has_any = true;

                        let root = std::path::Path::new(&result.root_dir);
                        let file_path = std::path::Path::new(&f.path);
                        let rel_path = file_path.strip_prefix(root).unwrap_or(file_path).to_string_lossy();

                        ui.label(
                            egui::RichText::new(format!("📄 {}", rel_path))
                                .size(14.0)
                                .color(theme::COLOR_TEXT_PRIMARY)
                                .strong()
                        );
                        ui.add_space(8.0);

                        for issue in &relevant_issues {
                            let (badge_text, badge_color, badge_bg) = match issue.severity {
                                crate::models::Severity::Critical => ("CRIT", theme::COLOR_ERROR, theme::COLOR_ERROR_CONTAINER),
                                crate::models::Severity::Error => ("ERR", theme::COLOR_PRIMARY, theme::COLOR_PRIMARY_CONTAINER),
                                crate::models::Severity::Warning => ("WARN", theme::COLOR_WARNING, theme::COLOR_WARNING.linear_multiply(0.2)),
                                crate::models::Severity::Info | crate::models::Severity::Note | crate::models::Severity::None => ("INFO", theme::COLOR_INFO, theme::COLOR_INFO.linear_multiply(0.2)),
                            };
                            
                            egui::Frame::new()
                                .fill(theme::COLOR_SURFACE)
                                .stroke(egui::Stroke::new(1.0, theme::COLOR_BORDER))
                                .corner_radius(6)
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());

                                    ui.horizontal(|ui| {
                                        // Badge
                                        egui::Frame::new()
                                            .fill(badge_bg)
                                            .corner_radius(4)
                                            .inner_margin(egui::Margin::symmetric(6, 2))
                                            .show(ui, |ui| {
                                                ui.label(egui::RichText::new(badge_text).size(10.0).color(badge_color).strong());
                                            });
                                            
                                        ui.add_space(8.0);
                                        ui.label(egui::RichText::new(format!("Line {}", issue.line)).size(11.0).color(theme::COLOR_TEXT_DIM).monospace());
                                        
                                        // Dynamic copy button per issue
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button(egui::RichText::new("📋").size(12.0).color(theme::COLOR_TEXT_DIM)).on_hover_text("Copy Issue").clicked() {
                                                let fix_text = issue.fix.as_deref().unwrap_or("None");
                                                let text = format!("[{}] {}:{} — {}\n💡 Fix: {}", badge_text, rel_path, issue.line, issue.message, fix_text);
                                                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                                    let _ = clipboard.set_text(&text);
                                                    self.status_msg = "Issue copied ✓".to_string();
                                                }
                                            }
                                        });
                                    });
                                    
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new(&issue.message).size(14.0).color(theme::COLOR_TEXT_PRIMARY).strong());
                                    
                                    if let Some(ref fix) = issue.fix {
                                        ui.add_space(12.0);
                                        egui::Frame::new()
                                            .fill(theme::COLOR_BG)
                                            .stroke(egui::Stroke::new(1.0, theme::COLOR_BORDER))
                                            .corner_radius(4)
                                            .inner_margin(12.0)
                                            .show(ui, |ui| {
                                                ui.set_width(ui.available_width());
                                                ui.label(egui::RichText::new("💡 SUGGESTED FIX").size(10.0).color(theme::COLOR_PRIMARY).strong());
                                                ui.add_space(4.0);
                                                ui.label(egui::RichText::new(fix).size(12.0).color(theme::COLOR_INFO).monospace());
                                            });
                                    }
                                });
                                
                            ui.add_space(12.0);
                        }
                    }
                    
                    if !has_any {
                        ui.add_space(40.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("✅ No issues found!")
                                    .size(18.0)
                                    .color(theme::COLOR_SUCCESS)
                            );
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("Your codebase looks clean for this analysis mode.")
                                    .size(13.0)
                                    .color(theme::COLOR_TEXT_DIM)
                            );
                        });
                    }
                });
            });
        });
    }

    fn render_help(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();

        ui.vertical_centered(|ui| {
            ui.add_space(available.y * 0.1);

            ui.label(
                egui::RichText::new("COMMAND INDEX")
                    .size(20.0)
                    .color(theme::COLOR_ACCENT)
                    .strong()
            );
            ui.add_space(20.0);
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            let sections = vec![
                ("Navigation", vec![
                    ("Click 'Open Project'", "Select a folder to analyze"),
                    ("Browse button", "Opens native file picker"),
                    ("❮ Back", "Return to previous screen"),
                ]),
                ("Analysis", vec![
                    ("Security & Vulnerability", "Scan for secrets, injections, crypto misuse"),
                    ("AI Code Detection", "Detect AI-generated code patterns"),
                    ("Code Quality", "Find complexity, duplication, dead code"),
                ]),
                ("Results", vec![
                    ("Scroll", "Mouse wheel or scrollbar"),
                    ("Copy", "Copy Markdown report to clipboard"),
                    ("❮ Back", "Return to analysis menu"),
                ]),
                ("CLI Usage", vec![
                    ("fuji .", "Launch GUI (default)"),
                    ("fuji --format json .", "JSON report to stdout"),
                    ("fuji --format md .", "Markdown report to stdout"),
                    ("fuji --ci .", "CI mode with exit codes"),
                ]),
            ];

            for (header, items) in &sections {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(*header)
                        .size(15.0)
                        .color(theme::COLOR_ACCENT2)
                        .strong()
                );
                ui.add_space(4.0);

                for (key, desc) in items {
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new(*key)
                                .size(13.0)
                                .color(theme::COLOR_ACCENT)
                                .monospace()
                        );
                        ui.label(
                            egui::RichText::new(format!("  —  {}", desc))
                                .size(13.0)
                                .color(theme::COLOR_TEXT_SECONDARY)
                        );
                    });
                }
            }

            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                if ui.add(
                    egui::Button::new(
                        egui::RichText::new("Back").size(14.0).color(theme::COLOR_TEXT_DIM)
                    ).fill(egui::Color32::TRANSPARENT)
                ).clicked() {
                    self.screen = Screen::Home;
                }
            });
        });
    }
}

impl eframe::App for FujiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply dark theme
        theme::apply_theme(ctx);

        // Keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            match self.screen {
                Screen::Home => {
                    std::process::exit(0);
                }
                Screen::Help => self.screen = Screen::Home,
                Screen::AnalysisMenu => self.screen = Screen::Home,
                Screen::Results => self.screen = Screen::AnalysisMenu,
                Screen::Loading => {}
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(theme::COLOR_BG))
            .show(ctx, |ui| {
                match self.screen.clone() {
                    Screen::Home => self.render_home(ui),
                    Screen::AnalysisMenu => self.render_analysis_menu(ui, ctx),
                    Screen::Loading => self.render_loading(ui, ctx),
                    Screen::Results => self.render_results(ui),
                    Screen::Help => self.render_help(ui),
                }
            });
    }
}
