use crate::cartridge::Cartridge;
use crate::cpu::*;
use crate::socket::RemoteServer;
use crate::util::*;
// use chrono::Utc;
use minifb::Key;
use pge::audio::Audio;
use pge::*;
use std::cell::RefCell;
use std::io::{Read, Write};
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

// For Logging:
// use std::io::Write;
// use crate::util::hex;

pub struct Nes {
    pub cpu: Cpu,
    pub cart: Rc<RefCell<Cartridge>>,
    pub selected_palette: u8,
    emulation_run: bool,
    draw_mode: bool,
    // residual_time: f32,
    cycles: u128,

    player1: bool,
    last_p1_send: u8,
    last_p2_send: u8,

    accumulated_time: f32,
}

static mut NES_PTR: *mut Nes = 0 as *mut Nes;
// static mut SOCKET_PTR: Option<Arc<Mutex<RemoteServer>>> = None;

impl State for Nes {
    fn on_user_create(&mut self) -> bool {
        self.cpu = Cpu::new();
        self.cart = Rc::new(RefCell::new(Cartridge::new("nestest.nes")));
        self.cpu.bus.insert_cartridge(self.cart.clone());
        self.cpu.reset();
        // let socket = RemoteServer::new();
        // unsafe {
        //     SOCKET_PTR = Some(Arc::new(Mutex::new(socket)));
        //     std::thread::spawn(|| match &SOCKET_PTR {
        //         Some(s) => {
        //             let mut guard = s.lock().unwrap();
        //             let server = guard.deref_mut();
        //             (*NES_PTR).set_player1(RemoteServer::connect_or_start(server));
        //             println!("Stop listening");
        //         }
        //         None => {}
        //     });
        // }

        // std::thread::spawn(move || {
        //     unsafe {
        //         loop {
        //             match &SOCKET_PTR {
        //                 Some(s) => {
        //                     // println!("Getting lock for read");
        //                     let server = s.lock().unwrap();
        //                     // println!("Got lock for read");
        //                     let mut data: [u8; 2] = [0; 2];
        //                     match server.stream.as_ref() {
        //                         Some(mut stream) => {
        //                             match stream.read_exact(&mut data) {
        //                                 Ok(_) => {
        //                                     (*NES_PTR).set_controller_state(data[0], data[1]);
        //                                     // self.cpu.bus.controller[0] = data[0];
        //                                     // println!("read success");
        //                                 }
        //                                 Err(_) => {
        //                                     // println!("read error{}", e);
        //                                 }
        //                             }
        //                         }
        //                         None => {}
        //                     }
        //                     // println!("Read Complete");
        //                 }
        //                 None => {}
        //             }
        //             std::thread::sleep(std::time::Duration::from_millis(33));
        //         }
        //     }
        // });

        // std::thread::spawn(move || unsafe {
        //     let mut lastp1 = 0;
        //     let mut lastp2 = 0;
        //     loop {
        //         match &SOCKET_PTR {
        //             Some(s) => {
        //                 // println!("Getting lock for write");
        //                 let server = s.lock().unwrap();
        //                 // println!("Got lock for write");
        //                 let data = (*NES_PTR).get_controllers();
        //                 if lastp1 == data[0] && lastp2 == data[1] {
        //                     continue;
        //                 } else {
        //                     match server.stream.as_ref() {
        //                         Some(mut stream) => match stream.write(&data) {
        //                             Ok(_) => {
        //                                 lastp1 = data[0];
        //                                 lastp2 = data[1];
        //                                 // println!("write success");
        //                             }
        //                             Err(e) => {
        //                                 // println!("write error{}", e);
        //                             }
        //                         },
        //                         None => {}
        //                     }
        //                 }
        //                 // println!("Write Complete");
        //             }
        //             None => {}
        //         }
        //         std::thread::sleep(std::time::Duration::from_millis(33));
        //     }
        // });

        unsafe {
            NES_PTR = self;
        }

        // self.cpu.disassemble(0x0000, 0xFFFF);
        self.cpu.bus.set_sample_frequency(44100);
        std::thread::spawn(move || {
            let mut audio = Audio::new();
            audio.initialise_audio(44100, 1, 8, 512);
            audio.set_user_synth_function(Self::sound_out);
            audio.audio_thread();
        });
        return true;
    }

