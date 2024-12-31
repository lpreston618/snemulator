// HELPER FUNCTIONS

use super::{scpu::Flag, Cpu65c816};



// void Cpu65C816::m16BIT(Address addressLo, Address addressHi) {
//     uint16_t result = accReg & readWord(addressLo, addressHi);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }

// void Cpu65C816::aBMI(Address address) {
//     if (isFlagSet(Flag::FlagN)) {
//         prgCnt = address.bankAddress;
//         branchTaken = true;
//     }
// }

// void Cpu65C816::aBNE(Address address) {
//     if (!isFlagSet(Flag::FlagZ)) {
//         prgCnt = address.bankAddress;
//         branchTaken = true;
//     }
// }

// void Cpu65C816::aBPL(Address address) {
//     if (!isFlagSet(Flag::FlagN)) {
//         prgCnt = address.bankAddress;
//         branchTaken = true;
//     }
// }

// void Cpu65C816::aBRA(Address address) {
//     prgCnt = address.bankAddress;
//     branchTaken = true;
// }

// void Cpu65C816::nBRK() {
//     nPush8(prgBank);
//     nPush16(prgCnt + 1); // push the address of the brk instruction + 2 (1 has already been added to pc)
//     nPush8(status);
//     setFlag(Flag::FlagI);

//     const Address N_BRK_VECTOR_LO = Address(0, 0xFFE6);
//     const Address N_BRK_VECTOR_HI = Address(0, 0xFFE7);

//     prgCnt = readWord(N_BRK_VECTOR_LO, N_BRK_VECTOR_HI);
// }
// void Cpu65C816::eBRK() {
//     nPush16(prgCnt + 1); // push the address of the brk instruction + 2 (1 has already been added to pc)
//     nPush8(status);
//     setFlag(Flag::FlagI);

//     const Address E_BRK_VECTOR_LO = Address(0, 0xFFFE);
//     const Address E_BRK_VECTOR_HI = Address(0, 0xFFFF);

//     prgCnt = readWord(E_BRK_VECTOR_LO, E_BRK_VECTOR_HI);
// }

// void Cpu65C816::aBVC(Address address) {
//     if (!isFlagSet(Flag::FlagV)) {
//         prgCnt = address.bankAddress;
//         branchTaken = true;
//     }
// }

// void Cpu65C816::aBVS(Address address) {
//     if (!isFlagSet(Flag::FlagV)) {
//         prgCnt = address.bankAddress;
//         branchTaken = true;
//     }
// }

// void Cpu65C816::aCLC() { clearFlag(Flag::FlagC); }

// void Cpu65C816::aCLD() { clearFlag(Flag::FlagD); }

// void Cpu65C816::aCLI() { clearFlag(Flag::FlagI); }

// void Cpu65C816::aCLV() { clearFlag(Flag::FlagV); }

// void Cpu65C816::m8CMP(Address address) {
//     uint8_t data = read(address);
//     uint8_t result = accReg - data;

//     self.set_flag_to_bool(Flag::FlagC, result > accReg);
//     self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }
// void Cpu65C816::m16CMP(Address addressLo, Address addressHi) {
//     uint16_t data = readWord(addressLo, addressHi);
//     uint16_t result = accReg - data;

//     self.set_flag_to_bool(Flag::FlagC, result > accReg);
//     self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }

// void Cpu65C816::nCOP(Address address) {
//     (void)read(address); // read is discarded here

//     nPush8(prgBank);
//     nPush16(prgCnt); // push the address of the COP instruction + 2 (2 has already been added to pc)
//     nPush8(status);
//     setFlag(Flag::FlagI);

//     const Address N_COP_VECTOR_LO = Address(0, 0xFFE4);
//     const Address N_COP_VECTOR_HI = Address(0, 0xFFE5);

//     prgCnt = readWord(N_COP_VECTOR_LO, N_COP_VECTOR_HI);
// }
// void Cpu65C816::eCOP(Address address) {
//     (void)read(address); // read is discarded here

//     nPush16(prgCnt); // push the address of the COP instruction + 2 (2 has already been added to pc)
//     nPush8(status);
//     setFlag(Flag::FlagI);

//     const Address E_COP_VECTOR_LO = Address(0, 0xFFF4);
//     const Address E_COP_VECTOR_HI = Address(0, 0xFFF5);

