use std::path::PathBuf;

use egui::{
    Color32, Context, CornerRadius, FontId, Stroke, Style, Visuals, epaint::AlphaFromCoverage, style::{Selection, TextCursorStyle, WidgetVisuals, Widgets}
};
use serde::Deserialize;
#[cfg(feature = "debug")]
use snemcore::Snemulator;

#[derive(Clone, Debug, Deserialize)]
pub struct AppTheme {
    // Base colors
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_tertiary: Color32,
    pub bg_elevated: Color32,
    
    // Text colors
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_disabled: Color32,
    
    // Accent colors
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_muted: Color32,
    
    // Borders
    pub border: Color32,
    pub border_focused: Color32,
    
    // Status colors
    pub error: Color32,
    pub warning: Color32,
    pub success: Color32,
    pub info: Color32,

    // Colors for icons (pause/play, debug buttons, etc.)
    pub icon_primary: Color32,
    pub icon_secondary: Color32,
    pub icon_tertiary: Color32,
    pub icon_hovered: Color32,
    pub icon_held: Color32,
    pub icon_disabled: Color32,
    
    // Debugger-specific colors
    pub highlight_line: Color32,      // Current execution line
    pub breakpoint: Color32,          // Breakpoint marker
    pub breakpoint_bg: Color32,       // Breakpoint line background
    pub watchpoint: Color32,          // Memory watchpoint
    pub modified: Color32,            // Recently modified value
    pub modified_bg: Color32,         // Modified disassembly instructions
    
    // Syntax highlighting (for disassembly/memory views)
    pub syntax_address: Color32,      // Memory addresses
    pub syntax_opcode: Color32,       // Instruction mnemonics
    pub syntax_register: Color32,     // CPU registers
    pub syntax_number: Color32,       // Numeric values
    pub syntax_label: Color32,        // Labels/symbols
    pub syntax_comment: Color32,      // Comments
    pub syntax_string: Color32,       // String data
    pub syntax_keyword: Color32,      // Special keywords
    pub syntax_directive: Color32,    // Assembler directives
    
    // Memory viewer specific
    pub memory_null: Color32,         // Zero bytes
    pub memory_ascii: Color32,        // Printable ASCII range
    pub memory_high: Color32,         // High bytes (0x80-0xFF)
    
    // Widget styling
    pub corner_radius: u8,
    pub widget_corner_radius: u8,
}