    fn on_user_destroy(&mut self) {}

    fn on_user_update(&mut self, engine: &mut PGE, elapsed: f32) -> bool {
        // engine.clear(&DARK_BLUE);
        self.accumulated_time = self.accumulated_time + elapsed;

        if self.accumulated_time >= 1.0 / 60.0 {
            self.accumulated_time = self.accumulated_time - (1.0 / 60.0);
        }

        self.cpu.bus.controller[0] = 0x00;
        self.set_controller(engine.get_key(Key::X).held, 0x80);
        self.set_controller(engine.get_key(Key::Z).held, 0x40);
        self.set_controller(engine.get_key(Key::A).held, 0x20);
        self.set_controller(engine.get_key(Key::S).held, 0x10);
        self.set_controller(engine.get_key(Key::Up).held, 0x08);
        self.set_controller(engine.get_key(Key::Down).held, 0x04);
        self.set_controller(engine.get_key(Key::Left).held, 0x02);
        self.set_controller(engine.get_key(Key::Right).held, 0x01);
        if engine.get_key(Key::Space).pressed {
            self.emulation_run = !self.emulation_run;
        }
        if engine.get_key(Key::R).pressed {
            self.cpu.reset();
        }
        if engine.get_key(Key::P).pressed {
            self.selected_palette = (self.selected_palette + 1) & 0x07;
        }
        if engine.get_key(Key::M).pressed {
            self.draw_mode = !self.draw_mode;
        }

        // if self.emulation_run {
        // if self.residual_time > 0.0 {
        //     self.residual_time = self.residual_time - elapsed;
        // } else {
        //     self.residual_time = self.residual_time + (1.0 / 60.0) - elapsed;
        //     let start_time = Utc::now().time();
        //     self.clock();
        //     let mut k = 0;
        //     while !self.cpu.bus.get_ppu().frame_complete {
        //         self.clock();
        //         k = k + 1;
        //     }
        //     let end_time = Utc::now().time();
        //     let diff = end_time - start_time;
        //     println!(
        //         "{} cycles took {} , with {} per frame",
        //         k,
        //         diff,
        //         (diff.num_milliseconds() as f32) / (k as f32)
        //     );

        //     self.cpu.bus.get_ppu().frame_complete = false;
        // }
        // } else {
        //     if engine.get_key(Key::C).pressed {
        //         println!("C");
        //         self.clock();
        //         while !self.cpu.is_complete() {
        //             self.clock();
        //         }
        //         self.clock();
        //         while self.cpu.is_complete() {
        //             self.clock();
        //         }
        //     }

        //     if engine.get_key(Key::F).pressed {
        //         println!("F");
        //         self.clock();
        //         while !self.cpu.bus.get_ppu().frame_complete {
        //             self.clock();
        //         }
        //         self.clock();
        //         while !self.cpu.is_complete() {
        //             self.clock();
        //         }

        //         self.cpu.bus.get_ppu().frame_complete = false;
        //     }
        // }

        // self.draw_cpu(engine, 516, 2);

        // self.draw_ram(engine, 516, 72, 0, 10, 10);

        // let swatch_size = 6;
        // for p in 0..8 {
        //     for s in 0..4 {
        //         engine.fill_rect(
        //             516 + p * (swatch_size * 5) + s * swatch_size,
        //             340,
        //             swatch_size,
        //             swatch_size,
        //             &self.cpu.bus.get_ppu().get_palette_color(p as u8, s as u8),
        //         );
        //     }
        // }
        // engine.draw_rect(
        //     (516 + (self.selected_palette as i32) * (swatch_size * 5) - 1).into(),
        //     339,
        //     swatch_size * 4,
        //     swatch_size,
        //     &WHITE,
        // );

        // self.draw_code(engine, 516, 72, 26);

        // for i in 0..48 {
        //     let s = hex2(i)
        //         + ": ("
        //         + &self
        //             .cpu
        //             .bus
        //             .get_ppu()
        //             .get_oam((i * 4 + 3) as usize)
        //             .to_string()
        //         + ", "
        //         + &self
        //             .cpu
        //             .bus
        //             .get_ppu()
        //             .get_oam((i * 4 + 0) as usize)
        //             .to_string()
        //         + ") "
        //         + "ID: "
        //         + &hex2(self.cpu.bus.get_ppu().get_oam((i * 4 + 1) as usize)).to_string()
        //         + " AT: "
        //         + &hex2(self.cpu.bus.get_ppu().get_oam((i * 4 + 2) as usize)).to_string();
        //     engine.draw_string(516, 4 + (i as i32) * 10, &String::from(s), &WHITE, 1);
        // }

        engine.draw_sprite(0, 0, &self.cpu.bus.get_ppu().spr_screen, 2);

        // let sp1 = &self
        //     .cpu
        //     .bus
        //     .get_ppu()
        //     .get_pattern_table(0, self.selected_palette);
        // let sp2 = &self
        //     .cpu
        //     .bus
        //     .get_ppu()
        //     .get_pattern_table(1, self.selected_palette);
        // engine.draw_sprite(516, 348, sp1, 1);
        // engine.draw_sprite(648, 348, sp2, 1);

        // if self.draw_mode {
        //     for y in 0..30 {
        //         for x in 0..32 {
        // let id = self.cpu.bus.get_ppu().tbl_name[0][(y * 32 + x) as usize];
        // engine.draw_parital_sprite(
        //     x * 16,
        //     y * 16,
        //     sp2,
        //     ((id & 0x0F) << 3) as i32,
        //     (((id >> 4) & 0x0F) << 3) as i32,
        //     8,
        //     8,
        //     2,
        // );
        // engine.draw_rect(x * 16, y * 16, 16, 16, &WHITE);
        // engine.draw_string(
        //     x * 16,
        //     y * 16,
        //     &hex2(id),
        //     &WHITE,
        //     1
        // );
        //     }
        // }
        // }
        return true;
    }
}