//     prgCnt = readWord(E_COP_VECTOR_LO, E_COP_VECTOR_HI);
// }

// void Cpu65C816::x8CPX(Address address) {
//     uint8_t data = read(address);
//     uint8_t result = xReg - data;

//     self.set_flag_to_bool(Flag::FlagC, result > xReg);
//     self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }
// void Cpu65C816::x16CPX(Address addressLo, Address addressHi) {
//     uint16_t data = readWord(addressLo, addressHi);
//     uint16_t result = xReg - data;

//     self.set_flag_to_bool(Flag::FlagC, result > xReg);
//     self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }

// void Cpu65C816::x8CPY(Address address) {
//     uint8_t data = read(address);
//     uint8_t result = yReg - data;

//     self.set_flag_to_bool(Flag::FlagC, result > yReg);
//     self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }
// void Cpu65C816::x16CPY(Address addressLo, Address addressHi) {
//     uint16_t data = readWord(addressLo, addressHi);
//     uint16_t result = yReg - data;

//     self.set_flag_to_bool(Flag::FlagC, result > yReg);
//     self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }

// void Cpu65C816::m8DECAcc() {
//     accLo--;

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }
// void Cpu65C816::m16DECAcc() {
//     accReg--;

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }
// void Cpu65C816::m8DECMem(Address address) {
//     uint8_t result = read(address) - 1;

//     write(address, result);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }
// void Cpu65C816::m16DECMem(Address addressLo, Address addressHi) {
//     uint16_t result = readWord(addressLo, addressHi) - 1;

//     write(addressLo, result & 0xFF);
//     write(addressHi, result >> 8);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }

// void Cpu65C816::x8DEX() {
//     xRegLo--;

//     self.set_flag_to_bool(Flag::FlagN, xReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);
// }
// void Cpu65C816::x16DEX() {
//     xReg--;

//     self.set_flag_to_bool(Flag::FlagN, xReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);
// }

// void Cpu65C816::x8DEY() {
//     yRegLo--;

//     self.set_flag_to_bool(Flag::FlagN, yReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yReg == 0);
// }
// void Cpu65C816::x16DEY() {
//     yReg--;

//     self.set_flag_to_bool(Flag::FlagN, yReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yReg == 0);
// }

// void Cpu65C816::m8EOR(Address address) {
//     uint8_t result = accReg ^ read(address);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
    
//     accReg = result;
// }
// void Cpu65C816::m16EOR(Address addressLo, Address addressHi) {
//     uint16_t result = accReg ^ readWord(addressLo, addressHi);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
    
//     accReg = result;
// }

// void Cpu65C816::m8INCAcc() {
//     accLo++;

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }
// void Cpu65C816::m16INCAcc() {
//     accReg++;

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }
// void Cpu65C816::m8INCMem(Address address) {
//     uint8_t result = read(address) + 1;

//     write(address, result);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }
// void Cpu65C816::m16INCMem(Address addressLo, Address addressHi) {
//     uint16_t result = readWord(addressLo, addressHi) + 1;

//     write(addressLo, result & 0xFF);
//     write(addressHi, result >> 8);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }

// void Cpu65C816::x8INX() {
//     xRegLo++;

//     self.set_flag_to_bool(Flag::FlagN, xReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);
// }
// void Cpu65C816::x16INX() {
//     xReg++;

//     self.set_flag_to_bool(Flag::FlagN, xReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);    
// }

// void Cpu65C816::x8INY() {
//     yRegLo++;

//     self.set_flag_to_bool(Flag::FlagN, yReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yReg == 0);
// }
// void Cpu65C816::x16INY() {
//     yReg++;

//     self.set_flag_to_bool(Flag::FlagN, yReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yReg == 0);
// }

// void Cpu65C816::aJMP(Address address) {
//     prgCnt = address.bankAddress;
//     branchTaken = true;
// }

// void Cpu65C816::nJSR(Address address) {
//     nPush16(prgCnt - 1); // push the address of the brk instruction + 2 (3 has already been added to pc)
//     prgCnt = address.bankAddress;
//     branchTaken = true;
// }
// void Cpu65C816::eJSR(Address address) {
//     ePush16(prgCnt - 1); // push the address of the brk instruction + 2 (3 has already been added to pc)
//     prgCnt = address.bankAddress;
//     branchTaken = true;
// }

