pub const LUA_INCLUDE: &str = r#"

CONSTS = {}
CONSTS.mmio = {}
CONSTS.mmio.INIDISP = 0x2100
CONSTS.mmio.OBJSEL = 0x2101
CONSTS.mmio.OAMADDL = 0x2102
CONSTS.mmio.OAMADDH = 0x2103
CONSTS.mmio.BGMODE = 0x2105
CONSTS.mmio.MOSAIC = 0x2106
CONSTS.mmio.BG1SC = 0x2107
CONSTS.mmio.BG2SC = 0x2108
CONSTS.mmio.BG3SC = 0x2109
CONSTS.mmio.BG4SC = 0x210A
CONSTS.mmio.BG12NBA = 0x210B
CONSTS.mmio.BG34NBA = 0x210C
CONSTS.mmio.BG1HOFS = 0x210D
CONSTS.mmio.M7HOFS = 0x210D
CONSTS.mmio.BG1VOFS = 0x210E
CONSTS.mmio.M7VOFS = 0x210E
CONSTS.mmio.BG2HOFS = 0x210F
CONSTS.mmio.BG2VOFS = 0x2110
CONSTS.mmio.BG3HOFS = 0x2111
CONSTS.mmio.BG3VOFS = 0x2112
CONSTS.mmio.BG4HOFS = 0x2113
CONSTS.mmio.BG4VOFS = 0x2114
CONSTS.mmio.VMAIN = 0x2115
CONSTS.mmio.VMADDL = 0x2116
CONSTS.mmio.VMADDH = 0x2117
CONSTS.mmio.M7SEL = 0x211A
CONSTS.mmio.M7A = 0x211B
CONSTS.mmio.M7B = 0x211C
CONSTS.mmio.M7C = 0x211D
CONSTS.mmio.M7D = 0x211E
CONSTS.mmio.M7X = 0x211F
CONSTS.mmio.M7Y = 0x2120
CONSTS.mmio.CGADD = 0x2121
CONSTS.mmio.W12SEL = 0x2123
CONSTS.mmio.W34SEL = 0x2124
CONSTS.mmio.WOBJSEL = 0x2125
CONSTS.mmio.WH0 = 0x2126
CONSTS.mmio.WH1 = 0x2127
CONSTS.mmio.WH2 = 0x2128
CONSTS.mmio.WH3 = 0x2129
CONSTS.mmio.WBGLOG = 0x212A
CONSTS.mmio.WOBJLOG = 0x212B
CONSTS.mmio.TM = 0x212C
CONSTS.mmio.TS = 0x212D
CONSTS.mmio.TMW = 0x212E
CONSTS.mmio.TSW = 0x212F
CONSTS.mmio.CGWSEL = 0x2130
CONSTS.mmio.CGADSUB = 0x2131
CONSTS.mmio.COLDATA = 0x2132
CONSTS.mmio.SETINI = 0x2133
CONSTS.mmio.MPYL = 0x2134
CONSTS.mmio.MPYM = 0x2135
CONSTS.mmio.MPYH = 0x2136
CONSTS.mmio.SLHV = 0x2137
CONSTS.mmio.OPHCT = 0x213C
CONSTS.mmio.OPVCT = 0x213D
CONSTS.mmio.STAT77 = 0x213E
CONSTS.mmio.STAT78 = 0x213F
CONSTS.mmio.APUIO0 = 0x2140
CONSTS.mmio.APUIO1 = 0x2141
CONSTS.mmio.APUIO2 = 0x2142
CONSTS.mmio.APUIO3 = 0x2143
CONSTS.mmio.WMDATA = 0x2180
CONSTS.mmio.WMADDL = 0x2181
CONSTS.mmio.WMADDM = 0x2182
CONSTS.mmio.WMADDH = 0x2183
CONSTS.mmio.JOYOUT = 0x4016
CONSTS.mmio.JOYSER0 = 0x4016
CONSTS.mmio.JOYSER1 = 0x4017
CONSTS.mmio.NMITIMEN = 0x4200
CONSTS.mmio.WRIO = 0x4201
CONSTS.mmio.WRMPYA = 0x4202
CONSTS.mmio.WRMPYB = 0x4203
CONSTS.mmio.WRDIVL = 0x4204
CONSTS.mmio.WRDIVH = 0x4205
CONSTS.mmio.WRDIVB = 0x4206
CONSTS.mmio.HTIMEL = 0x4207
CONSTS.mmio.HTIMEH = 0x4208
CONSTS.mmio.VTIMEL = 0x4209
CONSTS.mmio.VTIMEH = 0x420A
CONSTS.mmio.MDMAEN = 0x420B
CONSTS.mmio.HDMAEN = 0x420C
CONSTS.mmio.MEMSEL = 0x420D
CONSTS.mmio.RDNMI = 0x4210
CONSTS.mmio.TIMEUP = 0x4211
CONSTS.mmio.HVBJOY = 0x4212
CONSTS.mmio.RDIO = 0x4213
CONSTS.mmio.RDDIVL = 0x4214
CONSTS.mmio.RDDIVH = 0x4215
CONSTS.mmio.RDMPYL = 0x4216
CONSTS.mmio.RDMPYH = 0x4217
CONSTS.mmio.JOY1L = 0x4218
CONSTS.mmio.JOY1H = 0x4219
CONSTS.mmio.JOY2L = 0x421A
CONSTS.mmio.JOY2H = 0x421B
CONSTS.mmio.JOY3L = 0x421C
CONSTS.mmio.JOY3H = 0x421D
CONSTS.mmio.JOY4L = 0x421E
CONSTS.mmio.JOY4H = 0x421F
CONSTS.mmio.DMAP0 = 0x4300
CONSTS.mmio.BBAD0 = 0x4301
CONSTS.mmio.A1T0L = 0x4302
CONSTS.mmio.A1T0H = 0x4303
CONSTS.mmio.A1B0 = 0x4304
CONSTS.mmio.DAS0L = 0x4305
CONSTS.mmio.DAS0H = 0x4306
CONSTS.mmio.DASB0 = 0x4307
CONSTS.mmio.A2A0L = 0x4308
CONSTS.mmio.A2A0H = 0x4309
CONSTS.mmio.NLTR0 = 0x430A
CONSTS.mmio.UNUSED0 = 0x430B
CONSTS.mmio.DMAP1 = 0x4310
CONSTS.mmio.BBAD1 = 0x4311
CONSTS.mmio.A1T1L = 0x4312
CONSTS.mmio.A1T1H = 0x4313
CONSTS.mmio.A1B1 = 0x4314
CONSTS.mmio.DAS1L = 0x4315
CONSTS.mmio.DAS1H = 0x4316
CONSTS.mmio.DASB1 = 0x4317
CONSTS.mmio.A2A1L = 0x4318
CONSTS.mmio.A2A1H = 0x4319
CONSTS.mmio.NLTR1 = 0x431A
CONSTS.mmio.UNUSED1 = 0x431B
CONSTS.mmio.DMAP2 = 0x4320
CONSTS.mmio.BBAD2 = 0x4321
CONSTS.mmio.A1T2L = 0x4322
CONSTS.mmio.A1T2H = 0x4323
CONSTS.mmio.A1B2 = 0x4324
CONSTS.mmio.DAS2L = 0x4325
CONSTS.mmio.DAS2H = 0x4326
CONSTS.mmio.DASB2 = 0x4327
CONSTS.mmio.A2A2L = 0x4328
CONSTS.mmio.A2A2H = 0x4329
CONSTS.mmio.NLTR2 = 0x432A
CONSTS.mmio.UNUSED2 = 0x432B
CONSTS.mmio.DMAP3 = 0x4330
CONSTS.mmio.BBAD3 = 0x4331
CONSTS.mmio.A1T3L = 0x4332
CONSTS.mmio.A1T3H = 0x4333
CONSTS.mmio.A1B3 = 0x4334
CONSTS.mmio.DAS3L = 0x4335
CONSTS.mmio.DAS3H = 0x4336
CONSTS.mmio.DASB3 = 0x4337
CONSTS.mmio.A2A3L = 0x4338
CONSTS.mmio.A2A3H = 0x4339
CONSTS.mmio.NLTR3 = 0x433A
CONSTS.mmio.UNUSED3 = 0x433B
CONSTS.mmio.DMAP4 = 0x4340
CONSTS.mmio.BBAD4 = 0x4341
CONSTS.mmio.A1T4L = 0x4342
CONSTS.mmio.A1T4H = 0x4343
CONSTS.mmio.A1B4 = 0x4344
CONSTS.mmio.DAS4L = 0x4345
CONSTS.mmio.DAS4H = 0x4346
CONSTS.mmio.DASB4 = 0x4347
CONSTS.mmio.A2A4L = 0x4348
CONSTS.mmio.A2A4H = 0x4349
CONSTS.mmio.NLTR4 = 0x434A
CONSTS.mmio.UNUSED4 = 0x434B
CONSTS.mmio.DMAP5 = 0x4350
CONSTS.mmio.BBAD5 = 0x4351
CONSTS.mmio.A1T5L = 0x4352
CONSTS.mmio.A1T5H = 0x4353
CONSTS.mmio.A1B5 = 0x4354
CONSTS.mmio.DAS5L = 0x4355
CONSTS.mmio.DAS5H = 0x4356
CONSTS.mmio.DASB5 = 0x4357
CONSTS.mmio.A2A5L = 0x4358
CONSTS.mmio.A2A5H = 0x4359
CONSTS.mmio.NLTR5 = 0x435A
CONSTS.mmio.UNUSED5 = 0x435B
CONSTS.mmio.DMAP6 = 0x4360
CONSTS.mmio.BBAD6 = 0x4361
CONSTS.mmio.A1T6L = 0x4362
CONSTS.mmio.A1T6H = 0x4363
CONSTS.mmio.A1B6 = 0x4364
CONSTS.mmio.DAS6L = 0x4365
CONSTS.mmio.DAS6H = 0x4366
CONSTS.mmio.DASB6 = 0x4367
CONSTS.mmio.A2A6L = 0x4368
CONSTS.mmio.A2A6H = 0x4369
CONSTS.mmio.NLTR6 = 0x436A
CONSTS.mmio.UNUSED6 = 0x436B
CONSTS.mmio.DMAP7 = 0x4370
CONSTS.mmio.BBAD7 = 0x4371
CONSTS.mmio.A1T7L = 0x4372
CONSTS.mmio.A1T7H = 0x4373
CONSTS.mmio.A1B7 = 0x4374
CONSTS.mmio.DAS7L = 0x4375
CONSTS.mmio.DAS7H = 0x4376
CONSTS.mmio.DASB7 = 0x4377
CONSTS.mmio.A2A7L = 0x4378
CONSTS.mmio.A2A7H = 0x4379
CONSTS.mmio.NLTR7 = 0x437A
CONSTS.mmio.UNUSED7 = 0x437B

