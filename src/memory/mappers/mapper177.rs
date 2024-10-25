use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_rom(P0)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::fixed_ram(BankIndex::FIRST)),
]);

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

// BxROM with WorkRam and mirroring control.
pub struct Mapper177;

impl Mapper for Mapper177 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(32)
            .prg_bank_size(32 * KIBIBYTE)
            .prg_layout(PRG_LAYOUT)
            .chr_max_bank_count(1)
            .chr_bank_size(8 * KIBIBYTE)
            .chr_layout(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(value, "..mppppp");
                params.set_name_table_mirroring(MIRRORINGS[fields.m as usize]);
                params.set_bank_register(P0, fields.p);
            }
        }
    }
}
