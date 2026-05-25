require("snemulator_api")

function OnMemoryRead(addr, value)
    if addr >= 0x07f7db and addr <= 0x07fc36 then
        Log("Read circle coord")
        control:Break()
    end
end

function OnMemoryWrite(addr, value)
    -- if addr >= 0x2123 and addr <= 0x212B then
    --     Log(string.format("MMIO[$%04X] = %02X", addr, value))
    -- end
    if addr == CONSTS.mmio.A1B7 then
        Log("Wrote A1B7 with " .. value)
        control:Break()
    end
end

-- function OnHDMATransfer(channel, src_addr, dst_addr, value)
--     -- if dst_addr >= 0x2123 and dst_addr <= 0x212B then
--     --     Log(string.format("HDMA (%d): src = %06X, dst = %06X, val = %02X", channel, src_addr, dst_addr, value))
--     -- end
--     Log(string.format("HDMA (%d): src = %06X, dst = %06X, val = %02X", channel, src_addr, dst_addr, value))
-- end