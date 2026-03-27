use egui;
use std::sync::Mutex;

// ═══════════════════════════════════════════════════════════════
// ═══ COLOR PALETTE: MATERIAL YOU DARK (FUJI V2) ═════════════════
// ═══════════════════════════════════════════════════════════════

// Backgrounds & surfaces
pub const COLOR_BG: egui::Color32 = egui::Color32::from_rgb(19, 19, 21); // #131315
pub const COLOR_SURFACE: egui::Color32 = egui::Color32::from_rgb(32, 31, 34); // #201f22
pub const COLOR_SURFACE_LIGHT: egui::Color32 = egui::Color32::from_rgb(42, 42, 44); // #2a2a2c
pub const COLOR_BORDER: egui::Color32 = egui::Color32::from_rgb(53, 52, 55); // #353437
pub const COLOR_BORDER_FOCUS: egui::Color32 = egui::Color32::from_rgb(73, 68, 84); // #494454

// Text hierarchy
pub const COLOR_TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(229, 225, 228); // #e5e1e4
pub const COLOR_TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(203, 195, 215); // #cbc3d7
pub const COLOR_TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(149, 142, 160); // #958ea0
pub const COLOR_TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(100, 95, 110);

// Primary accents
pub const COLOR_PRIMARY: egui::Color32 = egui::Color32::from_rgb(208, 188, 255); // #d0bcff (Soft Purple)
pub const COLOR_PRIMARY_CONTAINER: egui::Color32 = egui::Color32::from_rgb(79, 55, 139); // #4f378b
#[allow(dead_code)]
pub const COLOR_ON_PRIMARY: egui::Color32 = egui::Color32::from_rgb(56, 30, 114); // #381e72
pub const COLOR_SECONDARY: egui::Color32 = egui::Color32::from_rgb(137, 206, 255); // #89ceff (Soft Blue)
pub const COLOR_ACCENT: egui::Color32 = COLOR_PRIMARY; // Alias
pub const COLOR_ACCENT2: egui::Color32 = COLOR_SECONDARY; // Alias

// Semantic colors
pub const COLOR_SUCCESS: egui::Color32 = egui::Color32::from_rgb(134, 239, 172); // Green
pub const COLOR_WARNING: egui::Color32 = egui::Color32::from_rgb(251, 191, 36);  // Amber
pub const COLOR_ERROR: egui::Color32 = egui::Color32::from_rgb(255, 180, 171);   // Red
pub const COLOR_ERROR_CONTAINER: egui::Color32 = egui::Color32::from_rgb(147, 0, 10); // #93000a
pub const COLOR_INFO: egui::Color32 = egui::Color32::from_rgb(137, 206, 255);    // Blue

// Title
#[allow(dead_code)]
pub const COLOR_TITLE: egui::Color32 = egui::Color32::from_rgb(229, 225, 228);   // Same as primary text now

// ═══════════════════════════════════════════════════════════════
// ═══ CYBERPUNK/HACKER THEME EXTENSIONS ═══════════════════════════
// ═══════════════════════════════════════════════════════════════

// ─── Matrix Rain Colors ──────────────────────────────────────
pub const MATRIX_GREEN: egui::Color32 = egui::Color32::from_rgb(0, 255, 65);     // Classic Matrix green
pub const MATRIX_GREEN_DIM: egui::Color32 = egui::Color32::from_rgb(0, 150, 40); // Dimmed Matrix green
pub const MATRIX_GREEN_GLOW: egui::Color32 = egui::Color32::from_rgb(100, 255, 150); // Glowing green
pub const MATRIX_TRAIL: egui::Color32 = egui::Color32::from_rgb(0, 100, 30);      // Trail fade

// ─── Neon/Cyberpunk Accents ────────────────────────────────────
pub const NEON_CYAN: egui::Color32 = egui::Color32::from_rgb(0, 255, 255);       // Cyan neon
pub const NEON_PINK: egui::Color32 = egui::Color32::from_rgb(255, 0, 255);       // Magenta neon
pub const NEON_AMBER: egui::Color32 = egui::Color32::from_rgb(255, 176, 0);      // Amber phosphor
pub const NEON_RED: egui::Color32 = egui::Color32::from_rgb(255, 50, 50);        // Alert red
pub const NEON_BLUE: egui::Color32 = egui::Color32::from_rgb(50, 150, 255);      // Electric blue