impl Nes {
    pub fn new() -> Self {
        return Nes {
            cpu: Cpu::new(),
            cart: Rc::new(RefCell::new(Cartridge::default())),
            emulation_run: false,
            // residual_time: 0.0,
            selected_palette: 0,
            draw_mode: false,
            cycles: 0,
            accumulated_time: 0.0,

            player1: true,

            last_p1_send: 0,
            last_p2_send: 0,
        };
    }

    //Static sound functions
    pub fn sound_out(channel: u32, _global_time: f32, _time_step: f32) -> f32 {
        unsafe {
            if channel == 0 {
                while !(*NES_PTR).clock() {}
                return ((*NES_PTR).cpu.bus.audio_sample) as f32;
            } else {
                return 0.0;
            }
        }
    }

    fn clock(&mut self) -> bool {
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

        return sample_ready;
    }

    fn _draw_ram(&mut self, engine: &mut PGE, x: i32, y: i32, address: u16, rows: i32, cols: i32) {
        let ramx = x;
        let mut ramy = y;
        let mut address = address;
        for _row in 0..rows {
            let mut drawstring: String = format!("${}:", hex(address));
            for _col in 0..cols {
                let cpuread = self.cpu.read(address, false).into();
                drawstring = format!("{} {}", drawstring, hex(cpuread));
                address = address + (1 as u16);
            }
            engine.draw_string(ramx, ramy, String::as_str(&drawstring), &WHITE, 1);
            ramy = ramy + 10;
        }
    }

    fn _get_flag_color(&self, flag: FLAGS6502) -> &Pixel {
        if self.cpu.get_flag(flag) > 0 {
            return &GREEN;
        } else {
            return &RED;
        }
    }

    fn _draw_code(&self, engine: &mut PGE, x: i32, y: i32, lines: i32) {
        // let ind = self.mapAsm.index(&self.cpu.pc);
        let mut line_y = (lines >> 1) * 10 + y;
        {
            let mut it = self.cpu.map_asm.iter();
            let mut value = it.find(|(k, _v)| k == &&self.cpu.pc);

            if value.is_some() {
                engine.draw_string(x, line_y, value.unwrap().1, &CYAN, 1);
                while line_y < (lines * 10) + y {
                    line_y = line_y + 10;
                    value = it.next();
                    if value.is_some() {
                        engine.draw_string(x, line_y, value.unwrap().1, &WHITE, 1);
                    }
                }
            }
        }

        {
            let mut it = self.cpu.map_asm.iter().rev();
            let mut value = it.find(|(k, _v)| k == &&self.cpu.pc);
            line_y = (lines >> 1) * 10 + y;
            if value.is_some() {
                while line_y > y {
                    line_y = line_y - 10;
                    value = it.next();
                    if value.is_some() {
                        engine.draw_string(x, line_y, value.unwrap().1, &WHITE, 1);
                    }
                }
            }
        }
    }

