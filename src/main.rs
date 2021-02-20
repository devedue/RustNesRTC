mod bus;
mod olc6502;
use bus::*;

fn main() {
    unsafe {
        println!("Hello, world!");
        let mut b = Bus::new();
        b.write(0, 5);
        println!("{}", b.read(0, false));
        b.write(0, 6);
        println!("{}", b.read(0, false));
    }
}
