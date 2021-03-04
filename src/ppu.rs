use crate::cartridge::*;
use pge::Pixel;
use pge::Sprite;
use std::cell::RefCell;
use std::rc::Rc;

use bitfield::*;

bitfield! {
    struct StatusBits(u8);
    u8;
    unused, _ : 4,0;
    sprite_overflow, set_sprite_overflow : 5;
    sprite_zero_hit, set_sprite_zero_hit : 6;
    vertical_blank, set_vertical_blank : 7;
}

bitfield! {
    struct MaskBits(u8);
    u8;
    grayscale, set_grayscale : 0;
    render_background_left, set_render_background_left : 1;
    render_sprites_left, set_render_sprites_left : 2;
    render_background, set_render_background : 3;
    render_sprites, set_render_sprites : 4;
    enhance_red, set_enhance_red : 5;
    enhance_green, set_enhance_green : 6;
    enhance_blue, set_enhance_blue : 7;
}

bitfield! {
    struct ControlBits(u8);
    u8;
    nametable_x, set_nametable_x : 0;
    nametable_y, set_nametable_y : 1;
    increment_mode, set_increment_mode : 2;
    pattern_sprite, set_pattern_sprite : 3;
    pattern_background, set_pattern_background : 4;
    sprite_size, set_sprite_size : 5;
    slave_mode, set_slave_mode: 6; // unused
    enable_nmi, set_enable_nmi : 7;
}
bitfield! {
    struct RegisterBits(u16);
    u16;
    coarse_x, set_coarse_x : 4,0;
    coarse_y, set_coarse_y : 9,5;
    nametable_x, set_nametable_x : 10;
    nametable_y, set_nametable_y : 11;
    fine_y, set_fine_y : 14,12;
    unused, set_unused : 15;
}

union Status {
    bits: StatusBits,
    reg: u8,
}

union Mask {
    bits: MaskBits,
    reg: u8,
}

union Control {
    bits: ControlBits,
    reg: u8,
}

union Register {
    bits: RegisterBits,
    reg: u16,
}

pub struct Ppu {
    cart: Option<Rc<RefCell<Cartridge>>>,
    _vram: [[u8; 1024]; 2],
    pub tbl_name: [[u8; 1024]; 2],
    tbl_pattern: [[u8; 4096]; 2],
    tbl_palette: [u8; 32],
    pal_screen: [Pixel; 0x40],
    pub spr_screen: Sprite,
    _spr_name_table: [Sprite; 2],
    spr_pattern_table: [Sprite; 2],
    scan_line: i16,
    cycle: i16,
    pub counter: u128,
    status: Status,
    mask: Mask,
    control: Control,
    vram_addr: Register,
    tram_addr: Register,

    fine_x: u8,

    address_latch: u8,
    ppu_data_buffer: u8,

    bg_next_tile_id: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,
    bg_shifter_pattern_lo: u16,
    bg_shifter_pattern_hi: u16,
    bg_shifter_attrib_lo: u16,
    bg_shifter_attrib_hi: u16,

    pub nmi: bool,

    // debug
    pub frame_complete: bool, // tbl_pattern: [[u8; 4096]; 2], olc future
}

