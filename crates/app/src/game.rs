use anyhow::Result;
use egui::TextureHandle;
use sdl3::video::GLProfile;

use crate::app::{self, thumbnail_fetcher};
use crate::app::theme::AppTheme;
use snemcore::sysinfo;
use crate::menu::MainMenuBar;
use crate::app::settings::Settings;
use crate::ui_window::UiWindow;
use crate::app::library::LibraryView;

pub const WINDOW_WIDTH: u32 = 1200;
pub const WINDOW_HEIGHT: u32 = 900;

pub struct MainWindow {
    egui_window: UiWindow,
    menu: MainMenuBar,
    game_texture: Option<TextureHandle>,
    pub library: LibraryView,
}

impl MainWindow {
    pub fn new(
        egui_window: UiWindow,
        video_subsystem: &sdl3::VideoSubsystem,
        settings: &Settings,
    ) -> Result<Self> {
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        gl_attr.set_context_flags().forward_compatible().set();
        gl_attr.set_double_buffer(true);

        video_subsystem.gl_set_swap_interval(
            if settings.vsync_en {
                sdl3::video::SwapInterval::VSync
            } else {
                sdl3::video::SwapInterval::Immediate
            }
        )?;

        Ok(Self {
            egui_window,
            menu: MainMenuBar::new(),
            game_texture: None,
            library: LibraryView::new(),
        })
    }

    pub fn rescan_library(&mut self, settings: &Settings) {
        self.library.scan(settings);
        // thumbnail_fetcher::resolve_thumbnails_for_library(&mut self.library.entries);
    }

    pub fn set_theme(&mut self, app_theme: &AppTheme) {
        app_theme.apply(&self.egui_window.egui_ctx);
    }

    fn update_game_texture(&mut self, frame_buffer: &[u8]) {
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [sysinfo::SCREEN_WIDTH as usize, sysinfo::SCREEN_HEIGHT as usize],
            frame_buffer,
        );

