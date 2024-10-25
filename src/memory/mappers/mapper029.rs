use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C0)),
]);

// Homebrew: Sealie Computing RET-CUFROM revD
// TODO: Reprogramming logic.
// FIXME: Untested!
pub struct Mapper029;

impl Mapper for Mapper029 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(8)
            .prg_bank_size(16 * KIBIBYTE)
            .prg_layout(PRG_LAYOUT)
            .chr_max_bank_count(4)
            .chr_bank_size(8 * KIBIBYTE)
            .chr_layout(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let banks = splitbits!(value, "...pppcc");
                params.set_bank_register(P0, banks.p);
                params.set_bank_register(C0, banks.c);
            }
        }
    }
}
