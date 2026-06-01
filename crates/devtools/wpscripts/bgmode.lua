require("app.src.debug.watchpoints.snemulator_api")

function OnMemoryWrite(a, v)
    if a == CONSTS.mmio.BGMODE then
        Log("Set BGMODE to " .. v)
        -- control:Break()
    end
end
