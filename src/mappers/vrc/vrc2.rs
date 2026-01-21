#![allow(clippy::needless_late_init)]

use std::collections::BTreeMap;

use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
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
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

pub struct Vrc2 {
    low_address_bank_register_ids: BTreeMap<CpuAddress, ChrBankRegisterId>,
    high_address_bank_register_ids: BTreeMap<CpuAddress, ChrBankRegisterId>,
    chr_bank_low_bit_behavior: BankLowBitBehavior,
}

impl Mapper for Vrc2 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            // TODO: Properly implement microwire interface.
            0x6000..=0x7FFF => { /* Do nothing. */ }
            // Set bank for 8000 through 9FFF.
            0x8000..=0x8003 => bus.set_prg_register(P0, value & 0b0001_1111),
            0x9000 => bus.set_name_table_mirroring(value & 1),
            // Set bank for A000 through AFFF.
            0xA000..=0xA003 => bus.set_prg_register(P1, value & 0b0001_1111),

            // Set a CHR bank mapping.
            0xB000..=0xEFFF => {
                let mut bank;
                let mask;
                let mut register_id = self.low_address_bank_register_ids.get(&addr);
                if register_id.is_some() {
                    bank = u16::from(value);
                    mask = Some(0b0000_1111);
                } else {
                    register_id = self.high_address_bank_register_ids.get(&addr);
                    bank = u16::from(value) << 4;
                    mask = Some(0b1111_0000);
                }

                if let (Some(&register_id), Some(mut mask)) = (register_id, mask) {
                    if self.chr_bank_low_bit_behavior == BankLowBitBehavior::Ignore {
                        bank >>= 1;
                        mask >>= 1;
                    }

                    bus.set_chr_bank_register_bits(register_id, bank, mask);
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
        bank_registers: &[(u16, u16, ChrBankRegisterId)],
        chr_bank_low_bit_behavior: BankLowBitBehavior,
    ) -> Self {
        // Convert the address-to-register mappings to maps for easy lookup.
        let mut low_address_bank_register_ids = BTreeMap::new();
        let mut high_address_bank_register_ids = BTreeMap::new();
        for &(low, high, register_id) in bank_registers {
            low_address_bank_register_ids.insert(CpuAddress::new(low), register_id);
            high_address_bank_register_ids.insert(CpuAddress::new(high), register_id);
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
