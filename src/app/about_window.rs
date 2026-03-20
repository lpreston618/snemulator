use anyhow::Result;
use log::info;

use crate::app::ui_window::UiWindow;

const ABOUT_WINDOW_WIDTH: u32 = 400;
const ABOUT_WINDOW_HEIGHT: u32 = 400;

pub struct AboutWindow {
    egui_window: UiWindow,
}

impl AboutWindow {
    pub fn new(video_subsystem: &sdl3::VideoSubsystem) -> Result<Self> {
        Ok(Self {
            egui_window: UiWindow::new(
                video_subsystem,
                "About",
                ABOUT_WINDOW_WIDTH,
                ABOUT_WINDOW_HEIGHT,
            )?,
        })
    }
        
    pub fn update_and_render(&mut self) {
        let full_output = self.egui_window.update_ui(|ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        // ui.add_space(20.0);
                        ui.heading("Snemulator");
                        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                        // ui.add_space(20.0);
                        ui.separator();
                        // ui.add_space(10.0);
                        ui.label("A Super Nintendo emulator");
                        ui.label("written in Rust");
                        if ui.button("TEST").clicked() {
                            info!("TEST button clicked");
                        }
                        // ui.add_space(20.0);
                    });
                });
            },
        );
        
        self.egui_window.clear();
        self.egui_window.render(full_output);
    }
    
    pub fn id(&self) -> u32 {
        self.egui_window.window().id()
    }
    
    pub fn handle_event(&mut self, event: &sdl3::event::Event, modifiers: &egui::Modifiers) {
        self.egui_window.handle_sdl_mouse_event(event, modifiers);
    }
}