impl Ppu {
    pub fn new() -> Self {
        let newppu = Ppu {
            cart: None,
            _vram: [[0; 1024]; 2],
            tbl_name: [[0; 1024]; 2],
            tbl_pattern: [[0; 4096]; 2],
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
            _spr_name_table: [Sprite::new(256, 240), Sprite::new(256, 240)],
            spr_pattern_table: [Sprite::new(256, 240), Sprite::new(256, 240)],
            scan_line: 0,
            cycle: 0,
            frame_complete: false,
            counter: 0,
            status: Status { reg: 0 },
            control: Control { reg: 0 },
            mask: Mask { reg: 0 },
            vram_addr: Register { reg: 0 },
            tram_addr: Register { reg: 0 },
            fine_x: 0x00,

            address_latch: 0x00,
            ppu_data_buffer: 0x00,

            bg_next_tile_id: 0x00,
            bg_next_tile_attrib: 0x00,
            bg_next_tile_lsb: 0x00,
            bg_next_tile_msb: 0x00,
            bg_shifter_pattern_lo: 0x0000,
            bg_shifter_pattern_hi: 0x0000,
            bg_shifter_attrib_lo: 0x0000,
            bg_shifter_attrib_hi: 0x0000,
            nmi: false
        };

        return newppu;
    }
    // Communications with cpu bus
    pub fn cpu_read(&mut self, addr: usize, _rdonly: bool) -> u8 {
        let mut data = 0x00;
        unsafe {
            match addr {
                0x0000 => {
                    data = self.control.reg;
                }
                0x0001 => {
                    data = self.mask.reg;
                }
                0x0002 => {
                    // Status register
                    data = (self.status.reg & 0xE0) | (self.ppu_data_buffer & 0x1F);
                    self.status.bits.set_vertical_blank(false);
                    self.address_latch = 0;
                }
                0x0003 => {}
                0x0004 => {}
                0x0005 => {}
                0x0006 => {}
                0x0007 => {
                    data = self.ppu_data_buffer;
                    self.ppu_data_buffer = self.ppu_read(self.vram_addr.reg, false);
                    if self.vram_addr.reg >= 0x3F00 {
                        data = self.ppu_data_buffer;
                    }
                    if self.control.bits.increment_mode() {
                        self.vram_addr.reg = self.vram_addr.reg.wrapping_add(32);
                    } else {
                        self.vram_addr.reg = self.vram_addr.reg.wrapping_add(1);
                    }
                }
                _ => {}
            }
        }
        return data;
    }

    pub fn cpu_write(&mut self, addr: usize, data: u8) {
        unsafe {
            match addr {
                0x0000 => {
                    self.control.reg = data;
                    self.tram_addr
                        .bits
                        .set_nametable_x(self.control.bits.nametable_x());
                    self.tram_addr
                        .bits
                        .set_nametable_y(self.control.bits.nametable_y());
                }
                0x0001 => {
                    self.mask.reg = data;
                }
                0x0002 => {}
                0x0003 => {}
                0x0004 => {}
                0x0005 => {
                    if self.address_latch == 0 {
                        self.fine_x = data & 0x07;
                        self.tram_addr.bits.set_coarse_x((data >> 2) as u16);
                        self.address_latch = 1;
                    } else {
                        self.tram_addr.bits.set_fine_y((data & 0x07).into());
                        self.tram_addr.bits.set_coarse_y((data >> 3) as u16);
                        self.address_latch = 0;
                    }
                }
                0x0006 => {
                    if self.address_latch == 0 {
                        self.tram_addr.reg = (((data & 0x3F) as u16) << 8) as u16
                            | ((self.tram_addr.reg & 0x00FF) as u16);
                        self.address_latch = 1;
                    } else {
                        self.tram_addr.reg = (self.tram_addr.reg & 0xFF00) | data as u16;
                        self.vram_addr.reg = self.tram_addr.reg;
                        self.address_latch = 0;
                    }
                }
                0x0007 => {
                    self.ppu_write(self.vram_addr.reg, data);
                    if self.control.bits.increment_mode() {
                        self.vram_addr.reg = self.vram_addr.reg.wrapping_add(32);
                    } else {
                        self.vram_addr.reg = self.vram_addr.reg.wrapping_add(1);
                    }
                }
                _ => {}
            }
        }
    }

