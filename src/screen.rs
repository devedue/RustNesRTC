use crate::audio::Audio;
use crate::audio::AUDIO_THREAD_ACTIVE;
use crate::cpu::Cpu;
use crate::gui::Message;
use crate::nes::NES_PTR;
use crate::nes::SPRITE_ARR_SIZE;
use iced::canvas::{self, Cache, Canvas, Cursor, Frame, Geometry};
use iced::Element;
use iced::Length;
use iced::Rectangle;
use iced_native::{Color, Point, Size};
use std::sync::atomic::Ordering;
use tokio::task::JoinHandle;

extern crate redis;

#[derive(Default)]
pub struct ScreenState {
    cache: Cache,
    pub scale: f32,
}

pub struct Screen {
    state: ScreenState,
    client: bool,
    pal_screen: [Color; 64],
    audio_thread: Option<JoinHandle<()>>,
}

impl Default for Screen {
    fn default() -> Self {
        Screen::new(false)
    }
}

impl Screen {
    pub fn new(client: bool) -> Self {
        Screen {
            client,
            state: ScreenState {
                scale: 2.0,
                ..ScreenState::default()
            },
            pal_screen: [
                Color::from_rgb8(84, 84, 84),
                Color::from_rgb8(0, 30, 116),
                Color::from_rgb8(8, 16, 144),
                Color::from_rgb8(48, 0, 136),
                Color::from_rgb8(68, 0, 100),
                Color::from_rgb8(92, 0, 48),
                Color::from_rgb8(84, 4, 0),
                Color::from_rgb8(60, 24, 0),
                Color::from_rgb8(32, 42, 0),
                Color::from_rgb8(8, 58, 0),
                Color::from_rgb8(0, 64, 0),
                Color::from_rgb8(0, 60, 0),
                Color::from_rgb8(0, 50, 60),
                Color::from_rgb8(0, 0, 0),
                Color::from_rgb8(0, 0, 0),
                Color::from_rgb8(0, 0, 0),
                Color::from_rgb8(152, 150, 152),
                Color::from_rgb8(8, 76, 196),
                Color::from_rgb8(48, 50, 236),
                Color::from_rgb8(92, 30, 228),
                Color::from_rgb8(136, 20, 176),
                Color::from_rgb8(160, 20, 100),
                Color::from_rgb8(152, 34, 32),
                Color::from_rgb8(120, 60, 0),
                Color::from_rgb8(84, 90, 0),
                Color::from_rgb8(40, 114, 0),
                Color::from_rgb8(8, 124, 0),
                Color::from_rgb8(0, 118, 40),
                Color::from_rgb8(0, 102, 120),
                Color::from_rgb8(0, 0, 0),
                Color::from_rgb8(0, 0, 0),
                Color::from_rgb8(0, 0, 0),
                Color::from_rgb8(236, 238, 236),
                Color::from_rgb8(76, 154, 236),
                Color::from_rgb8(120, 124, 236),
                Color::from_rgb8(176, 98, 236),
                Color::from_rgb8(228, 84, 236),
                Color::from_rgb8(236, 88, 180),
                Color::from_rgb8(236, 106, 100),
                Color::from_rgb8(212, 136, 32),
                Color::from_rgb8(160, 170, 0),
                Color::from_rgb8(116, 196, 0),
                Color::from_rgb8(76, 208, 32),
                Color::from_rgb8(56, 204, 108),
                Color::from_rgb8(56, 180, 204),
                Color::from_rgb8(60, 60, 60),
                Color::from_rgb8(0, 0, 0),
                Color::from_rgb8(0, 0, 0),
                Color::from_rgb8(236, 238, 236),
                Color::from_rgb8(168, 204, 236),
                Color::from_rgb8(188, 188, 236),
                Color::from_rgb8(212, 178, 236),
                Color::from_rgb8(236, 174, 236),
                Color::from_rgb8(236, 174, 212),
                Color::from_rgb8(236, 180, 176),
                Color::from_rgb8(228, 196, 144),
                Color::from_rgb8(204, 210, 120),
                Color::from_rgb8(180, 222, 120),
                Color::from_rgb8(168, 226, 144),
                Color::from_rgb8(152, 226, 180),
                Color::from_rgb8(160, 214, 228),
                Color::from_rgb8(160, 162, 160),
                Color::from_rgb8(0, 0, 0),
                Color::from_rgb8(0, 0, 0),
            ],
            audio_thread: None,
        }
    }

    pub fn init_nes(&self) {
        if self.client {
            return ();
        }
        let mut nes = NES_PTR.lock().unwrap();
        nes.cpu = Cpu::new();
        let cart = nes.cart.as_ref().unwrap().clone();
        // nes.cart = Some(cart.clone());
        nes.cpu.bus.insert_cartridge(cart.clone());
        nes.cpu.reset();

        // nes.cpu.disassemble(0x0000, 0xFFFF);
        nes.cpu.bus.set_sample_frequency(44100);
    }

    pub fn run_nes(&mut self, client: bool) {
        self.audio_thread = Some(tokio::spawn(async move {
            let mut audio = Audio::new(client);
            audio.initialise_audio(44100, 1, 8, 512);
            // audio.set_user_synth_function(sound_out);
            println!("Started");
            audio.run_thread().await;
            println!("Stopped");
            audio.destroy_audio();
        }));
    }

    pub fn stop_nes(&mut self) {
        AUDIO_THREAD_ACTIVE.store(false, Ordering::Relaxed);
        // self.audio_thread.take().join();
    }

    pub fn view(&mut self) -> Element<Message> {
        let scale = self.state.scale;
        Canvas::new(self)
            .width(Length::Units((256.0 * scale) as u16))
            .height(Length::Units((240.0 * scale) as u16))
            .into()
    }
    pub fn request_redraw(&mut self) {
        self.state.cache.clear()
    }
}

impl canvas::Program<Message> for Screen {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let nes = NES_PTR.lock().unwrap();
        if nes.pal_positions.len() < SPRITE_ARR_SIZE {
            // println!("No {}", nes.pal_positions.len());
            return vec![];
        }
        let content = self.state.cache.draw(bounds.size(), |frame: &mut Frame| {
            for i in 0..256 {
                for j in 0..240 {
                    frame.fill_rectangle(
                        Point::new(
                            i as f32 * self.state.scale as f32,
                            j as f32 * self.state.scale as f32,
                        ),
                        Size::new(1.0 * self.state.scale, 1.0 * self.state.scale),
                        self.pal_screen[nes.pal_positions[(j * 256) + i] as usize],
                    );
                }
            }
        });
        vec![content]
    }
}
