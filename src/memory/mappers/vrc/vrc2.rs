#![allow(clippy::needless_late_init)]

use std::collections::BTreeMap;

use crate::memory::mapper::*;

const PRG_WINDOWS: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM),
    PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
]);

const CHR_WINDOWS: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C0)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C1)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C2)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C3)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C4)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C5)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C6)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C7)),
]);

const NAME_TABLE_MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

pub struct Vrc2 {
    low_address_bank_register_ids: BTreeMap<u16, BankRegisterId>,
    high_address_bank_register_ids: BTreeMap<u16, BankRegisterId>,
    chr_bank_low_bit_behavior: ChrBankLowBitBehavior,
}

impl Mapper for Vrc2 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(32)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_windows(PRG_WINDOWS)
            .chr_max_bank_count(256)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            // TODO: Properly implement microwire interface.
            0x6000..=0x7FFF => params.write_prg(address, value),
            // Set bank for 8000 through 9FFF.
            0x8000..=0x8003 => params.set_bank_register(P0, value & 0b0001_1111),
            0x9000 => {
                let mirroring = NAME_TABLE_MIRRORINGS[usize::from(value & 0b0000_0001)];
                params.set_name_table_mirroring(mirroring);
            }
            // Set bank for A000 through AFFF.
            0xA000..=0xA003 => params.set_bank_register(P1, value & 0b0001_1111),

            // Set a CHR bank mapping.
            0xB000..=0xEFFF => {
                let mut bank;
                let mask;
                let mut register_id = self.low_address_bank_register_ids.get(&address.to_raw());
                if register_id.is_some() {
                    bank = u16::from(value);
                    mask = Some(0b0000_1111);
                } else {
                    register_id = self.high_address_bank_register_ids.get(&address.to_raw());
                    bank = u16::from(value) << 4;
                    mask = Some(0b1111_0000);
                }

                if let (Some(&register_id), Some(mut mask)) = (register_id, mask) {
                    if self.chr_bank_low_bit_behavior == ChrBankLowBitBehavior::Ignore {
                        bank >>= 1;
                        mask >>= 1;
                    }

                    params.set_bank_register_bits(register_id, bank, mask);
                }
            }

            0x4020..=0xFFFF => { /* All other writes do nothing. */ }
        }
    }
}

impl Vrc2 {
    pub fn new(
        bank_registers: &[(u16, u16, BankRegisterId)],
        chr_bank_low_bit_behavior: ChrBankLowBitBehavior,
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
pub enum ChrBankLowBitBehavior {
    Ignore,
    Keep,
}
