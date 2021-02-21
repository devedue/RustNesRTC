use crate::cpu::*;

pub struct Bus {
    pub cpu: Cpu,
    pub ram: [u8; 64 * 1024],
}

impl Bus {
    pub unsafe fn new() -> Bus {
        let mut cpu = Cpu::new();
        let mut b = Bus {
            cpu: Cpu::new(),
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
