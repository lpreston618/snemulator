-- snemulator_api.lua
-- This file provides IDE autocomplete support for snemulator watchpoint scripts
-- Include this at the top of your scripts with: require('snemulator_api')
-- Note that this script is ignored by the watchpoint engine. It is purely meant
-- as an aid to the IDE to make script development bearable.

---@alias u8 number Unsigned 8-bit value
---@alias u16 number Unsigned 16-bit value
---@alias CpuAddress number Unsigned 24-bit value

---@class Snemulator
core = {}

---@class Metadata
---@field frame number Current frame number
---@field rom_size number Size of the loaded ROM
core.meta = {}

---@class Snemulator.CPU
---@field pb u8         : Program Bank
---@field db u8         : Data Bank
---@field p u8          : Processor Status Byte
---@field apuio0 u8 Data from the CPU to the APU (Data from APU to CPU is accessible via `core.apu.apuio0`)
---@field apuio1 u8 Data from the CPU to the APU (Data from APU to CPU is accessible via `core.apu.apuio1`)
---@field apuio2 u8 Data from the CPU to the APU (Data from APU to CPU is accessible via `core.apu.apuio2`)
---@field apuio3 u8 Data from the CPU to the APU (Data from APU to CPU is accessible via `core.apu.apuio3`)
---@field prg0 u8 Alias for core.mem[cpu.full_pc], the current byte of the program (Read-only)
---@field prg1 u8 Alias for core.mem[cpu.full_pc + 1], the next byte of the program (address wrapped at bank) (Read-only)
---@field prg2 u8 Alias for core.mem[cpu.full_pc + 2], the next next byte of the program (address wrapped at bank) (Read-only)
---@field a u16 Accumulator
---@field x u16 X Index Register
---@field y u16 Y Index Register
---@field sp u16 Stack Pointer
---@field pc u16 Program Counter
---@field dp u16 Direct Page
---@field flagc boolean Carry Flag
---@field flagz boolean Zero Flag
---@field flagi boolean IRQ Disable Flag
---@field flagd boolean Decimal Mode Flag
---@field flagx boolean Index Register Size Flag
---@field flagm boolean Accumulator Size Flag
---@field flagv boolean Overflow Flag
---@field flagn boolean Negative Flag
---@field e boolean Emulation Mode Flag
---@field halted boolean CPU Halted
---@field stopped boolean CPU Stopped (For DMA/HDMA) (Read-only)
---@field nmi_pending boolean NMI Pending
---@field irq_pending boolean IRQ Pending
---@field waiting boolean Awaiting Interrupt Flag
---@field full_pc CpuAddress The concatenated pb and pc (i.e. (pb << 16) | pc). Useful for accessing program memory via `core.mem[core.cpu.full_pc]` (Read-only)
core.cpu = {}

---@class Snemulator.PPU
---@field screen_brightness u8 Screen brightness from 0-15 (INIDISP)
---@field obj_size u8 Object Size (0 = 8x8, 1 = 16x16) (OBJSEL)
---@field bg_mode u8 BG Mode from 0-7 (BGMODE)
---@field mosaic_size u8 Mosaic size from 0-15 (MOSAIC)
---@field cgram_addr u8 8-bit CGRAM address (CGADD)
---@field window1_left u8 Window 1 left position from 0-255 (WH0)
---@field window1_right u8 Window 1 right position from 0-255 (WH1)
---@field window2_left u8 Window 2 left position from 0-255 (WH2)
---@field window2_right u8 Window 2 right position from 0-255 (WH3)
---@field name_base_addr u16 Full 15-bit name base address from OBJSEL
---@field name_secondary_addr u16 Full 15-bit name base address + secondary select offset from OBJSEL
---@field oam_addr u16 Full 9-bit oam address from oamaddl and oamaddh
---@field bg1_tilemap_addr u16 From BG1SC
---@field bg2_tilemap_addr u16 From BG2SC
---@field bg3_tilemap_addr u16 From BG3SC
---@field bg4_tilemap_addr u16 From BG4SC
---@field bg1_hofs u16 From BG1HOFS
---@field bg1_vofs u16 From BG1VOFS
---@field bg2_hofs u16 From BG2HOFS
---@field bg2_vofs u16 From BG2VOFS
---@field bg3_hofs u16 From BG3HOFS
---@field bg3_vofs u16 From BG3VOFS
---@field bg4_hofs u16 From BG4HOFS
---@field bg4_vofs u16 From BG4VOFS
---@field m7_hofs u16 From M7HOFS
---@field m7_vofs u16 From M7VOFS
---@field vram_addr u16 From VMADDL and VMADDH
---@field m7_a u16 From M7A
---@field m7_b u16 From M7B
---@field m7_c u16 From M7C
---@field m7_d u16 From M7D
---@field m7_x u16 From M7X
---@field m7_y u16 From M7Y
---@field h_counter u16 From OPHCT
---@field v_counter u16 From OPVCT
---@field priority_rotation boolean From OAMADDH
---@field bg1_large_tiles boolean From BGMODE
---@field bg2_large_tiles boolean From BGMODE
---@field bg3_large_tiles boolean From BGMODE
---@field bg4_large_tiles boolean From BGMODE
---@field bg3_mode1_priority boolean From BGMODE
---@field bg1_mosaic_enable boolean From MOSAIC
---@field bg2_mosaic_enable boolean From MOSAIC
---@field bg3_mosaic_enable boolean From MOSAIC
---@field bg4_mosaic_enable boolean From MOSAIC
---@field bg1_main_enable boolean From TM
---@field bg2_main_enable boolean From TM
---@field bg3_main_enable boolean From TM
---@field bg4_main_enable boolean From TM
---@field obj_main_enable boolean From TM
---@field dot number Current dot, ranges from [0,339]
---@field scanline number Current scanline, ranges from [0,260]
---@field screen_x number Current dot position on the screen. When the dot is visible, this number ranges from [0,255]. Otherwise it is garbage.
---@field screen_y number Current scanline position on screen. When the scanline is visible, this number ranges from [0,223]. Otherwise it is garbage.
---@field multiply_result number Full 24-bit multiplication result from MPYL, MPYM, and MPYH
---@field f_blank boolean Whether the PPU is currently in forced blank.
---@field v_blank boolean Whether the PPU is currently in vertical blank.
---@field h_blank boolean Whether the PPU is currently in horizontal blank.
---@field v_blank_nmi boolean Whether the PPU will generate and NMI upon entering its v-blank period.
---@field hv_timer_mode number The current mode of the HV timer IRQ mechanism.
core.ppu = {}

