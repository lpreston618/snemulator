use egui::{Color32, Image, ImageSource, Vec2};

use crate::theme::AppTheme;

pub const CONTINUE: egui::ImageSource<'static> = egui::include_image!("../../assets/debug/continue.svg");
pub const SINGLE_STEP: egui::ImageSource<'static> = egui::include_image!("../../assets/debug/single_step.svg");
pub const PAUSE: egui::ImageSource<'static> = egui::include_image!("../../assets/debug/pause.svg");
pub const RUN_FRAME: egui::ImageSource<'static> = egui::include_image!("../../assets/debug/run_frame.svg");
pub const RUN_UNTIL_INTERRUPT: egui::ImageSource<'static> = egui::include_image!("../../assets/debug/run_until_interrupt.svg");
pub const STEP_INTO: egui::ImageSource<'static> = egui::include_image!("../../assets/debug/step_into.svg");
pub const STEP_OUT: egui::ImageSource<'static> = egui::include_image!("../../assets/debug/step_out.svg");
pub const STEP_OVER: egui::ImageSource<'static> = egui::include_image!("../../assets/debug/step_over.svg");

pub struct ThemedIcon<'a> {
    source: ImageSource<'a>,
    size: Vec2,
    enabled: bool,
}

impl<'a> ThemedIcon<'a> {
    pub fn new(source: ImageSource<'a>) -> Self {
        Self {
            source,
            size: egui::vec2(18.0, 18.0),
            enabled: true,
        }
    }
    
    pub fn size(mut self, size: impl Into<Vec2>) -> Self {
        self.size = size.into();
        self
    }
    
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// Show as a clickable button
    pub fn themed_button(self, ui: &mut egui::Ui, theme: &AppTheme) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(
            self.size + ui.spacing().button_padding * 2.0,
            egui::Sense::click(),
        );
        
        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            
            // Draw button background
            ui.painter().rect_filled(
                rect,
                visuals.corner_radius,
                if response.hovered() && self.enabled {
                    visuals.bg_fill
                } else {
                    egui::Color32::TRANSPARENT
                },
            );
            
            // Determine icon color based on state
            let tint = if !self.enabled {
                theme.icon_disabled
            } else if response.is_pointer_button_down_on() {
                theme.icon_held
            } else if response.hovered() {
                theme.icon_hovered
            } else {
                theme.icon_primary
            };
            
            // Draw icon
            let icon_rect = rect.shrink2(ui.spacing().button_padding);
            Image::new(self.source.clone())
                .fit_to_exact_size(icon_rect.size())
                .tint(tint)
                .paint_at(ui, icon_rect);
        }
        
        response
    }

    pub fn button(self, ui: &mut egui::Ui, tint: Color32) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(
            self.size + ui.spacing().button_padding * 2.0,
            egui::Sense::click(),
        );
        
        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            
            // Draw button background
            ui.painter().rect_filled(
                rect,
                visuals.corner_radius,
                if response.hovered() && self.enabled {
                    visuals.bg_fill
                } else {
                    egui::Color32::TRANSPARENT
                },
            );
            
            // Draw icon
            let icon_rect = rect.shrink2(ui.spacing().button_padding);
            Image::new(self.source.clone())
                .fit_to_exact_size(icon_rect.size())
                .tint(tint)
                .paint_at(ui, icon_rect);
        }
        
        response
    }
    
    /// Show as static icon (no interaction)
    pub fn show(self, ui: &mut egui::Ui, theme: &AppTheme) {
        let (rect, _response) = ui.allocate_exact_size(self.size, egui::Sense::hover());
        
        if ui.is_rect_visible(rect) {
            let tint = if self.enabled {
                theme.text_secondary
            } else {
                theme.text_disabled
            };
            
            Image::new(self.source)
                .fit_to_exact_size(rect.size())
                .tint(tint)
                .paint_at(ui, rect);
        }
    }
    
    /// Show with explicit color override
    pub fn show_colored(self, ui: &mut egui::Ui, color: Color32) {
        let (rect, _response) = ui.allocate_exact_size(self.size, egui::Sense::hover());
        
        if ui.is_rect_visible(rect) {
            Image::new(self.source)
                .fit_to_exact_size(rect.size())
                .tint(color)
                .paint_at(ui, rect);
        }
    }
}

// ============================================================================
// Convenience extension trait
// ============================================================================

pub trait IconExt {
    fn icon_button(
        &mut self,
        icon: ImageSource<'_>,
        theme: &AppTheme,
    ) -> egui::Response;
    
    fn icon_button_with_tint(
        &mut self,
        icon: ImageSource<'_>,
        tint: Color32,
    ) -> egui::Response;

    fn icon_button_with_tooltip(
        &mut self,
        icon: ImageSource<'_>,
        tooltip: &str,
        theme: &AppTheme,
    ) -> egui::Response;
    
    fn icon_toggle(
        &mut self,
        icon: ImageSource<'_>,
        active: bool,
        theme: &AppTheme,
    ) -> egui::Response;
}

impl IconExt for egui::Ui {
    fn icon_button(
        &mut self,
        icon: ImageSource<'_>,
        app_theme: &AppTheme,
    ) -> egui::Response {
        ThemedIcon::new(icon).themed_button(self, app_theme)
    }

    fn icon_button_with_tint(
        &mut self,
        icon: ImageSource<'_>,
        tint: Color32,
    ) -> egui::Response {
        ThemedIcon::new(icon).button(self, tint)
    }
    
    fn icon_button_with_tooltip(
        &mut self,
        icon: ImageSource<'_>,
        tooltip: &str,
        app_theme: &AppTheme,
    ) -> egui::Response {
        ThemedIcon::new(icon)
            .themed_button(self, app_theme)
            .on_hover_text(tooltip)
    }
    
    fn icon_toggle(
        &mut self,
        icon: ImageSource<'_>,
        active: bool,
        app_theme: &AppTheme,
    ) -> egui::Response {
        let (rect, response) = self.allocate_exact_size(
            egui::vec2(18.0, 18.0) + self.spacing().button_padding * 2.0,
            egui::Sense::click(),
        );
        
        if self.is_rect_visible(rect) {
            let tint = if active {
                app_theme.accent
            } else if response.hovered() {
                app_theme.text_primary
            } else {
                app_theme.text_secondary
            };
            
            // Background for active state
            if active {
                self.painter().rect_filled(
                    rect,
                    self.style().visuals.widgets.active.corner_radius,
                    app_theme.accent_muted,
                );
            }
            
            let icon_rect = rect.shrink2(self.spacing().button_padding);
            Image::new(icon)
                .fit_to_exact_size(icon_rect.size())
                .tint(tint)
                .paint_at(self, icon_rect);
        }
        
        response
    }
}