// void Cpu65C816::nJSL(Address address) {
//     nPush8(prgBank);
//     nPush16(prgCnt - 1); // push the address of the JSL instruction + 3 (4 has already been added to pc)

//     prgCnt = address.bankAddress;
//     prgBank = address.bank;

//     branchTaken = true;
// }
// void Cpu65C816::eJSL(Address address) {
//     ePush8(prgBank);
//     ePush16(prgCnt - 1); // push the address of the JSL instruction + 3 (4 has already been added to pc)

//     prgCnt = address.bankAddress;
//     prgBank = address.bank;

//     branchTaken = true;
// }

// void Cpu65C816::m8LDA(Address address) {
//     accReg = read(address);

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }
// void Cpu65C816::m16LDA(Address addressLo, Address addressHi) {
//     accReg = readWord(addressLo, addressHi);

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }

// void Cpu65C816::x8LDX(Address address) {
//     xReg = read(address);

//     self.set_flag_to_bool(Flag::FlagN, xReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);
// }
// void Cpu65C816::x16LDX(Address addressLo, Address addressHi) {
//     xReg = readWord(addressLo, addressHi);

//     self.set_flag_to_bool(Flag::FlagN, xReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);
// }

// void Cpu65C816::x8LDY(Address address) {
//     yReg = read(address);

//     self.set_flag_to_bool(Flag::FlagN, yReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yReg == 0);
// }
// void Cpu65C816::x16LDY(Address addressLo, Address addressHi) {
//     yReg = readWord(addressLo, addressHi);

//     self.set_flag_to_bool(Flag::FlagN, yReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yReg == 0);
// }

// void Cpu65C816::m8LSRAcc() {
//     self.set_flag_to_bool(Flag::FlagC, accLo & 0x01);
//     clearFlag(Flag::FlagN); // 0 shifted into high bit, result always positive

//     accLo >>= 1;

//     self.set_flag_to_bool(Flag::FlagZ, accLo == 0);
// }
// void Cpu65C816::m16LSRAcc() {
//     self.set_flag_to_bool(Flag::FlagC, accReg & 0x0001);
//     clearFlag(Flag::FlagN); // 0 shifted into high bit, result always positive

//     accReg >>= 1;

//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }
// void Cpu65C816::m8LSRMem(Address address) {
//     uint8_t data = read(address);
//     uint8_t result = data >> 1;

//     self.set_flag_to_bool(Flag::FlagC, data & 0x01);
//     clearFlag(Flag::FlagN); // 0 shifted into high bit, result always positive

//     write(address, result);

//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }
// void Cpu65C816::m16LSRMem(Address addressLo, Address addressHi) {
//     uint16_t data = readWord(addressLo, addressHi);
//     uint16_t result = data >> 1;

//     self.set_flag_to_bool(Flag::FlagC, data & 0x0001);
//     clearFlag(Flag::FlagN); // 0 shifted into high bit, result always positive

//     write(addressLo, result & 0xFF);
//     write(addressHi, result >> 8);

//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }

// void Cpu65C816::aMVN(Address srcAddress, Address destAddress) {
//     // Idx registers incremented in block move negative (it's backwards, I know)
//     // "Negative" actually refers to where the destination address is relative
//     // to the source address.
//     xReg++;
//     yReg++;

//     write(destAddress, read(srcAddress));

//     accReg--;

//     // This instruction essensially jumps to itself until it has moved accReg + 1
//     // bytes. accReg will be 0xFFFF after this instruction. The addressing mode
//     // of this instruction is always BlockMove, so the instruction is always 3 bytes.
//     if (accReg != 0xFFFF) {
//         prgCnt -= 3;
//     } else {
//         // Finished moving data
//         dataBank = destAddress.bank; // overwrites the dataBank register with the destination bank when finished
//     }
// }

// void Cpu65C816::aMVP(Address srcAddress, Address destAddress) {
//     // Idx registers decremented in block move positive (it's backwards, I know)
//     // "Positive" actually refers to where the destination address is relative
//     // to the source address.
//     xReg--;
//     yReg--;

//     write(destAddress, read(srcAddress));

//     accReg--;

//     // This instruction essensially jumps to itself until it has moved accReg + 1
//     // bytes. accReg will be 0xFFFF after this instruction. The addressing mode
//     // of this instruction is always BlockMove, so the instruction is always 3 bytes.
//     if (accReg != 0xFFFF) {
//         prgCnt -= 3;
//     } else {
//         // Finished moving data
//         dataBank = destAddress.bank; // overwrites the dataBank register with the destination bank when finished
//     }
// }

