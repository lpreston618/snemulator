require("snemulator_api")

function OnMemoryWrite(addr, value)
    if addr % 0x10000 == 0x2103 and value % 2 == 1 then
        Log("Write to 0x2103 with " .. value)
    end
end

-- function OnHDMATransfer(channel, src_addr, dst_addr, value)
--     -- if dst_addr >= 0x2123 and dst_addr <= 0x212B then
--     --     Log(string.format("HDMA (%d): src = %06X, dst = %06X, val = %02X", channel, src_addr, dst_addr, value))
--     -- end
--     Log(string.format("HDMA (%d): src = %06X, dst = %06X, val = %02X", channel, src_addr, dst_addr, value))
-- end