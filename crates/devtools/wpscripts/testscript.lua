require("snemulator_api")

local ff_speed = 4.0
local count = 0
local ff_enabled = false

local function enable_ff()
    Log("ff enabled")
    control:SetAudioEnabled(false)
    -- control:SetVideoEnabled(false)
    control:SetFastForwardEnabled(true)
    ff_enabled = true
end

local function disable_ff()
    Log("ff disabled")
    control:SetAudioEnabled(true)
    -- control:SetVideoEnabled(true)
    control:SetFastForwardEnabled(false)
    ff_enabled = false
end

disable_ff()

control:SetFastForwardSpeed(ff_speed)
Log("Fast forward speed set to " .. tostring(ff_speed))

local ppu = core.ppu

-- function OnFrame()
--     ff_enabled = control.ff_en

--     if ff_enabled then
--         if count % 1200 == 0 then
--             disable_ff()
--             count = 0
--             ff_enabled = false
--         end
--     else
--         if count % 300 == 0 then
--             enable_ff()
--             count = 0
--             ff_enabled = true
--         end
--     end

--     count = count + 1
-- end

function OnEmulationCycle()
    if ppu.scanline == 0 and ppu.dot == 0 then
        Log("Scanline, Dot: (0, 0)")
    end
end