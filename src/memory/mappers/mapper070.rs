use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(256 * KIBIBYTE)
    .chr_max_size(128 * KIBIBYTE)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .prg_layout(PrgLayout::new(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::switchable_rom(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
    ]))
    .chr_layout(ChrLayout::new(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C0)),
    ]))
    .build();

pub struct Mapper070;

impl Mapper for Mapper070 {
    fn has_bus_conflicts(&self) -> HasBusConflicts {
        HasBusConflicts::Yes
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => {
                let banks = splitbits!(value, "ppppcccc");
                params.set_bank_register(P0, banks.p);
                params.set_bank_register(C0, banks.c);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
