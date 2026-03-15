// use crate::app::ui_window::UiWindow;
// use crate::core;

// pub struct DisassemblyView {
//     pub follow_pc: bool,
//     anchor_addr: u32,        // last known-good address to decode from
//     cached_lines: Vec<DisasmLine>,
//     scroll_offset: usize,
// }

// pub struct DisasmLine {
//     pub addr: u32,
//     pub bytes: Vec<u8>,
//     pub mnemonic: String,
//     pub has_breakpoint: bool,
// }

// impl DisassemblyView {
//     /// Call when PC changes significantly or user navigates manually.
//     /// Decodes `count` instructions forward from `start_addr`.
//     pub fn decode_forward(start_addr: u32, count: usize, memory: &impl MemoryRead) -> Vec<DisasmLine> {
//         let mut lines = Vec::with_capacity(count);
//         let mut addr = start_addr;
//         for _ in 0..count {
//             let (mnemonic, len) = disassemble_one(addr, memory);
//             let bytes = (0..len).map(|i| memory.read_byte(addr + i as u32)).collect();
//             lines.push(DisasmLine { addr, bytes, mnemonic, has_breakpoint: false });
//             addr += len as u32;
//         }
//         lines
//     }
    
//     pub fn update(&mut self, pc: u32, memory: &impl MemoryRead) {
//         if self.follow_pc {
//             self.anchor_addr = pc;
//         }
//         self.cached_lines = Self::decode_forward(self.anchor_addr, 64, memory);
//     }
// }

// pub struct ChrViewer {
//     texture: Option<glow::Texture>,
//     bpp_mode: core::sppu::ColorDepth,
//     palette_index: usize,
// }

// impl ChrViewer {
//     // Call once during DebugWindow::new(), same pattern as game_texture init
//     pub fn init_texture(gl: &glow::Context) -> Option<glow::Texture> { ... }

//     // Decode VRAM tiles -> RGBA pixels, upload via tex_sub_image_2d
//     pub fn update_texture(&self, gl: &glow::Context, vram: &[u8], cgram: &[u8]) {
//         let mut pixels = vec![0u8; TILES_WIDE * TILES_TALL * 8 * 8 * 4];
//         // decode tiles from vram into pixels using self.bpp_mode
//         // ...
//         unsafe {
//             gl.bind_texture(glow::TEXTURE_2D, self.texture);
//             gl.tex_sub_image_2d( ... pixels ... );
//         }
//     }
    
//     // In egui, display with egui::Image using a TextureId registered via egui_painter
//     // NOTE: You'll need to register the raw GL texture with egui_glow to get a TextureId
// }

// pub struct DebugWindow {
//     egui_window: UiWindow,
//     disasm: DisassemblyView,
//     chr_viewer: ChrViewer,
//     selected_tab: DebugTab,
//     mem_region: MemRegion,
//     mem_scroll: f32,
// }

// #[derive(PartialEq)]
// enum DebugTab { Cpu, Memory, Disassembly, ChrRam, Ppu, Breakpoints }

// impl DebugWindow {
//     pub fn new(video_subsystem: &sdl3::VideoSubsystem) -> Result<Self> { ... }

//     pub fn update_and_render(&mut self, snes: &SnesCore) {
//         let gl = self.egui_window.gl();
//         self.chr_viewer.update_texture(gl, snes.vram(), snes.cgram());
        
//         let full_output = self.egui_window.update_ui(|ctx| {
//             egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
//                 ui.horizontal(|ui| {
//                     for tab in [DebugTab::Cpu, DebugTab::Memory, /* ... */] {
//                         ui.selectable_value(&mut self.selected_tab, tab, tab.label());
//                     }
//                 });
//             });
//             egui::CentralPanel::default().show(ctx, |ui| {
//                 match self.selected_tab {
//                     DebugTab::Memory     => self.render_memory_viewer(ui, snes),
//                     DebugTab::Disassembly => self.render_disassembly(ui, snes),
//                     DebugTab::ChrRam     => self.render_chr_viewer(ui),
//                     DebugTab::Cpu        => self.render_cpu_state(ui, snes),
//                     // ...
//                 }
//             });
//         });
        
//         self.egui_window.clear();
//         self.egui_window.render(full_output);
//     }
// }