---@type (u8?)[] Direct access to ROM. Size is unknown at compile time, but can be accessed via `core.rom_size`
--- Reads nil if address is out of bounds
core.rom = {}

---@type (u8?)[] Direct access to 128 KiB of WRAM (banks 7E and 7F of CPU-mapped memory).
--- Reads nil if address > 0x1FFFF
core.wram = {}

---@type (u16?)[] Access to 32 KiW (64 KiB) of VRAM as addressed by the SPPU.
--- Reads nil if address > 0x7FFF
core.vram = {}

---@type (u8?)[] Access to 64 KiB of ARAM as addressed by the SPC processor.
--- Reads nil if address > 0xFFFF
core.aram = {}

---@type (u8?)[] Access to 544 Bytes of Object Attribute Memory (OAM).
--- Reads nil if address > 0x21F
core.oam = {}

---@type (u16?)[] Access to 256 Words (512 Bytes) of CGRAM. Reads nil if address is > 0xFF
core.cgram = {}

---@type (u8|u16?)[] Access to raw MMIO register values by address. Reads nil if address is not a valid MMIO register address.
core.mmio = {}

---@class DMA
---@field dma_en boolean Whether this DMA channel has DMA enabled.
---@field hdma_en boolean Whether this DMA channel has HDMA enabled.
---@field addr_inc_mode u8 From DMAP
---@field transfer_pattern u8 From DMAP
---@field a_bus_bank u8 From A1TB
---@field hdma_table_start_bank u8 Alias for a_bus_bank
---@field hdma_indirect_table_bank u8 From DASB
---@field hdma_scanline_counter u8 From NLTR
---@field unused_reg u8 From UNUSED ($43nB/$43nF)
---@field b_bus_addr u16 High byte is always 0x21. From BBAD
---@field a_bus_offset u16 From A1TL, A1TH
---@field hdma_table_start_offset u16 Alias for a_bus_offset
---@field hdma_indirect_table_offset u16 From DASL, DASH
---@field hdma_table_offset u16 From A2AL and A2AH
---@field b_to_a boolean From DMAP
---@field indirect_hdma boolean From DMAP
---@field hdma_reload boolean From NLTR
---@field full_a_bus_addr CpuAddress From A1TL, A1TH, and A1B
---@field full_hdma_table_start_addr CpuAddress Alias for full_a_bus_addr
---@field full_hdma_indirect_table_addr CpuAddress From DASnL, DASnH, and DASBn

---@type (DMA?)[] Registers for DMA channels 0-7. `core.dma[n]` reads nil if n is not an integer from 0 to 7.
core.dma = {}

---@type DMA
core.dma[0] = {} ---@diagnostic disable-line: missing-fields
---@type DMA
core.dma[1] = {} ---@diagnostic disable-line: missing-fields
---@type DMA
core.dma[2] = {} ---@diagnostic disable-line: missing-fields
---@type DMA
core.dma[3] = {} ---@diagnostic disable-line: missing-fields
---@type DMA
core.dma[4] = {} ---@diagnostic disable-line: missing-fields
---@type DMA
core.dma[5] = {} ---@diagnostic disable-line: missing-fields
---@type DMA
core.dma[6] = {} ---@diagnostic disable-line: missing-fields
---@type DMA
core.dma[7] = {} ---@diagnostic disable-line: missing-fields

