use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(1024 * KIBIBYTE)
    .chr_max_size(8 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_rom(P0)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::fixed_ram(BankIndex::FIRST)),
    ])
    .build();

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

// BxROM with WorkRam and mirroring control.
pub struct Mapper177;

impl Mapper for Mapper177 {
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

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