// ─── Terminal/CRT Colors ─────────────────────────────────────
pub const TERMINAL_GREEN: egui::Color32 = egui::Color32::from_rgb(51, 255, 51);  // Phosphor green
pub const TERMINAL_AMBER: egui::Color32 = egui::Color32::from_rgb(255, 204, 0); // Phosphor amber
pub const CRT_SCANLINE: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 30); // Scanline overlay
pub const CRT_GLOW: egui::Color32 = egui::Color32::from_rgba_premultiplied(100, 255, 100, 20); // Screen glow

// ─── Hacker/Security Colors ────────────────────────────────────
pub const HACKER_DARK: egui::Color32 = egui::Color32::from_rgb(10, 20, 15);      // Deep hacker bg
pub const HACKER_GRID: egui::Color32 = egui::Color32::from_rgb(0, 80, 40);       // Grid lines
pub const SECURITY_CLEAR: egui::Color32 = egui::Color32::from_rgb(0, 255, 100);  // Clear/secure
pub const SECURITY_WARNING: egui::Color32 = egui::Color32::from_rgb(255, 165, 0); // Warning
pub const SECURITY_CRITICAL: egui::Color32 = egui::Color32::from_rgb(255, 0, 80);   // Critical

// ─── Binary/Data Visualization ─────────────────────────────────
pub const BINARY_0: egui::Color32 = egui::Color32::from_rgb(60, 60, 70);         // Zero bit
pub const BINARY_1: egui::Color32 = egui::Color32::from_rgb(0, 255, 100);        // One bit
pub const HEX_BYTE: egui::Color32 = egui::Color32::from_rgb(180, 180, 200);      // Hex values
pub const HEX_OFFSET: egui::Color32 = egui::Color32::from_rgb(100, 100, 120);    // Offset markers

// ═══════════════════════════════════════════════════════════════
// ═══ ANIMATION & EFFECTS STATE ═══════════════════════════════════
// ═══════════════════════════════════════════════════════════════

/// Global animation state for cyber effects
pub struct CyberAnimationState {
    pub matrix_drops: Vec<MatrixDrop>,
    pub last_update: f64,
    pub pulse_phase: f32,
    pub glitch_offset: (f32, f32),
    pub scanline_offset: f32,
}

impl Default for CyberAnimationState {
    fn default() -> Self {
        Self {
            matrix_drops: Vec::new(),
            last_update: 0.0,
            pulse_phase: 0.0,
            glitch_offset: (0.0, 0.0),
            scanline_offset: 0.0,
        }
    }
}

/// Single matrix rain drop
#[derive(Clone, Debug)]
pub struct MatrixDrop {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub char_code: char,
    pub brightness: f32,
    pub length: usize,
    pub trail: Vec<f32>,
}

impl MatrixDrop {
    pub fn new(x: f32, _screen_height: f32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self {
            x,
            y: rng.gen_range(-50.0..0.0),
            speed: rng.gen_range(2.0..8.0),
            char_code: rng.gen_range(0x30..0x7A) as u8 as char,
            brightness: rng.gen_range(0.3..1.0),
            length: rng.gen_range(5..20),
            trail: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32, screen_height: f32) {
        self.y += self.speed * dt * 60.0;
        self.trail.push(self.y);
        if self.trail.len() > self.length {
            self.trail.remove(0);
        }

        // Reset if off screen
        if self.y > screen_height + 50.0 {
            self.y = -20.0;
            self.trail.clear();
        }
    }
}

// Static storage for animation state
use std::sync::OnceLock;
static CYBER_STATE: OnceLock<Mutex<CyberAnimationState>> = OnceLock::new();

pub fn get_cyber_state() -> &'static Mutex<CyberAnimationState> {
    CYBER_STATE.get_or_init(|| Mutex::new(CyberAnimationState::default()))
}

// ═══════════════════════════════════════════════════════════════
// ═══ EFFECT RENDERING FUNCTIONS ═════════════════════════════════
// ═══════════════════════════════════════════════════════════════

