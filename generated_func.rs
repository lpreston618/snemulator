fn asl_mem_m8(cpu: &mut Cpu65c816, address: u32) {
    let data = cpu.read8(address);
    let result = data << 1;
    
    cpu.set_flag_to_bool(Flag::FlagC, data & 0x80 != 0);
    
    write(address, result);
    
    set_flag_n!(result, u8);
    set_flag_z!(result);
    
}

fn asl_mem_m16(cpu: &mut Cpu65c816, address_lo: u32, address_hi: u32) {
    let data = cpu.read16(address_lo, address_hi);
    let result = data << 1;
    
    cpu.set_flag_to_bool(Flag::FlagC, data & 0x80 != 0);
    
    write(address, result);
    
    set_flag_n!(result, u8);
    set_flag_z!(result);
    
}

