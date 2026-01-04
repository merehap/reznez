use log::info;
use ux::u4;

use crate::mapper::*;
use crate::mappers::common::kaiser202;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)), // P0 can only ever be 15 or 31
    ])
    .override_prg_bank_register(P1, 0b0001_0000)
    .override_prg_bank_register(P2, 0b0001_0000)
    .override_prg_bank_register(P3, 0b0001_0000)
    .override_prg_bank_register(P0, 0b0001_1111) // The last bank
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
    ])
    .build();

pub struct Mapper056 {
    irq_counter: ReloadDrivenCounter,
    selected_prg_bank: Option<PrgBankRegisterId>,
}

impl Mapper for Mapper056 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x8FFF => self.irq_counter.set_reload_value_lowest_nybble(u4::new(value & 0xF)),
            0x9000..=0x9FFF => self.irq_counter.set_reload_value_second_lowest_nybble(u4::new(value & 0xF)),
            0xA000..=0xAFFF => self.irq_counter.set_reload_value_second_highest_nybble(u4::new(value & 0xF)),
            0xB000..=0xBFFF => self.irq_counter.set_reload_value_highest_nybble(u4::new(value & 0xF)),
            0xC000..=0xCFFF => {
                bus.cpu_pinout.acknowledge_mapper_irq();

                let enabled = value != 0;
                self.irq_counter.set_enabled(enabled);
                if enabled {
                    self.irq_counter.force_reload();
                }
            }
            0xD000..=0xDFFF => {
                bus.cpu_pinout.acknowledge_mapper_irq();
            }
            0xE000..=0xEFFF => {
                match value & 0b111 {
                    0 | 5 | 7 => info!("Unknown bank select occurred: {}", value & 0b111),
                    1 => self.selected_prg_bank = Some(P1), // 0x8000
                    2 => self.selected_prg_bank = Some(P2), // 0xA000
                    3 => self.selected_prg_bank = Some(P3), // 0xC000
                    4 | 6 => self.selected_prg_bank = None,
                    _ => unreachable!(),
                }
            }
            0xF000..=0xFFFF => {
                if let Some(select_prg_bank) = self.selected_prg_bank {
                    bus.set_prg_bank_register_bits(select_prg_bank, value.into(), 0b0000_1111);
                }
            }
        }

        let addr = *addr as usize;
        let prg_id = [P1, P2, P3, P0][addr & 0b11];
        let chr_id = [C0, C1, C2, C3, C4, C5, C6, C7][addr & 0b111];

        // Overlapping registers in the 0xFXXX range.
        if matches!(addr & 0xFC03, 0xF000..=0xF003) && self.selected_prg_bank.is_some() {
            bus.set_prg_bank_register_bits(prg_id, u16::from(value & 0b0001_0000), 0b1111_0000);
        } else if addr & 0xFC00 == 0xF800 {
            bus.set_name_table_mirroring(value & 1);
        } else if matches!(addr & 0xFC07, 0xFC00..=0xFC07) {
            bus.set_chr_register(chr_id, value & 0b0111_1111);
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper056 {
    pub fn new() -> Self {
        Self {
            irq_counter: kaiser202::IRQ_COUNTER,
            selected_prg_bank: None,
        }
    }
}