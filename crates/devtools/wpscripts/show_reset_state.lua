require("app.src.debug.watchpoints.snemulator_api")

function OnInterrupt(kind)
    if kind == CONSTS.interrupts.RESET then
        -- Metadata
        Log(string.format("=== RESET INTERRUPT (Frame %d) ===", core.meta.frame))
        Log(string.format("ROM Size: %d bytes", core.meta.rom_size))

        -- CPU Registers
        Log("--- CPU ---")
        Log(string.format("PC: %04X  PB: %02X  DB: %02X  DP: %04X  SP: %04X",
            core.cpu.pc, core.cpu.pb, core.cpu.db, core.cpu.dp, core.cpu.sp))
        Log(string.format("Full PC: %06X", core.cpu.full_pc))
        Log(string.format("A: %04X  X: %04X  Y: %04X", core.cpu.a, core.cpu.x, core.cpu.y))
        Log(string.format("P: %02X  [N:%d V:%d M:%d X:%d D:%d I:%d Z:%d C:%d]  E:%d",
            core.cpu.p,
            core.cpu.flagn and 1 or 0,
            core.cpu.flagv and 1 or 0,
            core.cpu.flagm and 1 or 0,
            core.cpu.flagx and 1 or 0,
            core.cpu.flagd and 1 or 0,
            core.cpu.flagi and 1 or 0,
            core.cpu.flagz and 1 or 0,
            core.cpu.flagc and 1 or 0,
            core.cpu.e and 1 or 0))
        Log(string.format("PRG Bytes: %02X %02X %02X",
            core.cpu.prg0, core.cpu.prg1, core.cpu.prg2))
        Log(string.format("APU IO: %02X %02X %02X %02X",
            core.cpu.apuio0, core.cpu.apuio1, core.cpu.apuio2, core.cpu.apuio3))
        Log(string.format("Halted: %s  Stopped: %s  Waiting: %s  NMI: %s  IRQ: %s",
            tostring(core.cpu.halted),
            tostring(core.cpu.stopped),
            tostring(core.cpu.waiting),
            tostring(core.cpu.nmi_pending),
            tostring(core.cpu.irq_pending)))

        -- PPU State
        Log("--- PPU ---")
        Log(string.format("Dot: %d  Scanline: %d  Screen XY: (%d, %d)",
            core.ppu.dot, core.ppu.scanline, core.ppu.screen_x, core.ppu.screen_y))
        Log(string.format("Brightness: %d  F-Blank: %s  BG Mode: %d",
            core.ppu.screen_brightness,
            tostring(core.ppu.f_blank),
            core.ppu.bg_mode))
        Log(string.format("VRAM Addr: %04X  OAM Addr: %04X  CGRAM Addr: %02X",
            core.ppu.vram_addr, core.ppu.oam_addr, core.ppu.cgram_addr))
        Log(string.format("OBJ: Size=%d  NameBase=%04X  NameSec=%04X  PrioRot=%s",
            core.ppu.obj_size,
            core.ppu.name_base_addr,
            core.ppu.name_secondary_addr,
            tostring(core.ppu.priority_rotation)))
        Log(string.format("Mosaic Size: %d  BG Mosaic: [%s %s %s %s]",
            core.ppu.mosaic_size,
            core.ppu.bg1_mosaic_enable and "BG1" or "---",
            core.ppu.bg2_mosaic_enable and "BG2" or "---",
            core.ppu.bg3_mosaic_enable and "BG3" or "---",
            core.ppu.bg4_mosaic_enable and "BG4" or "---"))
        Log(string.format("BG Tilemap Addrs: BG1=%04X  BG2=%04X  BG3=%04X  BG4=%04X",
            core.ppu.bg1_tilemap_addr, core.ppu.bg2_tilemap_addr,
            core.ppu.bg3_tilemap_addr, core.ppu.bg4_tilemap_addr))
        Log(string.format("BG Scroll H/V: BG1=%d/%d  BG2=%d/%d  BG3=%d/%d  BG4=%d/%d",
            core.ppu.bg1_hofs, core.ppu.bg1_vofs,
            core.ppu.bg2_hofs, core.ppu.bg2_vofs,
            core.ppu.bg3_hofs, core.ppu.bg3_vofs,
            core.ppu.bg4_hofs, core.ppu.bg4_vofs))
        Log(string.format("Large Tiles: [%s %s %s %s]  BG3 M1 Prio: %s",
            core.ppu.bg1_large_tiles and "BG1" or "---",
            core.ppu.bg2_large_tiles and "BG2" or "---",
            core.ppu.bg3_large_tiles and "BG3" or "---",
            core.ppu.bg4_large_tiles and "BG4" or "---",
            tostring(core.ppu.bg3_mode1_priority)))
        Log(string.format("Main Enable: [%s %s %s %s %s]",
            core.ppu.bg1_main_enable and "BG1" or "---",
            core.ppu.bg2_main_enable and "BG2" or "---",
            core.ppu.bg3_main_enable and "BG3" or "---",
            core.ppu.bg4_main_enable and "BG4" or "---",
            core.ppu.obj_main_enable and "OBJ" or "---"))
        Log(string.format("Window1: [%d, %d]  Window2: [%d, %d]",
            core.ppu.window1_left, core.ppu.window1_right,
            core.ppu.window2_left, core.ppu.window2_right))
        Log(string.format("Mode7: A=%04X B=%04X C=%04X D=%04X  Origin=(%d,%d)  Scroll=(%d,%d)  Mul=%d",
            core.ppu.m7_a, core.ppu.m7_b, core.ppu.m7_c, core.ppu.m7_d,
            core.ppu.m7_x, core.ppu.m7_y,
            core.ppu.m7_hofs, core.ppu.m7_vofs,
            core.ppu.multiply_result))
        Log(string.format("H/V Counter: %d / %d", core.ppu.h_counter, core.ppu.v_counter))

        -- DMA Channels
        Log("--- DMA ---")
        for i = 0, 7 do
            local ch = core.dma[i]
            if ch then
                Log(string.format(
                    "CH%d: ABus=%06X  BBus=%04X  Pat=%d  Inc=%d  B->A=%s  Indirect=%s  Reload=%s",
                    i,
                    ch.full_a_bus_addr,
                    ch.b_bus_addr,
                    ch.transfer_pattern,
                    ch.addr_inc_mode,
                    tostring(ch.b_to_a),
                    tostring(ch.indirect_hdma),
                    tostring(ch.hdma_reload)))
                Log(string.format(
                    "  HDMA: TableStart=%06X  IndirectTable=%06X  TableOffset=%04X  ScanlineCtr=%d",
                    ch.full_hdma_table_start_addr,
                    ch.full_hdma_indirect_table_addr,
                    ch.hdma_table_offset,
                    ch.hdma_scanline_counter))
            end
        end

        Log("=== END RESET DUMP ===")
    end
end