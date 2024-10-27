use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .chr_max_size(32 * KIBIBYTE)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .prg_layout(PrgLayout::new(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_rom(P0)),
    ]))
    .chr_layout(ChrLayout::new(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C0)),
    ]))
    .build();

// Bit Corp.'s Crime Busters
// TODO: Oversize support
pub struct Mapper038;

impl Mapper for Mapper038 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let banks = splitbits!(value, "....ccpp");
                // Oversize CHR, matching FCEUX's implementation.
                params.set_bank_register(C0, banks.c);
                params.set_bank_register(P0, banks.p);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