impl AppTheme {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("snemulator").join("theme.toml"))
    }

    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::Dark => Self::dark(),
            ThemePreset::Light => Self::light(),
            ThemePreset::Retro => Self::retro(),
            _ => Self::dark(),
        }
    }

    pub fn load_or_preset(preset: ThemePreset) -> Self {
        let Some(path) = Self::config_path() else { return Self::from_preset(preset) };
        let Ok(text) = std::fs::read_to_string(&path) else { return Self::from_preset(preset) };
        
        log::trace!("Loaded theme from {}: {}", path.display(), text);

        let theme = toml::from_str(&text);

        if let Err(_) = theme {
            return Self::from_preset(preset);
        }

        theme.unwrap()
    }

    /// Dark theme inspired by SNES aesthetic (purple/indigo accents)
    pub fn dark() -> Self {
        Self {
            // Base - deep blue-gray
            bg_primary: Color32::from_rgb(18, 18, 24),
            bg_secondary: Color32::from_rgb(24, 24, 32),
            bg_tertiary: Color32::from_rgb(32, 32, 42),
            bg_elevated: Color32::from_rgb(40, 40, 52),
            
            // Text
            text_primary: Color32::from_rgb(230, 230, 240),
            text_secondary: Color32::from_rgb(180, 180, 195),
            text_muted: Color32::from_rgb(120, 120, 140),
            text_disabled: Color32::from_rgb(80, 80, 95),
            
            // Accent - SNES purple
            accent: Color32::from_rgb(130, 100, 200),
            accent_hover: Color32::from_rgb(155, 125, 225),
            accent_muted: Color32::from_rgb(90, 70, 140),
            
            // Borders
            border: Color32::from_rgb(55, 55, 70),
            border_focused: Color32::from_rgb(130, 100, 200),
            
            // Status
            error: Color32::from_rgb(230, 85, 85),
            warning: Color32::from_rgb(230, 180, 80),
            success: Color32::from_rgb(85, 200, 120),
            info: Color32::from_rgb(80, 160, 230),

            icon_primary: Color32::from_rgb(60, 180, 255),
            icon_secondary: Color32::from_rgb(40, 220, 120),
            icon_tertiary: Color32::from_rgb(230, 90, 90),
            icon_hovered: Color32::from_rgb(60, 180, 255),
            icon_held: Color32::from_rgb(0, 60, 180),
            icon_disabled: Color32::from_rgb(120, 120, 120),
            
            // Debugger
            highlight_line: Color32::from_rgba_unmultiplied(130, 100, 200, 40),
            breakpoint: Color32::from_rgb(230, 70, 70),
            breakpoint_bg: Color32::from_rgba_unmultiplied(230, 70, 70, 25),
            watchpoint: Color32::from_rgb(230, 180, 80),
            modified: Color32::from_rgb(255, 200, 100),
            modified_bg: Color32::from_rgba_unmultiplied(255, 200, 100, 25),
            
            // Syntax - vibrant but not harsh
            syntax_address: Color32::from_rgb(130, 170, 200),
            syntax_opcode: Color32::from_rgb(200, 150, 220),
            syntax_register: Color32::from_rgb(240, 180, 130),
            syntax_number: Color32::from_rgb(180, 220, 150),
            syntax_label: Color32::from_rgb(130, 200, 200),
            syntax_comment: Color32::from_rgb(100, 110, 130),
            syntax_string: Color32::from_rgb(200, 180, 130),
            syntax_keyword: Color32::from_rgb(200, 120, 150),
            syntax_directive: Color32::from_rgb(150, 150, 200),
            
            // Memory viewer
            memory_null: Color32::from_rgb(70, 70, 85),
            memory_ascii: Color32::from_rgb(180, 220, 150),
            memory_high: Color32::from_rgb(220, 150, 180),
            
            // Styling
            corner_radius: 6,
            widget_corner_radius: 4,
        }
    }
    
    /// Light theme - clean and professional
    pub fn light() -> Self {
        Self {
            // Base - warm white/gray
            bg_primary: Color32::from_rgb(250, 250, 252),
            bg_secondary: Color32::from_rgb(242, 242, 247),
            bg_tertiary: Color32::from_rgb(232, 232, 240),
            bg_elevated: Color32::from_rgb(255, 255, 255),
            
            // Text
            text_primary: Color32::from_rgb(30, 30, 40),
            text_secondary: Color32::from_rgb(70, 70, 85),
            text_muted: Color32::from_rgb(120, 120, 135),
            text_disabled: Color32::from_rgb(170, 170, 180),
            
            // Accent - deeper purple for contrast
            accent: Color32::from_rgb(100, 70, 170),
            accent_hover: Color32::from_rgb(120, 90, 190),
            accent_muted: Color32::from_rgb(180, 160, 210),
            
            // Borders
            border: Color32::from_rgb(210, 210, 220),
            border_focused: Color32::from_rgb(100, 70, 170),
            
            // Status
            error: Color32::from_rgb(210, 50, 50),
            warning: Color32::from_rgb(200, 140, 20),
            success: Color32::from_rgb(40, 160, 80),
            info: Color32::from_rgb(40, 120, 200),

            icon_primary: Color32::from_rgb(60, 180, 255),
            icon_secondary: Color32::from_rgb(40, 220, 120),
            icon_tertiary: Color32::from_rgb(230, 90, 90),
            icon_hovered: Color32::from_rgb(60, 180, 255),
            icon_held: Color32::from_rgb(0, 60, 180),
            icon_disabled: Color32::from_rgb(120, 120, 120),
            
            // Debugger
            highlight_line: Color32::from_rgba_unmultiplied(100, 70, 170, 30),
            breakpoint: Color32::from_rgb(210, 50, 50),
            breakpoint_bg: Color32::from_rgba_unmultiplied(210, 50, 50, 20),
            watchpoint: Color32::from_rgb(200, 140, 20),
            modified: Color32::from_rgb(200, 120, 0),
            modified_bg: Color32::from_rgba_unmultiplied(200, 120, 0, 25),
            
            // Syntax - darker/more saturated for light bg
            syntax_address: Color32::from_rgb(50, 100, 150),
            syntax_opcode: Color32::from_rgb(140, 70, 160),
            syntax_register: Color32::from_rgb(180, 100, 40),
            syntax_number: Color32::from_rgb(60, 130, 60),
            syntax_label: Color32::from_rgb(40, 130, 130),
            syntax_comment: Color32::from_rgb(130, 140, 155),
            syntax_string: Color32::from_rgb(160, 100, 40),
            syntax_keyword: Color32::from_rgb(170, 50, 80),
            syntax_directive: Color32::from_rgb(90, 90, 160),
            
            // Memory viewer
            memory_null: Color32::from_rgb(180, 180, 190),
            memory_ascii: Color32::from_rgb(60, 130, 60),
            memory_high: Color32::from_rgb(160, 70, 110),
            
            // Styling
            corner_radius: 6,
            widget_corner_radius: 4,
        }
    }
    
    /// SNES-inspired retro theme (CRT-like)
    pub fn retro() -> Self {
        Self {
            // Base - dark with slight green tint (CRT phosphor)
            bg_primary: Color32::from_rgb(12, 16, 14),
            bg_secondary: Color32::from_rgb(18, 24, 20),
            bg_tertiary: Color32::from_rgb(26, 34, 28),
            bg_elevated: Color32::from_rgb(34, 44, 38),
            
            // Text - phosphor green/amber mix
            text_primary: Color32::from_rgb(200, 230, 200),
            text_secondary: Color32::from_rgb(160, 190, 160),
            text_muted: Color32::from_rgb(100, 130, 100),
            text_disabled: Color32::from_rgb(70, 90, 70),
            
            // Accent - classic game console colors
            accent: Color32::from_rgb(80, 180, 120),
            accent_hover: Color32::from_rgb(100, 210, 140),
            accent_muted: Color32::from_rgb(50, 120, 80),
            
            // Borders
            border: Color32::from_rgb(50, 70, 55),
            border_focused: Color32::from_rgb(80, 180, 120),
            
            // Status
            error: Color32::from_rgb(255, 100, 100),
            warning: Color32::from_rgb(255, 200, 80),
            success: Color32::from_rgb(100, 255, 150),
            info: Color32::from_rgb(100, 180, 255),

            icon_primary: Color32::from_rgb(60, 180, 255),
            icon_secondary: Color32::from_rgb(40, 220, 120),
            icon_tertiary: Color32::from_rgb(230, 90, 90),
            icon_hovered: Color32::from_rgb(60, 180, 255),
            icon_held: Color32::from_rgb(0, 60, 180),
            icon_disabled: Color32::from_rgb(120, 120, 120),
            
            // Debugger
            highlight_line: Color32::from_rgba_unmultiplied(80, 180, 120, 35),
            breakpoint: Color32::from_rgb(255, 80, 80),
            breakpoint_bg: Color32::from_rgba_unmultiplied(255, 80, 80, 25),
            watchpoint: Color32::from_rgb(255, 200, 80),
            modified: Color32::from_rgb(255, 220, 100),
            modified_bg: Color32::from_rgba_unmultiplied(255, 220, 100, 25),
            
            // Syntax - retro terminal feel
            syntax_address: Color32::from_rgb(120, 200, 255),
            syntax_opcode: Color32::from_rgb(255, 200, 100),
            syntax_register: Color32::from_rgb(255, 150, 150),
            syntax_number: Color32::from_rgb(150, 255, 150),
            syntax_label: Color32::from_rgb(100, 220, 220),
            syntax_comment: Color32::from_rgb(80, 110, 85),
            syntax_string: Color32::from_rgb(255, 220, 180),
            syntax_keyword: Color32::from_rgb(255, 150, 200),
            syntax_directive: Color32::from_rgb(180, 180, 255),
            
            // Memory viewer
            memory_null: Color32::from_rgb(50, 65, 55),
            memory_ascii: Color32::from_rgb(150, 255, 150),
            memory_high: Color32::from_rgb(255, 180, 200),
            
            // Styling - sharper for retro feel
            corner_radius: 2,
            widget_corner_radius: 2,
        }
    }
    
    /// Apply theme to an egui context
    pub fn apply(&self, ctx: &Context) {
        let mut style = Style::default();
        
        // Visuals
        style.visuals = Visuals {
            dark_mode: self.is_dark(),
            
            override_text_color: None,
            
            widgets: Widgets {
                noninteractive: WidgetVisuals {
                    bg_fill: self.bg_secondary,
                    weak_bg_fill: self.bg_secondary,
                    bg_stroke: Stroke::new(1.0, self.border),
                    fg_stroke: Stroke::new(1.0, self.text_secondary),
                    corner_radius: CornerRadius::same(self.widget_corner_radius),
                    expansion: 0.0,
                },
                inactive: WidgetVisuals {
                    bg_fill: self.bg_tertiary,
                    weak_bg_fill: self.bg_tertiary,
                    bg_stroke: Stroke::new(1.0, self.border),
                    fg_stroke: Stroke::new(1.0, self.text_primary),
                    corner_radius: CornerRadius::same(self.widget_corner_radius),
                    expansion: 0.0,
                },
                hovered: WidgetVisuals {
                    bg_fill: self.bg_elevated,
                    weak_bg_fill: self.bg_elevated,
                    bg_stroke: Stroke::new(1.0, self.accent),
                    fg_stroke: Stroke::new(1.5, self.text_primary),
                    corner_radius: CornerRadius::same(self.widget_corner_radius),
                    expansion: 1.0,
                },
                active: WidgetVisuals {
                    bg_fill: self.accent_muted,
                    weak_bg_fill: self.accent_muted,
                    bg_stroke: Stroke::new(1.0, self.accent),
                    fg_stroke: Stroke::new(2.0, self.text_primary),
                    corner_radius: CornerRadius::same(self.widget_corner_radius),
                    expansion: 1.0,
                },
                open: WidgetVisuals {
                    bg_fill: self.bg_elevated,
                    weak_bg_fill: self.bg_elevated,
                    bg_stroke: Stroke::new(1.0, self.border),
                    fg_stroke: Stroke::new(1.0, self.text_primary),
                    corner_radius: CornerRadius::same(self.widget_corner_radius),
                    expansion: 0.0,
                },
            },
            
            selection: Selection {
                bg_fill: self.accent_muted,
                stroke: Stroke::new(1.0, self.accent),
            },
            
            hyperlink_color: self.info,
            faint_bg_color: self.bg_secondary,
            extreme_bg_color: self.bg_primary,
            code_bg_color: self.bg_tertiary,
            
            warn_fg_color: self.warning,
            error_fg_color: self.error,
            
            window_fill: self.bg_primary,
            window_stroke: Stroke::new(1.0, self.border),
            window_corner_radius: CornerRadius::same(self.corner_radius),
            window_shadow: egui::epaint::Shadow {
                offset: [0, 2],
                blur: 8,
                spread: 0,
                color: Color32::from_black_alpha(if self.is_dark() { 80 } else { 30 }),
            },
            window_highlight_topmost: true,
            
            menu_corner_radius: CornerRadius::same(self.widget_corner_radius),
            
            panel_fill: self.bg_primary,
            
            popup_shadow: egui::epaint::Shadow {
                offset: [0, 4],
                blur: 12,
                spread: 0,
                color: Color32::from_black_alpha(if self.is_dark() { 100 } else { 40 }),
            },
            
            resize_corner_size: 12.0,
            
            text_cursor: TextCursorStyle {
                stroke: Stroke::new(2.0, self.text_primary),
                preview: false,
                blink: true,
                on_duration: 0.5,
                off_duration: 0.5,
            },

            text_alpha_from_coverage: AlphaFromCoverage::Gamma(2.2),
            weak_text_alpha: 0.6,
            weak_text_color: Some(self.text_muted),
            text_edit_bg_color: Some(self.bg_tertiary),
            disabled_alpha: 0.4,
            
            clip_rect_margin: 3.0,
            button_frame: true,
            collapsing_header_frame: false,
            indent_has_left_vline: true,
            
            striped: true,
            
            slider_trailing_fill: true,
            
            handle_shape: egui::style::HandleShape::Circle,
            
            interact_cursor: None,
            
            image_loading_spinners: true,
            
            numeric_color_space: egui::style::NumericColorSpace::GammaByte,
        };
        
        // Spacing
        style.spacing.item_spacing = egui::vec2(8.0, 4.0);
        style.spacing.window_margin = egui::Margin::same(12);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
        style.spacing.indent = 18.0;
        style.spacing.scroll.bar_width = 10.0;
        style.spacing.scroll.bar_inner_margin = 2.0;
        style.spacing.scroll.bar_outer_margin = 2.0;
        
        ctx.set_style(style);
    }
    
    /// Determine if this is a dark theme based on background luminance
    pub fn is_dark(&self) -> bool {
        let r = self.bg_primary.r() as f32;
        let g = self.bg_primary.g() as f32;
        let b = self.bg_primary.b() as f32;
        let luminance = 0.299 * r + 0.587 * g + 0.114 * b;
        luminance < 128.0
    }
}

