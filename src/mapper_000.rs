use crate::mapper::Mapper;

#[derive(Default)]
pub struct Mapper000 {
    n_prg_banks: u8,
    _n_chr_banks: u8,
}

impl Mapper for Mapper000 {
    fn new(prg_banks: u8, chr_banks: u8) -> Mapper000 {
        return Mapper000 {
            n_prg_banks: prg_banks,
            _n_chr_banks: chr_banks,
            ..Default::default()
        };
    }
    fn cpu_map_read(&self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x8000 {
            if self.n_prg_banks > 1 {
                *mapped_addr = (addr & 0x7FFF) as u32;
            } else {
                *mapped_addr = (addr & 0x3FFF) as u32;
            }
            return true;
        }
        return false;
    }
    fn cpu_map_write(&self, addr: u16, mapped_addr: &mut u32) -> bool {
        self.cpu_map_read(addr, mapped_addr)
    }
    fn ppu_map_read(&self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr <= 0x1FFF {
            *mapped_addr = addr as u32;
            return true;
        }

        return false;
    }
    fn ppu_map_write(&self, addr: u16, mapped_addr: &mut u32) -> bool {
        self.ppu_map_read(addr, mapped_addr)
    }
}
