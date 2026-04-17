require("snemulator_api")

local function enable_ff()
    control:SetAudioEnabled(false)
    control:SetVideoEnabled(false)
    control:SetFastForwardEnabled(true)
end

local function disable_ff()
    control:SetAudioEnabled(true)
    control:SetVideoEnabled(true)
    control:SetFastForwardEnabled(false)
end

local ff_enabled = false

function OnFrame()
    if core.meta.frame % 300 == 0 then
        if ff_enabled then
            disable_ff()
        else
            enable_ff()
        end
        ff_enabled = not ff_enabled
    end
end