use crate::memory::mapper::*;
use crate::memory::mappers::vrc::vrc_irq_state::VrcIrqState;

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
    irq_state: VrcIrqState,
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
        self.irq_state.step();
    }

    fn irq_pending(&self) -> bool {
        self.irq_state.pending()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let low_bank = u16::from(value);
        let high_bank = u16::from(value) << 4;

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

            0xB000 => params.chr_memory_mut().set_bank_index_register_bits(C0, low_bank, 0b0_0000_1111),
            0xB001 | 0xB004 => params.chr_memory_mut().set_bank_index_register_bits(C0, high_bank, 0b1_1111_0000),
            0xB002 | 0xB008 => params.chr_memory_mut().set_bank_index_register_bits(C1, low_bank, 0b0_0000_1111),
            0xB003 | 0xB00C => params.chr_memory_mut().set_bank_index_register_bits(C1, high_bank, 0b1_1111_0000),
            0xC000 => params.chr_memory_mut().set_bank_index_register_bits(C2, low_bank, 0b0_0000_1111),
            0xC001 | 0xC004 => params.chr_memory_mut().set_bank_index_register_bits(C2, high_bank, 0b1_1111_0000),
            0xC002 | 0xC008 => params.chr_memory_mut().set_bank_index_register_bits(C3, low_bank, 0b0_0000_1111),
            0xC003 | 0xC00C => params.chr_memory_mut().set_bank_index_register_bits(C3, high_bank, 0b1_1111_0000),
            0xD000 => params.chr_memory_mut().set_bank_index_register_bits(C4, low_bank, 0b0_0000_1111),
            0xD001 | 0xD004 => params.chr_memory_mut().set_bank_index_register_bits(C4, high_bank, 0b1_1111_0000),
            0xD002 | 0xD008 => params.chr_memory_mut().set_bank_index_register_bits(C5, low_bank, 0b0_0000_1111),
            0xD003 | 0xD00C => params.chr_memory_mut().set_bank_index_register_bits(C5, high_bank, 0b1_1111_0000),
            0xE000 => params.chr_memory_mut().set_bank_index_register_bits(C6, low_bank, 0b0_0000_1111),
            0xE001 | 0xE004 => params.chr_memory_mut().set_bank_index_register_bits(C6, high_bank, 0b1_1111_0000),
            0xE002 | 0xE008 => params.chr_memory_mut().set_bank_index_register_bits(C7, low_bank, 0b0_0000_1111),
            0xE003 | 0xE00C => params.chr_memory_mut().set_bank_index_register_bits(C7, high_bank, 0b1_1111_0000),

            0xF000 => self.irq_state.set_reload_value_low_bits(value),
            0xF001 => self.irq_state.set_reload_value_high_bits(value),
            0xF002 => self.irq_state.set_mode(value),
            0xF003 => self.irq_state.acknowledge(),
            0x4020..=0xFFFF => { /* All other writes do nothing. */ }
        }
    }
}

impl Mapper023 {
    pub fn new() -> Self {
        Self {
            irq_state: VrcIrqState::new(),
        }
    }
}
