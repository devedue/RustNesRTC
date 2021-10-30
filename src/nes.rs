use crate::cpu::Cpu;
use crate::cartridge::Cartridge;
use std::sync::Mutex;

extern crate redis;
use std::sync::Arc;
// use tokio::sync::Mutex;
// For Logging:
// use std::io::Write;
// use crate::util::hex;

pub const SPRITE_ARR_SIZE: usize = 256 * 240;

lazy_static! {
    pub static ref NES_PTR: Arc<Mutex<Nes>> = Arc::new(Mutex::new(Nes::new()));
}

pub struct Nes {
    pub cpu: Cpu,
    pub selected_palette: u8,
    pub cart: Option<Arc<Mutex<Cartridge>>>,
    pub emulation_run: bool,
    pub draw_mode: bool,
    pub cycles: u128,
    player1: bool,
    pub accumulated_time: f32,
    // pub residual_time: f32,
    pub active_image: Vec<u8>,
    pub pal_positions: Vec<u8>
}

//Static sound functions
pub fn sound_out(channel: u32, _global_time: f32, _time_step: f32) -> f32 {
    if channel == 0 {
        let mut nes = NES_PTR.lock().unwrap();
        while !nes.clock() {}
        return nes.cpu.bus.audio_sample as f32;
    } else {
        return 0.0;
    }
}

impl Nes {
    pub fn new() -> Self {
        // let redis = redis::Client::open("redis://127.0.0.1").unwrap();
        return Nes {
            cpu: Cpu::new(),
            cart: None,
            emulation_run: true,
            selected_palette: 0,
            draw_mode: false,
            cycles: 0,
            accumulated_time: 0.0,
            // residual_time: 0.0,
            // redis_con: redis.get_connection().unwrap(),
            player1: true,
            active_image: vec!(),
            pal_positions: vec!()
        };
    }

    fn clock(&mut self) -> bool {
        // print!("Clock Start > ");
        self.cpu.bus.get_ppu().clock();
        self.cpu.bus.apu.clock();
        if self.cycles % 3 == 0 {
            if self.cpu.bus.dma_transfer {
                if self.cpu.bus.dma_dummy {
                    if self.cycles % 2 == 1 {
                        self.cpu.bus.dma_dummy = false;
                    }
                } else {
                    if self.cycles % 2 == 0 {
                        let page = self.cpu.bus.dma_page;
                        let addr = self.cpu.bus.dma_addr;
                        let data = self.cpu.read(((page as u16) << 8) | (addr as u16), false);
                        self.cpu.bus.dma_data = data;
                    } else {
                        let addr = self.cpu.bus.dma_addr;
                        let data = self.cpu.bus.dma_data;
                        self.cpu.bus.get_ppu().set_oam(addr as usize, data);
                        self.cpu.bus.dma_addr = addr.wrapping_add(1);

                        if self.cpu.bus.dma_addr == 0x00 {
                            self.cpu.bus.dma_transfer = false;
                            self.cpu.bus.dma_dummy = true;
                        }
                    }
                }
            } else {
                self.cpu.clock();
            }
        }

        let mut sample_ready = false;
        self.cpu.bus.audio_time += self.cpu.bus.audio_time_per_clock;
        if self.cpu.bus.audio_time >= self.cpu.bus.audio_time_per_sample {
            self.cpu.bus.audio_time -= self.cpu.bus.audio_time_per_sample;
            self.cpu.bus.audio_sample = self.cpu.bus.apu.get_output_sample();
            sample_ready = true;
        }

        if self.cpu.bus.get_ppu().nmi {
            self.cpu.bus.get_ppu().nmi = false;
            self.cpu.nmi();
        }
        self.cycles += 1;
        // println!("Sample Ready {}", sample_ready);
        return sample_ready;
    }

    pub fn set_controller(&mut self, set: bool, key: u8) {
        if set {
            if self.player1 {
                self.cpu.bus.controller[0] |= key;
            } else {
                self.cpu.bus.controller[1] |= key;
            }
        }
    }

    pub fn get_pal_positions(&mut self) -> Vec<u8>{
        return self.cpu.bus.get_ppu().pal_positions.to_vec();
    }

    pub fn construct_pal(&mut self) {
        for (pos,e) in self.pal_positions.iter().enumerate() {
            if pos > 61440 {
                break;
            }
            self.cpu.bus.get_ppu().spr_screen.data[pos] = self.cpu.bus.get_ppu().pal_screen[*e as usize];
        }
        // self.spr_screen.set_pixel(sprx.into(), spry.into(), &sprc);
    }
}
