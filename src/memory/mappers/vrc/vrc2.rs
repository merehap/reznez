#![allow(clippy::needless_late_init)]

use std::collections::BTreeMap;

use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM),
        Window::new(0x8000, 0x9FFF,  8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xA000, 0xBFFF,  8 * KIBIBYTE, Bank::ROM.switchable(P1)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(-2)),
    ])
    .chr_max_size(256 * KIBIBYTE)
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
        NameTableMirroring::Vertical,
        NameTableMirroring::Horizontal,
    ])
    .build();

pub struct Vrc2 {
    low_address_bank_register_ids: BTreeMap<u16, BankRegisterId>,
    high_address_bank_register_ids: BTreeMap<u16, BankRegisterId>,
    chr_bank_low_bit_behavior: BankLowBitBehavior,
}

impl Mapper for Vrc2 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            // TODO: Properly implement microwire interface.
            0x6000..=0x7FFF => params.write_prg(cpu_address, value),
            // Set bank for 8000 through 9FFF.
            0x8000..=0x8003 => params.set_bank_register(P0, value & 0b0001_1111),
            0x9000 => params.set_name_table_mirroring(value & 1),
            // Set bank for A000 through AFFF.
            0xA000..=0xA003 => params.set_bank_register(P1, value & 0b0001_1111),

            // Set a CHR bank mapping.
            0xB000..=0xEFFF => {
                let mut bank;
                let mask;
                let mut register_id = self.low_address_bank_register_ids.get(&cpu_address);
                if register_id.is_some() {
                    bank = u16::from(value);
                    mask = Some(0b0000_1111);
                } else {
                    register_id = self.high_address_bank_register_ids.get(&cpu_address);
                    bank = u16::from(value) << 4;
                    mask = Some(0b1111_0000);
                }

                if let (Some(&register_id), Some(mut mask)) = (register_id, mask) {
                    if self.chr_bank_low_bit_behavior == BankLowBitBehavior::Ignore {
                        bank >>= 1;
                        mask >>= 1;
                    }

                    params.set_bank_register_bits(register_id, bank, mask);
                }
            }

            0x4020..=0xFFFF => { /* All other writes do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Vrc2 {
    pub fn new(
        bank_registers: &[(u16, u16, BankRegisterId)],
        chr_bank_low_bit_behavior: BankLowBitBehavior,
    ) -> Self {
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
            chr_bank_low_bit_behavior,
        }
    }
}

#[derive(PartialEq)]
pub enum BankLowBitBehavior {
    Ignore,
    Keep,
}