/// Apply the Material You dark theme to egui with cyber enhancements
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Dark visuals with cyber tweaks
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = COLOR_BG;
    visuals.window_fill = COLOR_SURFACE;
    visuals.extreme_bg_color = COLOR_BG;

    visuals.widgets.noninteractive.bg_fill = COLOR_SURFACE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, COLOR_TEXT_SECONDARY);

    visuals.widgets.inactive.bg_fill = COLOR_SURFACE_LIGHT;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, COLOR_TEXT_PRIMARY);

    visuals.widgets.hovered.bg_fill = COLOR_BORDER_FOCUS;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, COLOR_ACCENT);

    visuals.widgets.active.bg_fill = COLOR_ACCENT;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 0, 145));

    visuals.selection.bg_fill = COLOR_ACCENT.linear_multiply(0.3);
    visuals.selection.stroke = egui::Stroke::new(1.0, COLOR_ACCENT);

    // Prototype uses tighter radii (4px mostly, 8px for containers)
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);

    style.visuals = visuals;

    // Fonts: Use proportional, increase sizes slightly
    style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::new(28.0, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Button, egui::FontId::new(13.0, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::new(12.0, egui::FontFamily::Monospace));

    // Spacing
    style.spacing.item_spacing = egui::vec2(12.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);

    ctx.set_style(style);
}

/// Update cyber animation state
pub fn update_cyber_animations(ctx: &egui::Context, rect: egui::Rect) {
    let time = ctx.input(|i| i.time);
    let dt = ctx.input(|i| i.stable_dt);

    if let Ok(mut state) = get_cyber_state().lock() {
        // Update pulse phase (cycles every 2 seconds)
        state.pulse_phase = (time % 2.0) as f32 / 2.0;

        // Update scanline
        state.scanline_offset += dt * 30.0;
        if state.scanline_offset > rect.height() {
            state.scanline_offset = 0.0;
        }

        // Update matrix drops
        state.matrix_drops.retain_mut(|drop| {
            drop.update(dt, rect.height());
            drop.y < rect.height() + 50.0
        });

        // Spawn new drops occasionally
        if state.matrix_drops.len() < 50 && time - state.last_update > 0.1 {
            state.last_update = time;
            let _cols = (rect.width() / 20.0) as i32;
            for _ in 0..3 {
                let x = rect.left() + (rand::random::<f32>() * rect.width());
                state.matrix_drops.push(MatrixDrop::new(x, rect.height()));
            }
        }
    }

    ctx.request_repaint();
}

/// Draw matrix rain background effect
pub fn draw_matrix_rain(ui: &mut egui::Ui, _rect: egui::Rect, intensity: f32) {
    let painter = ui.painter();

    if let Ok(state) = get_cyber_state().lock() {
        for drop in &state.matrix_drops {
            // Draw trail
            for (i, &y) in drop.trail.iter().enumerate() {
                let alpha = (i as f32 / drop.length as f32) * intensity * drop.brightness;
                let color = egui::Color32::from_rgba_premultiplied(
                    0,
                    (255.0 * alpha) as u8,
                    (100.0 * alpha) as u8,
                    (255.0 * alpha * 0.5) as u8,
                );
                painter.text(
                    egui::pos2(drop.x, y),
                    egui::Align2::CENTER_CENTER,
                    drop.char_code.to_string(),
                    egui::FontId::new(14.0, egui::FontFamily::Monospace),
                    color,
                );
            }

            // Draw head
            let head_color = egui::Color32::from_rgba_premultiplied(
                200,
                255,
                200,
                (255.0 * intensity * drop.brightness) as u8,
            );
            painter.text(
                egui::pos2(drop.x, drop.y),
                egui::Align2::CENTER_CENTER,
                drop.char_code.to_string(),
                egui::FontId::new(14.0, egui::FontFamily::Monospace),
                head_color,
            );
        }
    }
}

/// Draw CRT scanline effect
pub fn draw_scanlines(ui: &mut egui::Ui, rect: egui::Rect) {
    let painter = ui.painter();
    let line_height = 4.0;
    let gap = 2.0;

    for y in (rect.top() as i32..rect.bottom() as i32).step_by((line_height + gap) as usize) {
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.left(), y as f32),
                egui::vec2(rect.width(), line_height),
            ),
            egui::CornerRadius::ZERO,
            CRT_SCANLINE,
        );
    }
}