// void Cpu65C816::aNOP() {}

// void Cpu65C816::m8ORA(Address address) {
//     uint8_t result = accReg | read(address);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
    
//     accReg = result;
// }
// void Cpu65C816::m16ORA(Address addressLo, Address addressHi) {
//     uint16_t result = accReg | readWord(addressLo, addressHi);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
    
//     accReg = result;
// }

// void Cpu65C816::nPEX(Address address) {
//     nPush16(address.bankAddress);
// }
// void Cpu65C816::ePEX(Address address) {
//     ePush16(address.bankAddress);
// }

// void Cpu65C816::m8PHA() {
//     nPush8(accLo);
// }
// void Cpu65C816::m16PHA() {
//     nPush16(accReg);
// }
// void Cpu65C816::ePHA() {
//     ePush8(accLo);
// }

// void Cpu65C816::nPHB() {
//     nPush8(dataBank);
// }
// void Cpu65C816::ePHB() {
//     ePush8(dataBank);
// }

// void Cpu65C816::nPHD() {
//     nPush16(directPage);
// }
// void Cpu65C816::ePHD() {
//     ePush16(directPage);
// }

// void Cpu65C816::nPHK() {
//     nPush8(prgBank);
// }
// void Cpu65C816::ePHK() {
//     ePush8(prgBank);
// }

// void Cpu65C816::nPHP() {
//     nPush8(status);
// }
// void Cpu65C816::ePHP() {
//     ePush8(status);
// }

// void Cpu65C816::x8PHX() {
//     nPush8(xRegLo);
// }
// void Cpu65C816::x16PHX() {
//     nPush16(xReg);
// }
// void Cpu65C816::ePHX() {
//     ePush8(xRegLo);
// }

// void Cpu65C816::x8PHY() {
//     nPush8(yRegLo);
// }
// void Cpu65C816::x16PHY() {
//     nPush16(yReg);
// }
// void Cpu65C816::ePHY() {
//     ePush8(yRegLo);
// }

// void Cpu65C816::m8PLA() {
//     accReg = nPop8();

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }
// void Cpu65C816::m16PLA() {
//     accReg = nPop16();

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }
// void Cpu65C816::ePLA() {
//     accReg = ePop8();

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }

// void Cpu65C816::nPLB() {
//     dataBank = nPop8();

//     self.set_flag_to_bool(Flag::FlagN, dataBank & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, dataBank == 0);
// }
// void Cpu65C816::ePLB() {
//     dataBank = ePop8();

//     self.set_flag_to_bool(Flag::FlagN, dataBank & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, dataBank == 0);
// }

// void Cpu65C816::nPLD() {
//     directPage = nPop16();

//     self.set_flag_to_bool(Flag::FlagN, directPage & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, directPage == 0);
// }
// void Cpu65C816::ePLD() {
//     directPage = ePop16();

//     self.set_flag_to_bool(Flag::FlagN, directPage & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, directPage == 0);
// }

// void Cpu65C816::nPLP() {
//     status = nPop8();
// }
// void Cpu65C816::ePLP() {
//     status = ePop8() | Flag::FlagM | Flag::FlagX; // Emulation mode forces these flags on
// }

// void Cpu65C816::x8PLX() {
//     xReg = nPop8();

//     self.set_flag_to_bool(Flag::FlagN, xReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);
// }
// void Cpu65C816::x16PLX() {
//     xReg = nPop16();

//     self.set_flag_to_bool(Flag::FlagN, xReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);
// }
// void Cpu65C816::ePLX() {
//     xReg = ePop8();

//     self.set_flag_to_bool(Flag::FlagN, xReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);
// }

// void Cpu65C816::x8PLY() {
//     yReg = nPop8();

//     self.set_flag_to_bool(Flag::FlagN, yReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yReg == 0);
// }
// void Cpu65C816::x16PLY() {
//     yReg = nPop16();

//     self.set_flag_to_bool(Flag::FlagN, yReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yReg == 0);
// }
// void Cpu65C816::ePLY() {
//     yReg = ePop8();

//     self.set_flag_to_bool(Flag::FlagN, yReg & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yReg == 0);
// }

