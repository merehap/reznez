use crate::memory::mapper::*;

const PRG_WINDOWS: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::VariableBank(Rom, P0)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::LAST)),
]);

const CHR_WINDOWS: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::VariableBank(Rom, C0)),
]);

// Similar to Mapper070, but with one screen mirroring control.
pub struct Mapper152;

impl Mapper for Mapper152 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(8)
            .prg_bank_size(16 * KIBIBYTE)
            .prg_windows(PRG_WINDOWS)
            .chr_max_bank_count(16)
            .chr_bank_size(8 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroring::OneScreenLeftBank.to_source())
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => {
                if value & 0b1000_0000 == 0 {
                    params.set_name_table_mirroring(NameTableMirroring::OneScreenLeftBank);
                } else {
                    params.set_name_table_mirroring(NameTableMirroring::OneScreenRightBank);
                }

                params.prg_memory_mut().set_bank_index_register(P0, (value >> 4) & 0b0111);
                params.chr_memory_mut().set_bank_index_register(C0, value & 0b1111);
            }
        }
    }
}
