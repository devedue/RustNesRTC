// use crate::apu::Apu;
use crate::bus::*;
use crate::util::*;
use indexmap::IndexMap;

// For Logging: 
// use std::io::Write;
// use crate::util::hex;

pub enum FLAGS6502 {
    C = (1 << 0), // Carry Bit
    Z = (1 << 1), // Zero
    I = (1 << 2), // Disable Interrupts
    D = (1 << 3), // Decimal Mode
    B = (1 << 4), // Break
    U = (1 << 5), // Unused
    V = (1 << 6), // Overflow
    N = (1 << 7), // Negative
}

pub struct Instruction {
    pub name: String,
    pub operate: fn(&mut Cpu) -> u8,
    pub addrmode: fn(&mut Cpu) -> u8,
    pub cycles: u8,
}

impl Instruction {
    fn new_i(
        name: &str,
        operate: fn(&mut Cpu) -> u8,
        addrmode: fn(&mut Cpu) -> u8,
        cycles: u8,
    ) -> Instruction {
        return Instruction {
            name: name.to_owned(),
            operate: operate,
            addrmode: addrmode,
            cycles: cycles,
        };
    }
}

pub struct Cpu {
    pub bus: Bus,
    pub status: u8,
    pub a: u8,    // a register
    pub x: u8,    // x register
    pub y: u8,    // y register
    pub stkp: u8, // stack pointer
    pub pc: u16,  //program counter
    pub fetched: u8,
    pub addr_abs: u16,
    pub addr_rel: u16,
    pub opcode: u8,
    pub cycles: u8,
    pub lookup: Vec<Instruction>,
    pub map_asm: IndexMap<u16, String>,
}

#[allow(non_snake_case)]
impl Cpu {
    fn push_to_stack(&mut self, value: u8) {
        self.write(usize::from(0x0100 + self.stkp as u16), value);
        self.stkp = self.stkp.wrapping_sub(1);
    }

    fn pop_from_stack(&mut self) -> u8 {
        self.stkp = self.stkp.wrapping_add(1);
        let result = self.read((0x0100 as u16) + (self.stkp as u16), false);
        return result;
        // return 0;
    }

    /// Addressing Mode : Implied
    pub fn IMP(&mut self) -> u8 {
        self.fetched = self.a;
        return 0;
    }

    /// Addressing Mode : Immediate
    pub fn IMM(&mut self) -> u8 {
        self.addr_abs = self.pc;
        self.pc = self.pc + 1;
        return 0;
    }

    /// Addressing Mode: Zero Page
    pub fn ZP0(&mut self) -> u8 {
        self.addr_abs = self.pcread();
        self.addr_abs = self.addr_abs & 0x00FF;
        return 0;
    }

    /// Addressing Mode: Zero Page with X Offset
    pub fn ZPX(&mut self) -> u8 {
        self.addr_abs = self.pcread() + (self.x as u16);
        self.addr_abs = self.addr_abs & 0x00FF;
        return 0;
    }

    /// Addressing Mode: Zero Page with Y Offset
    pub fn ZPY(&mut self) -> u8 {
        self.addr_abs = self.pcread() + (self.y as u16);
        self.addr_abs = self.addr_abs & 0x00FF;
        return 0;
    }

    /// Addressing Mode: Relative
    pub fn REL(&mut self) -> u8 {
        self.addr_rel = self.pcread();
        if self.addr_rel & 0x80 > 0 {
            self.addr_rel = self.addr_rel | 0xFF00
        }
        return 0;
    }

    /// Addressing Mode: Absolute
    pub fn ABS(&mut self) -> u8 {
        let lo = self.pcread();
        let hi = self.pcread();

        self.addr_abs = (hi << 8) | lo;

        return 0;
    }

