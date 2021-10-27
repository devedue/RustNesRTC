// #![allow(dead_code)]
// #![windows_subsystem = "windows"]
#![feature(untagged_unions)]
#![feature(libc)]

use crate::screen::Screen;
use crate::nes::Nes;
use std::sync::Arc;
use std::sync::Mutex;
use crate::nes::NES_PTR;
use pge::PGE;

mod apu;
mod bus;
mod cartridge;
mod cpu;
mod mapper;
mod mapper_000;
mod nes;
mod ppu;
mod socket;
mod util;
mod rtc;
mod gui;
mod rtc_event;
mod audio;
mod screen;

#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() {
    let mut screen = Screen::new();
    let mut pge = PGE::construct("NES Emulator", 512, 480, 2, 2);
    pge.start(&mut screen);
}