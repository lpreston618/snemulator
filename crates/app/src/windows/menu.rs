use crate::app::{AppState, AppAction};

fn button_with_shortcut(ui: &mut egui::Ui, label: &str, shortcut: &str) -> egui::Response {
    ui.add(egui::Button::new(label).right_text(egui::RichText::new(shortcut).weak()))
}

pub struct MainMenuBar;

impl MainMenuBar {
    pub fn new() -> Self { Self {} }

    pub fn render(&self, ctx: &egui::Context, app_state: &AppState) -> AppAction {
        let mut app_action = AppAction::Continue;
    
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {                
                ui.menu_button("File", |ui| {
                    ui.set_width(100.0);
                    
                    ui.add_enabled_ui(!app_state.debug_active, |ui| {
                        if ui.button("Load Rom").clicked() {
                            app_action = AppAction::LoadRom;
                            ui.close();
                        }
                        if ui.button("Recent ROMs").clicked() {
                            log::warn!("Recent ROMs clicked.");
                            ui.close();
                        }
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
                    
                    ui.add_enabled_ui(!app_state.debug_active, |ui| {
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
                });
                
                ui.menu_button("View", |ui| {
                    ui.set_width(100.0);
                    
                    let window_size_text = if app_state.is_fullscreen { "Windowed" } else { "Fullscreen" };
                    if button_with_shortcut(ui, window_size_text, "F11").clicked() {
                        app_action = AppAction::ToggleFullscreen;
                        ui.close();
                    }
                });
                
                ui.menu_button("About", |ui| {
                    ui.set_width(100.0);
                    
                    if ui.button("About").clicked() {
                        app_action = AppAction::OpenAbout;
                        ui.close();
                    }
                })
            });
        });
    
        app_action
    }
}