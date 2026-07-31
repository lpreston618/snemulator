use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use egui::{Ui, Vec2};
use serde::{Deserialize, Serialize};

use crate::app::{self, MAX_SAVE_STATE_SLOTS};
use crate::app::settings::Settings;
use crate::app::theme::AppTheme;
use crate::app::thumbnail_fetcher::{self, ThumbnailResult};
use crate::app::AppAction;
use crate::app::rom_paths::{RomManifest, RomPathStem, RomPaths};

const MAX_ROM_DIR_SEARCH_DEPTH: usize = 3;
const STANDARD_THUMBNAIL_HEIGHT: f32 = 160.0;
const STANDARD_THUMBNAIL_WIDTH: f32 = 228.0;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum LibraryViewMode {
    List,
    Grid,
}

pub enum ThumbnailState {
    Loading,
    Ready(PathBuf),
    NotFound,
}

#[derive(Clone, Copy)]
pub enum GameDetailAction {
    Close,
    Play,
    Quickplay,
    DeleteSave,
    DeleteSlot(u32),
}

#[derive(Clone, Copy)]
enum DeleteConfirm {
    SaveData,
    SaveState(u32),
}

#[derive(Clone, Copy)]
enum LibraryGameLoad {
    Play,
    Quickplay,
}

impl LibraryGameLoad {
    fn to_app_action(self, path: PathBuf) -> AppAction {
        match self {
            LibraryGameLoad::Play => AppAction::LoadRomFromPath { path },
            LibraryGameLoad::Quickplay => AppAction::LoadRomAndQuickLoad { path },
        }
    }
}

struct GameDetailState {
    delete_confirm: Option<DeleteConfirm>,
    selected_save_state: Option<u32>,
}

impl GameDetailState {
    pub fn new() -> Self {
        Self {
            delete_confirm: None,
            selected_save_state: None,
        }
    }
}

pub struct LibraryEntry {
    pub path: PathBuf,
    pub display_name: String,
    pub uses_save: bool,
    pub has_save: bool,
    pub has_quicksave: bool,
    pub used_slots: Vec<u32>, // slot indices that exist on disk
    pub last_played: Option<u64>,
    pub play_time_secs: u64,
    pub thumbnail: ThumbnailState,
    pub file_size_bytes: usize,
    pub crc32: u32,
    pub mapping: String,
    pub coprocessor: String,
    pub egui_id: usize,
}

pub struct LibraryView {
    pub entries: Vec<LibraryEntry>,
    selected_entry: Option<usize>,
    detail_view_state: Option<GameDetailState>,
    thumbnail_rx: Option<Receiver<ThumbnailResult>>,
    next_entry_id: usize,
}

