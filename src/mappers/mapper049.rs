use crate::mapper::*;
use crate::mappers::mmc3::irq_state::Mmc3IrqState;
use crate::mappers::mmc3::mmc3;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_rom_outer_bank_size(128 * KIBIBYTE)
    // Same as MMC3 layout 0 except it can't have work RAM.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    // Same as MMC3 layout 1 except it can't have work RAM.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    // Identical layouts used for BigPrgWindow Mode.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P2))
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P2))
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_rom_outer_bank_size(128 * KIBIBYTE)
    .chr_layout(mmc3::CHR_BIG_WINDOWS_FIRST)
    .chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST)
    .name_table_mirrorings(mmc3::NAME_TABLE_MIRRORINGS)
    .build();

const MODES: [Mode; 2] = [Mode::BigPrgWindow, Mode::NormalMmc3];

// Super HIK 4-in-1
// FIXME: CHR banking is partially broken for games 1 and 3, and game 0 renders black. The wiki may be incorrect.
pub struct Mapper049 {
    mmc3: mmc3::Mapper004Mmc3,
    mode: Mode,
}

impl Mapper for Mapper049 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x6000..=0x7FFF => {
                if mem.prg_memory.bank_registers().write_status(W0) == WriteStatus::Enabled {
                    let fields = splitbits!(value, "oopp ...m");
                    mem.set_chr_rom_outer_bank_number(fields.o);
                    mem.set_prg_rom_outer_bank_number(fields.o);
                    log::info!("Changing outer banks to {}", fields.o);
                    mem.set_prg_register(P2, fields.p);
                    self.mode = MODES[fields.m as usize];
                }
            }
            0x8000..=0x9FFF if !addr.is_multiple_of(2)
                    && self.mode == Mode::BigPrgWindow
                    && matches!(self.mmc3.selected_register_id(), mmc3::RegId::Prg(_)) => {
                // Do nothing, PRG bank switching for NROM mode is not delegated to MMC3.
            }
            _ => {
                self.mmc3.write_register(mem, addr, value);
            }
        }

        // The PRG layout may have changed, either through a 0x6000 mode change, or through the MMC3.
        // Either way, fix it such that the mode setting is respected.
        let old_prg_layout = mem.prg_memory.layout_index();
        let new_prg_layout = match self.mode {
            Mode::BigPrgWindow => old_prg_layout | 0b10,
            Mode::NormalMmc3 => old_prg_layout & 0b01,
        };
        mem.set_prg_layout(new_prg_layout);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper049 {
    pub fn new() -> Self {
        Self {
            mmc3: mmc3::Mapper004Mmc3::new(Mmc3IrqState::SHARP_IRQ_STATE),
            mode: Mode::BigPrgWindow,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Mode {
    BigPrgWindow,
    NormalMmc3,
}