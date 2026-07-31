use crate::app::{AppAction, AppState, MAX_SAVE_STATE_SLOTS, settings::{HotkeyAction, Settings}};

fn button_with_shortcut(ui: &mut egui::Ui, label: &str, shortcut: &str) -> egui::Response {
    ui.add(egui::Button::new(label).right_text(egui::RichText::new(shortcut).weak()))
}

pub struct MainMenuBar;

impl MainMenuBar {
    pub fn new() -> Self { Self {} }

    pub fn render(&self, ui: &mut egui::Ui, app_state: &mut AppState, app_settings: &mut Settings) -> Option<AppAction> {
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

        let mut any_menu_open = false;

        // Top menu bar
        egui::Panel::top("menu_bar").show(ui, |ui| {
            let menu = egui::MenuBar::new().ui(ui, |ui| {
                let file_menu = ui.menu_button("File", |ui| {
                    ui.set_min_width(120.0);
                                        
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
                                    app_action = Some(AppAction::LoadRomFromPath { path: path.clone() });
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
                        let quick_save_shortcut_text = &app_settings.hotkeys
                            .action_button_label(HotkeyAction::QuickSave);

                        if button_with_shortcut(ui, "Quick Save", &quick_save_shortcut_text).clicked() {
                            app_action = Some(AppAction::QuickSave);
                            ui.close();
                        }

                        let quick_load_shortcut_text = &app_settings.hotkeys
                            .action_button_label(HotkeyAction::QuickLoad);

                        if button_with_shortcut(ui, "Quick Load", quick_load_shortcut_text).clicked() {
                            app_action = Some(AppAction::QuickLoad);
                            ui.close();
                        }

                        ui.menu_button("Save State", |ui| {
                            for slot in 0..MAX_SAVE_STATE_SLOTS {
                                if ui.button(format!("Slot {}", slot)).clicked() {
                                    app_action = Some(AppAction::SaveState { slot: slot as u32 });
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
                                    app_action = Some(AppAction::LoadState { slot: slot as u32 });
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
                        app_action = Some(AppAction::Exit);
                        ui.close();
                    }
                });
                
                if file_menu.inner.is_some() || file_menu.response.hovered() {
                    any_menu_open = true;
                }

                let emulation_menu = ui.menu_button("Emulation", |ui| {
                    ui.set_min_width(120.0);
                    
                    ui.add_enabled_ui(!debug_active, |ui| {
                        let pause_text = if app_state.is_paused { "Resume" } else { "Pause" };
                        let pause_shortcut_text = &app_settings.hotkeys
                            .action_button_label(HotkeyAction::TogglePause);
                        
                        if button_with_shortcut(ui, pause_text, pause_shortcut_text).clicked() {
                            app_action = Some(AppAction::SetPaused(!app_state.is_paused));
                            ui.close();
                        }

                        let ff_text = if app_settings.fast_forward_en { "Disable FF" } else { "Enable FF" };
                        let ff_shortcut_text = &app_settings.hotkeys
                            .action_button_label(HotkeyAction::ToggleFastForward);

                        if button_with_shortcut(ui, ff_text, ff_shortcut_text).clicked() {
                            app_action = Some(AppAction::ToggleFastForward);
                            ui.close();
                        }

                        ui.separator();

                        let reset_shortcut_text = &app_settings.hotkeys
                            .action_button_label(HotkeyAction::Reset);

                        if button_with_shortcut(ui, "Reset", reset_shortcut_text).clicked() {
                            app_action = Some(AppAction::ResetCore);
                            ui.close();
                        }

                        let hard_reset_shortcut_text = &app_settings.hotkeys
                            .action_button_label(HotkeyAction::HardReset);

                        if button_with_shortcut(ui, "Hard Reset", hard_reset_shortcut_text).clicked() {
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
                
                if emulation_menu.inner.is_some() || emulation_menu.response.hovered() {
                    any_menu_open = true;
                }

                let view_menu = ui.menu_button("View", |ui| {
                    ui.set_min_width(120.0);
                    
                    let window_size_text = if app_state.is_fullscreen { "Windowed" } else { "Fullscreen" };
                    let fullscreen_shortcut_text = &app_settings.hotkeys
                            .action_button_label(HotkeyAction::ToggleFullscreen);

                    if button_with_shortcut(ui, window_size_text, fullscreen_shortcut_text).clicked() {
                        app_action = Some(AppAction::ToggleFullscreen);
                        ui.close();
                    }

                    let show_fps_text = if app_settings.show_fps { "✔" } else { " " };
                    if button_with_shortcut(ui, "Show FPS", show_fps_text).clicked() {
                        app_settings.show_fps = !app_settings.show_fps;
                        ui.close();
                    }
                });

                if view_menu.inner.is_some() || view_menu.response.hovered() {
                    any_menu_open = true;
                }
            }).response;

            app_state.menu_in_use = menu.hovered() || any_menu_open;
        });

        app_action
    }
}