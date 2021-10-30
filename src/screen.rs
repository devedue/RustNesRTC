use crate::audio::Audio;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::nes::NES_PTR;
use minifb::Key;
use pge::*;
use std::sync::Mutex;

extern crate redis;

use std::sync::Arc;

pub struct Screen {
    client: bool,
}

impl Screen {
    pub fn new(client: bool) -> Self {
        Screen { client }
    }
}

impl State for Screen {
    fn on_user_create(&mut self) -> bool {
        if self.client {
            return true;
        }
        let mut nes = NES_PTR.lock().unwrap();
        nes.cpu = Cpu::new();
        let cart = Arc::new(Mutex::new(Cartridge::new("nestest.nes")));
        nes.cart = Some(cart.clone());
        nes.cpu.bus.insert_cartridge(cart.clone());
        nes.cpu.reset();

        // nes.cpu.disassemble(0x0000, 0xFFFF);
        nes.cpu.bus.set_sample_frequency(44100);
        std::thread::spawn(move || {
            let mut audio = Audio::new();
            audio.initialise_audio(44100, 1, 8, 512);
            // audio.set_user_synth_function(sound_out);
            audio.audio_thread();
        });
        println!("On User Created");
        return true;
    }

    fn on_user_destroy(&mut self) {}

    fn on_user_update(&mut self, engine: &mut PGE, elapsed: f32) -> bool {
        let mut nes = NES_PTR.lock().unwrap();

        engine.clear(&DARK_BLUE);
        if self.client {
            // let new_sprite: pge::Sprite = match bincode::deserialize(&nes.active_image) {
            //     Ok(res) => res,
            //     Err(_) => {
            //         return true;
            //     }
            // };
            engine.draw_sprite(0, 0, &nes.cpu.bus.get_ppu().spr_screen, 2);
            return true;
        }
        // print!("Update Start > ");
        nes.accumulated_time = nes.accumulated_time + elapsed;

        if nes.accumulated_time >= 1.0 / 60.0 {
            nes.accumulated_time = nes.accumulated_time - (1.0 / 60.0);
        }
        // if nes.residual_time > 0.0 {
        //     nes.residual_time = nes.residual_time - elapsed;
        // } else {
        //     nes.residual_time = nes.residual_time + (1.0 / 60.0) - elapsed;
        //     let start_time = Utc::now().time();
        //     nes.clock();
        //     let mut k = 0;
        //     while !nes.cpu.bus.get_ppu().frame_complete {
        //         nes.clock();
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

        //     nes.cpu.bus.get_ppu().frame_complete = false;
        // }

        nes.cpu.bus.controller[0] = 0x00;
        nes.set_controller(engine.get_key(Key::X).held, 0x80);
        nes.set_controller(engine.get_key(Key::Z).held, 0x40);
        nes.set_controller(engine.get_key(Key::A).held, 0x20);
        nes.set_controller(engine.get_key(Key::S).held, 0x10);
        nes.set_controller(engine.get_key(Key::Up).held, 0x08);
        nes.set_controller(engine.get_key(Key::Down).held, 0x04);
        nes.set_controller(engine.get_key(Key::Left).held, 0x02);
        nes.set_controller(engine.get_key(Key::Right).held, 0x01);
        if engine.get_key(Key::Space).pressed {
            nes.emulation_run = !nes.emulation_run;
        }
        if engine.get_key(Key::R).pressed {
            nes.cpu.reset();
        }
        if engine.get_key(Key::P).pressed {
            nes.selected_palette = (nes.selected_palette + 1) & 0x07;
        }
        if engine.get_key(Key::M).pressed {
            nes.draw_mode = !nes.draw_mode;
        }

        engine.draw_sprite(0, 0, &nes.cpu.bus.get_ppu().spr_screen, 2);
        let vecsprite = bincode::serialize(&nes.cpu.bus.get_ppu().spr_screen).unwrap();
        nes.active_image = vecsprite;

        // let new_sprite: pge::Sprite = bincode::deserialize(&nes.active_image).unwrap();

        return true;
    }
}