    // Communications with ppu bus
    pub fn ppu_read(&self, mut addr: u16, _: bool) -> u8 {
        let mut data = 0x00;
        addr = (addr as u16) & 0x3FFF;
        let cart = self.cart.as_ref().unwrap().borrow();

        if cart.ppu_read(addr, &mut data) {
        } else if addr <= 0x1FFF {
            data = self.tbl_pattern[((addr & 0x1000) >> 12) as usize][(addr & 0x0FFF) as usize];
        } else if addr <= 0x3EFF {
            addr = addr & 0x0FFF;
            let masked_addr = (addr & 0x03FF) as usize;
            match cart.mirror {
                Mirror::VERTICAL => {
                    if addr <= 0x03FF {
                        data = self.tbl_name[0][masked_addr];
                    }
                    if addr <= 0x07FF {
                        data = self.tbl_name[1][masked_addr];
                    }
                    if addr <= 0x0BFF {
                        data = self.tbl_name[0][masked_addr];
                    }
                    if addr <= 0x0FFF {
                        data = self.tbl_name[1][masked_addr];
                    }
                }
                Mirror::HORIZONTAL => {
                    if addr <= 0x03FF {
                        data = self.tbl_name[0][masked_addr];
                    }
                    if addr <= 0x07FF {
                        data = self.tbl_name[0][masked_addr];
                    }
                    if addr <= 0x0BFF {
                        data = self.tbl_name[1][masked_addr];
                    }
                    if addr <= 0x0FFF {
                        data = self.tbl_name[1][masked_addr];
                    }
                }
            }
        } else if addr <= 0x3FFF {
            addr = addr & 0x001F;
            if addr == 0x0010 {
                addr = 0x0000;
            }
            if addr == 0x0014 {
                addr = 0x0004;
            }
            if addr == 0x0018 {
                addr = 0x0008;
            }
            if addr == 0x001C {
                addr = 0x000C;
            }
            unsafe {
                if self.mask.bits.grayscale() {
                    data = self.tbl_palette[addr as usize] & 0x30;
                } else {
                    data = self.tbl_palette[addr as usize] & 0x3F;
                }
            }
        }

        return data;
    }
    pub fn ppu_write(&mut self, mut addr: u16, data: u8) {
        addr = (addr as u16) & 0x3FFF;
        let mut cart = self.cart.as_ref().unwrap().borrow_mut();

        if cart.ppu_write(addr as usize, data) {
        } else if addr <= 0x1FFF {
            self.tbl_pattern[((addr & 0x1000) >> 12) as usize][(addr & 0x0FFF) as usize] = data;
        } else if addr <= 0x3EFF {
            addr = addr & 0x0FFF;
            let masked_addr = (addr & 0x03FF) as usize;
            match cart.mirror {
                Mirror::VERTICAL => {
                    if addr <= 0x03FF {
                        self.tbl_name[0][masked_addr] = data;
                    }
                    if addr <= 0x07FF {
                        self.tbl_name[1][masked_addr] = data;
                    }
                    if addr <= 0x0BFF {
                        self.tbl_name[0][masked_addr] = data;
                    }
                    if addr <= 0x0FFF {
                        self.tbl_name[1][masked_addr] = data;
                    }
                }
                Mirror::HORIZONTAL => {
                    if addr <= 0x03FF {
                        self.tbl_name[0][masked_addr] = data;
                    }
                    if addr <= 0x07FF {
                        self.tbl_name[0][masked_addr] = data;
                    }
                    if addr <= 0x0BFF {
                        self.tbl_name[1][masked_addr] = data;
                    }
                    if addr <= 0x0FFF {
                        self.tbl_name[1][masked_addr] = data;
                    }
                }
            }
        } else if addr <= 0x3FFF {
            addr = addr & 0x001F;
            if addr == 0x0010 {
                addr = 0x0000;
            }
            if addr == 0x0014 {
                addr = 0x0004;
            }
            if addr == 0x0018 {
                addr = 0x0008;
            }
            if addr == 0x001C {
                addr = 0x000C;
            }
            self.tbl_palette[addr as usize] = data;
        }
    }

    pub fn connect_cartridge(&mut self, cart: Rc<RefCell<Cartridge>>) {
        self.cart = Some(cart.clone());
    }

    // pub fn get_decal(&self) -> &Decal {
    //     &Decal::create(Some(*self.spr_screen.deref()))
    // }

    pub fn get_palette_color(&self, palette: u8, pixel: u8) -> Pixel {
        let i = self.ppu_read(0x3F00 + (palette << 2) as u16 + (pixel as u16), false);
        return self.pal_screen[(i & 0x3F) as usize];
    }

    pub fn get_pattern_table(&mut self, i: u8, palette: u8) -> Sprite {
        for ty in 0..16 {
            for tx in 0..16 {
                let offset = ty * 256 + tx * 16;
                for row in 0..8 {
                    let mut tile_lsb = self.ppu_read((i as u16) * 0x1000 + offset + row + 0, false);
                    let mut tile_msb = self.ppu_read((i as u16) * 0x1000 + offset + row + 8, false);
                    for col in 0..8 {
                        let pixel = (tile_lsb & 0x01) + (tile_msb & 0x01);
                        tile_lsb = tile_lsb >> 1;
                        tile_msb = tile_msb >> 1;

                        self.spr_pattern_table[i as usize].set_pixel(
                            (tx * 8 + (7 - col)) as i32,
                            (ty * 8 + row) as i32,
                            &self.get_palette_color(palette, pixel),
                        );
                    }
                }
            }
        }

        return self.spr_pattern_table[i as usize].clone();
    }

    // pub fn get_name_table(&self, i: usize) -> Sprite {
    //     return self.spr_name_table[i].clone();
    // }

