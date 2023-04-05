use crate::memory::mapper::*;

const PRG_WINDOWS: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::LAST)),
]);

const CHR_WINDOWS: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrType::VariableBank(Rom, C0)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrType::VariableBank(Rom, C1)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C2)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C3)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C4)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C5)),
]);

// Taito's TC0190
pub struct Mapper033;

impl Mapper for Mapper033 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(64)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_windows(PRG_WINDOWS)
            .chr_max_bank_count(512)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x8000 => {
                let mirroring = if value & 0b0100_0000 == 0 {
                    NameTableMirroring::Vertical
                } else {
                    NameTableMirroring::Horizontal
                };
                params.set_name_table_mirroring(mirroring);
                params.prg_memory_mut().set_bank_index_register(P0, value & 0b0011_1111);
            }
            0x8001 => params.prg_memory_mut().set_bank_index_register(P1, value & 0b0011_1111),
            // Large CHR windows: this allows accessing 512KiB CHR by doubling the bank indexes.
            0x8002 => params.chr_memory_mut().set_bank_index_register(C0, 2 * u16::from(value)),
            0x8003 => params.chr_memory_mut().set_bank_index_register(C1, 2 * u16::from(value)),
            // Small CHR windows.
            0xA000 => params.chr_memory_mut().set_bank_index_register(C2, value),
            0xA001 => params.chr_memory_mut().set_bank_index_register(C3, value),
            0xA002 => params.chr_memory_mut().set_bank_index_register(C4, value),
            0xA003 => params.chr_memory_mut().set_bank_index_register(C5, value),
            _ => { /* Do nothing. */ }
        }
    }
}
