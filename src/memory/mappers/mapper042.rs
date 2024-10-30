use ux::u15;

use crate::memory::mapper::*;

// Used by most games.
const LAYOUT_WITH_SWITCHABLE_CHR_ROM: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .prg_layout(PrgLayout::new(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::switchable_rom(P0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
    ]))
    .chr_max_size(128 * KIBIBYTE)
    .chr_layout(ChrLayout::new(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C0)),
    ]))
    .build();

// Used by Bio Miracle Bokutte Upa, for example.
const LAYOUT_WITH_FIXED_CHR_RAM: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .prg_layout(PrgLayout::new(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::switchable_rom(P0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
    ]))
    .chr_max_size(128 * KIBIBYTE)
    .chr_layout(ChrLayout::new(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::fixed_ram(BankIndex::FIRST)),
    ]))
    .build();

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

// FDS games hacked into cartridge form.
// HACK: normally there should only be one Layout per mapper, but representing the option of having
// either switchable ROM or fixed RAM in the same layout seems excessive.
// Unknown if subject to bus conflicts.
// FIXME: Bottom status bar scrolls when it should be stationary in Bio Miracle Bokutte Upa.
pub struct Mapper042 {
    layout: Layout,
    irq_enabled: bool,
    irq_counter: u15,
}

impl Mapper for Mapper042 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() & 0xE003 {
            0x8000 => params.set_bank_register(C0, value & 0b1111),
            0xE000 => params.set_bank_register(P0, value & 0b1111),
            0xE001 => {
                let mirroring = splitbits_named!(value, "....m...");
                params.set_name_table_mirroring(MIRRORINGS[mirroring as usize]);
            }
            0xE002 => {
                self.irq_enabled = splitbits_named!(value, "......e.");
                if !self.irq_enabled {
                    self.irq_counter = 0.into();
                }
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {
        if self.irq_enabled {
            self.irq_counter = self.irq_counter.wrapping_add(1.into());
        }
    }

    fn irq_pending(&self) -> bool {
        // 0x6000 == 24576
        u16::from(self.irq_counter) >= 0x6000u16
    }

    fn layout(&self) -> Layout {
        self.layout.clone()
    }
}

impl Mapper042 {
    pub fn new(chr_ram_size: u32) -> Mapper042 {
        const CHR_RAM_SIZE: u32 = 8 * KIBIBYTE;

        let layout = match chr_ram_size {
            0 => LAYOUT_WITH_SWITCHABLE_CHR_ROM,
            CHR_RAM_SIZE => LAYOUT_WITH_FIXED_CHR_RAM,
            _ => panic!("Invalid CHR RAM size for mapper 42: {chr_ram_size} bytes."),
        };

        Mapper042 {
            layout,
            irq_enabled: false,
            irq_counter: 0.into(),
        }
    }
}
