use crate::debug::{breakpoints::BreakpointInfo, debugger::Debugger};
use common::app_utils::monospace_text;
use snemcore::{Snemulator, cartridge, scpu::{self, disassembler::DisasmLine}};

const DISASM_BLOCK_SIZE: usize = 64;

struct DisassemblyView {
    cached_lines: Option<Vec<scpu::disassembler::DisasmLine>>,
    // scroll_offset: usize,
    options: scpu::disassembler::DisassemblyOptions,
    follow_pc: bool,
    current_addr: u32,
}

impl DisassemblyView {
    fn new(rom_mapping_mode: cartridge::MappingMode) -> Self {        
        Self {
            cached_lines: None,
            // scroll_offset: 0,
            options: scpu::disassembler::DisassemblyOptions {
                use_hw_reg_names: true,
                show_rel_addr_dest: true,
                max_instr_count: DISASM_BLOCK_SIZE,
                forced_flag_x: None,
                forced_flag_m: None,
                forced_e: None,
                rom_mapping_mode,
            },
            follow_pc: true,
            current_addr: 0,
        }
    }

    fn update(&mut self,
        core: &Snemulator<Debugger>,
        options: &scpu::disassembler::DisassemblyOptions,
    ) {
        if self.follow_pc {
            self.current_addr = (core.cpu.pb as u32) << 16 | core.cpu.pc as u32;
        }

        self.cached_lines = Some(scpu::disassembler::disassemble_forward(core, options, self.current_addr));
    }
}

struct RomEdit {
    /// Line index of the first instruction included in this edit.
    lowest_idx: usize,
    /// Line index of the last instruction included in this edit.
    highest_idx: usize,
    /// Line index (within the disasm view) of the currently selected instruction.
    current_line: usize,
    /// Flat index into `bytes_strs` of the byte currently being edited.
    current_byte: usize,
    /// Number of bytes per instruction, in order from lowest_idx to highest_idx.
    /// Used to map a flat byte index back to a line index for up/down navigation.
    line_byte_counts: Vec<usize>,
    /// User-typed edits for each byte, as uppercase hex strings. Empty means unchanged.
    /// At most 2 hex chars
    bytes_strs: Vec<String>,
    bytes_originals: Vec<String>,
    just_went_down: bool,
    just_went_right: bool,
}

impl RomEdit {
    /// Returns true if every non-empty byte string is a valid 1-2 digit hex value.
    /// Empty strings are valid (they mean "keep original").
    fn is_valid(&self) -> bool {
        self.bytes_strs.iter().all(|s| {
            s.is_empty() || (s.len() <= 2 && s.chars().all(|c| c.is_ascii_hexdigit()))
        })
    }

    /// Pad the current byte to two chars if it only has one, treating it as "0X".
    /// If the field is empty it is left empty — the original will be used on commit.
    fn pad_current(&mut self) {
        let s = &mut self.bytes_strs[self.current_byte];
        if s.len() == 1 {
            *s = format!("0{}", s);
        }
    }

    /// Returns the effective hex string for `flat_i`: the user's input if non-empty,
    /// otherwise the original value.
    fn effective_str(&self, flat_i: usize) -> &str {
        let typed = &self.bytes_strs[flat_i];
        if typed.is_empty() {
            &self.bytes_originals[flat_i]
        } else {
            typed
        }
    }

    /// Returns the flat byte index of the first byte belonging to `line_idx`.
    /// `line_idx` must be within [lowest_idx, highest_idx].
    fn byte_offset_for_line(&self, line_idx: usize) -> usize {
        let relative = line_idx - self.lowest_idx;
        self.line_byte_counts[..relative].iter().sum()
    }

    /// Returns the line index (within the disasm view) that owns `byte_idx`.
    fn line_for_byte(&self, byte_idx: usize) -> usize {
        let mut remaining = byte_idx;
        for (i, &count) in self.line_byte_counts.iter().enumerate() {
            if remaining < count {
                return self.lowest_idx + i;
            }
            remaining -= count;
        }
        // Shouldn't happen if invariants hold, but fall back to highest line.
        self.highest_idx
    }
}

