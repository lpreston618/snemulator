use mlua::{UserData, Value};
use snemcore::{Snemulator, probe::DebugProbe, scpu::{self, dma::Direction}, sppu::TileSize};

pub struct SnemulatorInterface<P: DebugProbe> {
    core: *const Snemulator<P>
}

pub struct MetaInterface<P: DebugProbe> {
    core: *const Snemulator<P>
}

pub struct CpuInterface<P: DebugProbe> {
    core: *const Snemulator<P>
}

pub struct PpuInterface<P: DebugProbe> {
    core: *const Snemulator<P>
}

pub struct DmaInterface<P: DebugProbe> {
    core: *const Snemulator<P>,
    channel: usize,
}

impl<P: DebugProbe> SnemulatorInterface<P> {
    pub fn new(core: &Snemulator<P>) -> Self {
        Self { core }
    }
}

impl<P: DebugProbe + 'static> UserData for SnemulatorInterface<P> {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("meta", |lua, this| {
            lua.create_userdata(MetaInterface { core: this.core })
        });
        
        fields.add_field_method_get("cpu", |lua, this| {
            lua.create_userdata(CpuInterface { core: this.core })
        });
        
        fields.add_field_method_get("ppu", |lua, this| {
            lua.create_userdata(PpuInterface { core: this.core })
        });
        
        fields.add_field_method_get("dma", |lua, this| {
            let dma_table = lua.create_table()?;
            let dma_meta = lua.create_table()?;
            
            let core = this.core;
            
            dma_meta.set("__index", lua.create_function(move |lua, (_table, channel): (mlua::Table, usize)| {
                if channel > 7 {
                    Ok(Value::Nil)
                } else {
                    let dma_interface = lua.create_userdata(DmaInterface { 
                        core,
                        channel,
                    })?;
                    Ok(Value::UserData(dma_interface))
                }
            })?)?;
            
            dma_table.set_metatable(Some(dma_meta));
            
            Ok(dma_table)
        });
    }
}

impl<P: DebugProbe> UserData for MetaInterface<P> {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("frame", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.frame)
        });
        fields.add_field_method_get("rom_size", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cart.as_ref().unwrap().rom_slice().len())
        });
    }
}

impl<P: DebugProbe> UserData for CpuInterface<P> {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("pb", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.pb) 
        });
        fields.add_field_method_get("db", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.db) 
        });
        fields.add_field_method_get("p", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.p) 
        });
        fields.add_field_method_get("apuio0", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.apu_ports.cpuio0) 
        });
        fields.add_field_method_get("apuio1", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.apu_ports.cpuio1) 
        });
        fields.add_field_method_get("apuio2", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.apu_ports.cpuio2) 
        });
        fields.add_field_method_get("apuio3", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.apu_ports.cpuio3) 
        });
        fields.add_field_method_get("prg0", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu_read_mem(scpu::Address{ bank: core.cpu.pb, offset: core.cpu.pc + 0 }))
        });
        fields.add_field_method_get("prg1", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu_read_mem(scpu::Address{ bank: core.cpu.pb, offset: core.cpu.pc + 1 }))
        });
        fields.add_field_method_get("prg2", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu_read_mem(scpu::Address{ bank: core.cpu.pb, offset: core.cpu.pc + 2 }))
        });
        fields.add_field_method_get("a", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.a) 
        });
        fields.add_field_method_get("x", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.x) 
        });
        fields.add_field_method_get("y", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.y) 
        });
        fields.add_field_method_get("sp", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.sp) 
        });
        fields.add_field_method_get("pc", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.pc) 
        });
        fields.add_field_method_get("dp", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.dp) 
        });
        fields.add_field_method_get("flagc", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.is_flag_set(scpu::Flag::FlagC)) 
        });
        fields.add_field_method_get("flagz", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.is_flag_set(scpu::Flag::FlagZ)) 
        });
        fields.add_field_method_get("flagi", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.is_flag_set(scpu::Flag::FlagI)) 
        });
        fields.add_field_method_get("flagd", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.is_flag_set(scpu::Flag::FlagD)) 
        });
        fields.add_field_method_get("flagx", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.is_flag_set(scpu::Flag::FlagX)) 
        });
        fields.add_field_method_get("flagm", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.is_flag_set(scpu::Flag::FlagM)) 
        });
        fields.add_field_method_get("flagv", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.is_flag_set(scpu::Flag::FlagV)) 
        });
        fields.add_field_method_get("flagn", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.is_flag_set(scpu::Flag::FlagN)) 
        });
        fields.add_field_method_get("e", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.e) 
        });
        fields.add_field_method_get("halted", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.halted) 
        });
        fields.add_field_method_get("stopped", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.stopped) 
        });
        fields.add_field_method_get("nmi_pending", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.nmi_pending) 
        });
        fields.add_field_method_get("irq_pending", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.irq_pending) 
        });
        fields.add_field_method_get("waiting", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu.waiting_for_interrupt) 
        });
        fields.add_field_method_get("full_pc", |_, this| {
            let core = unsafe { &*this.core };
            Ok(scpu::Address {
                bank: core.cpu.pb,
                offset: core.cpu.pc,
            }.to_u32())
        });
    }
}

