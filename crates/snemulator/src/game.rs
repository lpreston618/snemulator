use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use egui::TextureHandle;
use sdl3::video::GLProfile;

use crate::app::messages::{Message, MessageKind, MessageQueue};
use crate::app::{self, AppAction};
use crate::app::theme::AppTheme;
use snemcore::sysinfo;
use crate::menu::MainMenuBar;
use crate::app::settings::Settings;
use crate::ui_window::UiWindow;
use crate::app::library::LibraryView;

pub const WINDOW_WIDTH: u32 = 900;
pub const WINDOW_HEIGHT: u32 = 675;

const MAX_DISPLAYED_MESSAGES: usize = 5;
const FADE: Duration = Duration::from_millis(250);
const SLIDE_DISTANCE: f32 = 30.0; // px the box travels during fade
const STACK_SPACING: f32 = 6.0;

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
    ) -> Result<Self> {
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        gl_attr.set_context_flags().forward_compatible().set();
        gl_attr.set_double_buffer(true);

        Ok(Self {
            egui_window,
            menu: MainMenuBar::new(),
            game_texture: None,
            library: LibraryView::new(),
        })
    }

    pub fn rescan_library(&mut self, roms_library_dir: &Option<PathBuf>) {
        self.library.scan(roms_library_dir);
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

    pub fn update_and_render(
        &mut self, 
        app_state: &app::AppState,
        app_theme: &AppTheme, 
        app_settings: &mut Settings,
        message_queue: &mut MessageQueue,
        frame_buffer: &[u8]
    ) -> Option<AppAction> {
        let mut menu_action: Option<AppAction> = None;
        let mut library_action: Option<AppAction> = None;

        // Upload texture inside the ui closure so we have access to ctx
        self.update_game_texture(frame_buffer);

        let full_output = self.egui_window.update_ui(|ctx| {
            if app_state.show_menu {
                menu_action = self.menu.render(ctx, app_state, app_settings);
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
                                library_action = self.library.render(ui, app_theme);
                            }
                            None => {
                                Self::render_empty_state(ui, &mut library_action, app_theme);
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

            message_queue.messages.retain(|m| m.alpha(FADE).is_some());

            if !message_queue.messages.is_empty() {
                Self::render_messages(&message_queue.messages, ctx, app_theme);
            }
        });

        // Use library action if there is one, else menu action. Should only ever
        // be one at a time anyways.
        let app_action = library_action.map_or(menu_action, |action| Some(action));

        self.egui_window.clear();
        self.egui_window.render(full_output);

        app_action
    }

    fn render_messages(messages: &Vec<Message>, ctx: &egui::Context, app_theme: &AppTheme) {
        let screen = ctx.viewport_rect();

        let anchor_x = screen.max.x - 20.0; // desired RIGHT edge of boxes
        let anchor_y = screen.max.y - 10.0; // desired BOTTOM edge of lowest box

        let mut cursor_y = anchor_y;

        let take = messages.len().min(MAX_DISPLAYED_MESSAGES);
        let start = messages.len() - take;

        for idx in start..messages.len() {
            let message = &messages[idx];
            let (alpha, slide) = match message.transition(FADE) {
                Some(t) => t,
                None => continue,
            };

            let msg_id = egui::Id::new(("msg", message.id));

            let height = ctx
                .data(|d| d.get_temp::<f32>(msg_id.with("h")))
                .unwrap_or(28.0);
            let width = ctx
                .data(|d| d.get_temp::<f32>(msg_id.with("w")))
                .unwrap_or(320.0);

            let target_bottom = cursor_y;
            cursor_y -= height + STACK_SPACING;

            let animated_bottom = ctx.animate_value_with_time(
                msg_id.with("y"),
                target_bottom,
                0.18,
            );

            let slide_x = (1.0 - slide) * SLIDE_DISTANCE;

            // fixed_pos = top-left corner, so convert from right/bottom edges.
            let box_top = animated_bottom - height;
            let pos = egui::pos2(anchor_x - width + slide_x, box_top);

            let fade_color = move |c: egui::Color32| {
                egui::Color32::from_rgba_unmultiplied(
                    c.r(), c.g(), c.b(),
                    ((c.a() as f32) * alpha) as u8,
                )
            };

            let (accent, bg) = message.kind.colors(app_theme);

            let response = egui::Area::new(msg_id)
                .fixed_pos(pos)          // <-- keep this
                // .anchor(...)          // <-- REMOVED: this was overriding fixed_pos
                .interactable(false)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    ui.set_max_width(320.0);
                    egui::Frame::NONE
                        .fill(fade_color(bg))
                        .stroke(egui::Stroke::new(1.5, fade_color(accent)))
                        .corner_radius(app_theme.widget_corner_radius)
                        .inner_margin(egui::Margin::symmetric(10, 6))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(message.display_text())
                                            .color(fade_color(egui::Color32::from_rgb(238, 238, 238)))
                                            .strong(),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Wrap),
                                );
                            });
                        });
                })
                .response;

            let measured = response.rect;
            ctx.data_mut(|d| {
                d.insert_temp(msg_id.with("h"), measured.height());
                d.insert_temp(msg_id.with("w"), measured.width());
            });
        }

        // Ensure animations keep ticking while messages exist.
        if !messages.is_empty() {
            ctx.request_repaint();
        }
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

    fn render_empty_state(ui: &mut egui::Ui, app_action: &mut Option<AppAction>, theme: &AppTheme) {
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
            *app_action = Some(AppAction::SelectRomsFolder);
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