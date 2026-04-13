require("snemulator_api")

local instr_count = 0
local bg_mode = 0

function OnFrame()
    Log("Frame: " .. core.meta.frame .. ", instr_cnt = " .. instr_count .. ", PC & 0xFF = " .. (core.cpu.pc & 0xFF))

    local ch = core.dma[0]

    Log("DMA0: b_to_a = " .. (ch.b_to_a and 1 or 0))

    if core.meta.frame == 300 then
        Log("Unregistering OnInstruction and OnMemoryRead hooks")
        _G.OnInstruction = nil
        _G.OnMemoryRead = nil
    end

    return ACTION.Continue
end

function OnInterrupt(kind)
    if kind == CONSTS.interrupts.RESET then
        instr_count = 0
        bg_mode = core.ppu.bg_mode
    end

    return ACTION.Continue
end

function OnInstruction()
    instr_count = instr_count + 1
    return ACTION.Continue
end

function OnMemoryRead(addr, value)
    if addr == CONSTS.mmio.BGMODE and core.ppu.bg_mode ~= bg_mode then
        bg_mode = core.ppu.bg_mode
        Log("Set bg mode to " .. bg_mode)
        return ACTION.Break
    end

    return ACTION.Continue
end