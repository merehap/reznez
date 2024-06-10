#![allow(clippy::needless_late_init)]

use std::collections::BTreeMap;

use crate::memory::mapper::*;
use crate::memory::mappers::vrc::vrc_irq_state::VrcIrqState;

const PRG_WINDOWS_SWITCHABLE_8000: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);

const PRG_WINDOWS_SWITCHABLE_C000: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);

const CHR_WINDOWS: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::switchable_rom(C0)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::switchable_rom(C1)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::switchable_rom(C2)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::switchable_rom(C3)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::switchable_rom(C4)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::switchable_rom(C5)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::switchable_rom(C6)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::switchable_rom(C7)),
]);

const PRG_LAYOUTS: [PrgLayout; 2] = [
    PRG_WINDOWS_SWITCHABLE_8000,
    PRG_WINDOWS_SWITCHABLE_C000,
];

const NAME_TABLE_MIRRORINGS: [NameTableMirroring; 4] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
    NameTableMirroring::OneScreenLeftBank,
    NameTableMirroring::OneScreenRightBank,
];

const RAM_STATUSES: [RamStatus; 2] = [
    RamStatus::Disabled,
    RamStatus::ReadWrite,
];

pub struct Vrc4 {
    low_address_bank_register_ids: BTreeMap<u16, BankRegisterId>,
    high_address_bank_register_ids: BTreeMap<u16, BankRegisterId>,
    irq_state: VrcIrqState,
}

impl Mapper for Vrc4 {
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
            // Set bank for 8000 through 9FFF (or C000 through DFFF).
            0x8000..=0x8003 => params.set_bank_register(P0, value & 0b0001_1111),
            0x9000 => {
                let mirroring = NAME_TABLE_MIRRORINGS[usize::from(value & 0b0000_0011)];
                params.set_name_table_mirroring(mirroring);
            }
            0x9002 => {
                let fields = splitbits!(value, ".... ..ps");
                params.set_prg_layout(PRG_LAYOUTS[fields.p as usize]);
                params.set_ram_status(S0, RAM_STATUSES[fields.s as usize]);
            }
            // Set bank for A000 through AFFF.
            0xA000..=0xA003 => params.set_bank_register(P1, value & 0b0001_1111),

            // Set a CHR bank mapping.
            0xB000..=0xEFFF => {
                let bank;
                let mask;
                let mut register_id = self.low_address_bank_register_ids.get(&address.to_raw());
                if register_id.is_some() {
                    bank = u16::from(value);
                    mask = Some(0b0_0000_1111);
                } else {
                    register_id = self.high_address_bank_register_ids.get(&address.to_raw());
                    bank = u16::from(value) << 4;
                    mask = Some(0b1_1111_0000);
                }

                if let (Some(&register_id), Some(mask)) = (register_id, mask) {
                    params.set_bank_register_bits(register_id, bank, mask);
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
