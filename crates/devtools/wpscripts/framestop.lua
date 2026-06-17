require("snemulator_api")

function OnFrame()
    if core.meta.frame == 655 then
        Log("Stopping on frame 655")
        control:Break()
    end
end
