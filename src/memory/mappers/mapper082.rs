use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x67FF, 2 * KIBIBYTE, Bank::WORK_RAM.fixed_index(0).status_register(S0)),
        Window::new(0x6800, 0x6FFF, 2 * KIBIBYTE, Bank::WORK_RAM.fixed_index(2).status_register(S1)),
        Window::new(0x7000, 0x73FF, 1 * KIBIBYTE, Bank::WORK_RAM.fixed_index(4).status_register(S2)),
        Window::new(0x7400, 0x7FFF, 3 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P2)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    // Large windows first.
    .chr_layout(&[
        Window::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::ROM.switchable(C0)),
        Window::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::ROM.switchable(C1)),
        Window::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::ROM.switchable(C2)),
        Window::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::ROM.switchable(C3)),
        Window::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C4)),
        Window::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C5)),
    ])
    // Small windows first.
    .chr_layout(&[
        Window::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::ROM.switchable(C2)),
        Window::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::ROM.switchable(C3)),
        Window::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C4)),
        Window::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C5)),
        Window::new(0x1000, 0x17FF, 2 * KIBIBYTE, Bank::ROM.switchable(C0)),
        Window::new(0x1800, 0x1FFF, 2 * KIBIBYTE, Bank::ROM.switchable(C1)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
    ])
    .ram_statuses(&[
        RamStatus::ReadOnlyZeros,
        RamStatus::ReadWrite,
    ])
    .build();

const READ_ONLY_ZEROS: u8 = 0;
const READ_WRITE: u8 = 1;

// Taito X1-017
// TODO: Read back 0 instead of open bus in all cases.
// TODO: Implement IRQ (even though it's not used in any commercial games).
pub struct Mapper082;

impl Mapper for Mapper082 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7EEF => { /* Do nothing. */ }
            0x7EF0 => params.set_bank_register(C0, value & 0b1111_1110),
            0x7EF1 => params.set_bank_register(C1, value & 0b1111_1110),
            0x7EF2 => params.set_bank_register(C2, value),
            0x7EF3 => params.set_bank_register(C3, value),
            0x7EF4 => params.set_bank_register(C4, value),
            0x7EF5 => params.set_bank_register(C5, value),
            0x7EF6 => {
                let fields = splitbits!(min=u8, value, "......lm");
                params.set_chr_layout(fields.l);
                params.set_name_table_mirroring(fields.m);
            }
            0x7EF7 => {
                let prg_ram_status = if value == 0xCA { READ_WRITE } else { READ_ONLY_ZEROS };
                params.set_ram_status(S0, prg_ram_status);
            }
            0x7EF8 => {
                let prg_ram_status = if value == 0x69 { READ_WRITE } else { READ_ONLY_ZEROS };
                params.set_ram_status(S1, prg_ram_status);
            }
            0x7EF9 => {
                let prg_ram_status = if value == 0x84 { READ_WRITE } else { READ_ONLY_ZEROS };
                params.set_ram_status(S2, prg_ram_status);
            }
            0x7EFA => params.set_bank_register(P0, splitbits_named!(value, "..pppp..")),
            0x7EFB => params.set_bank_register(P1, splitbits_named!(value, "..pppp..")),
            0x7EFC => params.set_bank_register(P2, splitbits_named!(value, "..pppp..")),
            0x7EFD => { /* IRQ not yet implemented. */ }
            0x7EFE => { /* IRQ not yet implemented. */ }
            0x7EFF => { /* IRQ not yet implemented. */ }
            0x7F00..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