    fn _draw_cpu(&self, engine: &mut PGE, x: i32, y: i32) {
        // let random = rand::random::<u16>();
        let st = "STATUS: ";
        engine.draw_string(x, y, &st.to_owned(), &WHITE, 1);
        engine.draw_string(x + 64, y, "N", self._get_flag_color(FLAGS6502::N), 1);
        engine.draw_string(x + 80, y, "V", self._get_flag_color(FLAGS6502::V), 1);
        engine.draw_string(x + 96, y, "-", self._get_flag_color(FLAGS6502::U), 1);
        engine.draw_string(x + 112, y, "B", self._get_flag_color(FLAGS6502::B), 1);
        engine.draw_string(x + 128, y, "D", self._get_flag_color(FLAGS6502::D), 1);
        engine.draw_string(x + 144, y, "I", self._get_flag_color(FLAGS6502::I), 1);
        engine.draw_string(x + 160, y, "Z", self._get_flag_color(FLAGS6502::Z), 1);
        engine.draw_string(x + 178, y, "C", self._get_flag_color(FLAGS6502::C), 1);
        engine.draw_string(
            x,
            y + 10,
            String::as_str(&format!("{}{}", "PC: $", hex(self.cpu.pc))),
            &WHITE,
            1,
        );
        engine.draw_string(
            x,
            y + 20,
            String::as_str(&format!("A: ${} [{}]", hex(self.cpu.a.into()), self.cpu.a)),
            &WHITE,
            1,
        );
        engine.draw_string(
            x,
            y + 30,
            String::as_str(&format!("X: ${} [{}]", hex(self.cpu.x.into()), self.cpu.x)),
            &WHITE,
            1,
        );
        engine.draw_string(
            x,
            y + 40,
            String::as_str(&format!("Y: ${} [{}]", hex(self.cpu.y.into()), self.cpu.y)),
            &WHITE,
            1,
        );
        engine.draw_string(
            x,
            y + 50,
            String::as_str(&format!("Stack P: $[{}]", hex(self.cpu.stkp.into()))),
            &WHITE,
            1,
        );
    }

    fn set_controller(&mut self, set: bool, key: u8) {
        if set {
            if self.player1 {
                self.cpu.bus.controller[0] |= key;
            } else {
                self.cpu.bus.controller[1] |= key;
            }
        }
        // unsafe {
        //     match &SOCKET_PTR {
        //         Some(s) => {
        //             let server = s.lock().unwrap();
        //             let data = (*NES_PTR).get_controllers();
        //             if (self.player1 && self.last_p1_send == data[0])
        //                 || (!self.player1 && self.last_p2_send == data[1])
        //             {
        //                 return;
        //             } else {
        //                 match server.stream.as_ref() {
        //                     Some(mut stream) => match stream.write(&data) {
        //                         Ok(_) => {
        //                             self.last_p1_send = data[0];
        //                             self.last_p2_send = data[1];
        //                             // println!("write success");
        //                         }
        //                         Err(_) => {
        //                             // println!("write error{}", e);
        //                         }
        //                     },
        //                     None => {}
        //                 }
        //             }
        //             // println!("Write Complete");
        //         }
        //         None => {}
        //     }
        // }
    }

    pub fn set_controller_state(&mut self, p1: u8, p2: u8) {
        if self.player1 {
            // println!("p2 {} {}", p1, p2);
            self.cpu.bus.controller[1] = p2;
        } else {
            // println!("p1 {} {}", p1, p2);
            self.cpu.bus.controller[0] = p1;
        }
    }

    pub fn get_controllers(&self) -> [u8; 2] {
        [self.cpu.bus.controller[0], self.cpu.bus.controller[1]]
    }

    pub fn set_player1(&mut self, p1: bool) {
        self.player1 = p1;
    }
}
