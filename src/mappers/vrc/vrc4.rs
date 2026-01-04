#![allow(clippy::needless_late_init)]

use std::collections::BTreeMap;

use crate::mapper::*;
use crate::mappers::vrc::vrc_irq_state::VrcIrqState;
use crate::memory::memory::Bus;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WORK_RAM.read_write_status(R0, W0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WORK_RAM.read_write_status(R0, W0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
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
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

pub struct Vrc4 {
    low_address_bank_register_ids: BTreeMap<CpuAddress, ChrBankRegisterId>,
    high_address_bank_register_ids: BTreeMap<CpuAddress, ChrBankRegisterId>,
    irq_state: VrcIrqState,
}

impl Mapper for Vrc4 {
    fn on_end_of_cpu_cycle(&mut self, bus: &mut Bus) {
        self.irq_state.step(bus);
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x6000..=0x7FFF => { /* Do nothing. */ }
            // Set bank for 8000 through 9FFF (or C000 through DFFF).
            0x8000..=0x8003 => bus.set_prg_register(P0, value & 0b0001_1111),
            0x9000 => {
                bus.set_name_table_mirroring(value & 0b11);
            }
            0x9002 => {
                let fields = splitbits!(value, "......pe");
                bus.set_prg_layout(fields.p as u8);
                bus.set_reads_enabled(R0, fields.e);
                bus.set_writes_enabled(W0, fields.e);
            }
            // Set bank for A000 through AFFF.
            0xA000..=0xA003 => bus.set_prg_register(P1, value & 0b0001_1111),

            // Set a CHR bank mapping.
            0xB000..=0xEFFF => {
                let bank;
                let mask;
                let mut register_id = self.low_address_bank_register_ids.get(&addr);
                if register_id.is_some() {
                    bank = u16::from(value);
                    mask = Some(0b0_0000_1111);
                } else {
                    register_id = self.high_address_bank_register_ids.get(&addr);
                    bank = u16::from(value) << 4;
                    mask = Some(0b1_1111_0000);
                }

                if let (Some(&register_id), Some(mask)) = (register_id, mask) {
                    bus.set_chr_bank_register_bits(register_id, bank, mask);
                }
            }

            0xF000 => self.irq_state.set_reload_value_low_bits(value),
            0xF001 => self.irq_state.set_reload_value_high_bits(value),
            0xF002 => self.irq_state.set_mode(bus, value),
            0xF003 => self.irq_state.acknowledge(bus),
            0x4020..=0xFFFF => { /* All other writes do nothing. */ }
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(self.irq_state.to_irq_counter_info())
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Vrc4 {
    pub fn new(bank_registers: &[(u16, u16, ChrBankRegisterId)]) -> Self {
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
            irq_state: VrcIrqState::new(),
        }
    }
}
