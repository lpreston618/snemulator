use anyhow::Result;
use egui::TextureHandle;
use sdl3::video::GLProfile;

use crate::app;
use crate::app::theme::AppTheme;
use snemcore::sysinfo;
use crate::menu::MainMenuBar;
use crate::app::settings::Settings;
use crate::ui_window::UiWindow;

pub struct MainWindow {
    egui_window: UiWindow,
    menu: MainMenuBar,
    game_texture: Option<TextureHandle>,
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
        })
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

    pub fn update_and_render(&mut self, app_state: &app::AppState, app_settings: &mut Settings, frame_buffer: &[u8]) -> app::AppAction {
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
                    if let Some(texture) = &self.game_texture {
                        let available = ui.available_size();
                        let available_aspect = available.x / available.y;

                        let render_size = if available_aspect > game_aspect {
                            egui::vec2(available.y * game_aspect, available.y)
                        } else {
                            egui::vec2(available.x, available.x / game_aspect)
                        };

                        // Center the image in the available space
                        let offset = (available - render_size) / 2.0;
                        ui.add_space(offset.y);
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.image(egui::load::SizedTexture::new(texture.id(), render_size));
                        });
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
}