/// Draw animated scanline sweep
pub fn draw_scanline_sweep(ui: &mut egui::Ui, rect: egui::Rect) {
    let painter = ui.painter();

    if let Ok(state) = get_cyber_state().lock() {
        let y = rect.top() + state.scanline_offset;
        if y < rect.bottom() {
            // Glow line
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.left(), y - 10.0),
                    egui::vec2(rect.width(), 20.0),
                ),
                egui::CornerRadius::ZERO,
                egui::Color32::from_rgba_premultiplied(0, 255, 100, 30),
            );
            // Core line
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.left(), y),
                    egui::vec2(rect.width(), 2.0),
                ),
                egui::CornerRadius::ZERO,
                MATRIX_GREEN,
            );
        }
    }
}

/// Draw hex grid background
pub fn draw_hex_grid(ui: &mut egui::Ui, rect: egui::Rect, hex_size: f32) {
    let painter = ui.painter();
    let sqrt3 = 1.73205;
    let hex_width = hex_size * 2.0;
    let hex_height = sqrt3 * hex_size;

    for row in 0..((rect.height() / hex_height) as i32 + 2) {
        for col in 0..((rect.width() / (hex_width * 0.75)) as i32 + 2) {
            let x_offset = if row % 2 == 0 { 0.0 } else { hex_width * 0.5 };
            let x = rect.left() + x_offset + col as f32 * hex_width * 0.75;
            let y = rect.top() + row as f32 * hex_height;

            if x > rect.right() || y > rect.bottom() {
                continue;
            }

            // Draw hex outline
            let points: Vec<egui::Pos2> = (0..6)
                .map(|i| {
                    let angle = (i as f32 * 60.0 - 30.0).to_radians();
                    egui::pos2(
                        x + hex_size * angle.cos(),
                        y + hex_size * angle.sin(),
                    )
                })
                .collect();

            painter.add(egui::Shape::line(
                points.windows(2).map(|w| [w[0], w[1]]).flatten().collect(),
                egui::Stroke::new(1.0, HACKER_GRID.linear_multiply(0.3)),
            ));
        }
    }
}

/// Draw digital noise/static effect
pub fn draw_digital_noise(ui: &mut egui::Ui, rect: egui::Rect, density: f32) {
    use rand::Rng;
    let painter = ui.painter();
    let mut rng = rand::thread_rng();

    let num_dots = (rect.area() * density) as i32;
    for _ in 0..num_dots {
        let x = rng.gen_range(rect.left()..rect.right());
        let y = rng.gen_range(rect.top()..rect.bottom());
        let size = rng.gen_range(1.0..3.0);
        let brightness = rng.gen_range(0.1..0.5);

        let color = if rng.gen_bool(0.7) {
            egui::Color32::from_rgba_premultiplied(
                (255.0 * brightness) as u8,
                (255.0 * brightness) as u8,
                (255.0 * brightness) as u8,
                (100.0 * brightness) as u8,
            )
        } else {
            egui::Color32::from_rgba_premultiplied(
                0,
                (255.0 * brightness) as u8,
                (100.0 * brightness) as u8,
                (150.0 * brightness) as u8,
            )
        };

        painter.circle_filled(egui::pos2(x, y), size, color);
    }
}

/// Create a glowing text style
pub fn glowing_text(text: &str, size: f32, color: egui::Color32, _glow_intensity: f32) -> egui::RichText {
    egui::RichText::new(text)
        .size(size)
        .color(color)
        .strong()
}

/// Create a cyber frame with neon border
pub fn cyber_frame(fill: egui::Color32, stroke_color: egui::Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(egui::Stroke::new(2.0, stroke_color))
        .corner_radius(4)
        .inner_margin(16.0)
}

/// Create a glowing neon frame
pub fn neon_frame(color: egui::Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(COLOR_SURFACE)
        .stroke(egui::Stroke::new(2.0, color))
        .corner_radius(6)
        .inner_margin(12.0)
}

/// Draw a pulsing glow effect around a rect
pub fn draw_pulse_glow(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32, intensity: f32) {
    let glow_size = 20.0 * intensity;

    for i in (1..=5).rev() {
        let expand = glow_size * (i as f32 / 5.0);
        let alpha = (50.0 * intensity * (i as f32 / 5.0)) as u8;
        let glow_rect = rect.expand(expand);
        let glow_color = egui::Color32::from_rgba_premultiplied(
            color.r(),
            color.g(),
            color.b(),
            alpha,
        );
        painter.rect_stroke(
            glow_rect,
            egui::CornerRadius::same(8),
            egui::Stroke::new(1.0, glow_color),
            egui::StrokeKind::Inside,
        );
    }
}

