use std::collections::BTreeMap;

use crate::memory::mapper::*;
use crate::memory::mappers::vrc::vrc_irq_state::VrcIrqState;

const PRG_WINDOWS_SWITCHABLE_8000: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);

// Only used for VRC4.
const PRG_WINDOWS_SWITCHABLE_C000: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
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

const NAME_TABLE_MIRRORINGS: [NameTableMirroring; 4] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
    NameTableMirroring::OneScreenLeftBank,
    NameTableMirroring::OneScreenRightBank,
];

pub struct Vrc2And4 {
    low_address_bank_index_register_ids: BTreeMap<u16, BankIndexRegisterId>,
    high_address_bank_index_register_ids: BTreeMap<u16, BankIndexRegisterId>,
    chr_bank_low_bit_behavior: ChrBankLowBitBehavior,
    irq_state: VrcIrqState,
}

impl Mapper for Vrc2And4 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(32)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_windows(PRG_WINDOWS_SWITCHABLE_8000)
            .chr_max_bank_count(512)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {
        self.irq_state.step();
    }

    fn irq_pending(&self) -> bool {
        self.irq_state.pending()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x6000..=0x7FFF => params.write_prg(address, value),
            // Set bank for 8000 through 9FFF (or C000 through DFFF, VRC4 only).
            0x8000..=0x8003 => params.set_bank_index_register(P0, value & 0b0001_1111),
            0x9000 => {
                // Wai Wai World writes a weird value here (all ones). Due to it being VRC2, only the last bit is used.
                // Every other ROM is well-behaved and uses the last 2 bits (VRC2 setting only the last bit).
                // https://forums.nesdev.org/viewtopic.php?f=3&t=13473
                let mask = if value == 0b1111_1111 {
                    0b0000_0001
                } else {
                    0b0000_0011
                };
                params.set_name_table_mirroring(NAME_TABLE_MIRRORINGS[usize::from(value & mask)]);
            }
            // VRC4-only
            0x9002 => {
                if value & 0b0000_0001 == 0 {
                    params.disable_work_ram(0x6000);
                } else {
                    params.enable_work_ram(0x6000);
                }

                let prg_layout = if value & 0b0000_0010 == 0 {
                    PRG_WINDOWS_SWITCHABLE_8000
                } else {
                    PRG_WINDOWS_SWITCHABLE_C000
                };
                params.set_prg_layout(prg_layout);
            }
            // Set bank for A000 through AFFF.
            0xA000..=0xA003 => params.set_bank_index_register(P1, value & 0b0001_1111),

            // Set a CHR bank mapping.
            0xB000..=0xEFFF => {
                let mut bank;
                let mask;
                let mut register_id = self.low_address_bank_index_register_ids.get(&address.to_raw());
                if register_id.is_some() {
                    bank = u16::from(value);
                    mask = Some(0b0_0000_1111)
                } else {
                    register_id = self.high_address_bank_index_register_ids.get(&address.to_raw());
                    bank = u16::from(value) << 4;
                    mask = Some(0b1_1111_0000)
                }

                if let (Some(&register_id), Some(mut mask)) = (register_id, mask) {
                    if self.chr_bank_low_bit_behavior == ChrBankLowBitBehavior::Ignore {
                        bank >>= 1;
                        mask >>= 1;
                    }

                    params.set_bank_index_register_bits(register_id, bank, mask);
                }
            }

            0xF000 => self.irq_state.set_reload_value_low_bits(value),
            0xF001 => self.irq_state.set_reload_value_high_bits(value),
            0xF002 => self.irq_state.set_mode(value),
            0xF003 => self.irq_state.acknowledge(),
            0x4020..=0xFFFF => { /* All other writes do nothing. */ }
        }
    }
}

impl Vrc2And4 {
    pub fn new(
        bank_index_registers: &[(u16, u16, BankIndexRegisterId)],
        chr_bank_low_bit_behavior: ChrBankLowBitBehavior,
    ) -> Self {
        // Convert the address-to-register mappings to maps for easy lookup.
        let mut low_address_bank_index_register_ids = BTreeMap::new();
        let mut high_address_bank_index_register_ids = BTreeMap::new();
        for &(low, high, register_id) in bank_index_registers {
            low_address_bank_index_register_ids.insert(low, register_id);
            high_address_bank_index_register_ids.insert(high, register_id);
        }

        Self {
            low_address_bank_index_register_ids,
            high_address_bank_index_register_ids,
            chr_bank_low_bit_behavior,
            irq_state: VrcIrqState::new(),
        }
    }
}

#[derive(PartialEq)]
pub enum ChrBankLowBitBehavior {
    Ignore,
    Keep,
}