impl<P: DebugProbe> UserData for PpuInterface<P> {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("screen_brightness", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.screen_brightness)
        });
        fields.add_field_method_get("obj_size", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.obj_sprite_size as u8)
        });
        fields.add_field_method_get("bg_mode", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_mode as u8)
        });
        fields.add_field_method_get("mosaic_size", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.mosaic_size)
        });
        fields.add_field_method_get("cgram_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.cgram_addr)
        });
        fields.add_field_method_get("window1_left", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.w1_left_pos)
        });
        fields.add_field_method_get("window1_right", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.w1_right_pos)
        });
        fields.add_field_method_get("window2_left", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.w2_left_pos)
        });
        fields.add_field_method_get("window2_right", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.w2_right_pos)
        });
        
        fields.add_field_method_get("name_base_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.name_base_addr)
        });
        fields.add_field_method_get("name_secondary_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.name_secondary_base_addr)
        });
        fields.add_field_method_get("oam_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.internal_oam_addr)
        });
        fields.add_field_method_get("bg1_tilemap_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[0].tilemap_base_addr)
        });
        fields.add_field_method_get("bg2_tilemap_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[1].tilemap_base_addr)
        });
        fields.add_field_method_get("bg3_tilemap_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[2].tilemap_base_addr)
        });
        fields.add_field_method_get("bg4_tilemap_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[3].tilemap_base_addr)
        });
        fields.add_field_method_get("bg1_hofs", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[0].scroll_x)
        });
        fields.add_field_method_get("bg1_vofs", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[0].scroll_y)
        });
        fields.add_field_method_get("bg2_hofs", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[1].scroll_x)
        });
        fields.add_field_method_get("bg2_vofs", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[1].scroll_y)
        });
        fields.add_field_method_get("bg3_hofs", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[2].scroll_x)
        });
        fields.add_field_method_get("bg3_vofs", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[2].scroll_y)
        });
        fields.add_field_method_get("bg4_hofs", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[3].scroll_x)
        });
        fields.add_field_method_get("bg4_vofs", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[3].scroll_y)
        });
        fields.add_field_method_get("m7_hofs", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.m7_scroll_x)
        });
        fields.add_field_method_get("m7_vofs", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.m7_scroll_y)
        });
        fields.add_field_method_get("vram_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.vram_addr)
        });
        fields.add_field_method_get("m7_a", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.m7_matrix_a)
        });
        fields.add_field_method_get("m7_b", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.m7_matrix_b)
        });
        fields.add_field_method_get("m7_c", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.m7_matrix_c)
        });
        fields.add_field_method_get("m7_d", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.m7_matrix_d)
        });
        fields.add_field_method_get("m7_x", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.m7_center_x)
        });
        fields.add_field_method_get("m7_y", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.m7_center_y)
        });
        fields.add_field_method_get("h_counter", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.h_counter)
        });
        fields.add_field_method_get("v_counter", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.v_counter)
        });
        
        fields.add_field_method_get("priority_rotation", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.priority_rotation)
        });
        fields.add_field_method_get("bg1_large_tiles", |_, this| {
            let core = unsafe { &*this.core };
            Ok(matches!(core.ppu_regs.bg_settings[0].chr_size, TileSize::Size16x16))
        });
        fields.add_field_method_get("bg2_large_tiles", |_, this| {
            let core = unsafe { &*this.core };
            Ok(matches!(core.ppu_regs.bg_settings[1].chr_size, TileSize::Size16x16))
        });
        fields.add_field_method_get("bg3_large_tiles", |_, this| {
            let core = unsafe { &*this.core };
            Ok(matches!(core.ppu_regs.bg_settings[2].chr_size, TileSize::Size16x16))
        });
        fields.add_field_method_get("bg4_large_tiles", |_, this| {
            let core = unsafe { &*this.core };
            Ok(matches!(core.ppu_regs.bg_settings[3].chr_size, TileSize::Size16x16))
        });
        fields.add_field_method_get("bg3_mode1_priority", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg3_mode1_priority)
        });
        fields.add_field_method_get("bg1_mosaic_enable", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[0].mosaic_en)
        });
        fields.add_field_method_get("bg2_mosaic_enable", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[1].mosaic_en)
        });
        fields.add_field_method_get("bg3_mosaic_enable", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[2].mosaic_en)
        });
        fields.add_field_method_get("bg4_mosaic_enable", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[3].mosaic_en)
        });
        fields.add_field_method_get("bg1_main_enable", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[0].main_en)
        });
        fields.add_field_method_get("bg2_main_enable", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[1].main_en)
        });
        fields.add_field_method_get("bg3_main_enable", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[2].main_en)
        });
        fields.add_field_method_get("bg4_main_enable", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.bg_settings[3].main_en)
        });
        fields.add_field_method_get("obj_main_enable", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.obj_settings.main_en)
        });
    
        fields.add_field_method_get("dot", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu.dot)
        });
        fields.add_field_method_get("scanline", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu.scanline)
        });
        fields.add_field_method_get("screen_x", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu.x)
        });
        fields.add_field_method_get("screen_y", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu.y)
        });
        fields.add_field_method_get("multiply_result", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.multiply_result)
        });
        
        fields.add_field_method_get("f_blank", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.ppu_regs.in_fblank)
        });
        fields.add_field_method_get("v_blank", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu_regs.vblank_flag)
        });
        fields.add_field_method_get("h_blank", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu_regs.hblank_flag)
        });
        fields.add_field_method_get("v_blank_nmi", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu_regs.vblank_nmi_en)
        });
        fields.add_field_method_get("hv_timer_mode", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.cpu_regs.hv_timer_irq_mode as u8)
        });
    }
}