    unsafe fn increment_scroll_x(&mut self) {
        if self.mask.bits.render_background() || self.mask.bits.render_sprites() {
            if self.vram_addr.bits.coarse_x() == 31 {
                self.vram_addr.bits.set_coarse_x(0);
                self.vram_addr
                    .bits
                    .set_nametable_x(!self.vram_addr.bits.nametable_x());
            } else {
                self.vram_addr
                    .bits
                    .set_coarse_x(self.vram_addr.bits.coarse_x() + 1);
            }
        }
    }

    unsafe fn increment_scroll_y(&mut self) {
        if self.mask.bits.render_background() || self.mask.bits.render_sprites() {
            // If possible, just increment the fine y offset
            if self.vram_addr.bits.fine_y() < 7 {
                self.vram_addr
                    .bits
                    .set_fine_y(self.vram_addr.bits.fine_y().wrapping_add(1));
            } else {
                self.vram_addr.bits.set_fine_y(0);

                if self.vram_addr.bits.coarse_y() == 29 {
                    self.vram_addr.bits.set_coarse_y(0);
                    self.vram_addr
                        .bits
                        .set_nametable_y(!self.vram_addr.bits.nametable_y());
                } else if self.vram_addr.bits.coarse_y() == 31 {
                    self.vram_addr.bits.set_coarse_y(0);
                } else {
                    self.vram_addr
                        .bits
                        .set_coarse_y(self.vram_addr.bits.coarse_y().wrapping_add(1));
                }
            }
        }
    }

    unsafe fn transfer_address_x(&mut self) {
        if self.mask.bits.render_background() || self.mask.bits.render_sprites() {
            self.vram_addr
                .bits
                .set_nametable_x(self.tram_addr.bits.nametable_x());
            self.vram_addr
                .bits
                .set_coarse_x(self.tram_addr.bits.coarse_x());
        }
    }

    unsafe fn transfer_address_y(&mut self) {
        if self.mask.bits.render_background() || self.mask.bits.render_sprites() {
            self.vram_addr.bits.set_fine_y(self.tram_addr.bits.fine_y());
            self.vram_addr
                .bits
                .set_nametable_y(self.tram_addr.bits.nametable_y());
            self.vram_addr
                .bits
                .set_coarse_y(self.tram_addr.bits.coarse_y());
        }
    }