// void Cpu65C816::nREP(Address address) {
//     status &= ~read(address);
// }
// void Cpu65C816::eREP(Address address) {
//     status &= ~read(address);
//     status |= Flag::FlagM | Flag::FlagX;
// }

// void Cpu65C816::m8ROLAcc() {
//     uint8_t c = isFlagSet(Flag::FlagC);
//     self.set_flag_to_bool(Flag::FlagC, accLo & 0x80);

//     accLo <<= 1;
//     accLo |= c;

//     self.set_flag_to_bool(Flag::FlagN, accLo & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accLo == 0);
// }
// void Cpu65C816::m16ROLAcc() {
//     uint16_t c = isFlagSet(Flag::FlagC);
//     self.set_flag_to_bool(Flag::FlagC, accReg & 0x8000);

//     accReg <<= 1;
//     accReg |= c;

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }
// void Cpu65C816::m8ROLMem(Address address) {
//     uint8_t data = read(address);
//     uint8_t result = (data << 1) | isFlagSet(Flag::FlagC);

//     self.set_flag_to_bool(Flag::FlagC, data & 0x80);

//     write(address, result);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }
// void Cpu65C816::m16ROLMem(Address addressLo, Address addressHi) {
//     uint16_t data = readWord(addressLo, addressHi);
//     uint16_t result = (data << 1) | isFlagSet(Flag::FlagC);

//     self.set_flag_to_bool(Flag::FlagC, data & 0x8000);

//     write(addressLo, result & 0xFF);
//     write(addressHi, result >> 8);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }

// void Cpu65C816::m8RORAcc() {
//     uint8_t c = isFlagSet(Flag::FlagC);
//     self.set_flag_to_bool(Flag::FlagC, accLo & 0x01);

//     accLo >>= 1;
//     accLo |= c << 7;

//     self.set_flag_to_bool(Flag::FlagN, accLo & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accLo == 0);
// }
// void Cpu65C816::m16RORAcc() {
//     uint16_t c = isFlagSet(Flag::FlagC);
//     self.set_flag_to_bool(Flag::FlagC, accReg & 0x0001);

//     accReg >>= 1;
//     accReg |= c << 15;

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }
// void Cpu65C816::m8RORMem(Address address) {
//     uint8_t c = isFlagSet(Flag::FlagC);

//     uint8_t data = read(address);
//     uint8_t result = (data >> 1) | (c << 7);

//     self.set_flag_to_bool(Flag::FlagC, data & 0x01);

//     write(address, result);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }
// void Cpu65C816::m16RORMem(Address addressLo, Address addressHi) {
//     uint16_t c = isFlagSet(Flag::FlagC);

//     uint16_t data = readWord(addressLo, addressHi);
//     uint16_t result = (data >> 1) | (c << 15);

//     self.set_flag_to_bool(Flag::FlagC, data & 0x0001);

//     write(addressLo, result & 0xFF);
//     write(addressHi, result >> 8);

//     self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }

// void Cpu65C816::nRTI() {
//     status = nPop8();
//     prgCnt = nPop16();
//     prgBank = nPop8();
// }
// void Cpu65C816::eRTI() {
//     status = ePop8() | Flag::FlagM | Flag::FlagX;
//     prgCnt = ePop16();
// }

// void Cpu65C816::nRTL() {
//     prgCnt = nPop16() + 1;
//     prgBank = nPop8();
// }
// void Cpu65C816::eRTL() {
//     prgCnt = ePop16() + 1;
//     prgBank = ePop8();
// }

// void Cpu65C816::nRTS() {
//     prgCnt = nPop16() + 1;
// }
// void Cpu65C816::eRTS() {
//     prgCnt = ePop16() + 1;
// }

// void Cpu65C816::m8SBC(Address address) {
//     uint8_t data = read(address);
//     uint8_t result;

//     if (isFlagSet(Flag::FlagD)) {
//         // One's place, ten's place
//         uint8_t oPlace, tPlace;
//         bool borrow = !isFlagSet(Flag::FlagC);

//         oPlace = bcdSubDigit(accReg&0x0F, data&0x0F, &borrow);
//         tPlace = bcdSubDigit((accReg >> 4)&0x0F, (data >> 4)&0x0F, &borrow);

//         result = (tPlace << 4) | oPlace;

//         self.set_flag_to_bool(Flag::FlagC, borrow);
//     } else {
//         result = accReg - data - !isFlagSet(Flag::FlagC);

