// #![allow(dead_code)]
#![feature(untagged_unions)]

mod bus;
mod cartridge;
mod cpu;
mod mapper;
mod mapper_000;
mod nes;
mod ppu;
mod util;
use nes::*;
use pge::PGE;

fn main() {
    
    let mut nes = Nes::new();
    let mut pge = PGE::construct("Demo Part #4", 780, 480, 2, 2);
    pge.start(&mut nes);
}
