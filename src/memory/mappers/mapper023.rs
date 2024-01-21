use crate::memory::mapper::*;

const PRG_WINDOWS_SWITCHABLE_8000: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);

const PRG_WINDOWS_SWITCHABLE_C000: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);

const CHR_WINDOWS: ChrWindows = ChrWindows::new(&[
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

// VRC2b and VRC4a and VRC4c
pub struct Mapper023 {
    chr_bank_lows: [u8; 8],
    chr_bank_highs: [u8; 8],

    irq_enabled: bool,
    irq_pending: bool,
    enable_irq_upon_acknowledgement: bool,
    irq_mode: IrqMode,
    irq_counter_reload_low_value: u8,
    irq_counter_reload_value: u8,
    irq_counter: u8,
}

impl Mapper for Mapper023 {
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
        if self.irq_enabled {
            if self.irq_counter == 0xFF {
                self.irq_pending = true;
                self.irq_counter = self.irq_counter_reload_value;
            } else {
                self.irq_counter += 1;
            }
        }
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            // Set bank for 8000 through 9FFF (or C000 through DFFF, VRC4 only).
            0x8000..=0x8003 => params.prg_memory_mut().set_bank_index_register(P0, value & 0b0001_1111),
            0x9000 =>
                params.set_name_table_mirroring(NAME_TABLE_MIRRORINGS[usize::from(value & 0b0000_0011)]),
            // VRC4-only
            0x9002 => {
                if value & 0b0000_0001 == 0 {
                    params.prg_memory_mut().disable_work_ram(0x6000);
                } else {
                    params.prg_memory_mut().enable_work_ram(0x6000);
                }

                let prg_layout = if value & 0b0000_0010 == 0 {
                    PRG_WINDOWS_SWITCHABLE_8000
                } else {
                    PRG_WINDOWS_SWITCHABLE_C000
                };
                params.prg_memory_mut().set_windows(prg_layout);
            }
            // Set bank for A000 through AFFF.
            0xA000..=0xA003 => params.prg_memory_mut().set_bank_index_register(P1, value & 0b0001_1111),

            0xB000 => self.chr_bank_lows[0] = value,
            0xB001 | 0xB004 => self.chr_bank_highs[0] = value,
            0xB002 | 0xB008 => self.chr_bank_lows[1] = value,
            0xB003 | 0xB00C => self.chr_bank_highs[1] = value,
            0xC000 => self.chr_bank_lows[2] = value,
            0xC001 | 0xC004 => self.chr_bank_highs[2] = value,
            0xC002 | 0xC008 => self.chr_bank_lows[3] = value,
            0xC003 | 0xC00C => self.chr_bank_highs[3] = value,
            0xD000 => self.chr_bank_lows[4] = value,
            0xD001 | 0xD004 => self.chr_bank_highs[4] = value,
            0xD002 | 0xD008 => self.chr_bank_lows[5] = value,
            0xD003 | 0xD00C => self.chr_bank_highs[5] = value,
            0xE000 => self.chr_bank_lows[6] = value,
            0xE001 | 0xE004 => self.chr_bank_highs[6] = value,
            0xE002 | 0xE008 => self.chr_bank_lows[7] = value,
            0xE003 | 0xE00C => self.chr_bank_highs[7] = value,

            0xF000 => self.irq_counter_reload_low_value = value & 0b0000_1111,
            0xF001 => self.irq_counter_reload_value = (value & 0b0000_1111) << 4 | self.irq_counter_reload_low_value,
            // IRQ mode.
            0xF002 => {
                self.irq_pending = false;
                self.irq_enabled = value & 0b0000_0001 != 0;
                if self.irq_enabled {
                    self.irq_counter = self.irq_counter_reload_value;
                }
                self.enable_irq_upon_acknowledgement = value & 0b0000_0010 != 0;
                self.irq_mode = if value & 0b0000_00100 == 0 { IrqMode::Scanline } else { IrqMode::Cycle };
            }
            // IRQ acknowledge.
            0xF003 => {
                self.irq_pending = false;
                if self.enable_irq_upon_acknowledgement {
                    self.irq_enabled = true;
                }
            }

            0x4020..=0xFFFF => { /* All other writes do nothing. */ }
        }

        // TODO: Get rid of temporary storage of lows and highs once merged with desktop
        // changes.
        params.chr_memory_mut().set_bank_index_register(C0, bank_index(self.chr_bank_lows[0], self.chr_bank_highs[0]));
        params.chr_memory_mut().set_bank_index_register(C1, bank_index(self.chr_bank_lows[1], self.chr_bank_highs[1]));
        params.chr_memory_mut().set_bank_index_register(C2, bank_index(self.chr_bank_lows[2], self.chr_bank_highs[2]));
        params.chr_memory_mut().set_bank_index_register(C3, bank_index(self.chr_bank_lows[3], self.chr_bank_highs[3]));
        params.chr_memory_mut().set_bank_index_register(C4, bank_index(self.chr_bank_lows[4], self.chr_bank_highs[4]));
        params.chr_memory_mut().set_bank_index_register(C5, bank_index(self.chr_bank_lows[5], self.chr_bank_highs[5]));
        params.chr_memory_mut().set_bank_index_register(C6, bank_index(self.chr_bank_lows[6], self.chr_bank_highs[6]));
        params.chr_memory_mut().set_bank_index_register(C7, bank_index(self.chr_bank_lows[7], self.chr_bank_highs[7]));
    }
}

fn bank_index(low: u8, high: u8) -> u16 {
    (u16::from(high & 0b0001_1111) << 4) | u16::from(low & 0b0000_1111)
}

#[derive(Debug)]
enum IrqMode {
    Scanline,
    Cycle,
}

impl Mapper023 {
    pub fn new() -> Self {
        Self {
            chr_bank_lows: [0; 8],
            chr_bank_highs: [0; 8],
            irq_enabled: false,
            irq_pending: false,
            enable_irq_upon_acknowledgement: false,
            irq_mode: IrqMode::Scanline,
            irq_counter_reload_low_value: 0,
            irq_counter_reload_value: 0,
            irq_counter: 0,
        }
    }
}
