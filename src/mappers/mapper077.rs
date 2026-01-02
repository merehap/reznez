use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(32 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::RAM.fixed_index(2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::RAM.fixed_index(3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::RAM.fixed_index(4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::RAM.fixed_index(5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::RAM.fixed_index(6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::RAM.fixed_index(7)),
    ])
    .four_screen_mirroring_definition(NameTableMirroring::new(
        NameTableSource::Ram { bank_number: BankNumber::from_u8(0) },
        NameTableSource::Ram { bank_number: BankNumber::from_u8(1) },
        NameTableSource::Ciram(CiramSide::Left),
        NameTableSource::Ciram(CiramSide::Right),
    ))
    .fixed_name_table_mirroring()
    .build();

// Irem (Napoleon Senki)
pub struct Mapper077;

impl Mapper for Mapper077 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let banks = splitbits!(value, "ccccpppp");
                mem.set_chr_register(C0, banks.c);
                mem.set_prg_register(P0, banks.p);
            }
        }
    }

    fn has_bus_conflicts(&self) -> bool {
        true
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
