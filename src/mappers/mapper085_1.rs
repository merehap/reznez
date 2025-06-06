use crate::mapper::*;
use crate::mappers::vrc::vrc_irq_state::VrcIrqState;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WORK_RAM.status_register(S0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    // TODO: Support CHR ROM and RAM
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .read_write_statuses(&[
        ReadWriteStatus::ReadOnly,
        ReadWriteStatus::ReadWrite,
    ])
    .build();

// Konami VRC7b
#[derive(Default)]
pub struct Mapper085_1 {
    irq_state: VrcIrqState,
}

impl Mapper for Mapper085_1 {
    fn on_end_of_cpu_cycle(&mut self, params: &mut MapperParams, _cycle: i64) {
        self.irq_state.step(params);
    }

    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x8000 => params.set_prg_register(P0, value & 0b0011_1111),
            0x8008 => params.set_prg_register(P1, value & 0b0011_1111),
            0x9000 => params.set_prg_register(P2, value & 0b0011_1111),
            0xA000 => params.set_chr_register(C0, value),
            0xA008 => params.set_chr_register(C1, value),
            0xB000 => params.set_chr_register(C2, value),
            0xB008 => params.set_chr_register(C3, value),
            0xC000 => params.set_chr_register(C4, value),
            0xC008 => params.set_chr_register(C5, value),
            0xD000 => params.set_chr_register(C6, value),
            0xD008 => params.set_chr_register(C7, value),
            0xE000 => {
                let fields = splitbits!(min=u8, value, "r.....mm");
                params.set_read_write_status(S0, fields.r);
                params.set_name_table_mirroring(fields.m);
            }
            0xE008 => self.irq_state.set_reload_value(value),
            0xF000 => self.irq_state.set_mode(params, value),
            0xF008 => self.irq_state.acknowledge(params),

            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
