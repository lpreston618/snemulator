require("app.src.debug.watchpoints.snemulator_api")

function OnInstruction()
    if core.cpu.prg0 == 0 then
        Log(string.format("Break instruction at 0x%02x%04x", core.cpu.pb, core.cpu.pc))
        control:Break()
    end
end
