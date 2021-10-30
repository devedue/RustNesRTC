// #![allow(dead_code)]
// #![windows_subsystem = "windows"]
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
mod rtc;
mod gui;
mod rtc_event;
mod audio;
mod screen;

#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() {
    println!("Start");
    gui::MainMenu::start_program();
    println!("End");
}