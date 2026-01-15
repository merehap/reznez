use crate::mapper::*;
use crate::mappers::mmc3::irq_state::Mmc3IrqState;
use crate::mappers::mmc3::mmc3;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    // Same PRG layouts as MMC3, except no RAM allowed.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-2)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(mmc3::CHR_BIG_WINDOWS_FIRST)
    .chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST)
    .name_table_mirrorings(mmc3::NAME_TABLE_MIRRORINGS)
    .build();

pub struct Mapper047 {
    mmc3: mmc3::Mapper004Mmc3,
}

impl Mapper for Mapper047 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        if matches!(*addr, 0x6000..=0x7FFF) {
            let regs = bus.prg_memory().bank_registers();
            // S0 isn't hooked up to any window, but its value is still set by MMC3 and used for this mapper.
            if regs.read_status(R0) == ReadStatus::Enabled && regs.write_status(W0) == WriteStatus::Enabled {
                let index = value & 1;
                bus.set_prg_rom_outer_bank_number(index);
                bus.set_chr_rom_outer_bank_number(index);
            }
        } else {
            self.mmc3.write_register(bus, addr, value);
        }
    }

    fn on_end_of_ppu_cycle(&mut self) {
        self.mmc3.on_end_of_ppu_cycle();
    }

    fn on_ppu_address_change(&mut self, bus: &mut Bus, address: PpuAddress) {
        self.mmc3.on_ppu_address_change(bus, address);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper047 {
    pub fn new() -> Self {
        Mapper047 {
            mmc3: mmc3::Mapper004Mmc3::new(Mmc3IrqState::SHARP_IRQ_STATE),
        }
    }
}