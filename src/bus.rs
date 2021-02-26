use crate::cartridge::*;
use crate::cpu::*;
use crate::ppu::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub cpu_ram: [u8; 2 * 1024],
    clock_counter: u32,
    pub cart: Rc<RefCell<Cartridge>>,
    system_clock_counter: u128,
    id: u16,
}

impl Bus {
    pub fn new() -> Self {
        let mut cpu = Cpu::new();
        let mut b = Bus {
            id: rand::random::<u16>(),
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            cpu_ram: [0; 2 * 1024],
            clock_counter: 0,
            cart: Rc::new(RefCell::new(Cartridge::default())),
            system_clock_counter: 0,
        };
        return b;
    }
    pub fn empty() -> Bus {
        let mut cpu = Cpu::new();
        let mut b = Bus {
            id: rand::random::<u16>(),
            cpu: Cpu::new(),
            ppu: Ppu::empty(),
            cpu_ram: [0; 2 * 1024],
            clock_counter: 0,
            cart: Rc::new(RefCell::new(Cartridge::default())),
            system_clock_counter: 0,
        };
        // cpu.connect_bus(&mut b);
        // b.cpu = cpu;
        return b;
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.system_clock_counter = 0;
    }

    pub fn clock(&mut self) {
        self.ppu.clock();

        // The CPU runs 3 times slower than the PPU so we only call its
        // clock() function every 3 times this function is called. We
        // have a global counter to keep track of this.
        if (self.system_clock_counter % 3 == 0) {
            self.cpu.clock();
        }

        self.system_clock_counter = self.system_clock_counter + 1;
    }

    pub fn cpu_write(&mut self, addr: usize, data: u8) {
        if self.cart.as_ref().borrow_mut().cpu_write(addr, data) {
        } else if addr <= 0x1FFF {
            self.cpu_ram[addr & 0x07FF] = data;
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            self.ppu.cpu_write(addr, data);
        }
    }
    pub fn cpu_read(&self, addr: usize, _: bool) -> u8 {
        let mut data = 0;
        if self.cart.as_ref().borrow_mut().cpu_read(addr, &mut data) {}
        if addr <= 0x1FFF {
            data = self.cpu_ram[addr & 0x07FF];
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            data = self.ppu.cpu_read(addr, false);
        }
        return data;
    }

    pub fn insert_cartridge(&mut self, cart: Rc<RefCell<Cartridge>>) {
        self.cart = cart.clone();
        self.ppu.connect_cartridge(self.cart.clone());
    }
}
