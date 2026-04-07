// use std::{io::{Read, Write}, path::PathBuf, time::{Duration, Instant}};

// use anyhow::Result;
// use common::UiWindow;
// use watchpoint_editor::WatchpointEditor;
// use rfd::FileDialog;

// const WINDOW_WIDTH: u32 = 600;
// const WINDOW_HEIGHT: u32 = 400;
// const TARGET_FPS: u32 = 30;
// const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);

// fn main() -> Result<()> {
//     let sdl_context = sdl3::init()?;
//     let video_subsystem = sdl_context.video()?;
//     let mut event_pump = sdl_context.event_pump()?;
    
//     let mut window = UiWindow::new(
//         &video_subsystem,
//         "Snem Watchpoint Editor",
//         WINDOW_WIDTH,
//         WINDOW_HEIGHT,
//     )?;
    
//     let mut wp_editor = WatchpointEditor::new();
    
//     let mut wp_file: Option<PathBuf> = None;
//     let mut saved = false;
    
//     'running: loop {
//         let frame_start = Instant::now();
        
//         let keyboard_state = event_pump.keyboard_state();

//         let modifiers = egui::Modifiers {
//             alt: keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::LAlt)
//                 || keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::RAlt),
//             ctrl: keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::LCtrl)
//                 || keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::RCtrl),
//             shift: keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::LShift)
//                 || keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::RShift),
//             mac_cmd: keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::LGui)
//                 || keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::RGui),
//             command: keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::LGui)
//                 || keyboard_state.is_scancode_pressed(sdl3::keyboard::Scancode::RGui),
//         };
        
//         for event in event_pump.poll_iter() {
//             match event {
//                 sdl3::event::Event::Quit { .. } => {
//                     if !saved {
                        
//                     }
                    
//                     break 'running;
//                 }
//                 _ => {
//                     window.handle_sdl_keyboard_event(&event);
//                     window.handle_sdl_mouse_event(&event, &modifiers);
//                 }
//             }
            
//             let full_output = window.update_ui(|ctx| {
//                 egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
//                     egui::MenuBar::new().ui(ui, |ui| {                
//                         ui.menu_button("File", |ui| {
//                             ui.set_width(100.0);
                            
//                             if ui.button("Open").clicked() {
//                                 // save_file(&mut wp_file, &wp_editor);
//                                 open_file(&mut wp_file, &mut wp_editor);
//                                 ui.close();
//                             }
//                         });
//                     });
//                 });
                
//                 egui::CentralPanel::default().show(ctx, |ui| {
//                     wp_editor.show(ui);        
//                 });
//             });
            
//             window.render(full_output);
            
//             let elapsed = frame_start.elapsed();
            
//             if elapsed < FRAME_DURATION {
//                 std::thread::sleep(FRAME_DURATION - elapsed);
//             }
//         }
//     }
    
//     Ok(())
// }

// fn save_file(wp_file: &mut Option<PathBuf>, wp_editor: &WatchpointEditor) -> Result<()> {
//     if let Some(path) = wp_file.take() {
//         let data = wp_editor.serialize();
        
//         let mut file = std::fs::File::create(path)?;
        
//         file.write(&data[..]);
//     }
    
//     Ok(())
// }

// fn open_file(wp_file: &mut Option<PathBuf>, wp_editor: &mut WatchpointEditor) -> Result<()> {
//     let wp_file_name = FileDialog::new()
//         .add_filter("Watchpoint File", &["swp"])
//         .set_directory("/")
//         .pick_file();
    
//     *wp_file = wp_file_name;
    
//     if wp_file.is_none() {
//         return Err(anyhow::anyhow!("Invalid file picked"))
//     }
    
//     let mut data = Vec::new();
//     let mut file = std::fs::File::open(wp_file.as_ref().unwrap())?;
    
//     file.read(&mut data);
    
//     wp_editor.deserialize(data);
    
//     Ok(())
// }

fn main() {}