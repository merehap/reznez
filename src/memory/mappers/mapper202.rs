use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::mirror_of(0x8000)),
    ])
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .chr_max_size(64 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::Vertical,
        NameTableMirroring::Horizontal,
    ])
    .build();

const PRG16: u8 = 0;
const PRG32: u8 = 1;

// 150-in-1 pirate cart
pub struct Mapper202;

impl Mapper for Mapper202 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, _value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                // Overlapping fields.
                let layout = splitbits_named!(
                    cpu_address, ".... .... .... l..l");
                let (bank_index, mirroring) = splitbits_named!(min=u8,
                    cpu_address, ".... .... .... rrrm");

                if layout == 3 {
                    params.set_prg_layout(PRG32);
                } else {
                    params.set_prg_layout(PRG16);
                }

                params.set_bank_register(P0, bank_index);
                params.set_bank_register(C0, bank_index);
                params.set_name_table_mirroring(mirroring);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
