use crate::memory::mapper::*;
use crate::memory::mappers::vrc::vrc_irq_state::VrcIrqState;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P2)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_max_size(256 * KIBIBYTE)
    // TODO: Support CHR ROM and RAM
    .chr_layout(&[
        Window::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::RAM.switchable(C0)),
        Window::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::RAM.switchable(C1)),
        Window::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::RAM.switchable(C2)),
        Window::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::RAM.switchable(C3)),
        Window::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::RAM.switchable(C4)),
        Window::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::RAM.switchable(C5)),
        Window::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::RAM.switchable(C6)),
        Window::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::RAM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::Vertical,
        NameTableMirroring::Horizontal,
        NameTableMirroring::OneScreenLeftBank,
        NameTableMirroring::OneScreenRightBank,
    ])
    .ram_statuses(&[
        RamStatus::ReadOnly,
        RamStatus::ReadWrite,
    ])
    .build();

// Konami VRC7a
// TODO: Expansion audio.
pub struct Mapper085_2 {
    irq_state: VrcIrqState,
}

impl Mapper for Mapper085_2 {
    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {
        self.irq_state.step();
    }

    fn irq_pending(&self) -> bool {
        self.irq_state.pending()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x8000 => params.set_bank_register(P0, value & 0b0011_1111),
            0x8010 => params.set_bank_register(P1, value & 0b0011_1111),
            0x9000 => params.set_bank_register(P2, value & 0b0011_1111),
            0x9010 | 0x9030 => { /* TODO: Expansion Audio */ }
            0xA000 => params.set_bank_register(C0, value),
            0xA010 => params.set_bank_register(C1, value),
            0xB000 => params.set_bank_register(C2, value),
            0xB010 => params.set_bank_register(C3, value),
            0xC000 => params.set_bank_register(C4, value),
            0xC010 => params.set_bank_register(C5, value),
            0xD000 => params.set_bank_register(C6, value),
            0xD010 => params.set_bank_register(C7, value),
            0xE000 => {
                // TODO: Silence expansion audio
                let fields = splitbits!(min=u8, value, "rs....mm");
                params.set_ram_status(S0, fields.r);
                params.set_name_table_mirroring(fields.m);
            }
            0xE010 => self.irq_state.set_reload_value(value),
            0xF000 => self.irq_state.set_mode(value),
            0xF010 => self.irq_state.acknowledge(),

            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper085_2 {
    pub fn new() -> Self {
        Self { irq_state: VrcIrqState::new() }
    }
}