//         self.set_flag_to_bool(Flag::FlagC, result > accReg);
//     }

//     self.set_flag_to_bool(Flag::FlagN, result & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
//     self.set_flag_to_bool(Flag::FlagV, (~(accReg ^ data))&(data ^ result)&0x80);

//     accReg = result;
// }
// void Cpu65C816::m16SBC(Address addressLo, Address addressHi) {
//     uint16_t data = readWord(addressLo, addressHi);
//     uint16_t result;

//     if (isFlagSet(Flag::FlagD)) {
//         // One's place, ten's place, hundred's place, thousand's place
//         uint16_t oPlace, tPlace, hPlace, thPlace;
//         bool borrow = !isFlagSet(Flag::FlagC);

//         oPlace = bcdSubDigit(accReg&0x0F, data&0x0F, &borrow);
//         tPlace = bcdSubDigit((accReg >> 4)&0x0F, (data >> 4)&0x0F, &borrow);
//         hPlace = bcdSubDigit((accReg >> 8)&0x0F, (data >> 8)&0x0F, &borrow);
//         thPlace = bcdSubDigit((accReg >> 12)&0x0F, (data >> 12)&0x0F, &borrow);

//         result = (thPlace << 12) | (hPlace << 8) | (tPlace << 4) | oPlace;

//         self.set_flag_to_bool(Flag::FlagC, borrow);
//     } else {
//         result = accReg - data - !isFlagSet(Flag::FlagC);

//         self.set_flag_to_bool(Flag::FlagC, result > accReg);
//     }

//     self.set_flag_to_bool(Flag::FlagN, result & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
//     self.set_flag_to_bool(Flag::FlagV, (~(accReg ^ data))&(data ^ result)&0x8000);

//     accReg = result;
// }

// void Cpu65C816::aSEC() { setFlag(Flag::FlagC); }

// void Cpu65C816::aSED() { setFlag(Flag::FlagD); }

// void Cpu65C816::aSEI() { setFlag(Flag::FlagI); }

// void Cpu65C816::aSEP(Address address) {
//     status |= read(address);
// }

// void Cpu65C816::m8STA(Address address) {
//     write(address, accLo);
// }
// void Cpu65C816::m16STA(Address addressLo, Address addressHi) {
//     write(addressLo, accLo);
//     write(addressHi, accHi);
// }

// void Cpu65C816::aSTP() { stopped = true; }

// void Cpu65C816::x8STX(Address address) {
//     write(address, xRegLo);
// }
// void Cpu65C816::x16STX(Address addressLo, Address addressHi) {
//     write(addressLo, xRegLo);
//     write(addressHi, xRegHi);
// }

// void Cpu65C816::x8STY(Address address) {
//     write(address, yRegLo);
// }
// void Cpu65C816::x16STY(Address addressLo, Address addressHi) {
//     write(addressLo, yRegLo);
//     write(addressHi, yRegHi);
// }

// void Cpu65C816::m8STZ(Address address) {
//     write(address, 0);
// }
// void Cpu65C816::m16STZ(Address addressLo, Address addressHi) {
//     write(addressLo, 0);
//     write(addressHi, 0);
// }

// void Cpu65C816::x8TAX() {
//     xRegLo = accLo;

//     self.set_flag_to_bool(Flag::FlagN, xRegLo & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xRegLo == 0);
// }
// void Cpu65C816::x16TAX() {
//     xReg = accReg;

//     self.set_flag_to_bool(Flag::FlagN, xReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);
// }

// void Cpu65C816::x8TAY() {
//     yRegLo = accLo;

//     self.set_flag_to_bool(Flag::FlagN, yRegLo & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yRegLo == 0);
// }
// void Cpu65C816::x16TAY() {
//     yReg = accReg;

//     self.set_flag_to_bool(Flag::FlagN, yReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yReg == 0);
// }

// void Cpu65C816::aTCD() {
//     directPage = accReg;

//     self.set_flag_to_bool(Flag::FlagN, directPage & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, directPage == 0);
// }

// void Cpu65C816::nTCS() { stkPtr = accReg; }
// void Cpu65C816::eTCS() { stkPtrLo = accLo; }

// void Cpu65C816::aTDC() {
//     accReg = directPage;

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }

// void Cpu65C816::m8TRB(Address address) {
//     uint8_t result = read(address) & (~accLo);

//     write(address, result);