    unsafe fn load_background_shifters(&mut self) {
        self.bg_shifter_pattern_lo =
            (self.bg_shifter_pattern_lo & 0xFF00) | (self.bg_next_tile_lsb as u16);
        self.bg_shifter_pattern_hi =
            (self.bg_shifter_pattern_hi & 0xFF00) | (self.bg_next_tile_msb as u16);
        if self.bg_next_tile_attrib & 0b01 > 0 {
            self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00) | 0x00FF;
        } else {
            self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00) | 0x00;
        }
        if self.bg_next_tile_attrib & 0b10 > 0 {
            self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00) | 0x00FF;
        } else {
            self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00) | 0x00;
        }
    }

    unsafe fn update_shifters(&mut self) {
        if self.mask.bits.render_background() {
            self.bg_shifter_pattern_lo = self.bg_shifter_pattern_lo << 1;
            self.bg_shifter_pattern_hi = self.bg_shifter_pattern_hi << 1;

            self.bg_shifter_attrib_lo = self.bg_shifter_attrib_lo << 1;
            self.bg_shifter_attrib_hi = self.bg_shifter_attrib_hi << 1;
        }
    }

    pub fn clock(&mut self) -> u128 {
        unsafe {
            if self.scan_line >= -1 && self.scan_line < 240 {
                if self.scan_line == 0 && self.cycle == 0 {
                    self.cycle = 1;
                }
                if self.scan_line == -1 && self.cycle == 1 {
                    self.status.bits.set_vertical_blank(false);
                }

                if (self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338)
                {
                    self.update_shifters();
                    let matcher = (self.cycle - 1) % 8;
                    match matcher {
                        0 => {
                            self.load_background_shifters();
                            self.bg_next_tile_id =
                                self.ppu_read(0x2000 | (self.vram_addr.reg & 0x0FFF), false);
                        }
                        2 => {
                            self.bg_next_tile_attrib = self.ppu_read(
                                0x23C0
                                    | ((self.vram_addr.bits.nametable_y() as u16) << 11)
                                    | ((self.vram_addr.bits.nametable_x() as u16) << 10)
                                    | (((self.vram_addr.bits.coarse_y() as u16) >> 2) << 3)
                                    | ((self.vram_addr.bits.coarse_x() as u16) >> 2),
                                false,
                            );
                            if (self.vram_addr.bits.coarse_y() as u16) & 0x02 > 0 {
                                self.bg_next_tile_attrib = self.bg_next_tile_attrib >> 4;
                            }
                            if (self.vram_addr.bits.coarse_x() as u16) & 0x02 > 0 {
                                self.bg_next_tile_attrib = self.bg_next_tile_attrib >> 2
                            };
                            self.bg_next_tile_attrib = self.bg_next_tile_attrib & 0x03;
                        }
                        4 => {
                            self.bg_next_tile_lsb = self.ppu_read(
                                ((self.control.bits.pattern_background() as u16) << 12)
                                    + ((self.bg_next_tile_id as u16) << 4)
                                    + (self.vram_addr.bits.fine_y() as u16)
                                    + 0,
                                false,
                            );
                        }
                        6 => {
                            self.bg_next_tile_msb = self.ppu_read(
                                ((self.control.bits.pattern_background() as u16) << 12)
                                    + ((self.bg_next_tile_id as u16) << 4)
                                    + (self.vram_addr.bits.fine_y() as u16)
                                    + 8,
                                false,
                            );
                        }
                        7 => {
                            self.increment_scroll_x();
                        }
                        _ => {}
                    }
                }

                if self.cycle == 256 {
                    self.increment_scroll_y();
                }

                if self.cycle == 257 {
                    self.load_background_shifters();
                    self.transfer_address_x();
                }

                if self.cycle == 338 || self.cycle == 340 {
                    self.bg_next_tile_id =
                        self.ppu_read(0x2000 | (self.vram_addr.reg & 0x0FFF), false);
                }

                if self.scan_line == -1 && self.cycle >= 280 && self.cycle < 305 {
                    self.transfer_address_y();
                }
            }

            if self.scan_line >= 241 && self.scan_line < 261 {
                if self.scan_line == 241 && self.cycle == 1 {
                    self.status.bits.set_vertical_blank(true);
                    if self.control.bits.enable_nmi() {
                        self.nmi = true;
                    }
                }
            }

            let mut bg_pixel = 0x00;
            let mut bg_palette = 0x00;

            if self.mask.bits.render_background() {
                let bit_mux = 0x8000 >> self.fine_x;

                let p0_pixel = (self.bg_shifter_pattern_lo & bit_mux) > 0;
                let p1_pixel = (self.bg_shifter_pattern_hi & bit_mux) > 0;

                bg_pixel = ((p1_pixel as u8) << 1) | (p0_pixel as u8);

                let bg_pal0 = (self.bg_shifter_attrib_lo & bit_mux) > 0;
                let bg_pal1 = (self.bg_shifter_attrib_hi & bit_mux) > 0;
                bg_palette = ((bg_pal1 as u8) << 1) | (bg_pal0 as u8);
            }

            let sprx = self.cycle - 1;
            let spry = self.scan_line;
            let sprc = self.get_palette_color(bg_palette, bg_pixel);
            self.spr_screen.set_pixel(sprx.into(), spry.into(), &sprc);
        }

        self.cycle = self.cycle + 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scan_line = self.scan_line + 1;
            if self.scan_line >= 261 {
                self.scan_line = -1;
                self.frame_complete = true;
            }
        }

        self.counter = self.counter + 1;
        return self.counter;
    }

    pub fn reset(&mut self) {
        self.fine_x = 0x00;
        self.address_latch = 0x00;
        self.ppu_data_buffer = 0x00;
        self.scan_line = 0;
        self.cycle = 0;
        self.bg_next_tile_id = 0x00;
        self.bg_next_tile_attrib = 0x00;
        self.bg_next_tile_lsb = 0x00;
        self.bg_next_tile_msb = 0x00;
        self.bg_shifter_pattern_lo = 0x0000;
        self.bg_shifter_pattern_hi = 0x0000;
        self.bg_shifter_attrib_lo = 0x0000;
        self.bg_shifter_attrib_hi = 0x0000;
        self.status.reg = 0x00;
        self.mask.reg = 0x00;
        self.control.reg = 0x00;
        self.vram_addr.reg = 0x0000;
        self.tram_addr.reg = 0x0000;
    }
}
