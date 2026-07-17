use crate::app::{AppAction, AppState, MAX_SAVE_STATE_SLOTS, settings::Settings};

fn button_with_shortcut(ui: &mut egui::Ui, label: &str, shortcut: &str) -> egui::Response {
    ui.add(egui::Button::new(label).right_text(egui::RichText::new(shortcut).weak()))
}

pub struct MainMenuBar;

impl MainMenuBar {
    pub fn new() -> Self { Self {} }

    pub fn render(&self, ui: &mut egui::Ui, app_state: &AppState, app_settings: &mut Settings) -> Option<AppAction> {
        let mut app_action: Option<AppAction> = None;

        let debug_active;

        #[cfg(feature = "debug")]
        {
            debug_active = app_state.debug_active;
        }
        #[cfg(not(feature = "debug"))]
        {
            debug_active = false;
        }


    
        // Top menu bar
        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {                
                ui.menu_button("File", |ui| {
                    ui.set_width(120.0);
                                        
                    ui.add_enabled_ui(!debug_active, |ui| {
                        if ui.button("Select ROMs Folder").clicked() {
                            app_action = Some(AppAction::SelectRomsFolder);
                            ui.close();
                        }
                        if ui.button("Load ROM").clicked() {
                            app_action = Some(AppAction::LoadRom);
                            ui.close();
                        }
                        ui.menu_button("Recent ROMs", |ui| {
                            for path in &app_settings.recent_roms {
                                let label = path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown");
                                if ui.button(label).clicked() {
                                    app_action = Some(AppAction::LoadRomFromPath(path.clone()));
                                    ui.close();
                                }
                            }
                            if app_settings.recent_roms.is_empty() {
                                ui.label("No recent ROMs");
                            }
                        });
                        if ui.button("Unload ROM").clicked() {
                            app_action = Some(AppAction::UnloadRom);
                            ui.close();
                        }
                    });
                    
                    ui.separator();

                    ui.add_enabled_ui(app_state.loaded_rom_data.is_some(), |ui| {
                        ui.menu_button("Save State", |ui| {
                            for slot in 0..MAX_SAVE_STATE_SLOTS {
                                if ui.button(format!("Slot {}", slot)).clicked() {
                                    app_action = Some(AppAction::SaveState { slot });
                                    ui.close();
                                }
                            }
                        });
                        ui.menu_button("Load State", |ui| {
                            for slot in 0..MAX_SAVE_STATE_SLOTS {
                                let save_exists = app_state.loaded_rom_data.as_ref().unwrap().used_save_state_slots[slot];
    
                                let resp = ui.add_enabled_ui(save_exists, |ui| {
                                    ui.button(format!("Slot {}", slot))
                                }).inner;
    
                                if resp.clicked() {
                                    app_action = Some(AppAction::LoadState { slot });
                                    ui.close();
                                }
                            }
                        });
                    });

                    ui.separator();
                    
                    if ui.button("Settings").clicked() {
                        app_action = Some(AppAction::OpenSettings);
                        ui.close();
                    }
                    
                    ui.separator();
                    
                    if button_with_shortcut(ui, "Exit", "Ctrl + Q").clicked() {
                        log::info!("Exit button clicked, exiting");
                        
                        app_action = Some(AppAction::Exit);
                        ui.close();
                    }
                });
                
                ui.menu_button("Emulation", |ui| {
                    ui.set_width(100.0);
                    
                    ui.add_enabled_ui(!debug_active, |ui| {
                        let pause_text = if app_state.is_paused { "Resume" } else { "Pause" };
                        if ui.button(pause_text).clicked() {
                            app_action = Some(AppAction::SetPaused(!app_state.is_paused));
                            ui.close();
                        }
                        if ui.button("Reset").clicked() {
                            app_action = Some(AppAction::ResetCore);
                            ui.close();
                        }
                        if ui.button("Hard Reset").clicked() {
                            app_action = Some(AppAction::PowerOnCore);
                            ui.close();
                        }
                    });
                    
                    #[cfg(feature = "debug")]
                    {
                        ui.separator();
                        
                        if app_state.debug_active {
                            if ui.button("Stop Debug").clicked() {
                                app_action = Some(AppAction::CloseDebug);
                                ui.close();
                            }
                        } else {
                            if ui.button("Debug").clicked() {
                                app_action = Some(AppAction::OpenDebug(None));
                                ui.close();
                            }
                        }
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.set_width(100.0);
                    
                    let window_size_text = if app_state.is_fullscreen { "Windowed" } else { "Fullscreen" };
                    if button_with_shortcut(ui, window_size_text, "F11").clicked() {
                        app_action = Some(AppAction::ToggleFullscreen);
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