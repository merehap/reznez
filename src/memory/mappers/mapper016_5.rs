use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::Empty),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C0)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C1)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C2)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C3)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C4)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C5)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C6)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C7)),
]);

// LZ93D50 ASIC
pub struct Mapper016_5 {
    irq_pending: bool,
    irq_counter_enabled: bool,
    irq_counter_latch: u16,
    irq_counter: u16,
}

impl Mapper for Mapper016_5 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(16)
            .prg_bank_size(16 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT)
            .chr_max_bank_count(256)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_windows(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() & 0x800F {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000 => params.set_bank_register(C0, value),
            0x8001 => params.set_bank_register(C1, value),
            0x8002 => params.set_bank_register(C2, value),
            0x8003 => params.set_bank_register(C3, value),
            0x8004 => params.set_bank_register(C4, value),
            0x8005 => params.set_bank_register(C5, value),
            0x8006 => params.set_bank_register(C6, value),
            0x8007 => params.set_bank_register(C7, value),
            0x8008 => params.set_bank_register(P0, value & 0b1111),
            0x8009 => {
                let mirroring = match value & 0b11 {
                    0 => NameTableMirroring::Vertical,
                    1 => NameTableMirroring::Horizontal,
                    2 => NameTableMirroring::OneScreenLeftBank,
                    3 => NameTableMirroring::OneScreenRightBank,
                    _ => unreachable!(),
                };
                params.set_name_table_mirroring(mirroring);
            }
            0x800A => {
                self.irq_pending = false;
                self.irq_counter_enabled = value & 1 == 1;
                self.irq_counter = self.irq_counter_latch;
                if self.irq_counter_enabled && self.irq_counter == 0 {
                    self.irq_pending = true;
                }
            }
            0x800B => {
                // Set the low byte.
                self.irq_counter_latch &= 0b1111_1111_0000_0000;
                self.irq_counter_latch |= u16::from(value);
            }
            0x800C => {
                // Set the high byte.
                self.irq_counter_latch &= 0b0000_0000_1111_1111;
                self.irq_counter_latch |= u16::from(value) << 8;
            }
            0x800D => todo!("Submapper 5 EEPROM Control."),
            0x800E..=0x9FFF => { /* Do nothing. */ }
            0xA000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {
        if self.irq_counter_enabled && self.irq_counter > 0 {
            self.irq_counter -= 1;
            if self.irq_counter == 0 {
                self.irq_pending = true;
            }
        }
    }
}

impl Mapper016_5 {
    pub fn new() -> Self {
        Self {
            irq_pending: false,
            irq_counter_enabled: false,
            irq_counter_latch: 0,
            irq_counter: 0,
        }
    }
}