/// Get a pulsing color based on time
pub fn pulse_color(base_color: egui::Color32, speed: f32, ctx: &egui::Context) -> egui::Color32 {
    let time = ctx.input(|i| i.time) as f32;
    let pulse = (time * speed).sin() * 0.5 + 0.5;
    base_color.linear_multiply(pulse)
}

/// Binary pattern generator for decoration
pub fn generate_binary_pattern(length: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| if rng.gen_bool(0.5) { '1' } else { '0' })
        .collect()
}

/// Draw a decorative binary border
pub fn draw_binary_border(ui: &mut egui::Ui, rect: egui::Rect) {
    let painter = ui.painter();
    let pattern = generate_binary_pattern((rect.width() / 10.0) as usize);

    // Top border
    painter.text(
        rect.left_top() + egui::vec2(rect.width() / 2.0, 0.0),
        egui::Align2::CENTER_TOP,
        &pattern,
        egui::FontId::new(10.0, egui::FontFamily::Monospace),
        HACKER_GRID,
    );

    // Bottom border
    painter.text(
        rect.left_bottom() + egui::vec2(rect.width() / 2.0, -10.0),
        egui::Align2::CENTER_BOTTOM,
        &pattern,
        egui::FontId::new(10.0, egui::FontFamily::Monospace),
        HACKER_GRID,
    );
}

// ═══════════════════════════════════════════════════════════════
// ═══ PREMIUM UI COMPONENTS & VECTOR ICONS ══════════════════════
// ═══════════════════════════════════════════════════════════════

pub fn draw_icon_copy(ui: &mut egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let p = ui.painter();
    let s = egui::Stroke::new(1.5, color);
    let r1 = egui::Rect::from_min_max(
        rect.min + egui::vec2(2.0, 2.0),
        rect.max - egui::vec2(4.0, 4.0),
    );
    let r2 = egui::Rect::from_min_max(
        rect.min + egui::vec2(4.0, 4.0),
        rect.max - egui::vec2(2.0, 2.0),
    );
    p.rect_stroke(r1, egui::CornerRadius::same(2), s, egui::StrokeKind::Middle);
    p.rect_stroke(r2, egui::CornerRadius::same(2), s, egui::StrokeKind::Middle);
}

pub fn draw_icon_back(ui: &mut egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let p = ui.painter();
    let s = egui::Stroke::new(2.0, color);
    let center = rect.center();
    let size = rect.width() * 0.3;
    p.line_segment([center + egui::vec2(-size, 0.0), center + egui::vec2(size, 0.0)], s);
    p.line_segment([center + egui::vec2(-size, 0.0), center + egui::vec2(0.0, -size)], s);
    p.line_segment([center + egui::vec2(-size, 0.0), center + egui::vec2(0.0, size)], s);
}

pub fn draw_icon_folder(ui: &mut egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let p = ui.painter();
    let s = egui::Stroke::new(1.5, color);
    let r = rect.shrink(2.0);
    let tab_right = r.left() + r.width() * 0.4;
    let tab_bottom = r.top() + r.height() * 0.25;
    
    let path = vec![
        r.left_bottom(),
        r.right_bottom(),
        egui::pos2(r.right(), tab_bottom),
        egui::pos2(tab_right, tab_bottom),
        egui::pos2(tab_right - 2.0, r.top()),
        r.left_top(),
        r.left_bottom(),
    ];
    p.add(egui::Shape::Path(egui::epaint::PathShape::line(path, s)));
}

pub fn draw_icon_check(ui: &mut egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let p = ui.painter();
    let s = egui::Stroke::new(2.0, color);
    let center = rect.center();
    let size = rect.width() * 0.3;
    p.line_segment([center + egui::vec2(-size, -size*0.2), center + egui::vec2(-size*0.2, size)], s);
    p.line_segment([center + egui::vec2(-size*0.2, size), center + egui::vec2(size, -size)], s);
}

