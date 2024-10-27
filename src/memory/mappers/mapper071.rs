use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(256 * KIBIBYTE)
    .chr_max_size(8 * KIBIBYTE)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .prg_layout(PrgLayout::new(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::switchable_rom(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
    ]))
    .chr_layout(ChrLayout::new(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
    ]))
    .build();

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::OneScreenLeftBank,
    NameTableMirroring::OneScreenRightBank,
];

// Similar to UxROM.
pub struct Mapper071;

impl Mapper for Mapper071 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let fields = splitbits!(value, "...mpppp");
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x8FFF => { /* Do nothing. */ }
            // https://www.nesdev.org/wiki/INES_Mapper_071#Mirroring_($8000-$9FFF)
            0x9000..=0x9FFF => params.set_name_table_mirroring(MIRRORINGS[fields.m as usize]),
            0xA000..=0xBFFF => { /* Do nothing. */ }
            0xC000..=0xFFFF => params.set_bank_register(P0, fields.p),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