impl<P: DebugProbe> UserData for DmaInterface<P> {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("addr_inc_mode", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].inc_mode as u8)
        });
        fields.add_field_method_get("transfer_pattern", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].transfer_pattern as u8)
        });
        fields.add_field_method_get("a_bus_bank", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].a_bus_addr.bank)
        });
        fields.add_field_method_get("hdma_table_start_bank", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].a_bus_addr.bank)
        });
        fields.add_field_method_get("hdma_indirect_table_bank", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].hdma_indirect_table_addr.bank)
        });
        fields.add_field_method_get("hdma_scanline_counter", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].scanline_counter)
        });
        fields.add_field_method_get("unused_reg", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].unused)
        });

        fields.add_field_method_get("b_bus_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(0x2100 | core.dma_regs[this.channel].b_bus_addr as u16)
        });
        fields.add_field_method_get("a_bus_offset", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].a_bus_addr.offset)
        });
        fields.add_field_method_get("hdma_table_start_offset", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].a_bus_addr.offset)
        });
        fields.add_field_method_get("hdma_indirect_table_offset", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].hdma_indirect_table_addr.offset)
        });
        fields.add_field_method_get("hdma_table_offset", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].hdma_table_offset)
        });

        fields.add_field_method_get("b_to_a", |_, this| {
            let core = unsafe { &*this.core };
            Ok(matches!(core.dma_regs[this.channel].direction, Direction::BtoA))
        });
        fields.add_field_method_get("indirect_hdma", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].indirect_hdma)
        });
        fields.add_field_method_get("hdma_reload", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].hdma_reload_flag)
        });

        fields.add_field_method_get("full_a_bus_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].a_bus_addr.to_u32())
        });
        fields.add_field_method_get("full_hdma_table_start_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].a_bus_addr.to_u32())
        });
        fields.add_field_method_get("full_hdma_table_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok((core.dma_regs[this.channel].a_bus_addr.bank as u32) << 16 | core.dma_regs[this.channel].hdma_table_offset as u32)
        });
        fields.add_field_method_get("full_hdma_indirect_table_addr", |_, this| {
            let core = unsafe { &*this.core };
            Ok(core.dma_regs[this.channel].hdma_indirect_table_addr.to_u32())
        });
    }
}