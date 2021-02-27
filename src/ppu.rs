use crate::cartridge::*;
use pge::Pixel;
use pge::Sprite;
use pge::RED;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Ppu {
    cart: Option<Rc<RefCell<Cartridge>>>,
    vram: [[u8; 1024]; 2],
    tbl_name: [[u8; 1024]; 2],
    tbl_palette: [u8; 32],
    pal_screen: [Pixel; 0x40],
    pub spr_screen: Sprite,
    spr_name_table: [Sprite; 2],
    spr_pattern_table: [Sprite; 2],
    scan_line: u16,
    cycle: u16,
    counter: u128,

    // debug
    pub frame_complete: bool, // tbl_pattern: [[u8; 4096]; 2], olc future
}

impl Ppu {
    pub fn new() -> Self {
        let newppu = Ppu {
            cart: None,
            vram: [[0; 1024]; 2],
            tbl_name: [[0; 1024]; 2],
            tbl_palette: [0; 32],
            pal_screen: [
                Pixel::rgb(84, 84, 84),
                Pixel::rgb(0, 30, 116),
                Pixel::rgb(8, 16, 144),
                Pixel::rgb(48, 0, 136),
                Pixel::rgb(68, 0, 100),
                Pixel::rgb(92, 0, 48),
                Pixel::rgb(84, 4, 0),
                Pixel::rgb(60, 24, 0),
                Pixel::rgb(32, 42, 0),
                Pixel::rgb(8, 58, 0),
                Pixel::rgb(0, 64, 0),
                Pixel::rgb(0, 60, 0),
                Pixel::rgb(0, 50, 60),
                Pixel::rgb(0, 0, 0),
                Pixel::rgb(0, 0, 0),
                Pixel::rgb(0, 0, 0),
                Pixel::rgb(152, 150, 152),
                Pixel::rgb(8, 76, 196),
                Pixel::rgb(48, 50, 236),
                Pixel::rgb(92, 30, 228),
                Pixel::rgb(136, 20, 176),
                Pixel::rgb(160, 20, 100),
                Pixel::rgb(152, 34, 32),
                Pixel::rgb(120, 60, 0),
                Pixel::rgb(84, 90, 0),
                Pixel::rgb(40, 114, 0),
                Pixel::rgb(8, 124, 0),
                Pixel::rgb(0, 118, 40),
                Pixel::rgb(0, 102, 120),
                Pixel::rgb(0, 0, 0),
                Pixel::rgb(0, 0, 0),
                Pixel::rgb(0, 0, 0),
                Pixel::rgb(236, 238, 236),
                Pixel::rgb(76, 154, 236),
                Pixel::rgb(120, 124, 236),
                Pixel::rgb(176, 98, 236),
                Pixel::rgb(228, 84, 236),
                Pixel::rgb(236, 88, 180),
                Pixel::rgb(236, 106, 100),
                Pixel::rgb(212, 136, 32),
                Pixel::rgb(160, 170, 0),
                Pixel::rgb(116, 196, 0),
                Pixel::rgb(76, 208, 32),
                Pixel::rgb(56, 204, 108),
                Pixel::rgb(56, 180, 204),
                Pixel::rgb(60, 60, 60),
                Pixel::rgb(0, 0, 0),
                Pixel::rgb(0, 0, 0),
                Pixel::rgb(236, 238, 236),
                Pixel::rgb(168, 204, 236),
                Pixel::rgb(188, 188, 236),
                Pixel::rgb(212, 178, 236),
                Pixel::rgb(236, 174, 236),
                Pixel::rgb(236, 174, 212),
                Pixel::rgb(236, 180, 176),
                Pixel::rgb(228, 196, 144),
                Pixel::rgb(204, 210, 120),
                Pixel::rgb(180, 222, 120),
                Pixel::rgb(168, 226, 144),
                Pixel::rgb(152, 226, 180),
                Pixel::rgb(160, 214, 228),
                Pixel::rgb(160, 162, 160),
                Pixel::rgb(0, 0, 0),
                Pixel::rgb(0, 0, 0),
            ],
            spr_screen: Sprite::new(256, 240),
            spr_name_table: [Sprite::new(256, 240), Sprite::new(256, 240)],
            spr_pattern_table: [Sprite::new(256, 240), Sprite::new(256, 240)],
            scan_line: 0,
            cycle: 0,
            frame_complete: false,
            counter: 0,
        };

        return newppu;
    }
    pub fn empty() -> Self {
        return Ppu {
            cart: None,
            vram: [[0; 1024]; 2],
            tbl_name: [[0; 1024]; 2],
            tbl_palette: [0; 32],
            pal_screen: [RED; 0x40],
            spr_screen: Sprite::new(256, 240),
            spr_name_table: [Sprite::new(256, 240), Sprite::new(256, 240)],
            spr_pattern_table: [Sprite::new(256, 240), Sprite::new(256, 240)],
            scan_line: 0,
            cycle: 0, // tbl_pattern: [[0; 4096]; 2], olc future
            frame_complete: false,
            counter: 0,
        };
    }
    // Communications with cpu bus
    pub fn cpu_read(&self, addr: usize, rdonly: bool) -> u8 {
        let data = 0x00;
        match addr {
            0x0000 => {}
            0x0001 => {}
            0x0002 => {}
            0x0003 => {}
            0x0004 => {}
            0x0005 => {}
            0x0006 => {}
            0x0007 => {}
            _ => {}
        }
        return data;
    }
    pub fn cpu_write(&mut self, addr: usize, data: u8) {
        let data = 0x00;
        match addr {
            0x0000 => {}
            0x0001 => {}
            0x0002 => {}
            0x0003 => {}
            0x0004 => {}
            0x0005 => {}
            0x0006 => {}
            0x0007 => {}
            _ => {}
        }
    }

    // Communications with ppu bus
    pub fn ppu_read(&self, mut addr: usize, _: bool) -> u8 {
        let mut data = 0;
        addr = ((addr as u16) & 0x3FFF) as usize;

        if self
            .cart
            .as_ref()
            .unwrap()
            .borrow()
            .ppu_read(addr, &mut data)
        {}

        return data;
    }
    pub fn ppu_write(&mut self, mut addr: usize, data: u8) {
        addr = ((addr as u16) & 0x3FFF) as usize;
        unsafe {
            if self
                .cart
                .as_ref()
                .unwrap()
                .borrow_mut()
                .ppu_write(addr, data)
            {}
        }
    }

    pub fn connect_cartridge(&mut self, cart: Rc<RefCell<Cartridge>>) {
        self.cart = Some(cart.clone());
    }

    // pub fn get_decal(&self) -> &Decal {
    //     &Decal::create(Some(*self.spr_screen.deref()))
    // }

    pub fn clock(&mut self) -> u128 {
        // Fake some noise for now
        let mut position: usize = 0;
        if rand::random::<u16>() % 2 > 0 {
            position = 0x3F;
        } else {
            position = 0x30;
        }
        let sprx = self.cycle.wrapping_sub(1);
        let spry = self.scan_line;
        let sprc = self.pal_screen[position];
        self.spr_screen.set_pixel(sprx.into(), spry.into(), &sprc);

        // Advance renderer - it never stops, it's relentless
        self.cycle = self.cycle + 1;
        if (self.cycle >= 341) {
            self.cycle = 0;
            self.scan_line = self.scan_line + 1;
            if (self.scan_line >= 261) {
                self.scan_line = 0;
                self.frame_complete = true;
            }
        }

        self.counter = self.counter + 1;
        return self.counter;
    }
}
