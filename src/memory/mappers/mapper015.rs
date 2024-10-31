use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .override_bank_register(P1, BankIndex::from_u8(0b10))
    .override_second_bank_register(P2, BankIndex::from_u8(0b1110))
    .prg_max_size(1024 * KIBIBYTE)
    // NROM-256
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
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::RAM.fixed_index(BankIndex::FIRST).status_register(S0)),
    ])
    .build();

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

// K-1029 and K-1030P (multicart)
// See https://www.nesdev.org/w/index.php?title=INES_Mapper_015&oldid=3854 for documentation, the
// latest version of that page is incomprehensible.
pub struct Mapper015;

impl Mapper for Mapper015 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let (sub_bank, mirroring, p) = splitbits_named!(min=u8, value, "smpppppp");
                let mut prg_bank = combinebits!("0pppppp0");

                let mut chr_ram_status = RamStatus::ReadWrite;
                let prg_layout_index = (address.to_raw() & 0b11) as u8;
                match prg_layout_index {
                    // NROM-256 and NROM-128
                    0 | 3 => chr_ram_status = RamStatus::ReadOnly,
                    // UNROM
                    1 => { /* Do nothing. */ }
                    // NROM-64
                    2 => prg_bank |= sub_bank,
                    _ => unreachable!(),
                }

                params.set_prg_layout(prg_layout_index);
                params.set_ram_status(S0, chr_ram_status);
                params.set_bank_register(P0, prg_bank);
                params.set_bank_register(P1, prg_bank | 0b10);
                params.set_bank_register(P2, prg_bank | 0b1110);
                params.set_name_table_mirroring(MIRRORINGS[mirroring as usize]);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