pub fn premium_button(ui: &mut egui::Ui, text: &str, base_color: egui::Color32, size: egui::Vec2) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    
    let is_hovered = response.hovered();
    let is_clicked = response.is_pointer_button_down_on();
    
    let hover_factor = ui.ctx().animate_bool(response.id.with("hover"), is_hovered);
    
    let current_color = if is_clicked {
        base_color.linear_multiply(0.3)
    } else {
        base_color.linear_multiply(0.1 + (hover_factor * 0.15))
    };
    
    let border_color = if is_hovered { 
        base_color.linear_multiply(0.8) 
    } else { 
        base_color.linear_multiply(0.3) 
    };
    
    ui.painter().rect_filled(rect, egui::CornerRadius::same(6), current_color);
    ui.painter().rect_stroke(rect, egui::CornerRadius::same(6), egui::Stroke::new(1.0, border_color), egui::StrokeKind::Inside);
    
    let text_color = if is_hovered { egui::Color32::WHITE } else { COLOR_TEXT_PRIMARY };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
        text_color,
    );
    
    response
}

pub fn premium_icon_button(ui: &mut egui::Ui, text: &str, draw_icon: fn(&mut egui::Ui, egui::Rect, egui::Color32), base_color: egui::Color32, size: egui::Vec2) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let is_hovered = response.hovered();
    let is_clicked = response.is_pointer_button_down_on();
    let hover_factor = ui.ctx().animate_bool(response.id.with("hover"), is_hovered);
    
    let current_color = if is_clicked {
        base_color.linear_multiply(0.3)
    } else {
        base_color.linear_multiply(0.1 + (hover_factor * 0.15))
    };
    let border_color = if is_hovered { base_color.linear_multiply(0.8) } else { base_color.linear_multiply(0.3) };
    
    ui.painter().rect_filled(rect, egui::CornerRadius::same(6), current_color);
    ui.painter().rect_stroke(rect, egui::CornerRadius::same(6), egui::Stroke::new(1.0, border_color), egui::StrokeKind::Inside);
    
    let text_color = if is_hovered { egui::Color32::WHITE } else { COLOR_TEXT_PRIMARY };
    
    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 12.0, rect.center().y - 8.0),
        egui::vec2(16.0, 16.0)
    );
    draw_icon(ui, icon_rect, text_color);
    
    ui.painter().text(
        egui::pos2(rect.left() + 36.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
        text_color,
    );
    
    response
}

pub fn premium_analysis_card(
    ui: &mut egui::Ui, 
    title: &str, 
    desc: &str, 
    base_color: egui::Color32, 
    selected: bool,
    draw_icon: fn(&mut egui::Ui, egui::Rect, egui::Color32)
) -> egui::Response {
    let size = egui::vec2(ui.available_width().min(600.0), 80.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    
    let is_hovered = response.hovered();
    let hover_factor = ui.ctx().animate_bool(response.id.with("hover"), is_hovered);
    
    let bg_color = if selected {
        base_color.linear_multiply(0.15)
    } else {
        base_color.linear_multiply(0.05 + (hover_factor * 0.05))
    };
    
    let border_color = if selected {
        base_color
    } else if is_hovered {
        base_color.linear_multiply(0.5)
    } else {
        COLOR_BORDER
    };
    
    ui.painter().rect_filled(rect, egui::CornerRadius::same(8), bg_color);
    ui.painter().rect_stroke(rect, egui::CornerRadius::same(8), egui::Stroke::new(if selected { 2.0 } else { 1.0 }, border_color), egui::StrokeKind::Inside);
    
    let icon_box = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 16.0, rect.center().y - 20.0),
        egui::vec2(40.0, 40.0)
    );
    ui.painter().rect_filled(icon_box, egui::CornerRadius::same(6), base_color.linear_multiply(0.2));
    ui.painter().rect_stroke(icon_box, egui::CornerRadius::same(6), egui::Stroke::new(1.0, base_color.linear_multiply(0.5)), egui::StrokeKind::Inside);
    
    draw_icon(ui, icon_box.shrink(10.0), base_color);
    
    ui.painter().text(
        egui::pos2(rect.left() + 72.0, rect.center().y - 12.0),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::new(16.0, egui::FontFamily::Proportional),
        egui::Color32::WHITE,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 72.0, rect.center().y + 10.0),
        egui::Align2::LEFT_CENTER,
        desc,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
        COLOR_TEXT_DIM,
    );
    
    response
}
