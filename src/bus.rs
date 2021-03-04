use crate::cartridge::*;
use crate::ppu::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    ppu: Ppu,
    cpu_ram: [u8; 2 * 1024],
    cart: Rc<RefCell<Cartridge>>,
    controller_state: [u8; 2],
    pub controller: [u8; 2],
}

impl Bus {
    pub fn new() -> Self {
        let b = Bus {
            ppu: Ppu::new(),
            cpu_ram: [0; 2 * 1024],
            cart: Rc::new(RefCell::new(Cartridge::default())),
            controller: [0;2],
            controller_state: [0;2],
        };
        return b;
    }

    pub fn reset(&mut self) {
        // self.cart.as_ref().borrow().reset();
        self.ppu.reset();
    }

    pub fn write(&mut self, addr: usize, data: u8) {
        if self.cart.as_ref().borrow_mut().cpu_write(addr, data) {
        } else if addr <= 0x1FFF {
            self.cpu_ram[addr & 0x07FF] = data;
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            self.ppu.cpu_write(addr & 0x0007, data);
        } else if addr >= 0x4016 && addr <= 0x4017 {
            self.controller_state[addr & 0x0001] = self.controller[addr & 0x0001];
        }
    }
    pub fn read(&mut self, addr: usize, _: bool) -> u8 {
        let mut data = 0;
        if self.cart.as_ref().borrow_mut().cpu_read(addr, &mut data) {}
        if addr <= 0x1FFF {
            data = self.cpu_ram[addr & 0x07FF];
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            data = self.ppu.cpu_read(addr & 0x0007, false);
        } else if addr >= 0x4016 && addr <= 0x4017 {
            if (self.controller_state[addr & 0x0001] & 0x80) > 0 {
                data = 1;
            } else {
                data = 0;
            }
            self.controller_state[addr & 0x0001] = self.controller_state[addr & 0x0001] << 1;
        }

        return data;
    }

    pub fn insert_cartridge(&mut self, cart: Rc<RefCell<Cartridge>>) {
        self.cart = cart.clone();
        self.ppu.connect_cartridge(self.cart.clone());
    }

    pub fn get_ppu(&mut self) -> &mut Ppu {
        return &mut self.ppu;
    }
}
