#![allow(dead_code)]
#![windows_subsystem = "windows"]
#![feature(untagged_unions)]
#![feature(libc)]

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
use nes::*;
use pge::PGE;
mod rtc;
mod gui;

#[macro_use]
extern crate lazy_static;

fn start_nes() {
    let mut nes = Nes::new();
    let mut pge = PGE::construct("NES Emulator", 512, 480, 2, 2);
    pge.start(&mut nes);
}

fn main() {
    println!("Start");
    gui::MainMenu::start_program();
    println!("End");
}