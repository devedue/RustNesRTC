use chrono::Utc;
use pge::audio::Audio;
use pge::*;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use minifb::Key;

pub struct SoundDemo {
    frequency: f32,
    duty_cycle: f32,
    harmonics: i32,
    // thread: Option<JoinHandle<()>>,
}

static mut soundptr: *mut SoundDemo = 0 as *mut SoundDemo;

impl SoundDemo {
    pub fn new() -> Self {
        let mut result = SoundDemo {
            frequency: 440.0,
            duty_cycle: 0.5,
            harmonics: 20,
            // thread: None,
        };

        return result;
    }

    pub fn sound_out(channels: u32, global_time: f32, time_step: f32) -> f32 {
        unsafe {
            return 0.1 * SoundDemo::sample_square_wave((*soundptr).frequency, global_time);
        }
    }

    fn sample_square_wave(f: f32, t: f32) -> f32 {
        let mut a = 0.0;
        let mut b = 0.0;
        let p = 0.5 * 2.0 * 3.14159;

        for n in 1..20 {
            let c = (n as f32) * f * 2.0 * 3.14159 * t;
            a = a + (c.sin() / (n as f32));
            b = b + ((c - p * (n as f32)).sin() / (n as f32));
        }
        return (2.0 / 3.14159) * (a - b);
    }
}

impl State for SoundDemo {
    fn on_user_create(&mut self) -> bool {
        unsafe {
            soundptr = self;
            println!("{}", (*self).frequency);
        }
        let handle = std::thread::spawn(move || {
            let mut audio = Audio::new();
            audio.initialise_audio(44100, 1, 8, 512);
            audio.set_user_synth_function(Self::sound_out);
            audio.audio_thread();
        });
        return true;
    }
    fn on_user_destroy(&mut self) {}

    fn on_user_update(&mut self, engine: &mut PGE, elapsed: f32) -> bool {
        if engine.get_key(Key::Up).held {
            self.frequency += 1.0;
        }
        if engine.get_key(Key::Down).held {
            self.frequency -= 1.0;
        }
        return true;
    }
}
