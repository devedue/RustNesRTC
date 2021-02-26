#![feature(try_trait)]
#![allow(dead_code)]
#![allow(unused)]
#![feature(once_cell)]
#![feature(new_uninit)]
#![feature(get_mut_unchecked)]

mod bus;
mod cartridge;
mod cpu;
mod mapper;
mod mapper_000;
mod nes;
mod ppu;
use crate::bus::Bus;
use nes::*;
use std::fs::File;
use pge::PGE;

fn main() {
    
    let mut nes = Nes::new();
    let mut pge = PGE::construct("Demo Part #3", 780, 480, 2, 2);
    pge.start(&mut nes);
}
