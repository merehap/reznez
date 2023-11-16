use crate::memory::mapper::*;

const PRG_WINDOWS: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::Empty),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
]);

const CHR_WINDOWS: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::Switchable(Rom, C0)),
]);

// Same as GNROM, except the writable port is moved to 0x6000 and more CHR banks are allowed.
pub struct Mapper140;

impl Mapper for Mapper140 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(4)
            .prg_bank_size(32 * KIBIBYTE)
            .prg_windows(PRG_WINDOWS)
            .chr_max_bank_count(16)
            .chr_bank_size(8 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => {
                assert_eq!(value & 0b1100_0000, 0);
                params.prg_memory_mut().set_bank_index_register(P0, (value & 0b0011_0000) >> 4);
                params.chr_memory_mut().set_bank_index_register(C0, value & 0b0000_1111);
            }
            0x8000..=0xFFFF => { /* Do nothing. */ }
        }
    }
}