pub struct CpuTab {
    disasm: DisassemblyView,
    bp_input: String,
    rom_changes: std::collections::HashMap<u32, u8>,
    rom_edit: Option<RomEdit>,
}

impl CpuTab {
    pub fn new(rom_mapping_mode: cartridge::MappingMode) -> Self {
        Self {
            disasm: DisassemblyView::new(rom_mapping_mode),
            bp_input: String::new(),
            rom_changes: std::collections::HashMap::new(),
            rom_edit: None,
        }
    }
    
    pub fn breakpoint_hit(&mut self, addr: u32) {
        self.disasm.current_addr = addr;
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, core: &mut Snemulator<Debugger>, jump_to_bps_on_hit: &mut bool) {
        self.update_disasm(core);
        
        let pc = (core.cpu.pb as u32) << 16 | core.cpu.pc as u32;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.disasm.options.use_hw_reg_names, "Use HW Reg Names");
                
                ui.checkbox(&mut self.disasm.options.show_rel_addr_dest, "Show Branch Dest Addr");

                ui.checkbox(&mut self.disasm.follow_pc, "Follow PC");

                if ui.button("Go to PC").clicked() {
                    self.disasm.current_addr = pc;
                    self.disasm.options.forced_flag_x = None;
                    self.disasm.options.forced_flag_m = None;
                    self.disasm.options.forced_e = None;
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("cpu_mode_sel")
                    .selected_text(
                        match self.disasm.options.forced_e {
                            Some(true) => "Emulation",
                            Some(false) => "Native",
                            None => if core.cpu.e { "Emulation" } else { "Native" },
                        })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.disasm.options.forced_e, Some(true), "Emulation");
                        ui.selectable_value(&mut self.disasm.options.forced_e, Some(false), "Native");
                        ui.selectable_value(&mut self.disasm.options.forced_e, None, "Current in Program");
                    });

                let (m_text, x_text, mx_en) = match self.disasm.options.forced_e {
                    Some(true) => {
                        ("m8", "x8", false)
                    }
                    None if core.cpu.e => {
                        ("m8", "x8", false)
                    }
                    _ => {
                        let m_text = match self.disasm.options.forced_flag_m {
                            Some(true) => "m8",
                            Some(false) => "m16",
                            None => if core.cpu.is_flag_set(scpu::Flag::FlagM) { "m8" } else { "m16" },
                        };
                        let x_text = match self.disasm.options.forced_flag_x {
                            Some(true) => "x8",
                            Some(false) => "x16",
                            None => if core.cpu.is_flag_set(scpu::Flag::FlagX) { "x8" } else { "x16" },
                        };
                        (m_text, x_text, true)
                    }
                };

                ui.add_enabled_ui(mx_en, |ui| {
                    egui::ComboBox::from_id_salt("m_flag_sel")
                        .selected_text(m_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.disasm.options.forced_flag_m, Some(true), "m8");
                            ui.selectable_value(&mut self.disasm.options.forced_flag_m, Some(false), "m16");
                            ui.selectable_value(&mut self.disasm.options.forced_flag_m, None, "Current in Program");
                        });
    
                    egui::ComboBox::from_id_salt("x_flag_sel")
                        .selected_text(x_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.disasm.options.forced_flag_x, Some(true), "x8");
                            ui.selectable_value(&mut self.disasm.options.forced_flag_x, Some(false), "x16");
                            ui.selectable_value(&mut self.disasm.options.forced_flag_x, None, "Current in Program");
                        });
                });
                
                ui.add_enabled_ui(!self.rom_changes.is_empty(), |ui| {
                    if ui.button("Reset ROM Data").clicked() {
                        let cart = core.cart.as_mut().unwrap();
                        
                        for (addr, value) in self.rom_changes.iter() {
                            cart.force_write(scpu::Address::from_u32(*addr), *value);
                        }
                        
                        self.rom_changes.clear();
                    }
                });
            });
        });
        ui.separator();

        let available_height = ui.available_height();
        
        ui.horizontal(|ui| {
            self.disasm_section(ui, core, available_height);

            ui.vertical(|ui| {
                self.cpu_state_section(ui, core);
                
                ui.add_space(10.0);
                
                self.breakpoints_section(ui, core, jump_to_bps_on_hit);
            });
        });
    }
    
    fn disasm_section(&mut self, ui: &mut egui::Ui, core: &mut Snemulator<Debugger>, available_height: f32) {
        ui.vertical(|ui| {
            egui::ScrollArea::vertical().id_salt("disasm_scroll").min_scrolled_height(available_height).show(ui, |ui| {
                let lines = self.disasm.cached_lines.take();
                
                if let Some(lines) = lines {
                    let line_count = lines.len();
                    
                    if let Some(re) = &mut self.rom_edit {
                        re.just_went_down = false;
                        re.just_went_right = false;
                    }
                    
                    for (i, line) in lines.iter().enumerate() {
                        ui.horizontal(|ui| {
                            match &self.rom_edit {
                                Some(rom_edit) if rom_edit.lowest_idx <= i && i <= rom_edit.highest_idx => {
                                    self.disasm_line_editable(ui, core, line, i, line_count);
                                }
                                _ => {
                                    self.disasm_line(ui, core, line, i);
                                }
                            }
                            
                            ui.add_space(10.0);
                        });
                    }
                } else {
                    ui.label("No disassembly available");
                }
            });
        });
    }
    
    fn disasm_line(&mut self, ui: &mut egui::Ui, core: &mut Snemulator<Debugger>, line: &DisasmLine, line_idx: usize) {
        let pc = (core.cpu.pb as u32) << 16 | core.cpu.pc as u32;
        
        let is_pc  = line.addr == pc;
        let has_bp = core.probe.as_ref().unwrap().breakpoints.contains(&BreakpointInfo::new(line.addr));
        let addr   = line.addr;
        
        let dot = if has_bp { "🔴" } else { "⚪" };
        if ui.small_button(dot).clicked() {
            if has_bp {
                core.probe.as_mut().unwrap().breakpoints.remove(&BreakpointInfo::new(addr));
            }
            else {
                self.add_breakpoint(addr, core);
            }
        }
        
        ui.label(" ");
        
        let style = egui::Style::default();
        let mut disasm_str = egui::text::LayoutJob::default();
        
        let bg_col = if self.rom_changes.contains_key(&line.addr) {
            egui::Color32::DARK_RED
        } else {    
            ui.visuals().window_fill
        };
        
        let addr_text_col = if has_bp {
            egui::Color32::RED
        } else if is_pc {
            egui::Color32::YELLOW
        } else {
            ui.visuals().text_color()
        };
        
        let addr_text = egui::RichText::new(format!("{:06X} ", line.addr))
            .monospace()
            .background_color(bg_col)
            .color(addr_text_col);
        
        let disasm_col = if has_bp && addr == self.disasm.current_addr {
            egui::Color32::RED
        } else if is_pc {
            egui::Color32::YELLOW
        } else {
            ui.visuals().text_color()
        };
        
        let bytes_str: String = line.bytes.iter().map(|b| format!("{:02X} ", b)).collect();
        let bytes_text = egui::RichText::new(format!("{:<12}", bytes_str))
            .monospace()
            .background_color(bg_col)
            .color(disasm_col)
            .weak();
        
        let instr_text = egui::RichText::new(&line.disasm_str)
            .monospace()
            .background_color(bg_col)
            .color(disasm_col);
        
        addr_text.append_to(&mut disasm_str, &style, egui::FontSelection::Default, egui::Align::Center);
        bytes_text.append_to(&mut disasm_str, &style, egui::FontSelection::Default, egui::Align::Center);
        instr_text.append_to(&mut disasm_str, &style, egui::FontSelection::Default, egui::Align::Center);
        
        ui.label(disasm_str)
            .context_menu(|ui| self.disasm_context_menu(ui, core, line_idx, &line));
    }
    
    fn disasm_line_editable(&mut self, ui: &mut egui::Ui, core: &mut Snemulator<Debugger>, line: &DisasmLine, line_idx: usize, total_lines: usize) {
        let pc = (core.cpu.pb as u32) << 16 | core.cpu.pc as u32;
        
        let is_pc  = line.addr == pc;
        let has_bp = core.probe.as_ref().unwrap().breakpoints.contains(&BreakpointInfo::new(line.addr));
        let addr   = line.addr;
        
        // Breakpoint toggle dot — same as normal line.
        let dot = if has_bp { "🔴" } else { "⚪" };
        if ui.small_button(dot).clicked() {
            if has_bp {
                core.probe.as_mut().unwrap().breakpoints.remove(&BreakpointInfo::new(addr));
            } else {
                self.add_breakpoint(addr, core);
            }
        }

        ui.label(" ");

        let addr_text_col = if has_bp {
            egui::Color32::RED
        } else if is_pc {
            egui::Color32::YELLOW
        } else {
            ui.visuals().text_color()
        };

        ui.label(
            egui::RichText::new(format!("{:06X} ", line.addr))
                .monospace()
                .color(addr_text_col)
        );

        let (byte_offset, byte_count, current_byte, current_line) = {
            let re = self.rom_edit.as_ref().unwrap();
            let offset = re.byte_offset_for_line(line_idx);
            let count  = re.line_byte_counts[line_idx - re.lowest_idx];
            (offset, count, re.current_byte, re.current_line)
        };

        // Collect key events before rendering widgets so we can act on them
        // after the focus check below.
        let press_enter  = ui.input(|i| i.key_pressed(egui::Key::Enter));
        let press_escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
        let press_left   = ui.input(|i| i.key_pressed(egui::Key::ArrowLeft));
        let press_right  = ui.input(|i| i.key_pressed(egui::Key::ArrowRight));
        let press_up     = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
        let press_down   = ui.input(|i| i.key_pressed(egui::Key::ArrowDown));

        // Render each byte in this instruction as a small TextEdit.
        let mut any_focused = false;
        for byte_i in 0..byte_count {
            let flat_i = byte_offset + byte_i;
            let is_current = flat_i == current_byte;

            // Background: bright gold for the active byte, muted blue for others.
            let bg = if is_current {
                egui::Color32::from_rgb(180, 140, 0)
            } else {
                egui::Color32::from_rgb(30, 50, 100)
            };
            let fg = if is_current {
                egui::Color32::BLACK
            } else {
                egui::Color32::LIGHT_GRAY
            };

            // A non-empty byte string that isn't valid hex is an error.
            let byte_str = &self.rom_edit.as_ref().unwrap().bytes_strs[flat_i];
            let byte_invalid = !byte_str.is_empty()
                && (byte_str.len() > 2 || !byte_str.chars().all(|c| c.is_ascii_hexdigit()));

            let actual_bg = if byte_invalid {
                egui::Color32::DARK_RED
            } else {
                bg
            };

            // The hint text shows the original value so the user knows what they
            // are replacing without having to manually clear a pre-filled field.
            let hint = self.rom_edit.as_ref().unwrap().bytes_originals[flat_i].clone();

            let widget_id = ui.id().with(("rom_edit_byte", flat_i));

            let te = egui::TextEdit::singleline(
                    &mut self.rom_edit.as_mut().unwrap().bytes_strs[flat_i]
                )
                .id(widget_id)
                .char_limit(2)
                .desired_width(22.0)
                .font(egui::TextStyle::Monospace)
                .text_color(fg)
                .background_color(actual_bg)
                .hint_text(hint);

            let resp = ui.add(te);

            // Request focus on the current byte every frame so arrow-key navigation
            // feels immediate without requiring a mouse click.
            if is_current {
                resp.request_focus();
            }

            if resp.has_focus() {
                any_focused = true;
            }

            // Filter non-hex characters and auto-advance only when the content
            // actually changed this frame, so empty fields never spuriously advance.
            if resp.changed() {
                {
                    let s = &mut self.rom_edit.as_mut().unwrap().bytes_strs[flat_i];
                    s.retain(|c| c.is_ascii_hexdigit());
                    *s = s.to_ascii_uppercase();
                    if s.len() > 2 {
                        s.truncate(2);
                    }
                }

                // Auto-advance when the user has just typed the second character.
                if is_current {
                    let len = self.rom_edit.as_ref().unwrap().bytes_strs[flat_i].len();
                    if len == 2 {
                        let re = self.rom_edit.as_mut().unwrap();
                        let total_bytes = re.bytes_strs.len();
                        if re.current_byte + 1 < total_bytes {
                            re.current_byte += 1;
                            // If the new current_byte is in the next line, advance current_line too.
                            re.current_line = re.line_for_byte(re.current_byte);
                        }
                    }
                }
            }
        }

        // ── Instruction mnemonic (read-only label, same as normal line) ──────
        let disasm_col = if is_pc { egui::Color32::YELLOW } else { ui.visuals().text_color() };
        ui.label(
            egui::RichText::new(&line.disasm_str)
                .monospace()
                .color(disasm_col)
        );

        // ── Commit / cancel / navigation ─────────────────────────────────────
        // Only process keys when a byte in this line is focused, so that key
        // events don't fire multiple times (once per editable line rendered).
        if !any_focused {
            return;
        }

        if press_escape {
            self.rom_edit = None;
            return;
        }

        if press_enter {
            let re = self.rom_edit.as_ref().unwrap();
            if re.is_valid() {
                // Build the list of (rom_address, new_byte) pairs to write.
                // Derive the range's start address from the current line's address
                // and how many bytes of earlier instructions are in the edit.
                let writes: Vec<(u32, u8)> = {
                    let cur_line_relative = line_idx - re.lowest_idx;
                    let bytes_before_cur: usize = re.line_byte_counts[..cur_line_relative].iter().sum();
                    let range_start_addr = line.addr - bytes_before_cur as u32;

                    (0..re.bytes_strs.len())
                        .map(|flat_i| {
                            let rom_addr = range_start_addr + flat_i as u32;
                            // Use the typed value if present, otherwise the original.
                            let new_val = u8::from_str_radix(re.effective_str(flat_i), 16).unwrap();
                            (rom_addr, new_val)
                        })
                        .collect()
                };

                let cart = core.cart.as_mut().unwrap();
                for (rom_addr, new_val) in writes {
                    // Record the original value before overwriting (if not already tracked).
                    self.rom_changes.entry(rom_addr).or_insert_with(|| {
                        cart.read(scpu::Address::from_u32(rom_addr))
                    });
                    cart.force_write(scpu::Address::from_u32(rom_addr), new_val);
                }

                self.rom_edit = None;
            }
            // If not valid, silently block commit — the red highlights show the problem.
            return;
        }

        // ── Arrow key navigation ──────────────────────────────────────────────
        let re = self.rom_edit.as_mut().unwrap();

        if press_left {
            re.pad_current();
            if re.current_byte > 0 {
                re.current_byte -= 1;
                re.current_line = re.line_for_byte(re.current_byte);
            }
        } else if press_right && !re.just_went_right {
            re.just_went_right = true;
            
            re.pad_current();
            let total = re.bytes_strs.len();
            if re.current_byte + 1 < total {
                re.current_byte += 1;
                re.current_line = re.line_for_byte(re.current_byte);
            }
        } else if press_up {
            re.pad_current();
            if re.current_line > re.lowest_idx {
                // Move to the first byte of the previous line within the current range.
                re.current_line -= 1;
                re.current_byte = re.byte_offset_for_line(re.current_line);
            } else if re.lowest_idx > 0 {
                // Already on the lowest line — expand the range upward.
                let new_line_idx = re.lowest_idx - 1;

                let cur_line_relative = line_idx - re.lowest_idx;
                let bytes_before_cur: usize = re.line_byte_counts[..cur_line_relative].iter().sum();
                let range_start_addr = line.addr - bytes_before_cur as u32;

                // Find the preceding instruction by trying lengths 1-4 and checking
                // whether the disassembler agrees on that length.
                let (prev_originals, prev_count) = {
                    let mut found: Option<Vec<u8>> = None;
                    for len in 1usize..=4 {
                        let candidate_addr = range_start_addr.wrapping_sub(len as u32);
                        let disasm_result = scpu::disassembler::disassemble_forward(
                            core,
                            &self.disasm.options,
                            candidate_addr,
                        );
                        if let Some(first) = disasm_result.first() {
                            if first.bytes.len() == len {
                                found = Some(first.bytes.clone());
                                break;
                            }
                        }
                    }
                    let bytes = found.unwrap_or_else(|| {
                        vec![core.cart.as_ref().unwrap()
                            .read(scpu::Address::from_u32(range_start_addr - 1))]
                    });
                    let count = bytes.len();
                    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
                    (strs, count)
                };

                // Prepend the new instruction's originals and empty typed strings.
                let mut new_originals = prev_originals;
                new_originals.extend(re.bytes_originals.drain(..));
                re.bytes_originals = new_originals;

                let new_typed: Vec<String> = std::iter::repeat_with(String::new)
                    .take(prev_count)
                    .chain(re.bytes_strs.drain(..))
                    .collect();
                re.bytes_strs = new_typed;

                re.line_byte_counts.insert(0, prev_count);
                re.lowest_idx = new_line_idx;
                re.current_line = new_line_idx;
                // Move cursor to the first byte of the newly added instruction.
                re.current_byte = 0;
            }
        } else if press_down && !re.just_went_down {
            re.just_went_down = true;
            
            re.pad_current();
            if re.current_line < re.highest_idx {
                // Move to the first byte of the next line within the current range.
                re.current_line += 1;
                re.current_byte = re.byte_offset_for_line(re.current_line);
            } else if re.highest_idx + 1 < total_lines {
                // Already on the highest line — expand the range downward.
                let cur_line_relative = line_idx - re.lowest_idx;
                let bytes_before_cur: usize = re.line_byte_counts[..cur_line_relative].iter().sum();
                let range_start_addr = line.addr - bytes_before_cur as u32;
                let total_bytes_in_range: usize = re.line_byte_counts.iter().sum();
                let next_instr_addr = range_start_addr + total_bytes_in_range as u32;

                let (next_originals, next_count) = {
                    let disasm_result = scpu::disassembler::disassemble_forward(
                        core,
                        &self.disasm.options,
                        next_instr_addr,
                    );
                    let bytes = if let Some(first) = disasm_result.first() {
                        first.bytes.clone()
                    } else {
                        vec![core.cart.as_ref().unwrap()
                            .read(scpu::Address::from_u32(next_instr_addr))]
                    };
                    let count = bytes.len();
                    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
                    (strs, count)
                };

                let first_new_byte_idx = re.bytes_strs.len();
                re.line_byte_counts.push(next_count);
                // Append empty typed strings and the originals for the new instruction.
                re.bytes_strs.extend(std::iter::repeat_with(String::new).take(next_count));
                re.bytes_originals.extend(next_originals);
                re.highest_idx += 1;
                re.current_line = re.highest_idx;
                // Move cursor to the first byte of the new instruction.
                re.current_byte = first_new_byte_idx;
            }
        }
    }
    
    fn disasm_context_menu(&mut self, ui: &mut egui::Ui, core: &mut Snemulator<Debugger>, line_idx: usize, line: &DisasmLine) {
        const NOP: u8 = 0xEA;
        
        let is_changed = self.rom_changes.contains_key(&line.addr);
        
        if is_changed {
            if ui.button("Revert").clicked() {
                let mut earliest_in_changed = line.addr;
                while self.rom_changes.contains_key(&(earliest_in_changed - 1)) {
                    earliest_in_changed -= 1;
                }
                
                let cart = core.cart.as_mut().unwrap();
                
                let mut addr = earliest_in_changed;
                while self.rom_changes.contains_key(&addr) {
                    cart.force_write(scpu::Address::from_u32(addr), self.rom_changes[&addr]);
                    self.rom_changes.remove(&addr);
                    addr += 1;
                }
                
                ui.close();
            }
        } else {
            if ui.button("Replace with NOPs").clicked() {
                let cart = core.cart.as_mut().unwrap();
    
                for i in 0..line.bytes.len() {
                    let addr = line.addr + i as u32;
                    self.rom_changes.insert(addr, cart.read(scpu::Address::from_u32(addr)));
                    cart.force_write(scpu::Address::from_u32(addr), NOP);
                }
                
                ui.close();
            }
        }
        
        if ui.button("Edit Bytes").clicked() {
            let originals: Vec<String> = line.bytes.iter().map(|b| format!("{:02X}", b)).collect();
            self.rom_edit = Some(RomEdit {
                lowest_idx: line_idx,
                highest_idx: line_idx,
                current_line: line_idx,
                current_byte: 0,
                line_byte_counts: vec![line.bytes.len()],
                bytes_strs: vec![String::new(); line.bytes.len()],
                bytes_originals: originals,
                just_went_down: false,
                just_went_right: false,
            });
            ui.close();
        }
    }
    
    fn cpu_state_section(&mut self, ui: &mut egui::Ui, core: &Snemulator<Debugger>) {
        ui.heading("CPU State");
        
        ui.separator();
        
        ui.horizontal(|ui| {
            let pb_text = egui::RichText::new(format!("PB: {:02X}", core.cpu.pb)).monospace();
            ui.label(pb_text);
            
            let pc_text = egui::RichText::new(format!("PC: {:04X}", core.cpu.pc)).monospace();
            ui.label(pc_text);
            
            let sp_text = egui::RichText::new(format!("SP: {:04X}", core.cpu.sp)).monospace();
            ui.label(sp_text);
            
            let db_text = egui::RichText::new(format!("DB: {:02X}", core.cpu.db)).monospace();
            ui.label(db_text);
            
            let dp_text = egui::RichText::new(format!("DP: {:04X}", core.cpu.dp)).monospace();
            ui.label(dp_text);
        });
        
        ui.horizontal(|ui| {
            let a_text = egui::RichText::new(format!("A: {:04X}", core.cpu.a)).monospace();
            ui.label(a_text);
            
            let x_text = egui::RichText::new(format!("X: {:04X}", core.cpu.x)).monospace();
            ui.label(x_text);
            
            let y_text = egui::RichText::new(format!("Y: {:04X}", core.cpu.y)).monospace();
            ui.label(y_text);
            
            let style = egui::Style::default();
            let mut status_str = egui::text::LayoutJob::default();
            
            let flag_col = |flag| if core.cpu.is_flag_set(flag) { egui::Color32::GREEN } else { egui::Color32::RED };
            
            let p_text = egui::RichText::new("P: ").color(ui.visuals().text_color()).monospace();
            let n_text = egui::RichText::new("N").color(flag_col(scpu::Flag::FlagN)).monospace();
            let v_text = egui::RichText::new("V").color(flag_col(scpu::Flag::FlagV)).monospace();
            let m_text = egui::RichText::new("M").color(flag_col(scpu::Flag::FlagM)).monospace();
            let x_text = egui::RichText::new("X").color(flag_col(scpu::Flag::FlagX)).monospace();
            let d_text = egui::RichText::new("D").color(flag_col(scpu::Flag::FlagD)).monospace();
            let i_text = egui::RichText::new("I").color(flag_col(scpu::Flag::FlagI)).monospace();
            let z_text = egui::RichText::new("Z").color(flag_col(scpu::Flag::FlagZ)).monospace();
            let c_text = egui::RichText::new("C").color(flag_col(scpu::Flag::FlagC)).monospace();
            
            p_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            n_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            v_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            m_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            x_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            d_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            i_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            z_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            c_text.append_to(&mut status_str, &style, egui::FontSelection::Default, egui::Align::Center);
            
            ui.label(status_str);
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            let mut halted = core.cpu.halted;
            let mut stopped = core.cpu.stopped;
            let mut waiting_for_interrupt = core.cpu.waiting_for_interrupt;
            
            ui.add_enabled(false,
                egui::Checkbox::new(&mut halted, "Halted")
            );
            ui.add_enabled(false,
                egui::Checkbox::new(&mut stopped, "Stopped")
            );
            ui.add_enabled(false,
                egui::Checkbox::new(&mut waiting_for_interrupt, "Waiting for Interrupt")
            );
        });
        
        ui.horizontal(|ui| {
            let mut irq_pending = core.cpu.irq_pending;
            let mut nmi_pending = core.cpu.nmi_pending;
            
            ui.add_enabled(false,
                egui::Checkbox::new(&mut irq_pending, "IRQ Pending")
            );
            ui.add_enabled(false,
                egui::Checkbox::new(&mut nmi_pending, "NMI Pending")
            );
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label(monospace_text("(APU→CPU)".to_string()));
            ui.label(monospace_text(format!("APUIO0: {:02X}", core.apu_ports.apuio0)));
            ui.label(monospace_text(format!("APUIO1: {:02X}", core.apu_ports.apuio1)));
            ui.label(monospace_text(format!("APUIO2: {:02X}", core.apu_ports.apuio2)));
            ui.label(monospace_text(format!("APUIO3: {:02X}", core.apu_ports.apuio3)));
        });
        
        ui.horizontal(|ui| {
            ui.label(monospace_text("(CPU→APU)".to_string()));
            ui.label(monospace_text(format!("CPUIO0: {:02X}", core.apu_ports.cpuio0)));
            ui.label(monospace_text(format!("CPUIO1: {:02X}", core.apu_ports.cpuio1)));
            ui.label(monospace_text(format!("CPUIO2: {:02X}", core.apu_ports.cpuio2)));
            ui.label(monospace_text(format!("CPUIO3: {:02X}", core.apu_ports.cpuio3)));
        });
    }
    
    fn breakpoints_section(&mut self, ui: &mut egui::Ui, core: &mut Snemulator<Debugger>, jump_to_bps_on_hit: &mut bool) {
        let breakpoints = &mut core.probe.as_mut().unwrap().breakpoints;
        
        ui.horizontal(|ui| {
            ui.heading("Breakpoints");
            
            if ui.button("Clear All").clicked() {
                breakpoints.clear();
            }
            
            ui.add_space(5.0);
            
            ui.checkbox(jump_to_bps_on_hit, "Show Breakpoints on Hit");
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Add:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.bp_input).hint_text("XXXXXX").char_limit(6)
            );
            let submitted = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui.button("Add").clicked() || submitted {
                if let Ok(addr) = u32::from_str_radix(self.bp_input.trim(), 16) {
                    self.add_breakpoint(addr, core);
                    self.bp_input.clear();
                }
            }
        });
        ui.separator();
        
        let breakpoints = &mut core.probe.as_mut().unwrap().breakpoints; // To appease borrow checker

        if breakpoints.is_empty() {
            ui.label("No breakpoints set.");
            return;
        }

        let mut to_remove: Option<&BreakpointInfo> = None;
        let mut sorted: Vec<BreakpointInfo> = breakpoints.iter().copied().collect();
        sorted.sort_unstable_by_key(|bp| bp.addr);

        egui::ScrollArea::vertical().id_salt("bp_scroll").show(ui, |ui| {
            for group in sorted.chunks(5) {
                ui.horizontal(|ui| {
                    for breakpoint in group {
                        ui.horizontal(|ui| {
                            if ui.small_button("❌").clicked() {
                                to_remove = Some(breakpoint);
                            }
                            // Clicking the address jumps the disassembly view to it
                            if ui.button(egui::RichText::new(format!("{:06X}", breakpoint.addr)).monospace()).clicked() {
                                let pc = ((core.cpu.pb as u32) << 16) | core.cpu.pc as u32;
                                self.disasm.follow_pc = breakpoint.addr == pc;
                                self.disasm.current_addr = breakpoint.addr;
                                self.disasm.options.forced_flag_m = Some(breakpoint.force_m);
                                self.disasm.options.forced_flag_x = Some(breakpoint.force_x);
                                self.disasm.options.forced_e = Some(breakpoint.force_e);
                            }
                        });
                    }
                });
            }
            
        });

        if let Some(breakpoint) = to_remove {
            core.probe.as_mut().unwrap().breakpoints.remove(breakpoint);
        }
    }
    
    fn update_disasm(&mut self, core: &Snemulator<Debugger>) {
        let options = self.disasm.options.clone();

        self.disasm.update(core, &options);
    }

    fn add_breakpoint(&mut self, addr: u32, core: &mut Snemulator<Debugger>) {
        let mut breakpoint = BreakpointInfo::new(addr);
        breakpoint.force_x = match self.disasm.options.forced_flag_x {
            Some(v) => v,
            None => core.cpu.is_flag_set(scpu::Flag::FlagX)
        };
        breakpoint.force_m = match self.disasm.options.forced_flag_m {
            Some(v) => v,
            None => core.cpu.is_flag_set(scpu::Flag::FlagM)
        };
        breakpoint.force_e = match self.disasm.options.forced_e {
            Some(v) => v,
            None => core.cpu.e
        };

        core.probe.as_mut().unwrap().breakpoints.insert(breakpoint);
    }
}