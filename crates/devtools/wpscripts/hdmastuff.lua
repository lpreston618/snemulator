require("app.src.debug.watchpoints.snemulator_api")

function OnHDMATransfer(channel, src_addr, dst_addr, val)
    Log(string.format("c:%d %04X -> %04X :: %0X", channel, src_addr, dst_addr, val))
end
