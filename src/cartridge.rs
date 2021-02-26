use crate::mapper::Mapper;
use crate::mapper_000::Mapper000;

use byteorder::ReadBytesExt; // 1.2.7
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

enum MIRROR {}

#[derive(Default)]
struct CartHeader {
    name: [u8; 4],
    prg_rom_chunks: u8,
    chr_rom_chunks: u8,
    mapper1: u8,
    mapper2: u8,
    prg_ram_size: u8,
    tv_system1: u8,
    tv_system2: u8,
    unused: [u8; 4],
}

#[derive(Default)]
pub struct Cartridge {
    header: CartHeader,
    mapper_id: u8,
    prg_banks: u8,
    chr_banks: u8,
    pub v_prg_memory: Vec<u8>,
    v_chr_memory: Vec<u8>,
    p_mapper: Mapper000,
}

impl Cartridge {
    pub fn new(file_name: &str) -> Self {
        let mut header: CartHeader = Default::default();

        let mut file = File::open(file_name).unwrap();
        let mut buf: [u8; 15] = [0; 15];
        let _ = file.read_exact(&mut header.name);
        header.prg_rom_chunks = file.read_u8().unwrap();
        header.chr_rom_chunks = file.read_u8().unwrap();
        header.mapper1 = file.read_u8().unwrap();
        header.mapper2 = file.read_u8().unwrap();
        header.prg_ram_size = file.read_u8().unwrap();
        header.tv_system1 = file.read_u8().unwrap();
        header.tv_system2 = file.read_u8().unwrap();
        let _ = file.read_exact(&mut header.unused);

        if header.mapper1 & 0x04 > 0 {
            let _ = file.seek(SeekFrom::Current(512));
        }

        let mapper_id = ((header.mapper2 >> 4) << 4) | (header.mapper1 >> 4);
        let file_type = 1;
        let mut prg_banks = 0;
        let mut v_prg_memory: Vec<u8> = Vec::new();
        let mut chr_banks = 0;
        let mut v_chr_memory: Vec<u8> = Vec::new();

        match file_type {
            0 => {}
            1 => {
                prg_banks = header.prg_rom_chunks;
                v_prg_memory.resize((prg_banks as u32 * 16384) as usize, 0);
                file.read_exact(&mut v_prg_memory).unwrap();

                chr_banks = header.chr_rom_chunks;
                v_chr_memory.resize((chr_banks as u32 * 8192) as usize, 0);
                file.read_exact(&mut v_chr_memory).unwrap();
            }
            2 => {}
            3 => {}
            _ => {}
        }

        let mut p_mapper = Mapper000::new(prg_banks, chr_banks);

        match mapper_id {
            0 => {
                p_mapper = Mapper000::new(prg_banks, chr_banks);
            }
            _ => {}
        }

        return Cartridge {
            header,
            mapper_id,
            prg_banks,
            v_prg_memory,
            chr_banks,
            v_chr_memory,
            p_mapper,
        };
    }
    // Communications with cpu bus
    pub fn cpu_read(&self, addr: usize, data: &mut u8) -> bool {
        let mut mapped_addr = 0 as u32;
        if (self.p_mapper).cpu_map_read(addr as u16, &mut mapped_addr) {
            *data = self.v_prg_memory[mapped_addr as usize];
            return true;
        }
        return false;
    }
    pub fn cpu_write(&mut self, addr: usize, data: u8) -> bool {
        let mut mapped_addr = 0 as u32;
        if (self.p_mapper).cpu_map_read(addr as u16, &mut mapped_addr) {
            self.v_prg_memory[mapped_addr as usize] = data;
            return true;
        }
        return false;
    }

    // Communications with ppu bus
    pub fn ppu_read(&self, addr: usize, data: &mut u8) -> bool {
        let mut mapped_addr = 0 as u32;
        if (self.p_mapper).cpu_map_read(addr as u16, &mut mapped_addr) {
            *data = self.v_chr_memory[mapped_addr as usize];
            return true;
        }
        return false;
    }
    pub fn ppu_write(&mut self, addr: usize, data: u8) -> bool {
        let mut mapped_addr = 0 as u32;
        if (self.p_mapper).cpu_map_read(addr as u16, &mut mapped_addr) {
            self.v_chr_memory[mapped_addr as usize] = data;
            return true;
        }
        return false;
    }
}
