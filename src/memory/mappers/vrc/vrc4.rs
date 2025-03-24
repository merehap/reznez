#![allow(clippy::needless_late_init)]

use std::collections::BTreeMap;

use crate::memory::mapper::*;
use crate::memory::mappers::vrc::vrc_irq_state::VrcIrqState;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-2)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-2)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::ROM.switchable(C0)),
        Window::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::ROM.switchable(C1)),
        Window::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C2)),
        Window::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C3)),
        Window::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::ROM.switchable(C4)),
        Window::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::ROM.switchable(C5)),
        Window::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C6)),
        Window::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .ram_statuses(&[
        RamStatus::Disabled,
        RamStatus::ReadWrite,
    ])
    .build();

pub struct Vrc4 {
    low_address_bank_register_ids: BTreeMap<u16, BankRegisterId>,
    high_address_bank_register_ids: BTreeMap<u16, BankRegisterId>,
    irq_state: VrcIrqState,
}

impl Mapper for Vrc4 {
    fn on_end_of_cpu_cycle(&mut self, params: &mut MapperParams, _cycle: i64) {
        self.irq_state.step(params);
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x6000..=0x7FFF => { /* Do nothing. */ }
            // Set bank for 8000 through 9FFF (or C000 through DFFF).
            0x8000..=0x8003 => params.set_bank_register(P0, value & 0b0001_1111),
            0x9000 => {
                params.set_name_table_mirroring(value & 0b11);
            }
            0x9002 => {
                let fields = splitbits!(min=u8, value, "......ps");
                params.set_prg_layout(fields.p);
                params.set_ram_status(S0, fields.s);
            }
            // Set bank for A000 through AFFF.
            0xA000..=0xA003 => params.set_bank_register(P1, value & 0b0001_1111),

            // Set a CHR bank mapping.
            0xB000..=0xEFFF => {
                let bank;
                let mask;
                let mut register_id = self.low_address_bank_register_ids.get(&cpu_address);
                if register_id.is_some() {
                    bank = u16::from(value);
                    mask = Some(0b0_0000_1111);
                } else {
                    register_id = self.high_address_bank_register_ids.get(&cpu_address);
                    bank = u16::from(value) << 4;
                    mask = Some(0b1_1111_0000);
                }

                if let (Some(&register_id), Some(mask)) = (register_id, mask) {
                    params.set_bank_register_bits(register_id, bank, mask);
                }
            }

            0xF000 => self.irq_state.set_reload_value_low_bits(value),
            0xF001 => self.irq_state.set_reload_value_high_bits(value),
            0xF002 => self.irq_state.set_mode(params, value),
            0xF003 => self.irq_state.acknowledge(params),
            0x4020..=0xFFFF => { /* All other writes do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Vrc4 {
    pub fn new(bank_registers: &[(u16, u16, BankRegisterId)]) -> Self {
        // Convert the address-to-register mappings to maps for easy lookup.
        let mut low_address_bank_register_ids = BTreeMap::new();
        let mut high_address_bank_register_ids = BTreeMap::new();
        for &(low, high, register_id) in bank_registers {
            low_address_bank_register_ids.insert(low, register_id);
            high_address_bank_register_ids.insert(high, register_id);
        }

        Self {
            low_address_bank_register_ids,
            high_address_bank_register_ids,
            irq_state: VrcIrqState::new(),
        }
    }
}
