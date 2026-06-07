use crate::{app::{AppAction, AppState}, windows::settings::Settings};

fn button_with_shortcut(ui: &mut egui::Ui, label: &str, shortcut: &str) -> egui::Response {
    ui.add(egui::Button::new(label).right_text(egui::RichText::new(shortcut).weak()))
}

pub struct MainMenuBar;

impl MainMenuBar {
    pub fn new() -> Self { Self {} }

    pub fn render(&self, ctx: &egui::Context, app_state: &AppState, app_settings: &mut Settings) -> AppAction {
        let mut app_action = AppAction::Continue;
    
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {                
                ui.menu_button("File", |ui| {
                    ui.set_width(120.0);
                    
                    let ui_en;
                    
                    #[cfg(feature = "debug")]
                    {
                        ui_en = !app_state.debug_active;
                    }
                    #[cfg(not(feature = "debug"))]
                    {
                        ui_en = true;
                    }
                    
                    ui.add_enabled_ui(ui_en, |ui| {
                        if ui.button("Load Rom").clicked() {
                            app_action = AppAction::LoadRom;
                            ui.close();
                        }
                        ui.menu_button("Recent ROMs", |ui| {
                            for path in &app_settings.recent_roms {
                                let label = path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown");
                                if ui.button(label).clicked() {
                                    app_action = AppAction::LoadRomFromPath(path.clone());
                                    ui.close();
                                }
                            }
                            if app_settings.recent_roms.is_empty() {
                                ui.label("No recent ROMs");
                            }
                        });
                    });
                    
                    ui.separator();
                    
                    if ui.button("Settings").clicked() {
                        app_action = AppAction::OpenSettings;
                        ui.close();
                    }
                    
                    ui.separator();
                    
                    if button_with_shortcut(ui, "Exit", "Ctrl + Q").clicked() {
                        log::info!("Exit button clicked, exiting");
                        
                        app_action = AppAction::Exit;
                        ui.close();
                    }
                });
                
                ui.menu_button("Emulation", |ui| {
                    ui.set_width(100.0);
                    
                    let ui_en;
                    
                    #[cfg(feature = "debug")]
                    {
                        ui_en = !app_state.debug_active;
                    }
                    #[cfg(not(feature = "debug"))]
                    {
                        ui_en = true;
                    }
                    
                    ui.add_enabled_ui(ui_en, |ui| {
                        let pause_text = if app_state.is_paused { "Resume" } else { "Pause" };
                        if ui.button(pause_text).clicked() {
                            app_action = AppAction::TogglePause;
                            ui.close();
                        }
                        if ui.button("Reset").clicked() {
                            app_action = AppAction::ResetCore;
                            ui.close();
                        }
                        
                        ui.separator();
                        
                        if ui.button("Save State").clicked() {
                            app_action = AppAction::SaveState;
                            ui.close();
                        }
                        if ui.button("Load State").clicked() {
                            app_action = AppAction::LoadState;
                            ui.close();
                        }
                    });
                    
                    #[cfg(feature = "debug")]
                    {
                        ui.separator();
                        
                        if app_state.debug_active {
                            if ui.button("Stop Debug").clicked() {
                                app_action = AppAction::CloseDebug;
                                ui.close();
                            }
                        } else {
                            if ui.button("Debug").clicked() {
                                app_action = AppAction::OpenDebug;
                                ui.close();
                            }
                        }
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.set_width(100.0);
                    
                    let window_size_text = if app_state.is_fullscreen { "Windowed" } else { "Fullscreen" };
                    if button_with_shortcut(ui, window_size_text, "F11").clicked() {
                        app_action = AppAction::ToggleFullscreen;
                        ui.close();
                    }

                    let show_fps_text = if app_settings.show_fps { "✔" } else { " " };
                    if button_with_shortcut(ui, "Show FPS", show_fps_text).clicked() {
                        app_settings.show_fps = !app_settings.show_fps;
                        ui.close();
                    }
                });
            });
        });
    
        app_action
    }
}