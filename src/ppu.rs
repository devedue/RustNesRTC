use crate::cartridge::{Cartridge, Mirror};
use pge::Pixel;
use pge::Sprite;
use std::fs::File;
use std::sync::Arc;
use std::sync::Mutex;

// For Logging:
// use std::io::Write;

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

#[derive(Default, Copy, Clone)]
struct ObjectAttributeMap {
    y: u8,
    id: u8,
    attribute: u8,
    x: u8,
}

#[derive(Copy, Clone)]
union ObjectAttributeMemory {
    map: ObjectAttributeMap,
    data: [u8; 4],
}

impl Default for ObjectAttributeMemory {
    fn default() -> ObjectAttributeMemory {
        ObjectAttributeMemory { data: [0xFF; 4] }
    }
}

pub struct Ppu {
    cart: Option<Arc<Mutex<Cartridge>>>,
    _vram: [[u8; 1024]; 2],
    pub tbl_name: [[u8; 1024]; 2],
    tbl_pattern: [[u8; 4096]; 2],
    tbl_palette: [u8; 32],
    pub pal_screen: [Pixel; 0x40],
    pub pal_positions: [u8; 65535],
    pub spr_screen: Sprite,
    _spr_name_table: [Sprite; 2],
    _spr_pattern_table: [Sprite; 2],
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

    // Foreground Rendering
    oam: [ObjectAttributeMemory; 64],
    oam_addr: u8,

    sprite_scanline: [ObjectAttributeMemory; 8],
    sprite_count: u8,

    sprite_shifter_pattern_lo: [u8; 8],
    sprite_shifter_pattern_hi: [u8; 8],

    zero_hit_possible: bool,
    zero_sprite_rendered: bool,

    // debug
    pub frame_complete: bool, // tbl_pattern: [[u8; 4096]; 2], olc future
    pub logfile: File,
}

impl Ppu {
    pub fn new() -> Self {
        let newppu = Ppu {
            cart: None,
            _vram: [[0; 1024]; 2],
            tbl_name: [[0; 1024]; 2],
            tbl_pattern: [[0; 4096]; 2],
            tbl_palette: [0; 32],
            pal_positions: [0; 65535],
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
            _spr_pattern_table: [Sprite::new(256, 240), Sprite::new(256, 240)],
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
            nmi: false,

            oam: [ObjectAttributeMemory::default(); 64],
            oam_addr: 0x00,

            sprite_scanline: [ObjectAttributeMemory::default(); 8],
            sprite_count: 0x00,

            sprite_shifter_pattern_lo: [0; 8],
            sprite_shifter_pattern_hi: [0; 8],

            zero_hit_possible: false,
            zero_sprite_rendered: false,

            logfile: File::create("ppu-log.txt").unwrap(),
        };

        return newppu;
    }

    pub fn get_oam(&self, addr: usize) -> u8 {
        unsafe {
            return self.oam[(addr / 4)].data[(addr % 4)];
        }
    }

    pub fn set_oam(&mut self, addr: usize, data: u8) {
        unsafe {
            self.oam[(addr / 4)].data[(addr % 4)] = data;
        }
    }

