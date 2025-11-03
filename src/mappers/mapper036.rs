use splitbits::splitbits_named_ux;
use ux::u2;

use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ])
    .fixed_name_table_mirroring()
    .build();

// TXC 01-22000-400
pub struct Mapper036 {
    invert_mode: bool,
    increment_mode: bool,
    rr: u2,
    pp: u2,
}

impl Mapper for Mapper036 {
    fn peek_register(&self, _mem: &Memory, addr: CpuAddress) -> ReadResult {
        if *addr & 0xE100 == 0x4100 {
            ReadResult::partial(u8::from(self.rr) << 4, 0b0011_0000)
        } else {
            ReadResult::OPEN_BUS
        }
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        if *addr & 0xE200 == 0x4200 {
            mem.set_chr_register(C0, value & 0b1111);
        } else if *addr & 0x8000 == 0x8000 {
            mem.set_prg_register(P0, self.rr);
        } else {
            match *addr & 0xE103 {
                0x4100 => {
                    self.rr = if self.increment_mode {
                        self.rr.wrapping_add(u2::new(1))
                    } else if self.invert_mode {
                        !self.pp
                    } else {
                        self.pp
                    };
                }
                0x4101 => self.invert_mode = splitbits_named!(value, "...i ...."),
                0x4102 => self.pp = splitbits_named_ux!(value, "..pp ...."),
                0x4103 => self.increment_mode = splitbits_named!(value, "...i ...."),
                _ => { /* Do nothing. */ }
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper036 {
    pub fn new() -> Self {
        Self {
            invert_mode: false,
            increment_mode: false,
            rr: u2::new(0),
            pp: u2::new(0),
        }
    }
}