---Log a message with the debug log level
---@param message string The message to log
function Log(message) end

---Event handler called every emulation cycle.
---WARNING: This is called millions of times per frame. Using it will lead to performance decrease.
function OnEmulationCycle() end

---Event handler called every PPU cycle.
---WARNING: This is called millions of times per frame. Using it will lead to performance decrease.
function OnDot() end

---Event handler called at start of each scanline.
function OnScanline() end

---Event handler called at the end of each frame (start of V-Blank)
function OnFrame() end

---Event handler called after every SCPU instruction.
---WARNING: This is called many, many times per frame and complex logic here can lead to performance decrease.
function OnInstruction() end

---Event handler called on SCPU memory writes.
---@param addr CpuAddress Memory address written
---@param value u8 Byte value written
function OnMemoryWrite(addr, value) end

---Event handler called on SCPU memory reads.
---@param addr CpuAddress Memory address read
---@param value u8 Byte value read from memory
function OnMemoryRead(addr, value) end

---Event handler called when an SCPU interrupt occurs.
---@param kind Interrupt The kind of interrupt that has occured.
function OnInterrupt(kind) end

---Event handler called when a DMA transfer starts on a given channel.
---@param channel number DMA channel number
function OnDMAStart(channel) end

---Event handler called on each DMA byte transfer.
---@param channel number DMA channel number
---@param src_addr CpuAddress Source memory address
---@param dst_addr CpuAddress Destination memory address
---@param value u8 Byte value transferred
function OnDMATransfer(channel, src_addr, dst_addr, value) end

---Event handler called when a DMA transfer ends on a given channel.
---@param channel number DMA channel number
function OnDMAEnd(channel) end

---Event handler called when an HDMA transfer starts on a given channel.
---@param channel number HDMA channel number
function OnHDMAStart(channel) end

---Event handler called on each HDMA byte transfer.
---@param channel number HDMA channel number
---@param src_addr CpuAddress Source memory address
---@param dst_addr CpuAddress Destination memory address
---@param value u8 Byte value transferred
function OnHDMATransfer(channel, src_addr, dst_addr, value) end

---Event handler called when an HDMA transfer ends on a given channel.
---@param channel number HDMA channel number
function OnHDMAEnd(channel) end

---Callbacks to control emulator output
control = {}

---Break/Pause emulation
function control:Break() end
---Register a callback to be called on an emulation event. This is automatically called for all
---callbacks when the script is loaded, and should only be used to re-register callbacks unregistered
---by the user.
function control:RegisterCallback(callback) end
---Unregister a callback that was previously registered with `control.RegisterCallback`.
function control:UnregisterCallback(callback) end
---Enable or disable audio output. When fast forwarding, this should be used to disable audio output.
function control:SetAudioEnabled(enabled) end
---Enable or disable video output. This can dramatically improve performance when debugging with a watchpoint script.
---At the obvious cost of not having a visible screen.
function control:SetVideoEnabled(enabled) end
---Enable or disable input processing.
function control:SetInputEnabled(enabled) end
---Enable or disable fast forwarding.
function control:SetFastForwardEnabled(enabled) end

