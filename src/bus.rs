use crate::olc6502::*;

pub struct Bus {
    pub cpu: Olc6502,
    pub ram: [u8; 64 * 1024],
}

impl Bus {
    pub unsafe fn new() -> Bus {
        let mut cpu = Olc6502::new();
        let mut b = Bus {
            cpu: Olc6502::new(),
            ram: [0; 64 * 1024],
        };
        cpu.connect_bus(&mut b);
        b.cpu = cpu;
        return b;
    }
    pub fn write(&mut self, addr: usize, data: u8) {
        if addr <= 0xFFFF {
            self.ram[addr] = data;
        }
    }
    pub fn read(&self, addr: usize, _: bool) -> u8 {
        return self.ram[addr];
    }
}
