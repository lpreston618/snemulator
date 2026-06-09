require("snemulator_api")

function OnMemoryWrite(address, value)
    if address == 0x10 and value == 0xc2 then
        Log("Test num: " .. string.format("%02X", value))
        control:Break()
    end
end