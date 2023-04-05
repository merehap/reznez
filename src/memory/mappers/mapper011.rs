use crate::memory::mapper::*;

const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(4)
    .prg_bank_size(32 * KIBIBYTE)
    .prg_windows(PRG_WINDOWS)
    .chr_max_bank_count(16)
    .chr_bank_size(8 * KIBIBYTE)
    .chr_windows(CHR_WINDOWS)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

const PRG_WINDOWS: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::VariableBank(Rom, P0)),
]);

const CHR_WINDOWS: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::VariableBank(Rom, C0)),
]);

// Color Dreams. Same as GxROM except with different register locations.
pub struct Mapper011;

impl Mapper for Mapper011 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => {
                params.prg_memory_mut().set_bank_index_register(P0, value & 0b0000_0011);
                params.chr_memory_mut().set_bank_index_register(C0, value >> 4);
            }
        }
    }
}

impl Mapper011 {
    pub fn new() -> (Self, InitialLayout) {
        (Self, INITIAL_LAYOUT)
    }
}