CONSTS.interrupts = {
    IRQ = 0,
    NMI = 1,
    BRK = 2,
    COP = 3,
    RESET = 4,
    ABORT = 5,
}

"#;

pub const EXAMPLE_SCRIPT: &str = r#"--[[

No script loaded. Here is an example script (not running):

]]--

require("snemulator_api")  -- Make sure the API is loaded for LSP autocomplete.
                           -- This import will be ignored by the emulator.

local bg_mode = 0

Log("Watchpoint script starting!") -- Use provided Log function to output at the debug level.

-- Reference api to see what callbacks are available.
function OnFrame()
    Log("Frame: " .. core.meta.frame)
    return ACTION.Continue         -- All callbacks return an Action (see api for available values)
end                                -- The script will not run and will cause the emulator to halt if
                                   -- this condition is not met, or if an error occurs.

function OnInterrupt(kind)
    if kind == CONSTS.interrupts.RESET then  -- Make use of provided CONSTS for interrupt
        instr_count = 0                      -- kinds and MMIO reg addresses.
        bg_mode = core.ppu.bg_mode
    end

    return ACTION.Continue
end

function OnMemoryRead(addr, value)
    if addr == CONSTS.mmio.BGMODE and core.ppu.bg_mode ~= bg_mode then
        bg_mode = core.ppu.bg_mode
        Log("Set bg mode to " .. bg_mode)
        return ACTION.Break
    end

    return ACTION.Continue
end"#;