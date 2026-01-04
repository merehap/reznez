use crate::mapper::*;
use crate::mappers::mmc3::mmc3;
use crate::mappers::mmc3::irq_state::Mmc3IrqState;
use crate::memory::bank::bank::ChrSourceRegisterId;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(mmc3::PRG_WINDOWS_8000_SWITCHABLE)
    .prg_layout(mmc3::PRG_WINDOWS_C000_SWITCHABLE)
    .chr_rom_max_size(256 * KIBIBYTE)
    // Same as MMC3, except each window can be dynamically toggled from ROM to RAM.
    .chr_layout(&[
        // Big windows.
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::with_switchable_source(CS0).switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::with_switchable_source(CS1).switchable(C1)),
        // Small windows.
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS2).switchable(C2)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS3).switchable(C3)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS4).switchable(C4)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS5).switchable(C5)),
    ])
    // Same as MMC3, except each window can be dynamically toggled from ROM to RAM.
    .chr_layout(&[
        // Small windows.
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS2).switchable(C2)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS3).switchable(C3)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS4).switchable(C4)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS5).switchable(C5)),
        // Big windows.
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::with_switchable_source(CS0).switchable(C0)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::with_switchable_source(CS1).switchable(C1)),
    ])
    // All the CHR banks start as ROM, and are only redirected to RAM on certain banks.
    .default_chr_source(ChrSource::Rom)
    .name_table_mirrorings(mmc3::NAME_TABLE_MIRRORINGS)
    .build();

const CHR_SOURCE_IDS: [ChrSourceRegisterId; 6] = [CS0, CS1, CS2, CS3, CS4, CS5];

// Waixing MMC3 clone with CHR RAM redirects
// TODO: Test. The only ROM file is non-NTSC.
pub struct Mapper074 {
    mmc3: mmc3::Mapper004Mmc3,
}

impl Mapper for Mapper074 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
            if matches!(*addr, 0x8000..=0x9FFF) && !addr.is_multiple_of(2) {
                match self.mmc3.selected_register_id() {
                    mmc3::RegId::Chr(cx) => {
                        let source_id = CHR_SOURCE_IDS[cx as usize];
                        if value == 8 || value == 9 {
                            bus.set_chr_source(source_id, ChrSource::WorkRam);
                            bus.set_chr_register(cx, value - 8);
                        } else {
                            bus.set_chr_source(source_id, ChrSource::Rom);
                            bus.set_chr_register(cx, value);
                        }
                    }
                    mmc3::RegId::Prg(px) => bus.set_prg_register(px, value),
                }
            } else {
                self.mmc3.write_register(bus, addr, value);
            }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper074 {
    pub fn new() -> Self {
        Self {
            mmc3: mmc3::Mapper004Mmc3::new(Mmc3IrqState::SHARP_IRQ_STATE),
        }
    }
}