//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }
// void Cpu65C816::m16TRB(Address addressLo, Address addressHi) {
//     uint16_t result = readWord(addressLo, addressHi) & (~accReg);

//     write(addressLo, result & 0xFF);
//     write(addressHi, result >> 8);

//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }

// void Cpu65C816::m8TSB(Address address) {
//     uint8_t result = read(address) | accLo;

//     write(address, result);

//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }
// void Cpu65C816::m16TSB(Address addressLo, Address addressHi) {
//     uint16_t result = readWord(addressLo, addressHi) | accReg;

//     write(addressLo, result & 0xFF);
//     write(addressHi, result >> 8);

//     self.set_flag_to_bool(Flag::FlagZ, result == 0);
// }

// void Cpu65C816::m8TSC() {
//     accReg = stkPtr & 0xFF; // 8-bit mode forces accHi to 0

//     clearFlag(Flag::FlagN); // the value transfered is always positive
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }
// void Cpu65C816::m16TSC() {
//     accReg = stkPtr;

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }
// void Cpu65C816::eTSC() {
//     accReg = stkPtr & 0xFF; // Emulation mode forces accHi to 0

//     clearFlag(Flag::FlagN); // the value transfered is always positive
//     clearFlag(Flag::FlagZ); // the value transfered is always non-zero
// }

// void Cpu65C816::x8TSX() {
//     xReg = stkPtr & 0xFF; // 8-bit mode forces xRegHi to 0

//     clearFlag(Flag::FlagN); // the value transfered is always positive
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);
// }
// void Cpu65C816::x16TSX() {
//     xReg = stkPtr;

//     self.set_flag_to_bool(Flag::FlagN, xReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);
// }
// void Cpu65C816::eTSX() {
//     xReg = stkPtr & 0xFF; // Emulation mode forces xRegHi to 0

//     clearFlag(Flag::FlagN); // the value transfered is always positive
//     clearFlag(Flag::FlagZ); // the value transfered is always non-zero
// }

// void Cpu65C816::m8TXA() {
//     accLo = xRegLo;

//     self.set_flag_to_bool(Flag::FlagN, accLo & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accLo == 0);
// }
// void Cpu65C816::m16TXA() {
//     accReg = xReg;

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }

// void Cpu65C816::nTXS() { stkPtr = xReg; }
// void Cpu65C816::eTXS() { stkPtrLo = xRegLo; }

// void Cpu65C816::x8TXY() {
//     yRegLo = xRegLo;

//     self.set_flag_to_bool(Flag::FlagN, yRegLo & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yRegLo == 0);
// }
// void Cpu65C816::x16TXY() {
//     yReg = xReg;

//     self.set_flag_to_bool(Flag::FlagN, yReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, yReg == 0);
// }

// void Cpu65C816::m8TYA() {
//     accLo = yRegLo;

//     self.set_flag_to_bool(Flag::FlagN, accLo & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accLo == 0);
// }
// void Cpu65C816::m16TYA() {
//     accReg = yReg;

//     self.set_flag_to_bool(Flag::FlagN, accReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, accReg == 0);
// }

// void Cpu65C816::x8TYX() {
//     xRegLo = yRegLo;

//     self.set_flag_to_bool(Flag::FlagN, xRegLo & 0x80 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xRegLo == 0);
// }
// void Cpu65C816::x16TYX() {
//     xReg = yReg;

//     self.set_flag_to_bool(Flag::FlagN, xReg & 0x8000 != 0);
//     self.set_flag_to_bool(Flag::FlagZ, xReg == 0);
// }

// void Cpu65C816::aWAI() { awaitingInterrupt = true; }

// void Cpu65C816::aWDM() {}

// void Cpu65C816::m8XBA() {
//     accReg = 0; // Has the effect of zeroing the accumulator in 8-bit mode (i think)
// }
// void Cpu65C816::m16XBA() {
//     uint8_t temp = accLo;
//     accLo = accHi;
//     accHi = temp;
// }

// void Cpu65C816::aXCE() {
//     bool newEmulationMode = isFlagSet(Flag::FlagC);
//     self.set_flag_to_bool(Flag::FlagC, emulationMode);
//     emulationMode = newEmulationMode;

//     if (emulationMode) {
//         accHi = 0;
//         xRegHi = 0;
//         yRegHi = 0;
//         stkPtrHi = 0x01;
//     }
// }