/// Syntax highlighting categories
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntaxKind {
    Address,
    Opcode,
    Register,
    Number,
    Label,
    Comment,
    String,
    Keyword,
    Directive,
}

// /// SNES-specific status flag display
// #[derive(Clone, Copy, Debug)]
// pub struct StatusFlags {
//     pub n: bool,  // Negative
//     pub v: bool,  // Overflow
//     pub m: bool,  // Accumulator size (65816)
//     pub x: bool,  // Index size (65816)
//     pub d: bool,  // Decimal
//     pub i: bool,  // IRQ disable
//     pub z: bool,  // Zero
//     pub c: bool,  // Carry
//     pub e: bool,  // Emulation mode (65816)
// }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemePreset {
    #[default]
    Dark,
    Light,
    Retro,
    Custom,
}

impl ThemePreset {
    pub fn name(&self) -> &'static str {
        match self {
            ThemePreset::Dark => "Dark",
            ThemePreset::Light => "Light",
            ThemePreset::Retro => "Retro",
            ThemePreset::Custom => "Custom",
        }
    }
    
    pub fn all() -> &'static [ThemePreset] {
        &[
            ThemePreset::Dark,
            ThemePreset::Light,
            ThemePreset::Retro,
        ]
    }
    
    pub fn to_theme(self) -> AppTheme {
        match self {
            ThemePreset::Dark | ThemePreset::Custom => AppTheme::dark(),
            ThemePreset::Light => AppTheme::light(),
            ThemePreset::Retro => AppTheme::retro(),
        }
    }
}