        match &mut self.game_texture {
            Some(handle) => handle.set(image, egui::TextureOptions::NEAREST),
            None => {
                self.game_texture = Some(self.egui_window.egui_ctx.load_texture(
                    "game_screen",
                    image,
                    egui::TextureOptions::NEAREST,
                ));
            }
        }
    }

    pub fn update_and_render(&mut self, app_state: &app::AppState, app_theme: &AppTheme, app_settings: &mut Settings, frame_buffer: &[u8]) -> app::AppAction {
        let mut app_action = app::AppAction::Continue;

        // Upload texture inside the ui closure so we have access to ctx
        self.update_game_texture(frame_buffer);

        let full_output = self.egui_window.update_ui(|ctx| {
            if app_state.show_menu {
                app_action = self.menu.render(ctx, app_state, app_settings);
            }

            let game_aspect = sysinfo::SCREEN_WIDTH as f32 / sysinfo::SCREEN_HEIGHT as f32;

            egui::CentralPanel::default()
                .frame(egui::Frame::new())
                .show(ctx, |ui| {
                    if app_state.loaded_rom_data.is_some() {
                        if let Some(texture) = &self.game_texture {
                            let available = ui.available_size();
                            let available_aspect = available.x / available.y;
                            let render_size = if available_aspect > game_aspect {
                                egui::vec2(available.y * game_aspect, available.y)
                            } else {
                                egui::vec2(available.x, available.x / game_aspect)
                            };
                            let offset = (available - render_size) / 2.0;
                            ui.add_space(offset.y);
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.image(egui::load::SizedTexture::new(texture.id(), render_size));
                            });
                        }
                    } else {
                        // Library mode
                        match &app_settings.roms_library_dir {
                            Some(_) => {
                                app_action = self.library.render(ui, app_theme);
                            }
                            None => {
                                Self::render_empty_state(ui, &mut app_action, app_theme);
                            }
                        }
                    }
                });

            if app_settings.show_fps {
                egui::Area::new(egui::Id::new("fps_counter_area"))
                    .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-20.0, 10.0))
                    .interactable(false)
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!("FPS: {}", app_state.display_fps))
                                    .color(egui::Color32::WHITE)
                                    .background_color(egui::Color32::from_black_alpha(150))
                                    .strong()
                            )
                            .wrap_mode(egui::TextWrapMode::Extend)
                        );
                    });
            }
        });

        self.egui_window.clear();
        self.egui_window.render(full_output);

        app_action
    }

    pub fn id(&self) -> u32 {
        self.egui_window.window().id()
    }

    pub fn handle_event(&mut self, event: &sdl3::event::Event, modifiers: &egui::Modifiers, app_state: &mut app::AppState) {
        if self.egui_window.handle_sdl_mouse_event(event, modifiers) {
            app_state.last_mouse_input_frame = app_state.frame_count;
        }
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()> {
        self.egui_window.window_mut().set_fullscreen(fullscreen).map_err(|e| e.into())
    }

    fn render_empty_state(ui: &mut egui::Ui, app_action: &mut app::AppAction, theme: &AppTheme) {
        let available = ui.available_size();

        // Allocate the entire panel as a single click target
        let (rect, response) = ui.allocate_exact_size(available, egui::Sense::click());

        let painter = ui.painter();

        painter.rect_filled(rect, 0.0, theme.bg_secondary);

        // Central box
        let box_size = egui::Vec2::splat(120.0);
        let box_rect = egui::Rect::from_center_size(
            egui::pos2(rect.center().x, rect.center().y - 30.0),
            box_size,
        );

        let border_color = theme.accent;

        // Dashed border
        let dash_len = 8.0;
        let gap_len  = 5.0;
        let cr = theme.corner_radius as f32;
        Self::paint_dashed_rect(painter, box_rect, cr, egui::Stroke::new(1.5, border_color), dash_len, gap_len);

        // + icon
        let arm = 18.0;
        let cx = box_rect.center();
        let stroke = egui::Stroke::new(2.0, border_color);
        painter.line_segment([egui::pos2(cx.x - arm, cx.y), egui::pos2(cx.x + arm, cx.y)], stroke);
        painter.line_segment([egui::pos2(cx.x, cx.y - arm), egui::pos2(cx.x, cx.y + arm)], stroke);

        // Label below the box
        let label_color = theme.text_secondary;
        painter.text(
            egui::pos2(rect.center().x, box_rect.max.y + 16.0),
            egui::Align2::CENTER_TOP,
            "Add ROMs Folder",
            egui::FontId::proportional(13.0),
            label_color,
        );

        if response.clicked() {
            *app_action = app::AppAction::SelectRomsFolder;
        }
    }

    fn paint_dashed_rect(painter: &egui::Painter, rect: egui::Rect, rounding: f32, stroke: egui::Stroke, dash: f32, gap: f32) {
        // Approximate rounded rect as 4 straight edges — close enough at this size
        let corners = [
            (egui::pos2(rect.min.x + rounding, rect.min.y), egui::pos2(rect.max.x - rounding, rect.min.y)), // top
            (egui::pos2(rect.max.x, rect.min.y + rounding), egui::pos2(rect.max.x, rect.max.y - rounding)), // right
            (egui::pos2(rect.max.x - rounding, rect.max.y), egui::pos2(rect.min.x + rounding, rect.max.y)), // bottom
            (egui::pos2(rect.min.x, rect.max.y - rounding), egui::pos2(rect.min.x, rect.min.y + rounding)), // left
        ];

        for (start, end) in corners {
            Self::paint_dashed_line(painter, start, end, stroke, dash, gap);
        }
    }

    fn paint_dashed_line(painter: &egui::Painter, start: egui::Pos2, end: egui::Pos2, stroke: egui::Stroke, dash: f32, gap: f32) {
        let delta = end - start;
        let len = delta.length();
        let dir = delta / len;
        let mut t = 0.0;
        while t < len {
            let dash_end = (t + dash).min(len);
            painter.line_segment(
                [start + dir * t, start + dir * dash_end],
                stroke,
            );
            t += dash + gap;
        }
    }
}