use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(2048 * KIBIBYTE)
    .chr_max_size(256 * KIBIBYTE)
    .override_bank_register(P1, BankIndex::SECOND)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::switchable_rom(C0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::switchable_rom(C1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::switchable_rom(C2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::switchable_rom(C3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::switchable_rom(C4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::switchable_rom(C5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::switchable_rom(C6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::switchable_rom(C7)),
    ])
    .build();


const CHR_REGISTER_IDS: [BankRegisterId; 8] = [C0, C1, C2, C3, C4, C5, C6, C7];

const MIRRORINGS: [NameTableMirroring; 4] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
    NameTableMirroring::OneScreenLeftBank,
    NameTableMirroring::OneScreenLeftBank,
];

// Irem's H3001
// FIXME: Daiku no Gen San 2 - small scanline flickering during intro.
pub struct Mapper065 {
    irq_enabled: bool,
    irq_pending: bool,
    irq_counter: u16,
    irq_reload_value: u16,
}

impl Mapper for Mapper065 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }

            0x8000 => params.set_bank_register(P0, value),
            0xA000 => params.set_bank_register(P1, value),
            0xB000..=0xB007 => {
                let reg_id = CHR_REGISTER_IDS[usize::from(cpu_address.to_raw() - 0xB000)];
                params.set_bank_register(reg_id, value);
            }
            0x9000 => params.set_prg_layout(value >> 7),
            0x9001 => params.set_name_table_mirroring(MIRRORINGS[usize::from(value >> 6)]),

            0x9003 => {
                self.irq_enabled = splitbits_named!(value, "i.......");
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

    fn layout(&self) -> Layout {
        LAYOUT
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
