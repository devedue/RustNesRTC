use crate::apu::Apu;
use crate::cartridge::*;
use crate::ppu::*;
use std::cell::RefCell;
use std::rc::Rc;

// For Logging:
// use std::io::Write;
// use crate::util::hex;

pub struct Bus {
    ppu: Ppu,
    pub apu: Apu,
    cpu_ram: [u8; 2 * 1024],
    cart: Rc<RefCell<Cartridge>>,
    controller_state: [u8; 2],
    pub controller: [u8; 2],

    // Foreground Sprites
    pub dma_page: u8,
    pub dma_addr: u8,
    pub dma_data: u8,

    pub dma_transfer: bool,
    pub dma_dummy: bool,

    // For Audio
    pub audio_time: f64,
    pub audio_global_time: f64,
    pub audio_time_per_clock: f64,
    pub audio_time_per_sample: f64,
    pub audio_sample: f64,
}

impl Bus {
    pub fn new() -> Self {
        let b = Bus {
            ppu: Ppu::new(),
            apu: Apu::new(),
            cpu_ram: [0; 2 * 1024],
            cart: Rc::new(RefCell::new(Cartridge::default())),
            controller: [0; 2],
            controller_state: [0; 2],
            dma_page: 0,
            dma_addr: 0,
            dma_data: 0,
            dma_transfer: false,
            dma_dummy: false,

            audio_time: 0.0,
            audio_global_time: 0.0,
            audio_time_per_clock: 0.0,
            audio_time_per_sample: 0.0,
            audio_sample: 0.0,
        };
        return b;
    }

    pub fn reset(&mut self) {
        self.dma_page = 0x00;
        self.dma_addr = 0x00;
        self.dma_data = 0x00;
        self.dma_dummy = true;
        self.dma_transfer = false;
        // self.cart.as_ref().borrow().reset();
        self.ppu.reset();
    }

    pub fn write(&mut self, addr: usize, data: u8) {
        if self.cart.as_ref().borrow_mut().cpu_write(addr, data) {
        } else if addr <= 0x1FFF {
            self.cpu_ram[addr & 0x07FF] = data;
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            self.ppu.cpu_write(addr & 0x0007, data);
        } else if addr <= 0x4013 || addr == 0x4015 {
            self.apu.cpu_write(addr as u16, data);
        } else if addr == 0x4014 {
            self.dma_page = data;
            self.dma_addr = 0x00;
            self.dma_transfer = true;
        } else if addr >= 0x4016 && addr <= 0x4017 {
            self.controller_state[addr & 0x0001] = self.controller[addr & 0x0001];
        }
    }
    pub fn read(&mut self, addr: usize, rdonly: bool) -> u8 {
        let mut data = 0;
        if self.cart.as_ref().borrow_mut().cpu_read(addr, &mut data) {
        } else if addr <= 0x1FFF {
            data = self.cpu_ram[addr & 0x07FF];
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            data = self.ppu.cpu_read(addr & 0x0007, rdonly);
        } else if addr==0x4015{
            data = self.apu.cpu_read(addr as u8);
        }else if addr >= 0x4016 && addr <= 0x4017 {
            if (self.controller_state[addr & 0x0001] & 0x80) > 0 {
                data = 1;
            } else {
                data = 0;
            }
            self.controller_state[addr & 0x0001] = self.controller_state[addr & 0x0001] << 1;
        }

        return data;
    }

    pub fn set_sample_frequency(&mut self, sample_rate: u32) {
        self.audio_time_per_sample = 1.0 / (sample_rate as f64);
        self.audio_time_per_clock = 1.0 / 5369318.0; // PPU Clock Frequency
    }

    pub fn insert_cartridge(&mut self, cart: Rc<RefCell<Cartridge>>) {
        self.cart = cart.clone();
        self.ppu.connect_cartridge(self.cart.clone());
    }

    pub fn get_ppu(&mut self) -> &mut Ppu {
        return &mut self.ppu;
    }
}
