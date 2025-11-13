use crate::mapper::*;
use crate::mappers::common::cony::Cony;

// Identical to the submapper 0 layout, except with 2KiB CHR banks and twice as much CHR ROM.
const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P3).read_status(R0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P3).read_status(R0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P3).read_status(R0)),
        PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF,  8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    // Same as above.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P3).read_status(R0)),
        PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF,  8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// Cony with 2 KiB CHR-ROM banking, no PRG work ram (ROM at 0x6000 instead), and with no outer banks.
pub struct Mapper083_1 {
    cony: Cony,
}

impl Mapper for Mapper083_1 {
    fn peek_register(&self, mem: &Memory, addr: CpuAddress) -> ReadResult {
        self.cony.peek_register(mem, addr)
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        if *addr & 0x8300 == 0x8100 {
            // The rest of the flags for this register are handled in Cony.
            let fields = splitbits!(value, "..r.....");
            mem.set_reads_enabled(R0, fields.r);
        } else if *addr & 0x8313 == 0x8303 {
            // P0, P1, P2, and P4 are handled in Cony.
            mem.set_prg_register(P3, value);
        } else if matches!(*addr & 0x831F, 0x8310..=0x8317) {
            // Different CHR setup from submappers 0 and 2.
            match *addr & 0x831F {
                0x8310 => mem.set_chr_register(C0, value),
                0x8311 => mem.set_chr_register(C1, value),
                0x8316 => mem.set_chr_register(C2, value),
                0x8317 => mem.set_chr_register(C3, value),
                _ => { /* Do nothing. */ }
            }
        }

        self.cony.write_register(mem, addr, value);
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        self.cony.on_end_of_cpu_cycle(mem);
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(self.cony.irq_counter_info())
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper083_1 {
    pub fn new() -> Self {
        Self { cony: Cony::new() }
    }
}