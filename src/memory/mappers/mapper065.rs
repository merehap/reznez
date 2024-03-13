use crate::memory::mapper::*;

const PRG_LAYOUT_8000_FIXED: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::Empty),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);
const PRG_LAYOUT_C000_FIXED: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::Empty),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
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

const CHR_REGISTER_IDS: [BankIndexRegisterId; 8] = [C0, C1, C2, C3, C4, C5, C6, C7];

// Irem's H3001
// FIXME: Daiku no Gen San 2 - small scanline flickering during intro.
pub struct Mapper065 {
    irq_enabled: bool,
    irq_pending: bool,
    irq_counter: u16,
    irq_reload_value: u16,
}

impl Mapper for Mapper065 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(256)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT_8000_FIXED)
            .chr_max_bank_count(256)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_windows(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .override_bank_index_register(P1, BankIndex::SECOND)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }

            0x8000 => params.set_bank_index_register(P0, value),
            0xA000 => params.set_bank_index_register(P1, value),
            0xB000..=0xB007 => {
                let reg_id = CHR_REGISTER_IDS[(cpu_address.to_raw() - 0xB000) as usize];
                params.set_bank_index_register(reg_id, value);
            }
            0x9000 => {
                let prg_windows = if value & 0b1000_0000 == 0 {
                    PRG_LAYOUT_8000_FIXED
                } else {
                    PRG_LAYOUT_C000_FIXED
                };
                params.set_prg_layout(prg_windows);
            }
            0x9001 => {
                let mirroring = match value & 0b1100_0000 {
                    0b0000_0000 => NameTableMirroring::Vertical,
                    0b1000_0000 => NameTableMirroring::Horizontal,
                    0b0100_0000 | 0b1100_0000 => NameTableMirroring::OneScreenLeftBank,
                    _ => unreachable!(),
                };
                params.set_name_table_mirroring(mirroring);
            }

            0x9003 => {
                self.irq_enabled = value & 0b1000_0000 != 0;
                self.irq_pending = false;
            }
            0x9004 => {
                self.irq_counter = self.irq_reload_value;
                self.irq_pending = false;
            }
            0x9005 => {
                self.irq_reload_value &= 0x00FF;
                self.irq_reload_value |= u16::from(value) << 8;
            }
            0x9006 => {
                self.irq_reload_value &= 0xFF00;
                self.irq_reload_value |= u16::from(value);
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {
        if self.irq_enabled && self.irq_counter > 0 {
            self.irq_counter -= 1;
            if self.irq_counter == 0 {
                self.irq_pending = true;
            }
        }
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }
}

impl Mapper065 {
    pub fn new() -> Self {
        Self {
            irq_enabled: false,
            irq_pending: false,
            irq_counter: 0,
            irq_reload_value: 0,
        }
    }
}