// Helper widgets using the theme
#[cfg(feature = "debug")]
impl AppTheme {
    /// Draw a breakpoint gutter marker
    pub fn draw_breakpoint_marker(
        &self,
        ui: &mut egui::Ui,
        has_breakpoint: bool,
        is_current_line: bool,
    ) -> egui::Response {
        let size = egui::vec2(16.0, 16.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            
            if has_breakpoint {
                // Filled red circle for breakpoint
                painter.circle_filled(center, 6.0, self.breakpoint);
            } else if response.hovered() {
                // Hollow circle on hover
                painter.circle_stroke(
                    center,
                    5.0,
                    Stroke::new(1.0, self.text_muted),
                );
            }
            
            if is_current_line {
                // Yellow arrow for current execution point
                let arrow_points = vec![
                    center + egui::vec2(-4.0, -4.0),
                    center + egui::vec2(4.0, 0.0),
                    center + egui::vec2(-4.0, 4.0),
                ];
                painter.add(egui::Shape::convex_polygon(
                    arrow_points,
                    self.warning,
                    Stroke::NONE,
                ));
            }
        }
        
        response
    }
    
    /// Styled separator for debugger panels
    pub fn debugger_separator(&self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(available_width, 1.0),
            egui::Sense::hover(),
        );
        
