require("snemulator_api")

function OnDMATransfer(ch, src, dst, val)
    if (val == 0x18 or val == 0x58) and (dst == 0x2118 or dst == 0x2119) then
        Log(string.format("DMA transfer: ch=%d, src=0x%06X, dst=0x%04X, val=0x%02X, vram_addr = 0x%04X", ch, src, dst, val, core.ppu.vram_addr))
        
        if core.ppu.vram_addr == 0x3D9A or core.ppu.vram_addr == 0x3D9B then
            Log("VRAM address is 0x3D9A or 0x3D9B during DMA transfer!")
            control:Break()
        end
    end
end

function OnMemoryWrite(addr, val)
    if addr == 0x2118 or addr == 0x2119 then
        if core.ppu.vram_addr == 0x3D9A or core.ppu.vram_addr == 0x3D9B then
            Log("VRAM address is 0x3D9A or 0x3D9B, data = 0x" .. string.format("%02X", val))
            control:Break()
        end
    end
end