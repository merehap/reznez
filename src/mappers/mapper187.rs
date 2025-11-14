use crate::mapper::*;
use crate::mappers::mmc3::mmc3;
use crate::mappers::mmc3::irq_state::Mmc3IrqState;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(mmc3::PRG_WINDOWS_8000_SWITCHABLE)
    .prg_layout(mmc3::PRG_WINDOWS_C000_SWITCHABLE)
    // NROM-128
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM.read_write_status(R0, W0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
    ])
    // NROM-256
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM.read_write_status(R0, W0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(mmc3:: CHR_BIG_WINDOWS_FIRST)
    .chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST)
    .name_table_mirrorings(mmc3::NAME_TABLE_MIRRORINGS)
    .build();

pub struct Mapper187 {
    mmc3: mmc3::Mapper004Mmc3,
    mmc3_prg_layout_index: u8,
    prg_layout_mode: PrgLayoutMode,
}

impl Mapper for Mapper187 {
    fn peek_register(&self, _mem: &Memory, addr: CpuAddress) -> ReadResult {
        match *addr {
            // "The actual values that are returned are unknown;
            //  The King of Fighters '96 reads from here and only expects bit 7 of the value being returned to be set."
            // TODO: Research. Mesen might implement this properly.
            0x5000..=0x5FFF => ReadResult::full(0b1000_0000),
            _ => ReadResult::OPEN_BUS
        }
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        if *addr & 0xF001 == 0x5000 {
            if self.prg_layout_mode == PrgLayoutMode::Mmc3 {
                // Remember what MMC3 layout was last used so we can switch back to it once NROM mode is over.
                self.mmc3_prg_layout_index = mem.prg_memory.layout_index();
            }

            let fields = splitbits!(value, "n.mp pppp");
            self.prg_layout_mode = if fields.n { PrgLayoutMode::Nrom } else { PrgLayoutMode::Mmc3 };
            let prg_layout_index = match self.prg_layout_mode {
                PrgLayoutMode::Mmc3 => self.mmc3_prg_layout_index,
                PrgLayoutMode::Nrom => fields.m as u8 + 2, // 2 is NROM128, 3 is NROM256
            };
            mem.set_prg_layout(prg_layout_index);
            mem.set_prg_register(P2, fields.p); // Bottom bit is ignored for NROM128, bottom two for NROM256

            return;
        }

        // For all other registers, use normal MMC3 behavior except for PRG layout selection.
        let prev_prg_layout_index = mem.prg_memory.layout_index();
        self.mmc3.write_register(mem, addr, value);
        if self.prg_layout_mode == PrgLayoutMode::Nrom {
            // Ignore/overwrite whatever layout MMC3 just set since we're not in MMC3 PRG layout mode.
            mem.set_prg_layout(prev_prg_layout_index);
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper187 {
    pub fn new() -> Self {
        Self {
            // Sharp IRQs assumed, but not verified.
            mmc3: mmc3::Mapper004Mmc3::new(Mmc3IrqState::SHARP_IRQ_STATE),
            mmc3_prg_layout_index: 0,
            prg_layout_mode: PrgLayoutMode::Mmc3,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PrgLayoutMode {
    Mmc3,
    Nrom,
}