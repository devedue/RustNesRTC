use crate::bus::*;

enum FLAGS6502 {
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
    pub operate: unsafe fn(&mut Olc6502) -> u8,
    pub addrmode: unsafe fn(&mut Olc6502) -> u8,
    pub cycles: u8,
}

impl Instruction {
    fn NewI(
        name: &str,
        operate: unsafe fn(&mut Olc6502) -> u8,
        addrmode: unsafe fn(&mut Olc6502) -> u8,
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

pub struct Olc6502 {
    bus: *mut Bus,
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
}
impl Olc6502 {

    // Addressing Modes
    pub unsafe fn IMP(&mut self) -> u8 {
        self.fetched = self.a;
        return 0;
    }
    pub fn IMM(&mut self) -> u8 {
        self.addr_abs = self.pc + 1;
        self.pc = self.pc + 1;
        return 0;
    }
    pub unsafe fn ZP0(&mut self) -> u8 {
        self.addr_abs = self.pcread();
        self.pc = self.pc + 1;
        self.addr_abs = self.addr_abs & 0x00FF;
        return 0;
    }
    pub unsafe fn ZPX(&mut self) -> u8 {
        self.addr_abs = self.pcread();
        self.pc = self.pc + 1;
        self.addr_abs = self.addr_abs & 0x00FF;
        return 0;
    }
    pub unsafe fn ZPY(&mut self) -> u8 {
        self.addr_abs = self.pcread();
        self.pc = self.pc + 1;
        self.addr_abs = self.addr_abs & 0x00FF;
        return 0;
    }
    pub unsafe fn REL(&mut self) -> u8 {
        self.addr_rel = self.pcread();
        self.pc = self.pc + 1;
        if self.addr_rel & 0x80 > 0 {
            self.addr_rel = self.addr_rel | 0xFF00
        }
        return 0;
    }
    pub unsafe fn ABS(&mut self) -> u8 {
        let lo = self.pcread();
        self.pc = self.pc + 1;
        let hi = self.pcread();
        self.pc = self.pc + 1;

        self.addr_abs = (hi << 8) | lo;

        return 0;
    }
    pub unsafe fn ABX(&mut self) -> u8 {
        let lo = self.pcread();
        self.pc = self.pc + 1;
        let hi = self.pcread();
        self.pc = self.pc + 1;

        let x16: u16 = self.x.into();
        self.addr_abs = ((hi << 8) | lo) + x16;
        if (self.addr_abs & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }
    pub unsafe fn ABY(&mut self) -> u8 {
        let lo = self.pcread();
        self.pc = self.pc + 1;
        let hi = self.pcread();
        self.pc = self.pc + 1;

        let y16: u16 = self.y.into();
        self.addr_abs = ((hi << 8) | lo) + y16;
        if (self.addr_abs & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
        return 0;
    }
    pub unsafe fn IND(&mut self) -> u8 {
        let lo = self.pcread();
        self.pc = self.pc + 1;
        let hi = self.pcread();
        self.pc = self.pc + 1;

        let ptr: u16 = (hi << 8) | lo;

        if lo == 0x00FF {
            let hi_addr: u16 = self.read(&(ptr + 1), false).into();
            let lo_addr: u16 = self.read(&ptr, false).into();
            self.addr_abs = hi_addr << 8 | lo_addr;
        } else {
            let hi_addr: u16 = self.read(&(ptr & 0x00FF), false).into();
            let lo_addr: u16 = self.read(&ptr, false).into();
            self.addr_abs = hi_addr << 8 | lo_addr;
        }

        return 0;
    }
    pub unsafe fn IZX(&mut self) -> u8 {
        let t = self.pcread();
        self.pc = self.pc + 1;

        let x16: u16 = self.x.into();
        let ptr = t + x16;

        let hi: u16 = self.read(&(ptr + 1 & 0x00FF), false).into();
        let lo: u16 = self.read(&(ptr & 0x00FF), false).into();
        self.addr_abs = hi << 8 | lo;

        return 0;
    }
    pub unsafe fn IZY(&mut self) -> u8 {
        let t = self.pcread();
        self.pc = self.pc + 1;

        let x16: u16 = self.x.into();
        let ptr = t + x16;

        let hi: u16 = self.read(&(ptr + 1 & 0x00FF), false).into();
        let lo: u16 = self.read(&(ptr & 0x00FF), false).into();
        self.addr_abs = hi << 8 | lo;
        let y16: u16 = self.y.into();
        self.addr_abs = self.addr_abs + y16;

        if (self.addr_abs & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }

    //Opcodes
    pub fn ADC(&mut self) -> u8 {
        return 0;
    }
    pub unsafe fn AND(&mut self) -> u8 {
        self.fetch();
        self.a = self.a & self.fetched;
        self.set_flag(FLAGS6502::Z, self.a == 0x00);
        self.set_flag(FLAGS6502::N, (self.a & 0x80) > 0);

        return 1;
    }
    pub fn ASL(&mut self) -> u8 {
        return 0;
    }
    pub fn BCC(&mut self) -> u8 {
        return 0;
    }
    pub fn BCS(&mut self) -> u8 {
        return 0;
    }
    pub fn BEQ(&mut self) -> u8 {
        return 0;
    }
    pub fn BIT(&mut self) -> u8 {
        return 0;
    }
    pub fn BMI(&mut self) -> u8 {
        return 0;
    }
    pub fn BNE(&mut self) -> u8 {
        return 0;
    }
    pub fn BPL(&mut self) -> u8 {
        return 0;
    }
    pub fn BRK(&mut self) -> u8 {
        return 0;
    }
    pub fn BVC(&mut self) -> u8 {
        return 0;
    }
    pub fn BVS(&mut self) -> u8 {
        return 0;
    }
    pub fn CLC(&mut self) -> u8 {
        return 0;
    }
    pub fn CLD(&mut self) -> u8 {
        return 0;
    }
    pub fn CLI(&mut self) -> u8 {
        return 0;
    }
    pub fn CLV(&mut self) -> u8 {
        return 0;
    }
    pub fn CMP(&mut self) -> u8 {
        return 0;
    }
    pub fn CPX(&mut self) -> u8 {
        return 0;
    }
    pub fn CPY(&mut self) -> u8 {
        return 0;
    }
    pub fn DEC(&mut self) -> u8 {
        return 0;
    }
    pub fn DEX(&mut self) -> u8 {
        return 0;
    }
    pub fn DEY(&mut self) -> u8 {
        return 0;
    }
    pub fn EOR(&mut self) -> u8 {
        return 0;
    }
    pub fn INC(&mut self) -> u8 {
        return 0;
    }
    pub fn INX(&mut self) -> u8 {
        return 0;
    }
    pub fn INY(&mut self) -> u8 {
        return 0;
    }
    pub fn JMP(&mut self) -> u8 {
        return 1;
    }
    pub fn JSR(&mut self) -> u8 {
        return 0;
    }
    pub fn LDA(&mut self) -> u8 {
        return 0;
    }
    pub fn LDX(&mut self) -> u8 {
        return 0;
    }
    pub fn LDY(&mut self) -> u8 {
        return 0;
    }
    pub fn LSR(&mut self) -> u8 {
        return 0;
    }
    pub fn NOP(&mut self) -> u8 {
        return 0;
    }
    pub fn ORA(&mut self) -> u8 {
        return 0;
    }
    pub fn PHA(&mut self) -> u8 {
        return 0;
    }
    pub fn PHP(&mut self) -> u8 {
        return 0;
    }
    pub fn PLA(&mut self) -> u8 {
        return 0;
    }
    pub fn PLP(&mut self) -> u8 {
        return 0;
    }
    pub fn ROL(&mut self) -> u8 {
        return 0;
    }
    pub fn ROR(&mut self) -> u8 {
        return 0;
    }
    pub fn RTI(&mut self) -> u8 {
        return 0;
    }
    pub fn RTS(&mut self) -> u8 {
        return 0;
    }
    pub fn SBC(&mut self) -> u8 {
        return 0;
    }
    pub fn SEC(&mut self) -> u8 {
        return 0;
    }
    pub fn SED(&mut self) -> u8 {
        return 0;
    }
    pub fn SEI(&mut self) -> u8 {
        return 0;
    }
    pub fn STA(&mut self) -> u8 {
        return 0;
    }
    pub fn STX(&mut self) -> u8 {
        return 0;
    }
    pub fn STY(&mut self) -> u8 {
        return 0;
    }
    pub fn TAX(&mut self) -> u8 {
        return 0;
    }
    pub fn TAY(&mut self) -> u8 {
        return 0;
    }
    pub fn TSX(&mut self) -> u8 {
        return 0;
    }
    pub fn TXA(&mut self) -> u8 {
        return 0;
    }
    pub fn TXS(&mut self) -> u8 {
        return 0;
    }
    pub fn TYA(&mut self) -> u8 {
        return 0;
    }
    pub fn XXX(&mut self) -> u8 {
        return 0;
    }
    //Interrupts
    unsafe fn clock(&mut self) {
        if self.cycles == 0 {
            self.opcode = self.read(&self.pc, false);

            let instr = &self.lookup[usize::from(self.opcode)];

            let addrfunc = instr.addrmode;
            let operfunc = instr.operate;

            let additional_cycle1 = addrfunc(self);
            let additional_cycle2 = operfunc(self);
            self.cycles = self.cycles + additional_cycle1 & additional_cycle2;
        }
        self.cycles = self.cycles - 1;
    }
    fn reset() {}
    fn irq() {}
    fn nmi() {}

    unsafe fn fetch(&mut self) -> u8 {
        let opsize = usize::from(self.opcode);
        if self.lookup[opsize].name != "IMP" {
            self.fetched = self.read(&self.addr_abs, false);
        }
        return self.fetched;
    }

    pub unsafe fn new() -> Olc6502 {
        type i = Instruction;
        return Olc6502 {
            bus: std::ptr::null_mut(),
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
                i::NewI("BRK", Self::BRK, Self::IMM, 7),
                i::NewI("ORA", Self::ORA, Self::IZX, 6),
                i::NewI("IMP", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 8),
                i::NewI("???", Self::NOP, Self::IMP, 3),
                i::NewI("ORA", Self::ORA, Self::ZP0, 3),
                i::NewI("ASL", Self::ASL, Self::ZP0, 5),
                i::NewI("???", Self::XXX, Self::IMP, 5),
                i::NewI("PHP", Self::PHP, Self::IMP, 3),
                i::NewI("ORA", Self::ORA, Self::IMM, 2),
                i::NewI("ASL", Self::ASL, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("ORA", Self::ORA, Self::ABS, 4),
                i::NewI("ASL", Self::ASL, Self::ABS, 6),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("BPL", Self::BPL, Self::REL, 2),
                i::NewI("ORA", Self::ORA, Self::IZY, 5),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 8),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("ORA", Self::ORA, Self::ZPX, 4),
                i::NewI("ASL", Self::ASL, Self::ZPX, 6),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("CLC", Self::CLC, Self::IMP, 2),
                i::NewI("ORA", Self::ORA, Self::ABY, 4),
                i::NewI("???", Self::NOP, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 7),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("ORA", Self::ORA, Self::ABX, 4),
                i::NewI("ASL", Self::ASL, Self::ABX, 7),
                i::NewI("???", Self::XXX, Self::IMP, 7),
                i::NewI("JSR", Self::JSR, Self::ABS, 6),
                i::NewI("AND", Self::AND, Self::IZX, 6),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 8),
                i::NewI("BIT", Self::BIT, Self::ZP0, 3),
                i::NewI("AND", Self::AND, Self::ZP0, 3),
                i::NewI("ROL", Self::ROL, Self::ZP0, 5),
                i::NewI("???", Self::XXX, Self::IMP, 5),
                i::NewI("PLP", Self::PLP, Self::IMP, 4),
                i::NewI("AND", Self::AND, Self::IMM, 2),
                i::NewI("ROL", Self::ROL, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("BIT", Self::BIT, Self::ABS, 4),
                i::NewI("AND", Self::AND, Self::ABS, 4),
                i::NewI("ROL", Self::ROL, Self::ABS, 6),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("BMI", Self::BMI, Self::REL, 2),
                i::NewI("AND", Self::AND, Self::IZY, 5),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 8),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("AND", Self::AND, Self::ZPX, 4),
                i::NewI("ROL", Self::ROL, Self::ZPX, 6),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("SEC", Self::SEC, Self::IMP, 2),
                i::NewI("AND", Self::AND, Self::ABY, 4),
                i::NewI("???", Self::NOP, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 7),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("AND", Self::AND, Self::ABX, 4),
                i::NewI("ROL", Self::ROL, Self::ABX, 7),
                i::NewI("???", Self::XXX, Self::IMP, 7),
                i::NewI("RTI", Self::RTI, Self::IMP, 6),
                i::NewI("EOR", Self::EOR, Self::IZX, 6),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 8),
                i::NewI("???", Self::NOP, Self::IMP, 3),
                i::NewI("EOR", Self::EOR, Self::ZP0, 3),
                i::NewI("LSR", Self::LSR, Self::ZP0, 5),
                i::NewI("???", Self::XXX, Self::IMP, 5),
                i::NewI("PHA", Self::PHA, Self::IMP, 3),
                i::NewI("EOR", Self::EOR, Self::IMM, 2),
                i::NewI("LSR", Self::LSR, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("JMP", Self::JMP, Self::ABS, 3),
                i::NewI("EOR", Self::EOR, Self::ABS, 4),
                i::NewI("LSR", Self::LSR, Self::ABS, 6),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("BVC", Self::BVC, Self::REL, 2),
                i::NewI("EOR", Self::EOR, Self::IZY, 5),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 8),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("EOR", Self::EOR, Self::ZPX, 4),
                i::NewI("LSR", Self::LSR, Self::ZPX, 6),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("CLI", Self::CLI, Self::IMP, 2),
                i::NewI("EOR", Self::EOR, Self::ABY, 4),
                i::NewI("???", Self::NOP, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 7),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("EOR", Self::EOR, Self::ABX, 4),
                i::NewI("LSR", Self::LSR, Self::ABX, 7),
                i::NewI("???", Self::XXX, Self::IMP, 7),
                i::NewI("RTS", Self::RTS, Self::IMP, 6),
                i::NewI("ADC", Self::ADC, Self::IZX, 6),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 8),
                i::NewI("???", Self::NOP, Self::IMP, 3),
                i::NewI("ADC", Self::ADC, Self::ZP0, 3),
                i::NewI("ROR", Self::ROR, Self::ZP0, 5),
                i::NewI("???", Self::XXX, Self::IMP, 5),
                i::NewI("PLA", Self::PLA, Self::IMP, 4),
                i::NewI("ADC", Self::ADC, Self::IMM, 2),
                i::NewI("ROR", Self::ROR, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("JMP", Self::JMP, Self::IND, 5),
                i::NewI("ADC", Self::ADC, Self::ABS, 4),
                i::NewI("ROR", Self::ROR, Self::ABS, 6),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("BVS", Self::BVS, Self::REL, 2),
                i::NewI("ADC", Self::ADC, Self::IZY, 5),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 8),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("ADC", Self::ADC, Self::ZPX, 4),
                i::NewI("ROR", Self::ROR, Self::ZPX, 6),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("SEI", Self::SEI, Self::IMP, 2),
                i::NewI("ADC", Self::ADC, Self::ABY, 4),
                i::NewI("???", Self::NOP, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 7),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("ADC", Self::ADC, Self::ABX, 4),
                i::NewI("ROR", Self::ROR, Self::ABX, 7),
                i::NewI("???", Self::XXX, Self::IMP, 7),
                i::NewI("???", Self::NOP, Self::IMP, 2),
                i::NewI("STA", Self::STA, Self::IZX, 6),
                i::NewI("???", Self::NOP, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("STY", Self::STY, Self::ZP0, 3),
                i::NewI("STA", Self::STA, Self::ZP0, 3),
                i::NewI("STX", Self::STX, Self::ZP0, 3),
                i::NewI("???", Self::XXX, Self::IMP, 3),
                i::NewI("DEY", Self::DEY, Self::IMP, 2),
                i::NewI("???", Self::NOP, Self::IMP, 2),
                i::NewI("TXA", Self::TXA, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("STY", Self::STY, Self::ABS, 4),
                i::NewI("STA", Self::STA, Self::ABS, 4),
                i::NewI("STX", Self::STX, Self::ABS, 4),
                i::NewI("???", Self::XXX, Self::IMP, 4),
                i::NewI("BCC", Self::BCC, Self::REL, 2),
                i::NewI("STA", Self::STA, Self::IZY, 6),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("STY", Self::STY, Self::ZPX, 4),
                i::NewI("STA", Self::STA, Self::ZPX, 4),
                i::NewI("STX", Self::STX, Self::ZPY, 4),
                i::NewI("???", Self::XXX, Self::IMP, 4),
                i::NewI("TYA", Self::TYA, Self::IMP, 2),
                i::NewI("STA", Self::STA, Self::ABY, 5),
                i::NewI("TXS", Self::TXS, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 5),
                i::NewI("???", Self::NOP, Self::IMP, 5),
                i::NewI("STA", Self::STA, Self::ABX, 5),
                i::NewI("???", Self::XXX, Self::IMP, 5),
                i::NewI("???", Self::XXX, Self::IMP, 5),
                i::NewI("LDY", Self::LDY, Self::IMM, 2),
                i::NewI("LDA", Self::LDA, Self::IZX, 6),
                i::NewI("LDX", Self::LDX, Self::IMM, 2),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("LDY", Self::LDY, Self::ZP0, 3),
                i::NewI("LDA", Self::LDA, Self::ZP0, 3),
                i::NewI("LDX", Self::LDX, Self::ZP0, 3),
                i::NewI("???", Self::XXX, Self::IMP, 3),
                i::NewI("TAY", Self::TAY, Self::IMP, 2),
                i::NewI("LDA", Self::LDA, Self::IMM, 2),
                i::NewI("TAX", Self::TAX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("LDY", Self::LDY, Self::ABS, 4),
                i::NewI("LDA", Self::LDA, Self::ABS, 4),
                i::NewI("LDX", Self::LDX, Self::ABS, 4),
                i::NewI("???", Self::XXX, Self::IMP, 4),
                i::NewI("BCS", Self::BCS, Self::REL, 2),
                i::NewI("LDA", Self::LDA, Self::IZY, 5),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 5),
                i::NewI("LDY", Self::LDY, Self::ZPX, 4),
                i::NewI("LDA", Self::LDA, Self::ZPX, 4),
                i::NewI("LDX", Self::LDX, Self::ZPY, 4),
                i::NewI("???", Self::XXX, Self::IMP, 4),
                i::NewI("CLV", Self::CLV, Self::IMP, 2),
                i::NewI("LDA", Self::LDA, Self::ABY, 4),
                i::NewI("TSX", Self::TSX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 4),
                i::NewI("LDY", Self::LDY, Self::ABX, 4),
                i::NewI("LDA", Self::LDA, Self::ABX, 4),
                i::NewI("LDX", Self::LDX, Self::ABY, 4),
                i::NewI("???", Self::XXX, Self::IMP, 4),
                i::NewI("CPY", Self::CPY, Self::IMM, 2),
                i::NewI("CMP", Self::CMP, Self::IZX, 6),
                i::NewI("???", Self::NOP, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 8),
                i::NewI("CPY", Self::CPY, Self::ZP0, 3),
                i::NewI("CMP", Self::CMP, Self::ZP0, 3),
                i::NewI("DEC", Self::DEC, Self::ZP0, 5),
                i::NewI("???", Self::XXX, Self::IMP, 5),
                i::NewI("INY", Self::INY, Self::IMP, 2),
                i::NewI("CMP", Self::CMP, Self::IMM, 2),
                i::NewI("DEX", Self::DEX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("CPY", Self::CPY, Self::ABS, 4),
                i::NewI("CMP", Self::CMP, Self::ABS, 4),
                i::NewI("DEC", Self::DEC, Self::ABS, 6),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("BNE", Self::BNE, Self::REL, 2),
                i::NewI("CMP", Self::CMP, Self::IZY, 5),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 8),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("CMP", Self::CMP, Self::ZPX, 4),
                i::NewI("DEC", Self::DEC, Self::ZPX, 6),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("CLD", Self::CLD, Self::IMP, 2),
                i::NewI("CMP", Self::CMP, Self::ABY, 4),
                i::NewI("NOP", Self::NOP, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 7),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("CMP", Self::CMP, Self::ABX, 4),
                i::NewI("DEC", Self::DEC, Self::ABX, 7),
                i::NewI("???", Self::XXX, Self::IMP, 7),
                i::NewI("CPX", Self::CPX, Self::IMM, 2),
                i::NewI("SBC", Self::SBC, Self::IZX, 6),
                i::NewI("???", Self::NOP, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 8),
                i::NewI("CPX", Self::CPX, Self::ZP0, 3),
                i::NewI("SBC", Self::SBC, Self::ZP0, 3),
                i::NewI("INC", Self::INC, Self::ZP0, 5),
                i::NewI("???", Self::XXX, Self::IMP, 5),
                i::NewI("INX", Self::INX, Self::IMP, 2),
                i::NewI("SBC", Self::SBC, Self::IMM, 2),
                i::NewI("NOP", Self::NOP, Self::IMP, 2),
                i::NewI("???", Self::SBC, Self::IMP, 2),
                i::NewI("CPX", Self::CPX, Self::ABS, 4),
                i::NewI("SBC", Self::SBC, Self::ABS, 4),
                i::NewI("INC", Self::INC, Self::ABS, 6),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("BEQ", Self::BEQ, Self::REL, 2),
                i::NewI("SBC", Self::SBC, Self::IZY, 5),
                i::NewI("???", Self::XXX, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 8),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("SBC", Self::SBC, Self::ZPX, 4),
                i::NewI("INC", Self::INC, Self::ZPX, 6),
                i::NewI("???", Self::XXX, Self::IMP, 6),
                i::NewI("SED", Self::SED, Self::IMP, 2),
                i::NewI("SBC", Self::SBC, Self::ABY, 4),
                i::NewI("NOP", Self::NOP, Self::IMP, 2),
                i::NewI("???", Self::XXX, Self::IMP, 7),
                i::NewI("???", Self::NOP, Self::IMP, 4),
                i::NewI("SBC", Self::SBC, Self::ABX, 4),
                i::NewI("INC", Self::INC, Self::ABX, 7),
                i::NewI("???", Self::XXX, Self::IMP, 7),
            ],
        };
    }

    pub fn connect_bus(&mut self, bus: &mut Bus) {
        self.bus = bus;
    }

    unsafe fn write(&mut self, addr: usize, data: u8) {
        let b = &mut *self.bus;
        b.write(addr, data);
    }
    unsafe fn pcread(&self) -> u16 {
        return self.read(&self.pc, false).into();
    }
    unsafe fn read(&self, addr: &u16, b_read_only: bool) -> u8 {
        let b = &*self.bus;
        let ua = usize::from(*addr);
        return b.read(ua, b_read_only);
    }

    pub fn get_flag(&self, f: FLAGS6502) -> u8 {
        return self.status & (f as u8);
    }
    pub fn set_flag(&mut self, f: FLAGS6502, v: bool) {
        if v {
            self.status = self.status | (f as u8);
        } else {
            self.status = self.status & (f as u8);
        }
    }
}