CONSTS = {}
CONSTS.mmio = {} -- MMIO Register Addresses
CONSTS.mmio.INIDISP = 0x2100 -- F... BBBB           | Forced blanking (F), screen brightness (B)
CONSTS.mmio.OBJSEL = 0x2101 -- SSSN NbBB           | OBJ sprite size (S), name secondary select (N), name base address (B)
CONSTS.mmio.OAMADDL = 0x2102 -- AAAA AAAA           | OAM word address (A)
CONSTS.mmio.OAMADDH = 0x2103 -- P... ...B           | Priority rotation (P), address high bit (B)
CONSTS.mmio.BGMODE = 0x2105 -- 4321 PMMM           | Tilemap size for BG layers 1,2,3, and 4, BG3 priority (P), BG mode (M)
CONSTS.mmio.MOSAIC = 0x2106 -- SSSS 4321           | Mosaic size (S), mosaic enable for BG layers 1,2,3, and 4
CONSTS.mmio.BG1SC = 0x2107 -- AAAA AAYX           | BG1 Tilemap VRAM address, vertical/horizontal tilemap count
CONSTS.mmio.BG2SC = 0x2108 -- AAAA AAYX           | BG2 Tilemap VRAM address, vertical/horizontal tilemap count
CONSTS.mmio.BG3SC = 0x2109 -- AAAA AAYX           | BG3 Tilemap VRAM address, vertical/horizontal tilemap count
CONSTS.mmio.BG4SC = 0x210A -- AAAA AAYX           | BG4 Tilemap VRAM address, vertical/horizontal tilemap count
CONSTS.mmio.BG12NBA = 0x210B -- BBBB AAAA           | BG2 CHR base address (B), BG1 CHR base addr (A)
CONSTS.mmio.BG34NBA = 0x210C -- DDDD CCCC           | BG4 CHR base address (D), BG3 CHR base addr (C)
CONSTS.mmio.BG1HOFS = 0x210D -- .... ..XX XXXX XXXX | BG1 horizontal scroll
CONSTS.mmio.M7HOFS = 0x210D -- .... XXXX XXXX XXXX | Mode 7 horizontal scroll
CONSTS.mmio.BG1VOFS = 0x210E -- .... ..YY YYYY YYYY | BG1 vertical scroll
CONSTS.mmio.M7VOFS = 0x210E -- .... YYYY YYYY YYYY | Mode 7 vertical scroll
CONSTS.mmio.BG2HOFS = 0x210F -- .... ..XX XXXX XXXX | BG horizontal scroll
CONSTS.mmio.BG2VOFS = 0x2110 -- .... ..YY YYYY YYYY | BG vertical scroll
CONSTS.mmio.BG3HOFS = 0x2111 -- .... ..XX XXXX XXXX | BG horizontal scroll
CONSTS.mmio.BG3VOFS = 0x2112 -- .... ..YY YYYY YYYY | BG vertical scroll
CONSTS.mmio.BG4HOFS = 0x2113 -- .... ..XX XXXX XXXX | BG horizontal scroll
CONSTS.mmio.BG4VOFS = 0x2114 -- .... ..YY YYYY YYYY | BG vertical scroll
CONSTS.mmio.VMAIN = 0x2115 -- M... RRII           | VRAM increment mode (M), remapping (R), increment size (I)
CONSTS.mmio.VMADDL = 0x2116 -- LLLL LLLL           | VRAM word address (low)
CONSTS.mmio.VMADDH = 0x2117 -- HHHH HHHH           | VRAM word address (high)
CONSTS.mmio.M7SEL = 0x211A -- RF.. ..YX           | Mode 7 settings (repeat, fill, flip X, flip Y)
CONSTS.mmio.M7A = 0x211B -- DDDD DDDD dddd dddd | Mode 7 matrix A
CONSTS.mmio.M7B = 0x211C -- DDDD DDDD dddd dddd | Mode 7 matrix B
CONSTS.mmio.M7C = 0x211D -- DDDD DDDD dddd dddd | Mode 7 matrix C
CONSTS.mmio.M7D = 0x211E -- DDDD DDDD dddd dddd | Mode 7 matrix D
CONSTS.mmio.M7X = 0x211F -- .... XXXX XXXX XXXX | Mode 7 center X
CONSTS.mmio.M7Y = 0x2120 -- .... YYYY YYYY YYYY | Mode 7 center Y
CONSTS.mmio.CGADD = 0x2121 -- AAAA AAAA           | CGRAM word address
CONSTS.mmio.W12SEL = 0x2123 -- DdCc BbAa           | Window enable/invert (EN/inv) BG1/BG2
CONSTS.mmio.W34SEL = 0x2124 -- DdCc BbAa           | Window enable/invert (EN/inv) BG3/BG4
CONSTS.mmio.WOBJSEL = 0x2125 -- LlKk JjIi           | Window enable/invert (EN/inv) OBJ/color
CONSTS.mmio.WH0 = 0x2126 -- LLLL LLLL           | Window 1 left position
CONSTS.mmio.WH1 = 0x2127 -- RRRR RRRR           | Window 1 right position
CONSTS.mmio.WH2 = 0x2128 -- LLLL LLLL           | Window 2 left position
CONSTS.mmio.WH3 = 0x2129 -- RRRR RRRR           | Window 2 right position
CONSTS.mmio.WBGLOG = 0x212A -- 4433 2211           | Window mask logic for BG layers
CONSTS.mmio.WOBJLOG = 0x212B -- .... CC00           | Window mask logic for OBJ/color
CONSTS.mmio.TM = 0x212C -- ...O 4321           | Main screen layer enable
CONSTS.mmio.TS = 0x212D -- ...O 4321           | Sub screen layer enable
CONSTS.mmio.TMW = 0x212E -- ...O 4321           | Main screen window enable
CONSTS.mmio.TSW = 0x212F -- ...O 4321           | Sub screen window enable
CONSTS.mmio.CGWSEL = 0x2130 -- MMSS ..AD           | Color math/window settings
CONSTS.mmio.CGADSUB = 0x2131 -- MHBO 4321           | Color math add/subtract
CONSTS.mmio.COLDATA = 0x2132 -- BGRC CCCC           | Fixed color channel select/value
CONSTS.mmio.SETINI = 0x2133 -- EX.. HOii           | Screen/EXTBG/interlace settings
CONSTS.mmio.MPYL = 0x2134 -- LLLL LLLL           | Multiplication result (low)
CONSTS.mmio.MPYM = 0x2135 -- MMMM MMMM           | Multiplication result (mid)
CONSTS.mmio.MPYH = 0x2136 -- HHHH HHHH           | Multiplication result (high)
CONSTS.mmio.SLHV = 0x2137 -- ........            | Latch H/V counters
CONSTS.mmio.OPHCT = 0x213C -- .... HHHH HHHH HHHH | Horizontal counter
CONSTS.mmio.OPVCT = 0x213D -- .... VVVV VVVV VVVV | Vertical counter
CONSTS.mmio.STAT77 = 0x213E -- TRM. VVVV           | Sprite overflow (T) tile overflow (R) master/slave (M) PPU1 ver. (V)
CONSTS.mmio.STAT78 = 0x213F -- FL.M VVVV           | Interlace field (F) counter latch value (L) NTSC/PAL (M) PPU2 ver. (V)
CONSTS.mmio.APUIO0 = 0x2140 -- DDDD DDDD | Data to/from APU.
CONSTS.mmio.APUIO1 = 0x2141 -- DDDD DDDD | Data to/from APU.
CONSTS.mmio.APUIO2 = 0x2142 -- DDDD DDDD | Data to/from APU.
CONSTS.mmio.APUIO3 = 0x2143 -- DDDD DDDD | Data to/from APU.
CONSTS.mmio.WMDATA = 0x2180 -- DDDD DDDD | Data to/from S-WRAM, increments WMADD.
CONSTS.mmio.WMADDL = 0x2181 -- LLLL LLLL | S-WRAM address for WMDATA access.
CONSTS.mmio.WMADDM = 0x2182 -- MMMM MMMM | S-WRAM address for WMDATA access.
CONSTS.mmio.WMADDH = 0x2183 -- .... ...H | S-WRAM address for WMDATA access.
CONSTS.mmio.JOYOUT = 0x4016 -- .... ...D | Output to joypads (latches standard controllers).
CONSTS.mmio.JOYSER0 = 0x4016 -- .... ..DD | Input from joypad 1.
CONSTS.mmio.JOYSER1 = 0x4017 -- ...1 11DD | Always 1 (1), input from joypad 2 (D).
CONSTS.mmio.NMITIMEN = 0x4200 -- N.VH ...J | Vblank NMI enable (N), timer IRQ mode (VH), joypad auto-read enable (J).
CONSTS.mmio.WRIO = 0x4201 -- 21DD DDDD | Joypad port 2 I/O (2), joypad port 1 I/O (1), unused I/O (D).
CONSTS.mmio.WRMPYA = 0x4202 -- DDDD DDDD | Unsigned multiplication factor A.
CONSTS.mmio.WRMPYB = 0x4203 -- DDDD DDDD | Unsigned multiplication factor B, starts 8-cycle multiplication.
CONSTS.mmio.WRDIVL = 0x4204 -- LLLL LLLL | Unsigned dividend.
CONSTS.mmio.WRDIVH = 0x4205 -- HHHH HHHH | Unsigned dividend.
CONSTS.mmio.WRDIVB = 0x4206 -- DDDD DDDD | Unsigned divisor, starts 16-cycle division.
CONSTS.mmio.HTIMEL = 0x4207 -- .... ...H | H counter target for timer IRQ.
CONSTS.mmio.HTIMEH = 0x4208 -- LLLL LLLL | H counter target for timer IRQ.
CONSTS.mmio.VTIMEL = 0x4209 -- .... ...H | V counter target for timer IRQ.
CONSTS.mmio.VTIMEH = 0x420A -- LLLL LLLL | V counter target for timer IRQ.
CONSTS.mmio.MDMAEN = 0x420B -- 7654 3210 | DMA enable.
CONSTS.mmio.HDMAEN = 0x420C -- 7654 3210 | HDMA enable.
CONSTS.mmio.MEMSEL = 0x420D -- .... ...F | FastROM enable (F).
CONSTS.mmio.RDNMI = 0x4210 -- N... VVVV | Vblank NMI flag (N), CPU version (V).
CONSTS.mmio.TIMEUP = 0x4211 -- T... .... | Timer IRQ flag (T).
CONSTS.mmio.HVBJOY = 0x4212 -- VH.. ...J | Vblank flag (V), hblank flag (H), joypad auto-read in-progress flag (J).
CONSTS.mmio.RDIO = 0x4213 -- 21DD DDDD | Joypad port 2 I/O (2), joypad port 1 I/O (1), unused I/O (D).
CONSTS.mmio.RDDIVL = 0x4214 -- LLLL LLLL | Unsigned quotient.
CONSTS.mmio.RDDIVH = 0x4215 -- HHHH HHHH | Unsigned quotient.
CONSTS.mmio.RDMPYL = 0x4216 -- LLLL LLLL | Unsigned product or unsigned remainder.
CONSTS.mmio.RDMPYH = 0x4217 -- HHHH HHHH | Unsigned product or unsigned remainder.
CONSTS.mmio.JOY1L = 0x4218 -- LLLL LLLL | 16-bit joypad auto-read result (first read high to last read low).
CONSTS.mmio.JOY1H = 0x4219 -- HHHH HHHH | 16-bit joypad auto-read result (first read high to last read low).
CONSTS.mmio.JOY2L = 0x421A -- LLLL LLLL | 16-bit joypad auto-read result (first read high to last read low).
CONSTS.mmio.JOY2H = 0x421B -- HHHH HHHH | 16-bit joypad auto-read result (first read high to last read low).
CONSTS.mmio.JOY3L = 0x421C -- LLLL LLLL | 16-bit joypad auto-read result (first read high to last read low).
CONSTS.mmio.JOY3H = 0x421D -- HHHH HHHH | 16-bit joypad auto-read result (first read high to last read low).
CONSTS.mmio.JOY4L = 0x421E -- LLLL LLLL | 16-bit joypad auto-read result (first read high to last read low).
CONSTS.mmio.JOY4H = 0x421F -- HHHH HHHH | 16-bit joypad auto-read result (first read high to last read low).
CONSTS.mmio.DMAP0 = 0x4300 -- DI.A APPP | Direction (D) Indirect HDMA (I) Addr inc mode (A) Transfer pattern (P)
CONSTS.mmio.BBAD0 = 0x4301 -- AAAA AAAA | B-bus address.
CONSTS.mmio.A1T0L = 0x4302 -- LLLL LLLL | DMA source address / HDMA table start address (low).
CONSTS.mmio.A1T0H = 0x4303 -- HHHH HHHH | DMA source address / HDMA table start address (high).
CONSTS.mmio.A1B0 = 0x4304 -- BBBB BBBB | DMA source address / HDMA table start address (bank).
CONSTS.mmio.DAS0L = 0x4305 -- LLLL LLLL | DMA byte count / HDMA indirect table address (low).
CONSTS.mmio.DAS0H = 0x4306 -- HHHH HHHH | DMA byte count / HDMA indirect table address (high).
CONSTS.mmio.DASB0 = 0x4307 -- BBBB BBBB | DMA byte count / HDMA indirect table address (bank).
CONSTS.mmio.A2A0L = 0x4308 -- LLLL LLLL | HDMA table current address within bank (low).
CONSTS.mmio.A2A0H = 0x4309 -- HHHH HHHH | HDMA table current address within bank (high).
CONSTS.mmio.NLTR0 = 0x430A -- RLLL LLLL | HDMA reload flag (R) and scanline counter (L).
CONSTS.mmio.UNUSED0 = 0x430B -- DDDD DDDD | Unused shared data byte (Same as $43nF).
CONSTS.mmio.DMAP1 = 0x4310 -- DI.A APPP | Direction (D) Indirect HDMA (I) Addr inc mode (A) Transfer pattern (P)
CONSTS.mmio.BBAD1 = 0x4311 -- AAAA AAAA | B-bus address.
CONSTS.mmio.A1T1L = 0x4312 -- LLLL LLLL | DMA source address / HDMA table start address (low).
CONSTS.mmio.A1T1H = 0x4313 -- HHHH HHHH | DMA source address / HDMA table start address (high).
CONSTS.mmio.A1B1 = 0x4314 -- BBBB BBBB | DMA source address / HDMA table start address (bank).
CONSTS.mmio.DAS1L = 0x4315 -- LLLL LLLL | DMA byte count / HDMA indirect table address (low).
CONSTS.mmio.DAS1H = 0x4316 -- HHHH HHHH | DMA byte count / HDMA indirect table address (high).
CONSTS.mmio.DASB1 = 0x4317 -- BBBB BBBB | DMA byte count / HDMA indirect table address (bank).
CONSTS.mmio.A2A1L = 0x4318 -- LLLL LLLL | HDMA table current address within bank (low).
CONSTS.mmio.A2A1H = 0x4319 -- HHHH HHHH | HDMA table current address within bank (high).
CONSTS.mmio.NLTR1 = 0x431A -- RLLL LLLL | HDMA reload flag (R) and scanline counter (L).
CONSTS.mmio.UNUSED1 = 0x431B -- DDDD DDDD | Unused shared data byte (Same as $43nF).
CONSTS.mmio.DMAP2 = 0x4320 -- DI.A APPP | Direction (D) Indirect HDMA (I) Addr inc mode (A) Transfer pattern (P)
CONSTS.mmio.BBAD2 = 0x4321 -- AAAA AAAA | B-bus address.
CONSTS.mmio.A1T2L = 0x4322 -- LLLL LLLL | DMA source address / HDMA table start address (low).
CONSTS.mmio.A1T2H = 0x4323 -- HHHH HHHH | DMA source address / HDMA table start address (high).
CONSTS.mmio.A1B2 = 0x4324 -- BBBB BBBB | DMA source address / HDMA table start address (bank).
CONSTS.mmio.DAS2L = 0x4325 -- LLLL LLLL | DMA byte count / HDMA indirect table address (low).
CONSTS.mmio.DAS2H = 0x4326 -- HHHH HHHH | DMA byte count / HDMA indirect table address (high).
CONSTS.mmio.DASB2 = 0x4327 -- BBBB BBBB | DMA byte count / HDMA indirect table address (bank).
CONSTS.mmio.A2A2L = 0x4328 -- LLLL LLLL | HDMA table current address within bank (low).
CONSTS.mmio.A2A2H = 0x4329 -- HHHH HHHH | HDMA table current address within bank (high).
CONSTS.mmio.NLTR2 = 0x432A -- RLLL LLLL | HDMA reload flag (R) and scanline counter (L).
CONSTS.mmio.UNUSED2 = 0x432B -- DDDD DDDD | Unused shared data byte (Same as $43nF).
CONSTS.mmio.DMAP3 = 0x4330 -- DI.A APPP | Direction (D) Indirect HDMA (I) Addr inc mode (A) Transfer pattern (P)
CONSTS.mmio.BBAD3 = 0x4331 -- AAAA AAAA | B-bus address.
CONSTS.mmio.A1T3L = 0x4332 -- LLLL LLLL | DMA source address / HDMA table start address (low).
CONSTS.mmio.A1T3H = 0x4333 -- HHHH HHHH | DMA source address / HDMA table start address (high).
CONSTS.mmio.A1B3 = 0x4334 -- BBBB BBBB | DMA source address / HDMA table start address (bank).
CONSTS.mmio.DAS3L = 0x4335 -- LLLL LLLL | DMA byte count / HDMA indirect table address (low).
CONSTS.mmio.DAS3H = 0x4336 -- HHHH HHHH | DMA byte count / HDMA indirect table address (high).
CONSTS.mmio.DASB3 = 0x4337 -- BBBB BBBB | DMA byte count / HDMA indirect table address (bank).
CONSTS.mmio.A2A3L = 0x4338 -- LLLL LLLL | HDMA table current address within bank (low).
CONSTS.mmio.A2A3H = 0x4339 -- HHHH HHHH | HDMA table current address within bank (high).
CONSTS.mmio.NLTR3 = 0x433A -- RLLL LLLL | HDMA reload flag (R) and scanline counter (L).
CONSTS.mmio.UNUSED3 = 0x433B -- DDDD DDDD | Unused shared data byte (Same as $43nF).
CONSTS.mmio.DMAP4 = 0x4340 -- DI.A APPP | Direction (D) Indirect HDMA (I) Addr inc mode (A) Transfer pattern (P)
CONSTS.mmio.BBAD4 = 0x4341 -- AAAA AAAA | B-bus address.
CONSTS.mmio.A1T4L = 0x4342 -- LLLL LLLL | DMA source address / HDMA table start address (low).
CONSTS.mmio.A1T4H = 0x4343 -- HHHH HHHH | DMA source address / HDMA table start address (high).
CONSTS.mmio.A1B4 = 0x4344 -- BBBB BBBB | DMA source address / HDMA table start address (bank).
CONSTS.mmio.DAS4L = 0x4345 -- LLLL LLLL | DMA byte count / HDMA indirect table address (low).
CONSTS.mmio.DAS4H = 0x4346 -- HHHH HHHH | DMA byte count / HDMA indirect table address (high).
CONSTS.mmio.DASB4 = 0x4347 -- BBBB BBBB | DMA byte count / HDMA indirect table address (bank).
CONSTS.mmio.A2A4L = 0x4348 -- LLLL LLLL | HDMA table current address within bank (low).
CONSTS.mmio.A2A4H = 0x4349 -- HHHH HHHH | HDMA table current address within bank (high).
CONSTS.mmio.NLTR4 = 0x434A -- RLLL LLLL | HDMA reload flag (R) and scanline counter (L).
CONSTS.mmio.UNUSED4 = 0x434B -- DDDD DDDD | Unused shared data byte (Same as $43nF).
CONSTS.mmio.DMAP5 = 0x4350 -- DI.A APPP | Direction (D) Indirect HDMA (I) Addr inc mode (A) Transfer pattern (P)
CONSTS.mmio.BBAD5 = 0x4351 -- AAAA AAAA | B-bus address.
CONSTS.mmio.A1T5L = 0x4352 -- LLLL LLLL | DMA source address / HDMA table start address (low).
CONSTS.mmio.A1T5H = 0x4353 -- HHHH HHHH | DMA source address / HDMA table start address (high).
CONSTS.mmio.A1B5 = 0x4354 -- BBBB BBBB | DMA source address / HDMA table start address (bank).
CONSTS.mmio.DAS5L = 0x4355 -- LLLL LLLL | DMA byte count / HDMA indirect table address (low).
CONSTS.mmio.DAS5H = 0x4356 -- HHHH HHHH | DMA byte count / HDMA indirect table address (high).
CONSTS.mmio.DASB5 = 0x4357 -- BBBB BBBB | DMA byte count / HDMA indirect table address (bank).
CONSTS.mmio.A2A5L = 0x4358 -- LLLL LLLL | HDMA table current address within bank (low).
CONSTS.mmio.A2A5H = 0x4359 -- HHHH HHHH | HDMA table current address within bank (high).
CONSTS.mmio.NLTR5 = 0x435A -- RLLL LLLL | HDMA reload flag (R) and scanline counter (L).
CONSTS.mmio.UNUSED5 = 0x435B -- DDDD DDDD | Unused shared data byte (Same as $43nF).
CONSTS.mmio.DMAP6 = 0x4360 -- DI.A APPP | Direction (D) Indirect HDMA (I) Addr inc mode (A) Transfer pattern (P)
CONSTS.mmio.BBAD6 = 0x4361 -- AAAA AAAA | B-bus address.
CONSTS.mmio.A1T6L = 0x4362 -- LLLL LLLL | DMA source address / HDMA table start address (low).
CONSTS.mmio.A1T6H = 0x4363 -- HHHH HHHH | DMA source address / HDMA table start address (high).
CONSTS.mmio.A1B6 = 0x4364 -- BBBB BBBB | DMA source address / HDMA table start address (bank).
CONSTS.mmio.DAS6L = 0x4365 -- LLLL LLLL | DMA byte count / HDMA indirect table address (low).
CONSTS.mmio.DAS6H = 0x4366 -- HHHH HHHH | DMA byte count / HDMA indirect table address (high).
CONSTS.mmio.DASB6 = 0x4367 -- BBBB BBBB | DMA byte count / HDMA indirect table address (bank).
CONSTS.mmio.A2A6L = 0x4368 -- LLLL LLLL | HDMA table current address within bank (low).
CONSTS.mmio.A2A6H = 0x4369 -- HHHH HHHH | HDMA table current address within bank (high).
CONSTS.mmio.NLTR6 = 0x436A -- RLLL LLLL | HDMA reload flag (R) and scanline counter (L).
CONSTS.mmio.UNUSED6 = 0x436B -- DDDD DDDD | Unused shared data byte (Same as $43nF).
CONSTS.mmio.DMAP7 = 0x4370 -- DI.A APPP | Direction (D) Indirect HDMA (I) Addr inc mode (A) Transfer pattern (P)
CONSTS.mmio.BBAD7 = 0x4371 -- AAAA AAAA | B-bus address.
CONSTS.mmio.A1T7L = 0x4372 -- LLLL LLLL | DMA source address / HDMA table start address (low).
CONSTS.mmio.A1T7H = 0x4373 -- HHHH HHHH | DMA source address / HDMA table start address (high).
CONSTS.mmio.A1B7 = 0x4374 -- BBBB BBBB | DMA source address / HDMA table start address (bank).
CONSTS.mmio.DAS7L = 0x4375 -- LLLL LLLL | DMA byte count / HDMA indirect table address (low).
CONSTS.mmio.DAS7H = 0x4376 -- HHHH HHHH | DMA byte count / HDMA indirect table address (high).
CONSTS.mmio.DASB7 = 0x4377 -- BBBB BBBB | DMA byte count / HDMA indirect table address (bank).
CONSTS.mmio.A2A7L = 0x4378 -- LLLL LLLL | HDMA table current address within bank (low).
CONSTS.mmio.A2A7H = 0x4379 -- HHHH HHHH | HDMA table current address within bank (high).
CONSTS.mmio.NLTR7 = 0x437A -- RLLL LLLL | HDMA reload flag (R) and scanline counter (L).
CONSTS.mmio.UNUSED7 = 0x437B  -- DDDD DDDD | Unused shared data byte (Same as $43nF).

---@enum Interrupt
CONSTS.interrupts = {
    IRQ = 0,
    NMI = 1,
    BRK = 2,
    COP = 3,
    RESET = 4,
    ABORT = 5,
}