impl LibraryView {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            selected_entry: None,
            detail_view_state: None,
            thumbnail_rx: None,
            next_entry_id: 0,
        }
    }

    pub fn update_entry(&mut self, path: &PathBuf) {
        let Some(entry) = self.entries.iter_mut().find(|e| e.path == *path) else { return };
        let stem = RomPathStem::from_path(path);
        let manifest = RomPaths::find_manifest_by_stem(&stem);
        let paths = RomPaths::new(&stem);

        entry.last_played = manifest.as_ref().and_then(|m| m.last_played);
        entry.play_time_secs = manifest.as_ref().map(|m| m.play_time_secs).unwrap_or(0);
        entry.has_save = paths.as_ref().map(|p| p.sav_path().exists()).unwrap_or(false);
        entry.used_slots = paths.map(|p| {
            (0..MAX_SAVE_STATE_SLOTS as u32)
                .filter(|&slot| p.state_path(slot).exists())
                .collect()
        }).unwrap_or_default();
        entry.thumbnail = match manifest.as_ref().and_then(|m| m.thumbnail_path.clone()) {
            Some(p) => ThumbnailState::Ready(p),
            None    => ThumbnailState::NotFound,
        };
    }

    /// Re-scan the library folder. Call this when the folder changes or on startup.
    pub fn scan(&mut self, roms_library_dir: &Option<PathBuf>) {
        self.entries.clear();
        self.selected_entry = None;
        let Some(lib_dir) = roms_library_dir else { return };
        Self::scan_dir(lib_dir, 0, &mut self.entries, &mut self.next_entry_id);
        self.entries.sort_by(|a, b| a.display_name.cmp(&b.display_name));

        // Collect stems that need thumbnails (all Loading entries after scan)
        let stems: Vec<(RomPathStem, PathBuf)> = self.entries.iter()
            .filter(|e| matches!(e.thumbnail, ThumbnailState::Loading))
            .filter_map(|e| {
                let stem = RomPathStem::from_path(&e.path);
                Some((stem, e.path.clone()))
            })
            .collect();

        log::trace!("Stems found with no thumbnails: {}", stems.len());

        if !stems.is_empty() {
            let (tx, rx) = mpsc::channel();
            self.thumbnail_rx = Some(rx);
            thumbnail_fetcher::spawn_thumbnail_resolver(stems, tx);
            log::trace!("Spawned thumbnail fetcher thread");
        }
    }

    fn scan_dir(dir: &PathBuf, depth: usize, entries: &mut Vec<LibraryEntry>, next_id: &mut usize) {
        let Ok(read_dir) = std::fs::read_dir(dir) else { return };

        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() && depth < MAX_ROM_DIR_SEARCH_DEPTH {
                Self::scan_dir(&path, depth + 1, entries, next_id);
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "sfc" | "smc") { continue; }

            let stem = RomPathStem::from_path(&path);
            let paths = RomPaths::new(&stem);

            let manifest = match RomPaths::find_manifest_by_stem(&stem) {
                Some(m) => m,
                None => {
                    let bytes: Option<Vec<u8>> = std::fs::read(&path).ok();

                    // First time seeing this ROM — derive title from header if possible
                    let rom_header_meta = snemcore::cartridge::get_rom_meta(bytes.as_ref().map(|b| b.as_slice()));

                    let crc: u32 = bytes.as_ref()
                        .and_then(|bytes| Some(crc32fast::hash(&bytes)))
                        .unwrap_or_default();

                    let m = RomManifest {
                        rom_crc: crc,
                        display_name: rom_header_meta.title.trim().to_owned(),
                        saves_game: rom_header_meta.saves_game,
                        coprocessor: rom_header_meta.coprocessor_name,
                        mapping: rom_header_meta.mapping_name,
                        rom_size_bytes: rom_header_meta.rom_size_bytes,
                        ..Default::default()
                    };

                    if let Some(p) = &paths {
                        let _ = p.ensure_dirs();
                        p.write_manifest(&m);
                    }

                    m
                }
            };

            let has_save = paths.as_ref().map(|p| p.sav_path().exists()).unwrap_or(false);
            let has_quicksave = paths.as_ref().map(|p| p.quicksave_path().exists()).unwrap_or(false);
            let used_slots = paths.map(|p| {
                (0..MAX_SAVE_STATE_SLOTS as u32)
                    .filter(|&slot| p.state_path(slot).exists())
                    .collect()
            }).unwrap_or_default();

            log::trace!("Read manifest for '{}', thumbnail_path: {:?}, exists: {:?}",
                stem.sanitized_name(),
                &manifest.thumbnail_path,
                manifest.thumbnail_path.as_ref().map(|p| std::fs::exists(p))
            );

            entries.push(LibraryEntry {
                path,
                display_name: manifest.display_name,
                uses_save: manifest.saves_game,
                has_save,
                has_quicksave,
                used_slots,
                last_played: manifest.last_played,
                play_time_secs: manifest.play_time_secs,
                thumbnail: match &manifest.thumbnail_path {
                    Some(p) => {
                        if std::fs::exists(p).ok().unwrap_or(false) {
                            ThumbnailState::Ready(p.clone())
                        } else {
                            ThumbnailState::Loading
                        }
                    },
                    None    => ThumbnailState::Loading,
                },
                file_size_bytes: manifest.rom_size_bytes,
                crc32: manifest.rom_crc,
                mapping: manifest.mapping,
                coprocessor: manifest.coprocessor,
                egui_id: *next_id
            });

            *next_id += 1;
        }
    }

    pub fn render(&mut self, ui: &mut Ui, app_settings: &Settings, app_theme: &AppTheme) -> Option<AppAction> {
        if let Some(rx) = &self.thumbnail_rx {
            'receive_thumbnails: loop {
                match rx.try_recv() {
                    Ok(result) => {
                        log::trace!("Received thumbnail: found={}, path={:?}", result.path.is_some(), result.path);
        
                        if let Some(entry) = self.entries.iter_mut()
                            .find(|e| RomPathStem::from_path(&e.path).raw_name() == result.stem.raw_name())
                        {
                            entry.thumbnail = match result.path {
                                Some(p) => ThumbnailState::Ready(p),
                                None    => ThumbnailState::NotFound,
                            };
                        }
                    }
                    Err(TryRecvError::Empty) => { break 'receive_thumbnails; }
                    Err(TryRecvError::Disconnected) => {
                        self.thumbnail_rx = None;

                        for entry in self.entries.iter_mut() {
                            if matches!(entry.thumbnail, ThumbnailState::Loading) {
                                entry.thumbnail = ThumbnailState::NotFound;
                            }
                        }

                        break 'receive_thumbnails;
                    }
                }

            }
        }

        let mut action: Option<AppAction>;

        action = ui.add_enabled_ui(self.detail_view_state.is_none(), |ui| {
            match app_settings.library_view_mode {
                LibraryViewMode::List => self.render_library_list(ui, app_theme),
                LibraryViewMode::Grid => { None }, //self.render_library_grid(ui, app_theme),
            }
        }).inner;

        if self.detail_view_state.is_some() {
            if self.selected_entry.is_none() {
                self.detail_view_state = None;
                return action;
            }

            let entry = &mut self.entries[self.selected_entry.unwrap()];
            
            let detail_view_action = Self::show_game_detail_panel(ui, entry, self.detail_view_state.as_mut().unwrap(), app_theme);

            if let Some(detail_action) = detail_view_action {
                match detail_action {
                    GameDetailAction::Close => { self.detail_view_state = None; },
                    GameDetailAction::Play => {
                        if let Some(slot) = self.detail_view_state.as_ref().unwrap().selected_save_state {
                            action = Some(AppAction::LoadRomAndState {
                                path: entry.path.clone(),
                                slot,
                            });
                        } else {
                            action = Some(AppAction::LoadRomFromPath{ path: entry.path.clone() });
                        }
                    },
                    GameDetailAction::Quickplay => {
                        action = Some(AppAction::LoadRomAndQuickLoad { path: entry.path.clone() });
                    },
                    GameDetailAction::DeleteSave => {
                        action = Some(AppAction::DeleteSaveData { path: entry.path.clone() });

                        entry.has_save = false;
                    },
                    GameDetailAction::DeleteSlot(slot) => {
                        action = Some(AppAction::DeleteStateForRom {
                            path: entry.path.clone(),
                            slot,
                        });

                        entry.used_slots.retain(|&s| s != slot);

                        if let Some(detail_view_state) = &mut self.detail_view_state {
                            if Some(slot) == detail_view_state.selected_save_state {
                                detail_view_state.selected_save_state = None;
                            }
                        }
                    },
                }
            }
        }

        action
    }

    fn show_game_detail_panel(
        ctx: &egui::Context,
        entry: &mut LibraryEntry,
        state: &mut GameDetailState,
        app_theme: &AppTheme,
    ) -> Option<GameDetailAction> {
        let mut action: Option<GameDetailAction> = None;

        let screen_rect = ctx.content_rect();
        
        let tab_width = 24.0;
        let tab_height = 64.0;
        let content_width = 640.0;
        let total_width = tab_width + content_width;

        egui::Window::new("game_detail_panel")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .fixed_size(Vec2::new(total_width, screen_rect.height()))
            .fixed_pos(egui::pos2(screen_rect.right() - total_width, screen_rect.top()))
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let available_rect = ui.available_rect_before_wrap();
                
                // Define the tab rect (vertically centered on the left edge)
                let tab_rect = egui::Rect::from_center_size(
                    egui::pos2(
                        available_rect.min.x + tab_width / 2.0,
                        available_rect.center().y,
                    ),
                    Vec2::new(tab_width, tab_height),
                );
                
                // Define the content rect (right side, full height)
                let content_rect = egui::Rect::from_min_size(
                    egui::pos2(available_rect.min.x + tab_width, available_rect.min.y),
                    Vec2::new(content_width, available_rect.height()),
                );
                
                // Fill background for content area first (so tab appears on top)
                ui.painter().rect_filled(content_rect, 0.0, app_theme.bg_elevated);
                
                // Draw and handle tab
                let tab_response = ui.allocate_rect(tab_rect, egui::Sense::click());
                let painter = ui.painter();
                
                let tab_bg = if tab_response.hovered() {
                    app_theme.bg_elevated.linear_multiply(1.3)
                } else {
                    app_theme.bg_elevated
                };
                
                // Rounded corners only on the left side
                let tab_corner_radius = egui::CornerRadius {
                    nw: 8,
                    sw: 8,
                    ne: 0,
                    se: 0,
                };
                
                painter.rect_filled(tab_rect, tab_corner_radius, tab_bg);
                
                // Optional: subtle border around the tab
                painter.rect_stroke(
                    tab_rect,
                    tab_corner_radius,
                    egui::Stroke::new(1.0, app_theme.border),
                    egui::StrokeKind::Outside,
                );
                
                let chevron_color = if tab_response.hovered() {
                    app_theme.text_primary
                } else {
                    app_theme.text_muted
                };
                
                painter.text(
                    tab_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "❯",
                    egui::FontId::proportional(16.0),
                    chevron_color,
                );
                
                if tab_response.clicked() {
                    action = Some(GameDetailAction::Close);
                }
                
                // Create a child UI for the content area
                let mut content_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(content_rect)
                        .layout(egui::Layout::top_down(egui::Align::LEFT))
                );
                
                egui::Frame::NONE
                    .inner_margin(16.0)
                    .show(&mut content_ui, |ui| {
                        Self::render_game_detail_panel(ui, entry, state, app_theme, &mut action);
                    });
            });

        if state.delete_confirm.is_some() {
            Self::render_delete_confirm_dialog(ctx, entry, state, app_theme, &mut action);
        }

        action
    }

    fn render_library_list(&mut self, ui: &mut Ui, app_theme: &AppTheme) -> Option<AppAction> {
        let mut action: Option<AppAction> = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            for (i, entry) in self.entries.iter_mut().enumerate() {
                let is_selected = self.selected_entry == Some(i);
                let entry_id = entry.egui_id;

                ui.push_id(entry_id, |ui| {
                    let (response, game_load_action) = Self::render_list_entry(ui, entry, is_selected, app_theme);
                    
                    if let Some(load_action) = game_load_action {
                        action = Some(load_action.to_app_action(entry.path.clone()));
                    } else {
                        if response.clicked() {
                            if self.selected_entry != Some(i) {
                                self.selected_entry = Some(i);
                            }
                            
                            self.detail_view_state = Some(GameDetailState::new());
                        }
        
                        #[cfg(feature = "debug")]
                        response.context_menu(|ui| {
                            if ui.button("Debug").clicked() {
                                action = Some(AppAction::OpenDebug(Some(entry.path.clone())));
                                ui.close();
                            }
                        });
                    }
                });

            }
        });

        action
    }

    fn render_list_entry(
        ui: &mut Ui,
        entry: &mut LibraryEntry,
        is_selected: bool,
        app_theme: &AppTheme,
    ) -> (egui::Response, Option<LibraryGameLoad>) {
        const THUMBNAIL_SCALE: f32 = 0.75;
        const SCALED_THUMBNAIL_HEIGHT: f32 = THUMBNAIL_SCALE * STANDARD_THUMBNAIL_HEIGHT;
        const SCALED_THUMBNAIL_WIDTH: f32 = THUMBNAIL_SCALE * STANDARD_THUMBNAIL_WIDTH;
        const THUMBNAIL_MARGIN: f32 = 12.0;
        const ROW_HEIGHT: f32 = SCALED_THUMBNAIL_HEIGHT + THUMBNAIL_MARGIN;

        let mut game_load_action: Option<LibraryGameLoad> = None;

        let thumbnail_size = Vec2::new(SCALED_THUMBNAIL_WIDTH, SCALED_THUMBNAIL_HEIGHT);

        let available_width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(available_width, ROW_HEIGHT),
            egui::Sense::click(),
        );

        if !ui.is_rect_visible(rect) {
            return (response, None);
        }

        let painter = ui.painter();
        let cr = app_theme.corner_radius as f32;

        // Background
        if is_selected {
            painter.rect_filled(rect, 0.0, app_theme.bg_elevated);
            painter.rect_filled(
                egui::Rect::from_min_size(rect.min, Vec2::new(3.0, ROW_HEIGHT)),
                0.0,
                app_theme.accent,
            );
        } else if response.hovered() {
            painter.rect_filled(rect, 0.0, app_theme.bg_secondary);
        }

        // Separator
        painter.line_segment(
            [egui::pos2(rect.min.x, rect.max.y - 0.5), egui::pos2(rect.max.x, rect.max.y - 0.5)],
            egui::Stroke::new(0.5, app_theme.border),
        );

        // Thumbnail
        let thumb_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + THUMBNAIL_MARGIN, rect.min.y + (ROW_HEIGHT - SCALED_THUMBNAIL_HEIGHT) / 2.0),
            thumbnail_size,
        );
        painter.rect_filled(thumb_rect, cr, app_theme.bg_tertiary);
        painter.rect_stroke(
            thumb_rect, 
            cr, 
            egui::Stroke::new(1.0, app_theme.border),
            egui::StrokeKind::Outside,
        );

        Self::render_box_art(ui, thumb_rect, entry, app_theme);

        let painter = ui.painter();

        let text_x = thumb_rect.max.x + 10.0;
        let mid_y = rect.center().y;

        let title_size = 12.0;
        let filename_size = 10.0;

        let max_stem_len = 40;

        // Title + subtitle
        painter.text(
            egui::pos2(text_x, mid_y - 16.0),
            egui::Align2::LEFT_CENTER,
            &entry.display_name,
            app::theme::bold_font(title_size),
            if is_selected { app_theme.text_primary } else { app_theme.text_secondary },
        );
        let stem = entry.path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let trimmed_stem = if stem.len() <= max_stem_len {
            stem.to_string()
        } else {
            stem.chars()
                .take(max_stem_len - 3)
                .collect::<String>()
                + "..."
        };
        if stem != entry.display_name {
            painter.text(
                egui::pos2(text_x, mid_y + 3.0),
                egui::Align2::LEFT_CENTER,
                trimmed_stem,
                egui::FontId::proportional(filename_size),
                app_theme.text_muted,
            );
        }

        // Quick Play Button
        const BUTTON_SIZE: f32 = 24.0;
        
        let play_rect = egui::Rect::from_min_size(
            egui::pos2(text_x, mid_y + 16.0),
            Vec2::splat(BUTTON_SIZE),
        );

        let play_id = response.id.with("mini_play_button");
        let play_response = ui.interact(play_rect, play_id, egui::Sense::click())
            .on_hover_text("Play");

        let button_fill = if play_response.hovered() {
            app_theme.success.linear_multiply(1.15)
        } else {
            app_theme.success
        };

        painter.rect_filled(play_rect, app_theme.widget_corner_radius as f32, button_fill);
        painter.text(
            play_rect.center(),
            egui::Align2::CENTER_CENTER,
            "▶",
            egui::FontId::proportional(11.0),
            egui::Color32::WHITE,
        );

        if play_response.clicked() {
            game_load_action = Some(LibraryGameLoad::Play);
        }

        if entry.has_quicksave {
            let quickplay_rect = egui::Rect::from_min_size(
                egui::pos2(text_x + BUTTON_SIZE + 8.0, mid_y + 16.0),
                Vec2::splat(BUTTON_SIZE),
            );

            let quickplay_id = response.id.with("mini_quickplay_button");
            let quickplay_response = ui.interact(quickplay_rect, quickplay_id, egui::Sense::click())
                .on_hover_text("Quick Load");

            let button_fill = if quickplay_response.hovered() {
                app_theme.info.linear_multiply(1.15)
            } else {
                app_theme.info
            };

            painter.rect_filled(quickplay_rect, app_theme.widget_corner_radius as f32, button_fill);
            painter.text(
                quickplay_rect.center(),
                egui::Align2::CENTER_CENTER,
                "▶",
                egui::FontId::proportional(11.0),
                egui::Color32::WHITE,
            );

            if quickplay_response.clicked() {
                game_load_action = Some(LibraryGameLoad::Quickplay);
            }
        }

        // Additional Play Data (Right side)
        const RIGHT_MARGIN: f32 = 24.0;
        const LINE_HEIGHT: f32 = 20.0;

        let right_edge = rect.max.x - RIGHT_MARGIN;
        let block_height = LINE_HEIGHT * 3.0;
        let block_top = mid_y - block_height / 2.0;

        // Row 1: save status
        let (save_text, save_color) = if !entry.uses_save {
            ("--", app_theme.text_muted)
        } else if entry.has_save {
            ("Yes", app_theme.success)
        } else {
            ("No", app_theme.warning)
        };

        let mut job = egui::text::LayoutJob::default();
        job.append(
            "Save: ",
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(11.0),
                color: app_theme.text_muted,
                ..Default::default()
            },
        );
        job.append(
            save_text,
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(11.0),
                color: save_color,
                ..Default::default()
            },
        );

        let galley = ui.fonts_mut(|f| f.layout_job(job));
        let pos = egui::pos2(
            right_edge - galley.size().x,
            block_top + LINE_HEIGHT * 0.5 - galley.size().y / 2.0,
        );
        painter.galley(pos, galley, app_theme.text_muted);

        // Row 2: last played
        painter.text(
            egui::pos2(right_edge, block_top + LINE_HEIGHT * 1.5),
            egui::Align2::RIGHT_CENTER,
            entry.last_played.map(format_timestamp).unwrap_or_else(|| "Never".to_string()),
            egui::FontId::proportional(11.0),
            app_theme.text_muted,
        );

        // Row 3: time played
        painter.text(
            egui::pos2(right_edge, block_top + LINE_HEIGHT * 2.5),
            egui::Align2::RIGHT_CENTER,
            format_play_time(entry.play_time_secs),
            egui::FontId::proportional(11.0),
            app_theme.text_muted,
        );

        (response, game_load_action)
    }

    fn render_game_detail_panel(
        ui: &mut Ui,
        entry: &mut LibraryEntry,
        state: &mut GameDetailState,
        app_theme: &AppTheme,
        action: &mut Option<GameDetailAction>,
    ) {
        ui.add_space(8.0);

        // Box art
        let art_size = Vec2::new(ui.available_width(), 240.0);
        let (art_rect, _) = ui.allocate_exact_size(art_size, egui::Sense::hover());
        Self::render_box_art(ui, art_rect, entry, app_theme);

        ui.add_space(12.0);

        // Title on the left, technical details right-aligned to the panel's edge
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&entry.display_name)
                    .font(app::theme::bold_font(20.0))
                    .color(app_theme.text_primary),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    let detail_font = egui::FontId::proportional(10.0);

                    ui.label(
                        egui::RichText::new(format_file_size(entry.file_size_bytes))
                            .font(detail_font.clone())
                            .color(app_theme.text_muted),
                    );
                    ui.label(
                        egui::RichText::new(format!("CRC32: {:08X}", entry.crc32))
                        .font(detail_font.clone())
                        .color(app_theme.text_muted),
                    );
                    ui.label(
                        egui::RichText::new(format!("Mapping: {}", entry.mapping))
                        .font(detail_font.clone())
                        .color(app_theme.text_muted),
                    );
                    ui.label(
                        egui::RichText::new(format!("Coprocessor: {}", entry.coprocessor))
                            .font(detail_font.clone())
                            .color(app_theme.text_muted),
                    );
                });
            });
        });


        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        // Save data status
        ui.horizontal(|ui| {
            let (save_text, save_color) = if !entry.uses_save {
                ("--", app_theme.text_muted)
            } else if entry.has_save {
                ("Yes", app_theme.success)
            } else {
                ("No", app_theme.warning)
            };

            let mut job = egui::text::LayoutJob::default();
            job.append(
                "Save Data: ",
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(13.0),
                    color: app_theme.text_muted,
                    ..Default::default()
                },
            );
            job.append(
                save_text,
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(13.0),
                    color: save_color,
                    ..Default::default()
                },
            );
            ui.label(job);

            if entry.uses_save && entry.has_save {
                ui.add_space(8.0);
                if ui
                    .add(egui::Button::new(egui::RichText::new("🗑").color(app_theme.error)).frame(false))
                    .on_hover_text("Delete save data")
                    .clicked()
                {
                    state.delete_confirm = Some(DeleteConfirm::SaveData);
                }
            }
        });

        ui.add_space(10.0);

        // Save state slots
        ui.label(egui::RichText::new("Save States").color(app_theme.text_muted).size(11.0));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for slot in 0..MAX_SAVE_STATE_SLOTS as u32 {
                ui.push_id(slot, |ui| {
                    let used = entry.used_slots.contains(&slot);
                    let selected = state.selected_save_state == Some(slot);
                    
                    ui.add_enabled_ui(used, |ui| {
                        let (rect, resp) = ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::click());
                        let painter = ui.painter();
        
                        let fill = if resp.hovered() {
                            app_theme.accent.linear_multiply(0.45)
                        } else {
                            app_theme.accent.linear_multiply(0.3)
                        };
        
                        painter.rect_filled(rect, 4.0, fill);
                        
                        if used && selected {
                            painter.rect_stroke(rect, 6.0, egui::Stroke::new(1.0, app_theme.border_focused), egui::StrokeKind::Outside);
                        } else {
                            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, app_theme.border), egui::StrokeKind::Outside);
                        }
                        
                        if used {
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{}", slot),
                                egui::FontId::proportional(11.0),
                                app_theme.text_secondary,
                            );
                        }
        
                        if resp.clicked() {
                            state.selected_save_state = if selected { None } else { Some(slot) };
                        }
                    });
                });
            }
        });

        ui.add_space(16.0);

        // Play / Load State button, bottom-right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
            if let Some(slot) = state.selected_save_state {
                let load_button = egui::Button::new(
                    egui::RichText::new(format!("▶  Load State {}", slot))
                        .size(16.0)
                        .color(egui::Color32::WHITE),
                )
                .fill(app_theme.success)
                .corner_radius(app_theme.widget_corner_radius as f32);

                if ui.add_sized(Vec2::new(160.0, 40.0), load_button).clicked() {
                    *action = Some(GameDetailAction::Play);
                }

                ui.add_space(8.0);

                let delete_slot_button = egui::Button::new(
                    egui::RichText::new(format!("🗑 Delete State {}", slot))
                        .size(16.0)
                        .color(app_theme.error),
                )
                .corner_radius(app_theme.widget_corner_radius as f32)
                .stroke(egui::Stroke::new(2.0, app_theme.error));

                if ui.add_sized(Vec2::new(160.0, 40.0), delete_slot_button).clicked() {
                    state.delete_confirm = Some(DeleteConfirm::SaveState(slot));
                }
            } else {
                if entry.has_quicksave {
                    let continue_button = egui::Button::new(
                        egui::RichText::new("▶ Quick Load").size(16.0).color(egui::Color32::WHITE),
                    )
                    .fill(app_theme.info)
                    .corner_radius(app_theme.widget_corner_radius as f32);

                    if ui.add_sized(Vec2::new(140.0, 40.0), continue_button).clicked() {
                        *action = Some(GameDetailAction::Quickplay);
                    }

                    ui.add_space(8.0);
                }

                let play_button = egui::Button::new(
                    egui::RichText::new("▶  Play").size(16.0).color(egui::Color32::WHITE),
                )
                .fill(app_theme.success)
                .corner_radius(app_theme.widget_corner_radius as f32);

                if ui.add_sized(Vec2::new(120.0, 40.0), play_button).clicked() {
                    *action = Some(GameDetailAction::Play);
                }
            }
        });
    }

    fn render_delete_confirm_dialog(
        ctx: &egui::Context,
        entry: &LibraryEntry,
        state: &mut GameDetailState,
        app_theme: &AppTheme,
        action: &mut Option<GameDetailAction>,
    ) {
        if state.delete_confirm.is_none() { return; }

        let prompt = match state.delete_confirm.unwrap() {
            DeleteConfirm::SaveData => format!(
                "Are you sure you want to delete the save data for \"{}\"? This cannot be undone.",
                entry.display_name
            ),
            DeleteConfirm::SaveState(slot) => format!(
                "Are you sure you want to delete the data for \"{}\" save state slot {}? This cannot be undone.",
                entry.display_name,
                slot,
            ),
        };

        let delete_action = match state.delete_confirm.unwrap() {
            DeleteConfirm::SaveData => GameDetailAction::DeleteSave,
            DeleteConfirm::SaveState(slot) => GameDetailAction::DeleteSlot(slot),
        };

        egui::Window::new("Please Confirm")
            .max_height(64.0)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(prompt);
                ui.add_space(12.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let delete_button = egui::Button::new(egui::RichText::new("Delete").color(egui::Color32::WHITE))
                        .fill(app_theme.warning);

                    if ui.add(delete_button).clicked() {
                        *action = Some(delete_action);
                        state.delete_confirm = None;
                    }
                    if ui.button("Cancel").clicked() {
                        state.delete_confirm = None;
                    }
                });
            });
    }

    fn render_box_art(ui: &mut Ui, rect: egui::Rect, entry: &mut LibraryEntry, app_theme: &AppTheme) {
        let cr = app_theme.corner_radius as f32;
        let painter = ui.painter();
        painter.rect_filled(rect, cr, app_theme.bg_tertiary);
        painter.rect_stroke(rect, cr, egui::Stroke::new(1.0, app_theme.border), egui::StrokeKind::Outside);

        match &mut entry.thumbnail {
            ThumbnailState::Ready(path) => {
                let uri = format!("file:///{}", path.to_string_lossy().replace('\\', "/"));
                let image = egui::Image::new(&uri).fit_to_exact_size(rect.size()).corner_radius(cr);
                match image.load_for_size(ui.ctx(), rect.size()) {
                    Ok(_) => { ui.put(rect, image); }
                    Err(e) => {
                        log::error!("box art load failed for {}: {e:?}", &uri);
                        entry.thumbnail = ThumbnailState::NotFound;
                    }
                }
            }
            ThumbnailState::Loading => {
                // Spinning arc loader
                let center = rect.center();
                let radius = 14.0;
                let t = ui.ctx().input(|i| i.time) as f32;
                let start_angle = t * 2.5;
                let sweep = std::f32::consts::PI * 1.4;

                let steps = 32usize;
                let points: Vec<egui::Pos2> = (0..=steps)
                    .map(|i| {
                        let angle = start_angle + (i as f32 / steps as f32) * sweep;
                        egui::pos2(
                            center.x + radius * angle.cos(),
                            center.y + radius * angle.sin(),
                        )
                    })
                    .collect();

                for pair in points.windows(2) {
                    let alpha = (pair[0].x - center.x + radius) / (2.0 * radius);
                    painter.line_segment(
                        [pair[0], pair[1]],
                        egui::Stroke::new(2.5, app_theme.accent.linear_multiply(alpha.clamp(0.2, 1.0))),
                    );
                }

                ui.ctx().request_repaint();
            }
            ThumbnailState::NotFound => {
                let initial = entry
                    .display_name
                    .chars()
                    .next()
                    .and_then(|c| c.to_uppercase().next())
                    .unwrap_or('?');
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    initial.to_string(),
                    egui::FontId::proportional(64.0),
                    app_theme.text_muted,
                );
            }
        }
    }
}

fn format_play_time(secs: u64) -> String {
    if secs == 0 { return "—".to_string(); }
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 { format!("{}h {}m", h, m) } else { format!("{}m", m) }
}

fn format_timestamp(unix_secs: u64) -> String {
    // Without a date library we do a rough conversion.
    // Good enough for display; swap for chrono if you add it.
    let days_since_epoch = unix_secs / 86400;
    // Rough Gregorian: good to ~2100
    let year  = 1970 + days_since_epoch / 365;
    let day_of_year = days_since_epoch % 365;
    let month = (day_of_year / 30).min(11) + 1;
    let day   = (day_of_year % 30) + 1;
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn format_file_size(size: usize) -> String {
    const MB: usize = 1024 * 1024;
    const KB: usize = 1024;

    let mb = size / MB;
    let kb = (size % MB) / KB;

    if mb == 0 {
        return format!("{kb} KiB");
    }

    if kb == 0 {
        return format!("{mb} MiB");
    }

    let mb = (mb as f32) + (kb as f32) / 1024.0;

    format!("{mb:.02} MiB")
}