    // Communications with cpu bus
    pub fn cpu_read(&mut self, addr: usize, rdonly: bool) -> u8 {
        let mut data = 0x00;
        unsafe {
            if rdonly {
                match addr {
                    0x0000 => {
                        data = self.control.reg;
                    }
                    0x0001 => {
                        data = self.mask.reg;
                    }
                    0x0002 => {
                        data = self.status.reg;
                    }
                    0x0003 => {}
                    0x0004 => {}
                    0x0005 => {}
                    0x0006 => {}
                    0x0007 => {}
                    _ => {}
                }
            } else {
                match addr {
                    0x0000 => {}
                    0x0001 => {}
                    0x0002 => {
                        // Status register
                        data = (self.status.reg & 0xE0) | (self.ppu_data_buffer & 0x1F);
                        self.status.bits.set_vertical_blank(false);
                        self.address_latch = 0;
                    }
                    0x0003 => {}
                    0x0004 => {
                        data = self.get_oam(self.oam_addr as usize);
                    }
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
                0x0003 => {
                    self.oam_addr = data;
                }
                0x0004 => {
                    self.set_oam(self.oam_addr as usize, data);
                }
                0x0005 => {
                    if self.address_latch == 0 {
                        self.fine_x = data & 0x07;
                        self.tram_addr.bits.set_coarse_x((data >> 3) as u16);
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
        let cart = self.cart.as_ref().unwrap().lock().unwrap();

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
                    } else if addr <= 0x07FF {
                        data = self.tbl_name[1][masked_addr];
                    } else if addr <= 0x0BFF {
                        data = self.tbl_name[0][masked_addr];
                    } else if addr <= 0x0FFF {
                        data = self.tbl_name[1][masked_addr];
                    }
                }
                Mirror::HORIZONTAL => {
                    if addr <= 0x03FF {
                        data = self.tbl_name[0][masked_addr];
                    } else if addr <= 0x07FF {
                        data = self.tbl_name[0][masked_addr];
                    } else if addr <= 0x0BFF {
                        data = self.tbl_name[1][masked_addr];
                    } else if addr <= 0x0FFF {
                        data = self.tbl_name[1][masked_addr];
                    }
                }
            }
        } else if addr <= 0x3FFF {
            addr = addr & 0x001F;
            match addr {
                0x0010 => {
                    addr = 0x0000;
                }
                0x0014 => {
                    addr = 0x0004;
                }
                0x0018 => {
                    addr = 0x0008;
                }
                0x001C => {
                    addr = 0x000C;
                }
                _ => {}
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
        let mut cart = self.cart.as_ref().unwrap().lock().unwrap();

        if (*cart).ppu_write(addr as usize, data) {
        } else if addr <= 0x1FFF {
            self.tbl_pattern[((addr & 0x1000) >> 12) as usize][(addr & 0x0FFF) as usize] = data;
        } else if addr <= 0x3EFF {
            addr = addr & 0x0FFF;
            let masked_addr = (addr & 0x03FF) as usize;
            match cart.mirror {
                Mirror::VERTICAL => {
                    if addr <= 0x03FF {
                        self.tbl_name[0][masked_addr] = data;
                    } else if addr <= 0x07FF {
                        self.tbl_name[1][masked_addr] = data;
                    } else if addr <= 0x0BFF {
                        self.tbl_name[0][masked_addr] = data;
                    } else if addr <= 0x0FFF {
                        self.tbl_name[1][masked_addr] = data;
                    }
                }
                Mirror::HORIZONTAL => {
                    if addr <= 0x03FF {
                        self.tbl_name[0][masked_addr] = data;
                    } else if addr <= 0x07FF {
                        self.tbl_name[0][masked_addr] = data;
                    } else if addr <= 0x0BFF {
                        self.tbl_name[1][masked_addr] = data;
                    } else if addr <= 0x0FFF {
                        self.tbl_name[1][masked_addr] = data;
                    }
                }
            }
        } else if addr <= 0x3FFF {
            addr = addr & 0x001F;
            match addr {
                0x0010 => {
                    addr = 0x0000;
                }
                0x0014 => {
                    addr = 0x0004;
                }
                0x0018 => {
                    addr = 0x0008;
                }
                0x001C => {
                    addr = 0x000C;
                }
                _ => {}
            }
            self.tbl_palette[addr as usize] = data;
        }
    }

    pub fn connect_cartridge(&mut self, cart: Arc<Mutex<Cartridge>>) {
        self.cart = Some(Arc::clone(&cart));
    }

    // pub fn get_decal(&self) -> &Decal {
    //     &Decal::create(Some(*self.spr_screen.deref()))
    // }

    pub fn get_pal_position(&self, palette: u8, pixel: u8) -> usize {
        let i = self.ppu_read(0x3F00 + ((palette as u16) << 2) + (pixel as u16), false);
        return (i & 0x3F) as usize;
    }

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

        if self.mask.bits.render_sprites() && self.cycle >= 1 && self.cycle < 258 {
            for i in 0..self.sprite_count {
                if self.sprite_scanline[i as usize].map.x > 0 {
                    self.sprite_scanline[i as usize].map.x =
                        self.sprite_scanline[i as usize].map.x - 1;
                } else {
                    self.sprite_shifter_pattern_lo[i as usize] =
                        self.sprite_shifter_pattern_lo[i as usize] << 1;
                    self.sprite_shifter_pattern_hi[i as usize] =
                        self.sprite_shifter_pattern_hi[i as usize] << 1;
                }
            }
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
                    self.status.bits.set_sprite_overflow(false);
                    self.status.bits.set_sprite_zero_hit(false);
                    for i in 0..8 {
                        self.sprite_shifter_pattern_hi[i as usize] = 0;
                        self.sprite_shifter_pattern_lo[i as usize] = 0;
                    }
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

                //Foreground Rendering
                if self.cycle == 257 && self.scan_line >= 0 {
                    self.sprite_scanline = [ObjectAttributeMemory::default(); 8];
                    self.sprite_count = 0;

                    for i in 0..8 {
                        self.sprite_shifter_pattern_lo[i] = 0;
                        self.sprite_shifter_pattern_hi[i] = 0;
                    }

                    let mut entry = 0;

                    self.zero_hit_possible = false;

                    while entry < 64 && self.sprite_count < 9 {
                        let diff =
                            (self.scan_line as i32) - (self.oam[entry as usize].map.y as i32);

                        if diff >= 0
                            && ((self.control.bits.sprite_size() && diff < 16)
                                || (!self.control.bits.sprite_size() && diff < 8))
                        {
                            if self.sprite_count < 8 {
                                if entry == 0 {
                                    self.zero_hit_possible = true;
                                }
                                self.sprite_scanline[self.sprite_count as usize] =
                                    self.oam[entry as usize].clone();
                                self.sprite_count = self.sprite_count + 1;
                            }
                        }
                        entry = entry + 1;
                    }
                    self.status.bits.set_sprite_overflow(self.sprite_count > 8);
                }

                if self.cycle == 340 {
                    for i in 0..self.sprite_count {
                        let mut sprite_bits_lo: u8;
                        let mut sprite_bits_hi: u8;
                        let sprite_addr_lo: u16;
                        let sprite_addr_hi: u16;

                        let current_sprite = self.sprite_scanline[i as usize];

                        if !self.control.bits.sprite_size() {
                            if current_sprite.map.attribute & 0x80 == 0 {
                                sprite_addr_lo = ((self.control.bits.pattern_sprite() as u16)
                                    << 12)
                                    | ((current_sprite.map.id as u16) << 4)
                                    | (self.scan_line as u16)
                                        .wrapping_sub(current_sprite.map.y as u16);
                            } else {
                                sprite_addr_lo = ((self.control.bits.pattern_sprite() as u16)
                                    << 12)
                                    | ((current_sprite.map.id as u16) << 4)
                                    | ((7 as u16).wrapping_sub(
                                        (self.scan_line as u16)
                                            .wrapping_sub(current_sprite.map.y as u16),
                                    ));
                            }
                        } else {
                            if current_sprite.map.attribute & 0x80 == 0 {
                                if self.scan_line.wrapping_sub(current_sprite.map.y as i16) < 8 {
                                    sprite_addr_lo = (((current_sprite.map.id & 0x01) as u16)
                                        << 12)
                                        | (((current_sprite.map.id & 0xFE) as u16) << 4)
                                        | ((self.scan_line as u16)
                                            .wrapping_sub(current_sprite.map.y as u16)
                                            & 0x07);
                                } else {
                                    sprite_addr_lo = (((current_sprite.map.id & 0x01) as u16)
                                        << 12)
                                        | ((((current_sprite.map.id & 0xFE) + 1) as u16) << 4)
                                        | ((self.scan_line as u16).wrapping_sub(
                                            self.sprite_scanline[i as usize].map.y as u16,
                                        ) & 0x07);
                                }
                            } else {
                                if self.scan_line.wrapping_sub(current_sprite.map.y as i16) < 8 {
                                    sprite_addr_lo = (((current_sprite.map.id & 0x01) as u16)
                                        << 12)
                                        | ((((current_sprite.map.id & 0xFE) + 1) as u16) << 4)
                                        | (7 - (((self.scan_line as u16)
                                            - (current_sprite.map.y as u16))
                                            & 0x07));
                                } else {
                                    sprite_addr_lo = (((current_sprite.map.id & 0x01) as u16)
                                        << 12)
                                        | (((current_sprite.map.id & 0xFE) as u16) << 4)
                                        | (7 - (((self.scan_line as u16)
                                            - (current_sprite.map.y as u16))
                                            & 0x07));
                                }
                            }
                        }

                        sprite_addr_hi = sprite_addr_lo.wrapping_add(8);
                        sprite_bits_lo = self.ppu_read(sprite_addr_lo, false);
                        sprite_bits_hi = self.ppu_read(sprite_addr_hi, false);

                        if current_sprite.map.attribute & 0x40 > 0 {
                            let flipbyte = |mut b: u8| -> u8 {
                                b = ((b & 0xF0) >> 4) | ((b & 0x0F) << 4);
                                b = ((b & 0xCC) >> 2) | ((b & 0x33) << 2);
                                b = ((b & 0xAA) >> 1) | ((b & 0x55) << 1);
                                return b;
                            };
                            sprite_bits_lo = flipbyte(sprite_bits_lo);
                            sprite_bits_hi = flipbyte(sprite_bits_hi);
                        }

                        self.sprite_shifter_pattern_lo[i as usize] = sprite_bits_lo;
                        self.sprite_shifter_pattern_hi[i as usize] = sprite_bits_hi;
                    }
                }
            }

            if self.scan_line == 240 {}

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

            let mut fg_pixel = 0x00;
            let mut fg_palette = 0x00;
            let mut priority = false;

            if self.mask.bits.render_sprites() {
                self.zero_sprite_rendered = false;
                for i in 0 as usize..self.sprite_count as usize {
                    if self.sprite_scanline[i].map.x == 0 {
                        let fg_pixel_lo = (self.sprite_shifter_pattern_lo[i] & 0x80) > 0;
                        let fg_pixel_hi = (self.sprite_shifter_pattern_hi[i] & 0x80) > 0;

                        fg_pixel = ((fg_pixel_hi as u8) << 1) | (fg_pixel_lo as u8);
                        fg_palette = (self.sprite_scanline[i].map.attribute & 0x03) + 0x04;
                        priority = (self.sprite_scanline[i].map.attribute & 0x20) == 0;

                        if fg_pixel != 0 {
                            if i == 0 {
                                self.zero_sprite_rendered = true;
                            }
                            break;
                        }
                    }
                }
            }

            let pixel;
            let palette;

            if fg_pixel == 0 && bg_pixel == 0 {
                pixel = 0;
                palette = 0;
            } else if fg_pixel > 0 && bg_pixel == 0 {
                pixel = fg_pixel;
                palette = fg_palette;
            } else if fg_pixel == 0 && bg_pixel > 0 {
                pixel = bg_pixel;
                palette = bg_palette;
            } else {
                if priority {
                    pixel = fg_pixel;
                    palette = fg_palette;
                } else {
                    pixel = bg_pixel;
                    palette = bg_palette;
                }

                if self.zero_hit_possible
                    && self.zero_sprite_rendered
                    && self.mask.bits.render_background()
                    && self.mask.bits.render_sprites()
                {
                    if (!((self.mask.bits.render_background_left() as u8)
                        | (self.mask.bits.render_sprites_left() as u8)))
                        > 0
                    {
                        if self.cycle >= 9 && self.cycle < 258 {
                            self.status.bits.set_sprite_zero_hit(true);
                        }
                    } else {
                        if self.cycle >= 1 && self.cycle < 258 {
                            self.status.bits.set_sprite_zero_hit(true);
                        }
                    }
                }
            }

            let sprx = self.cycle - 1;
            let spry = self.scan_line;
            if sprx >= 0 && (sprx as i32) < self.spr_screen.width as i32 && spry >= 0 && (spry as i32) < self.spr_screen.height as i32 {
                let cpa = self.get_pal_position(palette, pixel);
                let sprc = self.pal_screen[cpa];
                self.pal_positions
                    [(spry as i32 * self.spr_screen.width as i32 + sprx as i32) as usize] =
                    cpa as u8;
                self.spr_screen.set_pixel(sprx.into(), spry.into(), &sprc);
            }
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
        println!("PPU Reset Start");
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
        println!("PPU Reset End");
    }
}
