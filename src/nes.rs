use crate::bus::*;
use crate::cartridge::Cartridge;
use crate::cpu::*;
use minifb::Key;
use pge::{Pixel, PixelMode, State, PGE};
use pge::{DARK_BLUE, GREEN, RED, WHITE};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

pub struct Nes {
    pub cpu: Cpu,
    pub cart: Rc<RefCell<Cartridge>>,
    emulation_run: bool,
    residual_time: f32,
}

impl State for Nes {
    fn on_user_create(&mut self) -> bool {
        self.cpu = Cpu::new();
        self.cart = Rc::new(RefCell::new(Cartridge::new("nestest.nes")));
        self.cpu.bus.insert_cartridge(self.cart.clone());
        self.cpu.reset();

        return true;
    }
    // fn on_user_destroy(&mut self) -> bool {
    //     return true;
    // }

    fn on_user_update(&mut self, engine: &mut PGE, elapsed: f32) -> bool {
        engine.clear(&DARK_BLUE);

        if self.emulation_run {
            if self.residual_time > 0.0 {
                self.residual_time = self.residual_time - elapsed;
            } else {
                self.residual_time = self.residual_time + (1.0 / 60.0) - elapsed;
                self.clock();
                while (!self.cpu.bus.ppu.frame_complete) {
                    self.clock();
                }
                self.cpu.bus.ppu.frame_complete = false;
            }
        } else {
            if (engine.get_key(Key::C).pressed) {
                self.clock();
                while (!self.cpu.is_complete()) {
                    self.clock();
                }
                self.clock();
                while (self.cpu.is_complete()) {
                    self.clock();
                }
            }

            // Emulate one whole frame
            if (engine.get_key(Key::F).pressed) {
                // Clock enough times to draw a single frame
                self.clock();
                while (!self.cpu.bus.ppu.frame_complete) {
                    self.clock();
                }
                self.clock();
                while (!self.cpu.is_complete()) {
                    self.clock();
                }

                self.cpu.bus.ppu.frame_complete = false;
            }
        }

        if (engine.get_key(Key::Space).pressed) {
            self.emulation_run = !self.emulation_run;
        }
        if (engine.get_key(Key::R).pressed) {
            self.cpu.reset();
        }

        self.draw_cpu(engine, 516, 2);

        self.draw_ram(engine, 516, 72, 0, 10, 10);

        engine.draw_sprite(0, 0, &self.cpu.bus.ppu.spr_screen, 2);
        return true;
    }
}

impl Nes {
    pub fn new() -> Self {
        return Nes {
            cpu: Cpu::new(),
            cart: Rc::new(RefCell::new(Cartridge::default())),
            emulation_run: false,
            residual_time: 0.0,
        };
    }

    fn clock(&mut self) {
        let ppu_count = self.cpu.bus.ppu.clock();
        if (ppu_count % 3 == 0) {
            self.cpu.clock();
        }
    }

    fn hex(&self, n: u16) -> String {
        return (format!("{:X}", n));
    }

    fn draw_ram(&self, engine: &mut PGE, x: i32, y: i32, address: u16, rows: i32, cols: i32) {
        let mut ramx = x;
        let mut ramy = y;
        let mut address = address;
        for row in 0..rows {
            let mut drawstring: String = format!("${}:", self.hex(address));
            for col in 0..cols {
                drawstring = format!(
                    "{} {}",
                    drawstring,
                    self.hex(self.cpu.read(address, false).into())
                );
                address = address + (1 as u16);
            }
            engine.draw_string(ramx, ramy, String::as_str(&drawstring), &WHITE, 1);
            ramy = ramy + 10;
        }
    }

    fn get_flag_color(&self, flag: FLAGS6502) -> &Pixel {
        if (self.cpu.get_flag(flag) > 0) {
            return &GREEN;
        } else {
            return &RED;
        }
    }

    fn draw_cpu(&self, engine: &mut PGE, x: i32, y: i32) {
        // let random = rand::random::<u16>();
        let st = "STATUS: ";
        engine.draw_string(x, y, &st.to_owned(), &WHITE, 1);
        engine.draw_string(x + 64, y, "N", self.get_flag_color(FLAGS6502::N), 1);
        engine.draw_string(x + 80, y, "V", self.get_flag_color(FLAGS6502::V), 1);
        engine.draw_string(x + 96, y, "-", self.get_flag_color(FLAGS6502::U), 1);
        engine.draw_string(x + 112, y, "B", self.get_flag_color(FLAGS6502::B), 1);
        engine.draw_string(x + 128, y, "D", self.get_flag_color(FLAGS6502::D), 1);
        engine.draw_string(x + 144, y, "I", self.get_flag_color(FLAGS6502::I), 1);
        engine.draw_string(x + 160, y, "Z", self.get_flag_color(FLAGS6502::Z), 1);
        engine.draw_string(x + 178, y, "C", self.get_flag_color(FLAGS6502::C), 1);
        engine.draw_string(
            x,
            y + 10,
            String::as_str(&format!("{},{}", "PC: $", self.hex(self.cpu.pc))),
            &WHITE,
            1,
        );
        engine.draw_string(
            x,
            y + 20,
            String::as_str(&format!("A: ${} [{}]", self.hex(self.cpu.a.into()), self.cpu.a)),
            &WHITE,
            1,
        );
        engine.draw_string(
            x,
            y + 30,
            String::as_str(&format!("X: ${} [{}]", self.hex(self.cpu.x.into()), self.cpu.x)),
            &WHITE,
            1,
        );
        engine.draw_string(
            x,
            y + 40,
            String::as_str(&format!("Y: ${} [{}]", self.hex(self.cpu.y.into()), self.cpu.y)),
            &WHITE,
            1,
        );
        engine.draw_string(
            x,
            y + 50,
            String::as_str(&format!("Stack P: $[{}]", self.hex(self.cpu.stkp.into()))),
            &WHITE,
            1,
        );
    }
}
