use crate::cartridge::resolved_metadata::ResolvedMetadata;
use crate::mapper::*;
use crate::memory::memory::Memory;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM.fixed_index(0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

const IRQ_COUNTER: IncrementingCounter = IncrementingCounterBuilder::new()
    .auto_triggered_by(IncAutoTriggeredBy::EndingOnTarget)
    .trigger_target(0x6000)
    .when_target_reached(WhenTargetReached::ContinueThenClearAfter(0x7FFF))
    .when_disabled_prevent(WhenDisabledPrevent::TickingAndTriggering)
    .build();

// FDS games hacked into cartridge form.
// Unknown if subject to bus conflicts.
// FIXME: Bottom status bar scrolls when it should be stationary in Bio Miracle Bokutte Upa.
pub struct Mapper042 {
    chr_board: ChrBoard,
    irq_counter: IncrementingCounter,
}

impl Mapper for Mapper042 {
    fn init_mapper_params(&self, mem: &mut Memory) {
        mem.set_chr_layout(self.chr_board as u8);
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr & 0xE003 {
            0x8000 => mem.set_chr_register(C0, value & 0b1111),
            0xE000 => mem.set_prg_register(P0, value & 0b1111),
            0xE001 => {
                let mirroring = splitbits_named!(value, "....m...");
                mem.set_name_table_mirroring(mirroring as u8);
            }
            0xE002 => {
                if value & 0b0000_0010 == 0 {
                    self.irq_counter.disable();
                    self.irq_counter.clear();
                    mem.cpu_pinout.acknowledge_mapper_irq();
                } else {
                    self.irq_counter.enable();
                }
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        let triggered = self.irq_counter.tick();
        if triggered {
            mem.cpu_pinout.generate_mapper_irq();
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(self.irq_counter.to_irq_counter_info())
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper042 {
    pub fn new(metadata: &ResolvedMetadata) -> Self {
        const CHR_RAM_SIZE: u32 = 8 * KIBIBYTE;

        let chr_board = match metadata.chr_work_ram_size {
            0 => ChrBoard::SwitchableRom,
            CHR_RAM_SIZE => ChrBoard::FixedRam,
            _ => panic!("Bad CHR RAM size for mapper 42: {}", metadata.chr_work_ram_size),
        };

        Self {
            chr_board,
            irq_counter: IRQ_COUNTER,
        }
    }
}

#[derive(Clone, Copy)]
enum ChrBoard {
    // Ai Senshi Nicol, for example
    SwitchableRom = 0,
    // Bio Miracle Bokutte Upa, for example.
    FixedRam = 1,
}
