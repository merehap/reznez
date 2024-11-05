use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .override_bank_register(P1, 0b10)
    .override_bank_register(P2, 0b1110)
    // NROM-256
    .prg_max_size(1024 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        // P1 = P0 | 0b10
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P1)),
    ])
    // UNROM
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        // P2 = P0 | 0b1110
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P2)),
    ])
    // NROM-64
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::mirror_of(0x8000)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::mirror_of(0x8000)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::mirror_of(0x8000)),
    ])
    // NROM-128
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::mirror_of(0x8000)),
    ])
    .chr_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::RAM.fixed_index(0).status_register(S0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::Vertical,
        NameTableMirroring::Horizontal,
    ])
    .ram_statuses(&[
        RamStatus::ReadOnly,
        RamStatus::ReadWrite,
    ])
    .build();

// K-1029 and K-1030P (multicart)
// See https://www.nesdev.org/w/index.php?title=INES_Mapper_015&oldid=3854 for documentation, the
// latest version of that page is incomprehensible.
pub struct Mapper015;

impl Mapper for Mapper015 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let prg_layout_index = (cpu_address & 0b11) as u8;
                params.set_prg_layout(prg_layout_index);
                let chr_ram_writable = matches!(prg_layout_index, 1 | 2);
                params.set_ram_status(S0, chr_ram_writable as u8);

                let (s, mirroring, p) = splitbits_named!(min=u8, value, "smpppppp");
                let prg_bank = if prg_layout_index == 2 {
                    // NROM-64
                    combinebits!("0pppppps")
                } else {
                    combinebits!("0pppppp0")
                };

                params.set_name_table_mirroring(mirroring);
                params.set_bank_register(P0, prg_bank);
                params.set_bank_register(P1, prg_bank | 0b10);
                params.set_bank_register(P2, prg_bank | 0b1110);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