    /// Addressing Mode: Absolute with X offset
    pub fn ABX(&mut self) -> u8 {
        let lo = self.pcread();
        let hi = self.pcread();

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs = self.addr_abs.wrapping_add(self.x as u16);
        if (self.addr_abs & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }

    /// Addressing Mode: Absolute with Y offset
    pub fn ABY(&mut self) -> u8 {
        let lo = self.pcread();
        let hi = self.pcread();

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs = self.addr_abs.wrapping_add(self.y as u16);
        if (self.addr_abs & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }

    /// Addressing Mode: Indirect
    pub fn IND(&mut self) -> u8 {
        let ptr_lo = self.pcread();
        let ptr_hi = self.pcread();

        let ptr: u16 = (ptr_hi << 8) | ptr_lo;

        if ptr_lo == 0x00FF {
            let hi_addr: u16 = self.read(ptr & 0xFF00, false).into();
            let lo_addr: u16 = self.read(ptr, false).into();
            self.addr_abs = (hi_addr << 8) | lo_addr;
        } else {
            let hi_addr: u16 = self.read(ptr + 1, false).into();
            let lo_addr: u16 = self.read(ptr, false).into();
            self.addr_abs = (hi_addr << 8) | lo_addr;
        }

        return 0;
    }

    /// Addressing Mode: Indirect x offset
    pub fn IZX(&mut self) -> u8 {
        let t = self.pcread();

        let ptr = t + (self.x as u16);

        let lo: u16 = self.read(ptr & 0x00FF, false).into();
        let hi: u16 = self.read((ptr + 1) & 0x00FF, false).into();

        self.addr_abs = (hi << 8) | lo;

        return 0;
    }

    /// Addressing Mode: Indirect Y offset
    pub fn IZY(&mut self) -> u8 {
        let t = self.pcread();

        let lo: u16 = self.read(t & 0x00FF, false).into();
        let hi: u16 = self.read((t + 1) & 0x00FF, false).into();
        self.addr_abs = (hi << 8) | lo;
        self.addr_abs = self.addr_abs.wrapping_add(self.y as u16);

        if (self.addr_abs & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }

    //Opcodes

    /// Instruction: Add with carry in
    pub fn ADC(&mut self) -> u8 {
        self.fetch();
        let temp: u16 =
            (self.a as u16) + (self.fetched as u16) + (self.get_flag(FLAGS6502::C) as u16);
        self.set_flag(FLAGS6502::C, temp > 255);
        self.set_flag(FLAGS6502::Z, (temp & 0x00FF) == 0);
        self.set_flag(
            FLAGS6502::V,
            (!(self.a as u16 ^ self.fetched as u16) & (self.a as u16 ^ temp)) & 0x0080 > 0,
        );
        self.set_flag(FLAGS6502::N, temp & 0x80 > 0);
        self.a = (temp & 0x00FF) as u8;
        return 1;
    }

    /// Instruction: fetch and AND accumulator
    pub fn AND(&mut self) -> u8 {
        self.fetch();
        self.a = self.a & self.fetched;
        self.set_flag(FLAGS6502::Z, self.a == 0x00);
        self.set_flag(FLAGS6502::N, (self.a & 0x80) > 0);

        return 1;
    }

    /// Instruction: Accumulator shift left
    pub fn ASL(&mut self) -> u8 {
        self.fetch();
        let temp: u16 = (self.fetched as u16) << 1;
        self.set_flag(FLAGS6502::C, (temp & 0xFF00) > 0);
        self.set_flag(FLAGS6502::Z, (temp & 0x00FF) == 0x00);
        self.set_flag(FLAGS6502::N, (temp & 0x80) > 0);
        if self.lookup[usize::from(self.opcode)].addrmode as usize == Self::IMP as usize {
            self.a = (temp & 0x00FF) as u8;
        } else {
            self.write(usize::from(self.addr_abs), (temp & 0x00FF) as u8);
        }
        return 0;
    }

    /// Instruction: Branch if carry is clear
    pub fn BCC(&mut self) -> u8 {
        self.branchrel(self.get_flag(FLAGS6502::C) == 0);
        return 0;
    }

    /// Instruction: Branch if carry bit is set
    pub fn BCS(&mut self) -> u8 {
        self.branchrel(self.get_flag(FLAGS6502::C) == 1);
        return 0;
    }

    /// Instruction: Branch if equal
    pub fn BEQ(&mut self) -> u8 {
        self.branchrel(self.get_flag(FLAGS6502::Z) == 1);
        return 0;
    }

    pub fn BIT(&mut self) -> u8 {
        self.fetch();
        let temp = self.a & self.fetched;
        self.set_flag(FLAGS6502::Z, (temp & 0x00FF) == 0);
        self.set_flag(FLAGS6502::V, self.fetched & 0x40 > 0);
        self.set_flag(FLAGS6502::N, self.fetched & 0x80 > 0);
        return 0;
    }

    /// Instruction: Branch if minus
    pub fn BMI(&mut self) -> u8 {
        self.branchrel(self.get_flag(FLAGS6502::N) == 1);
        return 0;
    }

    /// Instruction: Branch if not equal
    pub fn BNE(&mut self) -> u8 {
        self.branchrel(self.get_flag(FLAGS6502::Z) == 0);
        return 0;
    }

    /// Instruction: Branch if positive
    pub fn BPL(&mut self) -> u8 {
        self.branchrel(self.get_flag(FLAGS6502::N) == 0);
        return 0;
    }

    /// Instruction: Force Interrupt
    pub fn BRK(&mut self) -> u8 {
        self.pc = self.pc.wrapping_add(1);

        self.set_flag(FLAGS6502::I, true);
        self.push_to_stack(((self.pc >> 8) & 0x00FF) as u8);
        self.push_to_stack((self.pc & 0x00FF) as u8);

        self.set_flag(FLAGS6502::B, true);
        self.push_to_stack(self.status);
        self.set_flag(FLAGS6502::B, false);

        self.pc = self.read(0xFFFE as u16, false) as u16
            | ((self.read(0xFFFF as u16, false) as u16) << 8) as u16;
        return 0;
    }

    /// Instruction: Branch if overflowed
    pub fn BVC(&mut self) -> u8 {
        self.branchrel(self.get_flag(FLAGS6502::V) == 0);
        return 0;
    }

    /// Instruction: Branch if  not overflowed
    pub fn BVS(&mut self) -> u8 {
        self.branchrel(self.get_flag(FLAGS6502::V) == 1);
        return 0;
    }

    /// Instruction: Clear carry flag
    pub fn CLC(&mut self) -> u8 {
        self.set_flag(FLAGS6502::C, false);
        return 0;
    }

    /// Instruction: Clear decimal flag
    pub fn CLD(&mut self) -> u8 {
        self.set_flag(FLAGS6502::D, false);
        return 0;
    }

    /// Instruction: Clear interrupt block flag
    pub fn CLI(&mut self) -> u8 {
        self.set_flag(FLAGS6502::I, false);
        return 0;
    }

    /// Instruction: Clear Overflow flag
    pub fn CLV(&mut self) -> u8 {
        self.set_flag(FLAGS6502::V, false);
        return 0;
    }

    /// Instruction: Compare memory with A register
    pub fn CMP(&mut self) -> u8 {
        self.fetch();
        let temp = self.a.wrapping_sub(self.fetched);
        self.set_flag(FLAGS6502::C, self.a >= self.fetched);
        self.set_flag(FLAGS6502::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(FLAGS6502::N, temp & 0x0080 > 0);

        return 1;
    }

    /// Instruction: Compare memory with X register
    pub fn CPX(&mut self) -> u8 {
        self.fetch();
        let temp = self.x.wrapping_sub(self.fetched);
        self.set_flag(FLAGS6502::C, self.x >= self.fetched);
        self.set_flag(FLAGS6502::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(FLAGS6502::N, temp & 0x0080 > 0);

        return 0;
    }

    /// Instruction: Compare memory with Y register
    pub fn CPY(&mut self) -> u8 {
        self.fetch();
        let temp = self.y.wrapping_sub(self.fetched);
        self.set_flag(FLAGS6502::C, self.y >= self.fetched);
        self.set_flag(FLAGS6502::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(FLAGS6502::N, temp & 0x0080 > 0);

        return 0;
    }

    /// Instruction: Decrement value at memory
    pub fn DEC(&mut self) -> u8 {
        self.fetch();

        let result = self.fetched.wrapping_sub(1);
        self.write(self.addr_abs as usize, (result & 0x00FF) as u8);
        self.set_flag(FLAGS6502::Z, (result & 0x00FF) == 0);
        self.set_flag(FLAGS6502::N, (result & 0x0080) > 0);

        return 0;
    }
    /// Instruction: Decrement X register
    pub fn DEX(&mut self) -> u8 {
        self.x = self.x.wrapping_sub(1);
        self.set_flag(FLAGS6502::Z, self.x == 0);
        self.set_flag(FLAGS6502::N, (self.x & 0x80) > 0);

        return 0;
    }

    /// Instruction: Decrement Y register
    pub fn DEY(&mut self) -> u8 {
        self.y = self.y.wrapping_sub(1);
        self.set_flag(FLAGS6502::Z, self.y == 0);
        self.set_flag(FLAGS6502::N, (self.y & 0x80) > 0);

        return 0;
    }

    /// Instruction: XOR A and M
    pub fn EOR(&mut self) -> u8 {
        self.fetch();
        self.a = self.a ^ self.fetched;
        self.set_flag(FLAGS6502::Z, self.a == 0x00);
        self.set_flag(FLAGS6502::N, (self.a & 0x80) > 0);

        return 1;
    }

    /// Instruction: Increment Memory
    pub fn INC(&mut self) -> u8 {
        self.fetch();

        let result = self.fetched.wrapping_add(1);
        self.write(self.addr_abs as usize, result & 0x00FF);
        self.set_flag(FLAGS6502::Z, result == 0);
        self.set_flag(FLAGS6502::N, (result & 0x80) > 0);

        return 0;
    }

    /// Instruction: Increment X
    pub fn INX(&mut self) -> u8 {
        self.x = self.x.wrapping_add(1);
        self.set_flag(FLAGS6502::Z, self.x == 0);
        self.set_flag(FLAGS6502::N, (self.x & 0x80) > 0);

        return 0;
    }

    /// Instruction: Increment Y
    pub fn INY(&mut self) -> u8 {
        self.y = self.y.wrapping_add(1);
        self.set_flag(FLAGS6502::Z, self.y == 0);
        self.set_flag(FLAGS6502::N, (self.y & 0x80) > 0);

        return 0;
    }

    /// Instruction: Jump to address
    pub fn JMP(&mut self) -> u8 {
        self.pc = self.addr_abs;
        return 0;
    }

    /// Instruction: Jump to subroutine
    pub fn JSR(&mut self) -> u8 {
        self.pc = self.pc - 1;

        self.push_to_stack(((self.pc >> 8) & 0x00FF) as u8);
        self.push_to_stack((self.pc & 0x00FF) as u8);
        self.pc = self.addr_abs;
        return 0;
    }

    /// Instruction: Load memory in A
    pub fn LDA(&mut self) -> u8 {
        self.fetch();
        self.a = self.fetched;
        self.set_flag(FLAGS6502::Z, self.a == 0);
        self.set_flag(FLAGS6502::N, (self.a & 0x80) > 0);
        return 1;
    }

    /// Instruction: Load memory in X
    pub fn LDX(&mut self) -> u8 {
        self.fetch();
        self.x = self.fetched;
        self.set_flag(FLAGS6502::Z, self.x == 0);
        self.set_flag(FLAGS6502::N, (self.x & 0x80) > 0);
        return 1;
    }

    /// Instruction: Load memory in Y
    pub fn LDY(&mut self) -> u8 {
        self.fetch();
        self.y = self.fetched;
        self.set_flag(FLAGS6502::Z, self.y == 0);
        self.set_flag(FLAGS6502::N, (self.y & 0x80) > 0);
        return 1;
    }

    /// Instruction: Logical shift right
    pub fn LSR(&mut self) -> u8 {
        self.fetch();
        self.set_flag(FLAGS6502::C, (self.fetched & 0x0001) > 0);
        let temp: u16 = (self.fetched as u16) >> 1;
        self.set_flag(FLAGS6502::Z, (temp & 0x00FF) == 0x00);
        self.set_flag(FLAGS6502::N, (temp & 0x80) > 0);
        if self.lookup[usize::from(self.opcode)].addrmode as usize == Self::IMP as usize {
            self.a = (temp & 0x00FF) as u8;
        } else {
            self.write(usize::from(self.addr_abs), (temp & 0x00FF) as u8);
        }
        return 0;
    }

    /// Instruction: No operation
    pub fn NOP(&mut self) -> u8 {
        match self.opcode {
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                return 1;
            }
            _ => {
                return 0;
            }
        }
    }

    /// Instruction: OR A and M
    pub fn ORA(&mut self) -> u8 {
        self.fetch();
        self.a = self.a | self.fetched;
        self.set_flag(FLAGS6502::Z, self.a == 0x00);
        self.set_flag(FLAGS6502::N, (self.a & 0x80) > 0);
        return 0;
    }

    /// Instruction: Push A to stack
    pub fn PHA(&mut self) -> u8 {
        self.push_to_stack(self.a);
        return 0;
    }

    /// Instruction: Push status to stack
    pub fn PHP(&mut self) -> u8 {
        self.push_to_stack(self.status | (FLAGS6502::B as u8) | (FLAGS6502::U as u8));
        self.set_flag(FLAGS6502::B, false);
        self.set_flag(FLAGS6502::U, false);
        return 0;
    }

    /// Instruction: Pop from stack to accumulator
    pub fn PLA(&mut self) -> u8 {
        self.a = self.pop_from_stack();
        self.set_flag(FLAGS6502::Z, self.a == 0);
        self.set_flag(FLAGS6502::N, (self.a & 0x80) > 0);
        return 0;
    }

    /// Instruction: Pop from stack to status
    pub fn PLP(&mut self) -> u8 {
        self.status = self.pop_from_stack();
        self.set_flag(FLAGS6502::U, true);
        return 0;
    }

    /// Instruction: Rotate Left
    pub fn ROL(&mut self) -> u8 {
        self.fetch();
        let temp: u16 = ((self.fetched as u16) << 1) | (self.get_flag(FLAGS6502::C) as u16);
        self.set_flag(FLAGS6502::C, (temp & 0xFF00) > 0);
        self.set_flag(FLAGS6502::Z, (temp & 0x00FF) == 0x00);
        self.set_flag(FLAGS6502::N, (temp & 0x80) > 0);
        if self.lookup[usize::from(self.opcode)].addrmode as usize == Self::IMP as usize {
            self.a = (temp & 0x00FF) as u8;
        } else {
            self.write(usize::from(self.addr_abs), (temp & 0x00FF) as u8);
        }
        return 0;
    }

    /// Instruction: Rotate Right
    pub fn ROR(&mut self) -> u8 {
        self.fetch();
        let cval = self.get_flag(FLAGS6502::C);
        let temp: u16 = ((self.fetched as u16) >> 1) | ((cval as u16) << 7);
        self.set_flag(FLAGS6502::C, (self.fetched & 0x01) > 0);
        self.set_flag(FLAGS6502::Z, (temp & 0x00FF) == 0x00);
        self.set_flag(FLAGS6502::N, (temp & 0x0080) > 0);
        if self.lookup[usize::from(self.opcode)].addrmode as usize == Self::IMP as usize {
            self.a = (temp & 0x00FF) as u8;
        } else {
            self.write(usize::from(self.addr_abs), (temp & 0x00FF) as u8);
        }
        return 0;
    }

    /// Instruction: Return from interrupt
    pub fn RTI(&mut self) -> u8 {
        self.status = self.pop_from_stack();

        self.set_flag(FLAGS6502::B, false);
        self.set_flag(FLAGS6502::U, false);

        let lo: u16 = self.pop_from_stack() as u16;
        let hi: u16 = (self.pop_from_stack() as u16) << 8;
        self.pc = hi | lo;
        return 0;
    }

    /// Instruction: Return from subroutine
    pub fn RTS(&mut self) -> u8 {
        let lo: u16 = self.pop_from_stack() as u16;
        let hi: u16 = (self.pop_from_stack() as u16) << 8;
        self.pc = hi | lo;
        self.pc = self.pc.wrapping_add(1);
        return 0;
    }

    /// Instruction: Subtract with carry
    pub fn SBC(&mut self) -> u8 {
        self.fetch();
        let value: u16 = (self.fetched as u16) ^ 0x00FF;

        let temp = (self.a as u16) + (value as u16) + (self.get_flag(FLAGS6502::C) as u16);
        self.set_flag(FLAGS6502::C, temp & 0xFF00 > 0);
        self.set_flag(FLAGS6502::Z, (temp & 0x00FF) == 0);
        self.set_flag(
            FLAGS6502::V,
            (temp ^ (self.a as u16)) & (temp ^ value) & 0x0080 > 0,
        );
        self.set_flag(FLAGS6502::N, temp & 0x0080 > 0);
        self.a = (temp & 0x00FF) as u8;
        return 0;
    }

    /// Instruction: Set carry flag
    pub fn SEC(&mut self) -> u8 {
        self.set_flag(FLAGS6502::C, true);
        return 0;
    }

    /// Instruction: Set Decimal flag
    pub fn SED(&mut self) -> u8 {
        self.set_flag(FLAGS6502::D, true);
        return 0;
    }

    /// Instruction: Set interrupt block flag
    pub fn SEI(&mut self) -> u8 {
        self.set_flag(FLAGS6502::I, true);
        return 0;
    }

    /// Instruction: Store accumulator to memory
    pub fn STA(&mut self) -> u8 {
        self.write(self.addr_abs as usize, self.a);
        return 0;
    }

    /// Instruction: Store X to memory
    pub fn STX(&mut self) -> u8 {
        self.write(self.addr_abs as usize, self.x);
        return 0;
    }

    /// Instruction: Store Y to memory
    pub fn STY(&mut self) -> u8 {
        self.write(self.addr_abs as usize, self.y);
        return 0;
    }

    /// Instruction: Transfer A to X
    pub fn TAX(&mut self) -> u8 {
        self.x = self.a;
        self.set_flag(FLAGS6502::Z, self.x == 0);
        self.set_flag(FLAGS6502::N, (self.x & 0x0080) > 0);
        return 0;
    }

    /// Instruction: Transfer A to Y
    pub fn TAY(&mut self) -> u8 {
        self.y = self.a;
        self.set_flag(FLAGS6502::Z, self.y == 0);
        self.set_flag(FLAGS6502::N, (self.y & 0x0080) > 0);
        return 0;
    }

    /// Instruction: Transfer stkp to X
    pub fn TSX(&mut self) -> u8 {
        self.x = self.stkp;
        self.set_flag(FLAGS6502::Z, self.x == 0);
        self.set_flag(FLAGS6502::N, (self.x & 0x0080) > 0);
        return 0;
    }

    /// Instruction: Transfer X to A
    pub fn TXA(&mut self) -> u8 {
        self.a = self.x;
        self.set_flag(FLAGS6502::Z, self.a == 0);
        self.set_flag(FLAGS6502::N, (self.a & 0x0080) > 0);
        return 0;
    }

    /// Instruction: Transfer X to Stack
    pub fn TXS(&mut self) -> u8 {
        self.stkp = self.x;
        return 0;
    }

    /// Instruction: Transfer Y to A
    pub fn TYA(&mut self) -> u8 {
        self.a = self.y;
        self.set_flag(FLAGS6502::Z, self.a == 0);
        self.set_flag(FLAGS6502::N, (self.a & 0x0080) > 0);
        return 0;
    }

    /// Instruction: Invalid operation
    pub fn XXX(&mut self) -> u8 {
        return 0;
    }

    //Interrupts
    pub fn clock(&mut self) {
        if self.cycles == 0 {
            self.opcode = self.read(self.pc, false).into();

            self.set_flag(FLAGS6502::U, true);
            self.pc = self.pc.wrapping_add(1);

            let instr = &self.lookup[usize::from(self.opcode)];

            self.cycles = instr.cycles;

            let addrfunc = instr.addrmode;
            let operfunc = instr.operate;
            let additional_cycle1 = addrfunc(self);
            let additional_cycle2 = operfunc(self);

            self.cycles = self.cycles + (additional_cycle1 & additional_cycle2);

            self.set_flag(FLAGS6502::U, true);
        }
        self.cycles = self.cycles - 1;
    }

    pub fn is_complete(&mut self) -> bool {
        return self.cycles == 0;
    }

    /// reset cpu to a known state
    pub fn reset(&mut self) {
        self.addr_abs = 0xFFFC;

        let loc1 = self.addr_abs + 0;
        let loc2 = self.addr_abs + 1;
        let lo = self.read(loc1, false);
        let hi = self.read(loc2, false);

        self.pc = ((hi as u16) << 8) | (lo as u16);

        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.stkp = 0xFD;
        self.status = 0x00 | (FLAGS6502::U as u8);

        self.addr_abs = 0;
        self.addr_rel = 0;
        self.fetched = 0;

        self.cycles = 8;

        self.bus.reset();
    }

    /// interrupt only if I=0
    fn _irq(&mut self) {
        if self.get_flag(FLAGS6502::I) == 0 {
            self.push_to_stack(((self.pc >> 8) & 0x00FF) as u8);
            self.push_to_stack((self.pc & 0x00FF) as u8);

            self.set_flag(FLAGS6502::B, false);
            self.set_flag(FLAGS6502::U, true);
            self.set_flag(FLAGS6502::I, true);
            self.push_to_stack(self.status);

            self.addr_abs = 0xFFFE;
            let lo = self.read(self.addr_abs + 0, false);
            let hi = self.read(self.addr_abs + 1, false);
            self.pc = ((hi as u16) << 8) | (lo as u16);
            self.cycles = 7;
        }
    }

    /// Non maskable interrupt
    pub fn nmi(&mut self) {
        self.push_to_stack(((self.pc >> 8) & 0x00FF) as u8);
        self.push_to_stack((self.pc & 0x00FF) as u8);

        self.set_flag(FLAGS6502::B, false);
        self.set_flag(FLAGS6502::U, true);
        self.set_flag(FLAGS6502::I, true);
        self.push_to_stack(self.status);

        self.addr_abs = 0xFFFA;
        let lo = self.read(self.addr_abs + 0, false);
        let hi = self.read(self.addr_abs + 1, false);

        self.pc = ((hi as u16) << 8) | (lo as u16);
        self.cycles = 8;
    }

    fn fetch(&mut self) {
        let opsize = self.opcode as usize;
        if self.lookup[opsize].addrmode as usize != Self::IMP as usize {
            self.fetched = self.read(self.addr_abs, false);
        }
    }

    pub fn new() -> Self {
        type I = Instruction;
        return Cpu {
            bus: Bus::new(),
            status: 0,
            a: 0,
            x: 0,
            y: 0,
            stkp: 0,
            pc: 0,
            fetched: 0,
            addr_abs: 0,
            addr_rel: 0,
            opcode: 0,
            cycles: 0,
            lookup: vec![
                I::new_i("BRK", Self::BRK, Self::IMM, 7),
                I::new_i("ORA", Self::ORA, Self::IZX, 6),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 8),
                I::new_i("???", Self::NOP, Self::IMP, 3),
                I::new_i("ORA", Self::ORA, Self::ZP0, 3),
                I::new_i("ASL", Self::ASL, Self::ZP0, 5),
                I::new_i("???", Self::XXX, Self::IMP, 5),
                I::new_i("PHP", Self::PHP, Self::IMP, 3),
                I::new_i("ORA", Self::ORA, Self::IMM, 2),
                I::new_i("ASL", Self::ASL, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("ORA", Self::ORA, Self::ABS, 4),
                I::new_i("ASL", Self::ASL, Self::ABS, 6),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("BPL", Self::BPL, Self::REL, 2),
                I::new_i("ORA", Self::ORA, Self::IZY, 5),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 8),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("ORA", Self::ORA, Self::ZPX, 4),
                I::new_i("ASL", Self::ASL, Self::ZPX, 6),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("CLC", Self::CLC, Self::IMP, 2),
                I::new_i("ORA", Self::ORA, Self::ABY, 4),
                I::new_i("???", Self::NOP, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 7),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("ORA", Self::ORA, Self::ABX, 4),
                I::new_i("ASL", Self::ASL, Self::ABX, 7),
                I::new_i("???", Self::XXX, Self::IMP, 7),
                I::new_i("JSR", Self::JSR, Self::ABS, 6),
                I::new_i("AND", Self::AND, Self::IZX, 6),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 8),
                I::new_i("BIT", Self::BIT, Self::ZP0, 3),
                I::new_i("AND", Self::AND, Self::ZP0, 3),
                I::new_i("ROL", Self::ROL, Self::ZP0, 5),
                I::new_i("???", Self::XXX, Self::IMP, 5),
                I::new_i("PLP", Self::PLP, Self::IMP, 4),
                I::new_i("AND", Self::AND, Self::IMM, 2),
                I::new_i("ROL", Self::ROL, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("BIT", Self::BIT, Self::ABS, 4),
                I::new_i("AND", Self::AND, Self::ABS, 4),
                I::new_i("ROL", Self::ROL, Self::ABS, 6),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("BMI", Self::BMI, Self::REL, 2),
                I::new_i("AND", Self::AND, Self::IZY, 5),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 8),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("AND", Self::AND, Self::ZPX, 4),
                I::new_i("ROL", Self::ROL, Self::ZPX, 6),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("SEC", Self::SEC, Self::IMP, 2),
                I::new_i("AND", Self::AND, Self::ABY, 4),
                I::new_i("???", Self::NOP, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 7),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("AND", Self::AND, Self::ABX, 4),
                I::new_i("ROL", Self::ROL, Self::ABX, 7),
                I::new_i("???", Self::XXX, Self::IMP, 7),
                I::new_i("RTI", Self::RTI, Self::IMP, 6),
                I::new_i("EOR", Self::EOR, Self::IZX, 6),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 8),
                I::new_i("???", Self::NOP, Self::IMP, 3),
                I::new_i("EOR", Self::EOR, Self::ZP0, 3),
                I::new_i("LSR", Self::LSR, Self::ZP0, 5),
                I::new_i("???", Self::XXX, Self::IMP, 5),
                I::new_i("PHA", Self::PHA, Self::IMP, 3),
                I::new_i("EOR", Self::EOR, Self::IMM, 2),
                I::new_i("LSR", Self::LSR, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("JMP", Self::JMP, Self::ABS, 3),
                I::new_i("EOR", Self::EOR, Self::ABS, 4),
                I::new_i("LSR", Self::LSR, Self::ABS, 6),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("BVC", Self::BVC, Self::REL, 2),
                I::new_i("EOR", Self::EOR, Self::IZY, 5),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 8),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("EOR", Self::EOR, Self::ZPX, 4),
                I::new_i("LSR", Self::LSR, Self::ZPX, 6),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("CLI", Self::CLI, Self::IMP, 2),
                I::new_i("EOR", Self::EOR, Self::ABY, 4),
                I::new_i("???", Self::NOP, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 7),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("EOR", Self::EOR, Self::ABX, 4),
                I::new_i("LSR", Self::LSR, Self::ABX, 7),
                I::new_i("???", Self::XXX, Self::IMP, 7),
                I::new_i("RTS", Self::RTS, Self::IMP, 6),
                I::new_i("ADC", Self::ADC, Self::IZX, 6),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 8),
                I::new_i("???", Self::NOP, Self::IMP, 3),
                I::new_i("ADC", Self::ADC, Self::ZP0, 3),
                I::new_i("ROR", Self::ROR, Self::ZP0, 5),
                I::new_i("???", Self::XXX, Self::IMP, 5),
                I::new_i("PLA", Self::PLA, Self::IMP, 4),
                I::new_i("ADC", Self::ADC, Self::IMM, 2),
                I::new_i("ROR", Self::ROR, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("JMP", Self::JMP, Self::IND, 5),
                I::new_i("ADC", Self::ADC, Self::ABS, 4),
                I::new_i("ROR", Self::ROR, Self::ABS, 6),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("BVS", Self::BVS, Self::REL, 2),
                I::new_i("ADC", Self::ADC, Self::IZY, 5),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 8),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("ADC", Self::ADC, Self::ZPX, 4),
                I::new_i("ROR", Self::ROR, Self::ZPX, 6),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("SEI", Self::SEI, Self::IMP, 2),
                I::new_i("ADC", Self::ADC, Self::ABY, 4),
                I::new_i("???", Self::NOP, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 7),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("ADC", Self::ADC, Self::ABX, 4),
                I::new_i("ROR", Self::ROR, Self::ABX, 7),
                I::new_i("???", Self::XXX, Self::IMP, 7),
                I::new_i("???", Self::NOP, Self::IMP, 2),
                I::new_i("STA", Self::STA, Self::IZX, 6),
                I::new_i("???", Self::NOP, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("STY", Self::STY, Self::ZP0, 3),
                I::new_i("STA", Self::STA, Self::ZP0, 3),
                I::new_i("STX", Self::STX, Self::ZP0, 3),
                I::new_i("???", Self::XXX, Self::IMP, 3),
                I::new_i("DEY", Self::DEY, Self::IMP, 2),
                I::new_i("???", Self::NOP, Self::IMP, 2),
                I::new_i("TXA", Self::TXA, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("STY", Self::STY, Self::ABS, 4),
                I::new_i("STA", Self::STA, Self::ABS, 4),
                I::new_i("STX", Self::STX, Self::ABS, 4),
                I::new_i("???", Self::XXX, Self::IMP, 4),
                I::new_i("BCC", Self::BCC, Self::REL, 2),
                I::new_i("STA", Self::STA, Self::IZY, 6),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("STY", Self::STY, Self::ZPX, 4),
                I::new_i("STA", Self::STA, Self::ZPX, 4),
                I::new_i("STX", Self::STX, Self::ZPY, 4),
                I::new_i("???", Self::XXX, Self::IMP, 4),
                I::new_i("TYA", Self::TYA, Self::IMP, 2),
                I::new_i("STA", Self::STA, Self::ABY, 5),
                I::new_i("TXS", Self::TXS, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 5),
                I::new_i("???", Self::NOP, Self::IMP, 5),
                I::new_i("STA", Self::STA, Self::ABX, 5),
                I::new_i("???", Self::XXX, Self::IMP, 5),
                I::new_i("???", Self::XXX, Self::IMP, 5),
                I::new_i("LDY", Self::LDY, Self::IMM, 2),
                I::new_i("LDA", Self::LDA, Self::IZX, 6),
                I::new_i("LDX", Self::LDX, Self::IMM, 2),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("LDY", Self::LDY, Self::ZP0, 3),
                I::new_i("LDA", Self::LDA, Self::ZP0, 3),
                I::new_i("LDX", Self::LDX, Self::ZP0, 3),
                I::new_i("???", Self::XXX, Self::IMP, 3),
                I::new_i("TAY", Self::TAY, Self::IMP, 2),
                I::new_i("LDA", Self::LDA, Self::IMM, 2),
                I::new_i("TAX", Self::TAX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("LDY", Self::LDY, Self::ABS, 4),
                I::new_i("LDA", Self::LDA, Self::ABS, 4),
                I::new_i("LDX", Self::LDX, Self::ABS, 4),
                I::new_i("???", Self::XXX, Self::IMP, 4),
                I::new_i("BCS", Self::BCS, Self::REL, 2),
                I::new_i("LDA", Self::LDA, Self::IZY, 5),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 5),
                I::new_i("LDY", Self::LDY, Self::ZPX, 4),
                I::new_i("LDA", Self::LDA, Self::ZPX, 4),
                I::new_i("LDX", Self::LDX, Self::ZPY, 4),
                I::new_i("???", Self::XXX, Self::IMP, 4),
                I::new_i("CLV", Self::CLV, Self::IMP, 2),
                I::new_i("LDA", Self::LDA, Self::ABY, 4),
                I::new_i("TSX", Self::TSX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 4),
                I::new_i("LDY", Self::LDY, Self::ABX, 4),
                I::new_i("LDA", Self::LDA, Self::ABX, 4),
                I::new_i("LDX", Self::LDX, Self::ABY, 4),
                I::new_i("???", Self::XXX, Self::IMP, 4),
                I::new_i("CPY", Self::CPY, Self::IMM, 2),
                I::new_i("CMP", Self::CMP, Self::IZX, 6),
                I::new_i("???", Self::NOP, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 8),
                I::new_i("CPY", Self::CPY, Self::ZP0, 3),
                I::new_i("CMP", Self::CMP, Self::ZP0, 3),
                I::new_i("DEC", Self::DEC, Self::ZP0, 5),
                I::new_i("???", Self::XXX, Self::IMP, 5),
                I::new_i("INY", Self::INY, Self::IMP, 2),
                I::new_i("CMP", Self::CMP, Self::IMM, 2),
                I::new_i("DEX", Self::DEX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("CPY", Self::CPY, Self::ABS, 4),
                I::new_i("CMP", Self::CMP, Self::ABS, 4),
                I::new_i("DEC", Self::DEC, Self::ABS, 6),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("BNE", Self::BNE, Self::REL, 2),
                I::new_i("CMP", Self::CMP, Self::IZY, 5),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 8),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("CMP", Self::CMP, Self::ZPX, 4),
                I::new_i("DEC", Self::DEC, Self::ZPX, 6),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("CLD", Self::CLD, Self::IMP, 2),
                I::new_i("CMP", Self::CMP, Self::ABY, 4),
                I::new_i("NOP", Self::NOP, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 7),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("CMP", Self::CMP, Self::ABX, 4),
                I::new_i("DEC", Self::DEC, Self::ABX, 7),
                I::new_i("???", Self::XXX, Self::IMP, 7),
                I::new_i("CPX", Self::CPX, Self::IMM, 2),
                I::new_i("SBC", Self::SBC, Self::IZX, 6),
                I::new_i("???", Self::NOP, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 8),
                I::new_i("CPX", Self::CPX, Self::ZP0, 3),
                I::new_i("SBC", Self::SBC, Self::ZP0, 3),
                I::new_i("INC", Self::INC, Self::ZP0, 5),
                I::new_i("???", Self::XXX, Self::IMP, 5),
                I::new_i("INX", Self::INX, Self::IMP, 2),
                I::new_i("SBC", Self::SBC, Self::IMM, 2),
                I::new_i("NOP", Self::NOP, Self::IMP, 2),
                I::new_i("???", Self::SBC, Self::IMP, 2),
                I::new_i("CPX", Self::CPX, Self::ABS, 4),
                I::new_i("SBC", Self::SBC, Self::ABS, 4),
                I::new_i("INC", Self::INC, Self::ABS, 6),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("BEQ", Self::BEQ, Self::REL, 2),
                I::new_i("SBC", Self::SBC, Self::IZY, 5),
                I::new_i("???", Self::XXX, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 8),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("SBC", Self::SBC, Self::ZPX, 4),
                I::new_i("INC", Self::INC, Self::ZPX, 6),
                I::new_i("???", Self::XXX, Self::IMP, 6),
                I::new_i("SED", Self::SED, Self::IMP, 2),
                I::new_i("SBC", Self::SBC, Self::ABY, 4),
                I::new_i("NOP", Self::NOP, Self::IMP, 2),
                I::new_i("???", Self::XXX, Self::IMP, 7),
                I::new_i("???", Self::NOP, Self::IMP, 4),
                I::new_i("SBC", Self::SBC, Self::ABX, 4),
                I::new_i("INC", Self::INC, Self::ABX, 7),
                I::new_i("???", Self::XXX, Self::IMP, 7),
            ],
            map_asm: IndexMap::new(),
        };
    }

    fn pcread(&mut self) -> u16 {
        let result = self.read(self.pc, false).into();
        self.pc = self.pc + 1;
        return result;
    }

    fn branchrel(&mut self, flag: bool) {
        if flag {
            self.cycles = self.cycles + 1;
            self.addr_abs = self.addr_rel.wrapping_add(self.pc);
            if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles = self.cycles + 1;
            }

            self.pc = self.addr_abs;
        }
    }

    pub fn read(&mut self, addr: u16, b_read_only: bool) -> u8 {
        let data = self.bus.read(usize::from(addr), b_read_only);
        return data;
    }

    pub fn write(&mut self, addr: usize, data: u8) {
        self.bus.write(addr, data);
    }

    pub fn get_flag(&self, f: FLAGS6502) -> u8 {
        if self.status & (f as u8) > 0 {
            return 1;
        }
        return 0;
    }
    pub fn set_flag(&mut self, f: FLAGS6502, v: bool) {
        if v {
            self.status = self.status | (f as u8);
        } else {
            self.status = self.status & (!(f as u8));
        }
    }

    pub fn _disassemble(&mut self, nStart: u16, nStop: u16) {
        let nStop = nStop as u32;
        let mut addr = nStart as u32;
        let mut value: u8;
        let mut lo;
        let mut hi;
        let mut mapLines: IndexMap<u16, String> = IndexMap::new();
        let mut line_addr;

        while addr < nStop {
            line_addr = addr;

            let mut sInst = format!("{}{}{}", "$", hex(addr as u16), ":           ");

            let opcode = self.read(addr as u16, true) as usize;
            let inst = &self.lookup[opcode];
            addr = addr + 1;
            sInst = format!("{}{} ", sInst, String::from(&inst.name));

            if self.lookup[opcode].addrmode as usize == Self::IMP as usize {
                sInst = format!("{}{}", sInst, "{IMP}");
            } else if self.lookup[opcode].addrmode as usize == Self::IMM as usize {
                value = self.read(addr as u16, true);
                addr = addr + 1;
                sInst = format!("{}{}{}{}", sInst, "#$", hex(value.into()), " {IMM}");
            } else if self.lookup[opcode].addrmode as usize == Self::ZP0 as usize {
                lo = self.read(addr as u16, true);
                addr = addr + 1;
                sInst = format!("{}{}{}{}", sInst, "$", hex(lo.into()), " {ZP0}");
            } else if self.lookup[opcode].addrmode as usize == Self::ZPX as usize {
                lo = self.read(addr as u16, true);
                addr = addr + 1;
                sInst = format!("{}{}{}{}", sInst, "$", hex(lo.into()), ", X {ZPX}");
            } else if self.lookup[opcode].addrmode as usize == Self::ZPY as usize {
                lo = self.read(addr as u16, true);
                addr = addr + 1;
                sInst = format!("{}{}{}{}", sInst, "$", hex(lo.into()), ", Y {ZPY}");
            } else if self.lookup[opcode].addrmode as usize == Self::IZX as usize {
                lo = self.read(addr as u16, true);
                addr = addr + 1;
                sInst = format!("{}{}{}{}", sInst, "$", hex(lo.into()), "{IZX}");
            } else if self.lookup[opcode].addrmode as usize == Self::IZY as usize {
                lo = self.read(addr as u16, true);
                addr = addr + 1;
                sInst = format!("{}{}{}{}", sInst, "$", hex(lo.into()), "{IZY}");
            } else if self.lookup[opcode].addrmode as usize == Self::ABS as usize {
                lo = self.read(addr as u16, true);
                addr = addr + 1;
                hi = self.read(addr as u16, true);
                addr = addr + 1;
                let comb = ((hi as u16) << 8) | (lo as u16);
                sInst = format!("{}{}{}{}", sInst, "$", hex(comb), " {ABS}");
            } else if self.lookup[opcode].addrmode as usize == Self::ABX as usize {
                lo = self.read(addr as u16, true);
                addr = addr + 1;
                hi = self.read(addr as u16, true);
                addr = addr + 1;
                sInst = format!(
                    "{}{}{}{}",
                    sInst,
                    "$",
                    hex((((hi as u16) << 8) | lo as u16).into()),
                    ", X {ABX}"
                );
            } else if self.lookup[opcode].addrmode as usize == Self::ABY as usize {
                lo = self.read(addr as u16, true);
                addr = addr + 1;
                hi = self.read(addr as u16, true);
                addr = addr + 1;
                sInst = format!(
                    "{}{}{}{}",
                    sInst,
                    "$",
                    hex((((hi as u16) << 8) | lo as u16).into()),
                    ", Y {ABY}"
                );
            } else if self.lookup[opcode].addrmode as usize == Self::IND as usize {
                lo = self.read(addr as u16, true);
                addr = addr + 1;
                hi = self.read(addr as u16, true);
                addr = addr + 1;
                sInst = format!(
                    "{}{}{}{}",
                    sInst,
                    "$",
                    hex((((hi as u16) << 8) | (lo as u16)).into()),
                    "{IND}"
                );
            } else if self.lookup[opcode].addrmode as usize == Self::REL as usize {
                value = self.read(addr as u16, true);
                addr = addr + 1;
                let k = addr.wrapping_add(value as u32);

                sInst = format!(
                    "{}{}{}{}",
                    sInst,
                    "$",
                    hex(k.wrapping_sub(0x0100) as u16),
                    " {REL}"
                );
            }
            mapLines.insert(line_addr as u16, String::from(sInst));
        }

        self.map_asm = mapLines;
    }
}
