require("app.src.debug.watchpoints.snemulator_api")

function OnInstruction()
    if core.cpu.prg0 == 0x28 and core.meta.frame > 275 then
        Log(string.format("plp instruction at 0x%02x%04x, sp before:0x%04X", core.cpu.pb, core.cpu.pc, core.cpu.sp))
        control:Break()
    end
end

function OnInterrupt(kind)
    Log(string.format("%s interrupt at pc=0x%02x%04x; sp=%04x", kind, core.cpu.pb, core.cpu.pc, core.cpu.sp))
end
