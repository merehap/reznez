use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(32 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 *KIBIBYTE, PrgBank::ROM.fixed_index(0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.fixed_index(0))
    ])
    .fixed_name_table_mirroring()
    .build();

// NROM circuit board with simple copy protection
pub struct Mapper143;

impl Mapper for Mapper143 {
    fn peek_register(&self, _mem: &Memory, addr: CpuAddress) -> ReadResult {
        if *addr & 0b1110_0001_0000_0000 == 0b0100_0001_0000_0000 {
            // A simple copy-protection measure: return the inverted low 6 bits of the address.
            ReadResult::partial(!*addr as u8, 0b0011_1111)
        } else {
            ReadResult::OPEN_BUS
        }
    }

    fn write_register(&mut self, _mem: &mut Memory, addr: CpuAddress, _value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* Only NROM does nothing here. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}