        if ui.is_rect_visible(rect) {
            ui.painter().hline(
                rect.x_range(),
                rect.center().y,
                Stroke::new(1.0, self.border),
            );
        }
    }
    
    /// Section header for debugger panels
    pub fn section_header(&self, ui: &mut egui::Ui, text: &str) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(text)
                    .color(self.text_primary)
                    .strong()
                    .size(14.0),
            );
        });
        self.debugger_separator(ui);
        ui.add_space(4.0);
    }

    /// Get color for a byte value in memory viewer
    pub fn memory_byte_color(&self, byte: u8) -> Color32 {
        match byte {
            0x00 => self.memory_null,
            0x20..=0x7E => self.memory_ascii,
            0x80..=0xFF => self.memory_high,
            _ => self.text_secondary,
        }
    }

    pub fn memory_word_color(&self, word: u16) -> Color32 {
        match word {
            0x00 => self.memory_null,
            0x2000..=0x7E00 => self.memory_ascii,
            0x8000..=0xFFFF => self.memory_high,
            _ => self.text_secondary,
        }
    }

        /// Create a text format for syntax highlighting
    pub fn syntax_format(&self, kind: SyntaxKind) -> egui::text::TextFormat {
        let color = match kind {
            SyntaxKind::Address => self.syntax_address,
            SyntaxKind::Opcode => self.syntax_opcode,
            SyntaxKind::Register => self.syntax_register,
            SyntaxKind::Number => self.syntax_number,
            SyntaxKind::Label => self.syntax_label,
            SyntaxKind::Comment => self.syntax_comment,
            SyntaxKind::String => self.syntax_string,
            SyntaxKind::Keyword => self.syntax_keyword,
            SyntaxKind::Directive => self.syntax_directive,
        };
        
        egui::text::TextFormat {
            font_id: FontId::monospace(13.0),
            color,
            ..Default::default()
        }
    }
    
    /// Get appropriate icon tint based on state
    pub fn icon_color(&self, enabled: bool, hovered: bool, active: bool) -> Color32 {
        if !enabled {
            self.text_disabled
        } else if active {
            self.accent
        } else if hovered {
            self.text_primary
        } else {
            self.text_secondary
        }
    }
    
    /// Format a register display with change highlighting
    pub fn format_register(
        &self,
        name: &str,
        value: u16,
        bits: u8,
        changed: bool,
    ) -> egui::text::LayoutJob {
        use egui::text::{LayoutJob, TextFormat};
        
        let mut job = LayoutJob::default();
        let mono = FontId::monospace(13.0);
        
        // Register name
        job.append(
            &format!("{}: ", name),
            0.0,
            TextFormat {
                font_id: mono.clone(),
                color: self.syntax_register,
                ..Default::default()
            },
        );
        
        // Value
        let value_str = match bits {
            8 => format!("{:02X}", value as u8),
            16 => format!("{:04X}", value),
            24 => format!("{:06X}", value as u32),
            _ => format!("{:X}", value),
        };
        
        job.append(
            &value_str,
            0.0,
            TextFormat {
                font_id: mono.clone(),
                color: if changed { self.modified } else { self.syntax_number },
                background: if changed {
                    Color32::from_rgba_unmultiplied(
                        self.modified.r(),
                        self.modified.g(),
                        self.modified.b(),
                        30,
                    )
                } else {
                    Color32::TRANSPARENT
                },
                ..Default::default()
            },
        );
        
        job
    }

    /// Format CPU status flags with color coding
    pub fn format_status_flags(&self, core: &Snemulator) -> egui::text::LayoutJob {
        use egui::text::{LayoutJob, TextFormat};
        
        let mut job = LayoutJob::default();
        let mono = FontId::monospace(13.0);
        
        let flag_items = [
            ('N', core.cpu.is_flag_set(snemcore::scpu::Flag::FlagN)),
            ('V', core.cpu.is_flag_set(snemcore::scpu::Flag::FlagV)),
            ('M', core.cpu.is_flag_set(snemcore::scpu::Flag::FlagM)),
            ('X', core.cpu.is_flag_set(snemcore::scpu::Flag::FlagX)),
            ('D', core.cpu.is_flag_set(snemcore::scpu::Flag::FlagD)),
            ('I', core.cpu.is_flag_set(snemcore::scpu::Flag::FlagI)),
            ('Z', core.cpu.is_flag_set(snemcore::scpu::Flag::FlagZ)),
            ('C', core.cpu.is_flag_set(snemcore::scpu::Flag::FlagC)),
        ];
        
        for (name, set) in flag_items {
            job.append(
                &format!("{}", name),
                0.0,
                TextFormat {
                    font_id: mono.clone(),
                    color: if set { self.success } else { self.text_disabled },
                    ..Default::default()
                },
            );
        }
        
        // Emulation mode indicator
        job.append(
            " ",
            0.0,
            TextFormat::default(),
        );
        job.append(
            if core.cpu.e { "EMU" } else { "NAT" },
            0.0,
            TextFormat {
                font_id: mono.clone(),
                color: if core.cpu.e { self.warning } else { self.info },
                ..Default::default()
            },
        );
        
        job
    }
    
    /// Format memory hex dump line
    pub fn format_memory_line(
        &self,
        address: u32,
        bytes: &[u8; 16],
        modified_mask: u16, // Bitmask of which bytes were recently modified
    ) -> egui::text::LayoutJob {
        use egui::text::{LayoutJob, TextFormat};
        
        let mut job = LayoutJob::default();
        let mono = FontId::monospace(13.0);
        
        // Address
        job.append(
            &format!("{:06X}  ", address),
            0.0,
            TextFormat {
                font_id: mono.clone(),
                color: self.syntax_address,
                ..Default::default()
            },
        );
        
        // Hex bytes
        for (i, &byte) in bytes.iter().enumerate() {
            let modified = (modified_mask >> i) & 1 != 0;
            let color = if modified {
                self.modified
            } else {
                self.memory_byte_color(byte)
            };
            
            job.append(
                &format!("{:02X}", byte),
                0.0,
                TextFormat {
                    font_id: mono.clone(),
                    color,
                    background: if modified {
                        Color32::from_rgba_unmultiplied(
                            self.modified.r(),
                            self.modified.g(),
                            self.modified.b(),
                            25,
                        )
                    } else {
                        Color32::TRANSPARENT
                    },
                    ..Default::default()
                },
            );
            
            // Spacing between bytes, extra space at midpoint
            let spacing = if i == 7 { "  " } else { " " };
            job.append(
                spacing,
                0.0,
                TextFormat {
                    font_id: mono.clone(),
                    color: self.text_muted,
                    ..Default::default()
                },
            );
        }
        
        // ASCII representation
        job.append(
            " |",
            0.0,
            TextFormat {
                font_id: mono.clone(),
                color: self.text_muted,
                ..Default::default()
            },
        );
        
        for &byte in bytes {
            let ch = if (0x20..=0x7E).contains(&byte) {
                byte as char
            } else {
                '.'
            };
            
            job.append(
                &ch.to_string(),
                0.0,
                TextFormat {
                    font_id: mono.clone(),
                    color: self.memory_byte_color(byte),
                    ..Default::default()
                },
            );
        }
        
        job.append(
            "|",
            0.0,
            TextFormat {
                font_id: mono.clone(),
                color: self.text_muted,
                ..Default::default()
            },
        );
        
        job
    }
}