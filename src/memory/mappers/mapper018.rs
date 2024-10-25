use crate::memory::mapper::*;

const PRG_WINDOWS: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::switchable_rom(P2)),
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

const NAME_TABLE_MIRRORINGS: [NameTableMirroring; 4] = [
    NameTableMirroring::Horizontal,
    NameTableMirroring::Vertical,
    NameTableMirroring::OneScreenLeftBank,
    NameTableMirroring::OneScreenRightBank,
];

// Jaleco SS 88006
// TODO: Expansion Audio
// TODO: PRG RAM chip enable/disable
pub struct Mapper018 {
    work_ram_write_enabled: bool,

    irq_enabled: bool,
    irq_pending: bool,
    irq_counter: u16,
    irq_counter_mask: u16,
    irq_reload_value: u16,
}

impl Mapper for Mapper018 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(64)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_layout(PRG_WINDOWS)
            .chr_max_bank_count(256)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_layout(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {
        // Disch: When enabled, the IRQ counter counts down every CPU cycle.
        //        When it wraps, an IRQ is generated.
        if self.irq_enabled {
            let mut new_counter = self.irq_counter & self.irq_counter_mask;
            if new_counter == 0 {
                self.irq_pending = true;
            }

            new_counter -= 1;
            set_bits(&mut self.irq_counter, new_counter, self.irq_counter_mask);
        }
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        if matches!(address.to_raw(), 0x6000..=0x7FFF) {
            if self.work_ram_write_enabled {
                params.write_prg(address, value);
            }

            return;
        }

        let address = address.to_raw();
        let value = u16::from(value);
        match address & 0b1111_0000_0000_0011 {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => unreachable!(),
            0x8000 => params.set_bank_register_bits(P0, value     , 0b0000_1111),
            0x8001 => params.set_bank_register_bits(P0, value << 4, 0b0011_0000),
            0x8002 => params.set_bank_register_bits(P1, value     , 0b0000_1111),
            0x8003 => params.set_bank_register_bits(P1, value << 4, 0b0011_0000),
            0x9000 => params.set_bank_register_bits(P2, value     , 0b0000_1111),
            0x9001 => params.set_bank_register_bits(P2, value << 4, 0b0011_0000),
            0x9002 => self.work_ram_write_enabled = value & 0b0000_0010 != 0,
            0x9003 => { /* Do nothing */ }
            0xA000 => params.set_bank_register_bits(C0, value     , 0b0000_1111),
            0xA001 => params.set_bank_register_bits(C0, value << 4, 0b1111_0000),
            0xA002 => params.set_bank_register_bits(C1, value     , 0b0000_1111),
            0xA003 => params.set_bank_register_bits(C1, value << 4, 0b1111_0000),
            0xB000 => params.set_bank_register_bits(C2, value     , 0b0000_1111),
            0xB001 => params.set_bank_register_bits(C2, value << 4, 0b1111_0000),
            0xB002 => params.set_bank_register_bits(C3, value     , 0b0000_1111),
            0xB003 => params.set_bank_register_bits(C3, value << 4, 0b1111_0000),
            0xC000 => params.set_bank_register_bits(C4, value     , 0b0000_1111),
            0xC001 => params.set_bank_register_bits(C4, value << 4, 0b1111_0000),
            0xC002 => params.set_bank_register_bits(C5, value     , 0b0000_1111),
            0xC003 => params.set_bank_register_bits(C5, value << 4, 0b1111_0000),
            0xD000 => params.set_bank_register_bits(C6, value     , 0b0000_1111),
            0xD001 => params.set_bank_register_bits(C6, value << 4, 0b1111_0000),
            0xD002 => params.set_bank_register_bits(C7, value     , 0b0000_1111),
            0xD003 => params.set_bank_register_bits(C7, value << 4, 0b1111_0000),
            0xE000 => set_bits(&mut self.irq_reload_value, value      , 0b0000_0000_0000_1111),
            0xE001 => set_bits(&mut self.irq_reload_value, value <<  4, 0b0000_0000_1111_0000),
            0xE002 => set_bits(&mut self.irq_reload_value, value <<  8, 0b0000_1111_0000_0000),
            0xE003 => set_bits(&mut self.irq_reload_value, value << 12, 0b1111_0000_0000_0000),
            0xF000 => {
                self.irq_counter = self.irq_reload_value;
                self.irq_pending = false;
            }
            0xF001 => {
                if value & 0b0000_1000 != 0 {
                    self.irq_counter_mask = 0b0000_0000_0000_1111;
                } else if value & 0b0000_0100 != 0 {
                    self.irq_counter_mask = 0b0000_0000_1111_1111;
                } else if value & 0b0000_0010 != 0 {
                    self.irq_counter_mask = 0b0000_1111_1111_1111;
                } else { // Full IRQ counter will be used.
                    self.irq_counter_mask = 0b1111_1111_1111_1111;
                }

                self.irq_enabled = value & 0b0000_0001 != 0;
                self.irq_pending = false;
            }
            0xF002 => {
                let mirroring = NAME_TABLE_MIRRORINGS[usize::from(value) & 0b11];
                params.set_name_table_mirroring(mirroring);
            }
            0xF003 => todo!("Expansion audio."),
            _ => unreachable!(),
        }
    }
}

impl Mapper018 {
    pub fn new() -> Self {
        Self {
            // TODO: Verify this power-on value.
            work_ram_write_enabled: false,

            irq_enabled: false,
            irq_pending: false,
            irq_counter: 0,
            irq_counter_mask: 0,
            irq_reload_value: 0,
        }
    }
}

fn set_bits(value: &mut u16, new_bits: u16, mask: u16) {
    *value = (*value & !mask) | (new_bits & mask);
}
