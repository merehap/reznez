use crate::cartridge::resolved_metadata::ResolvedMetadata;
use crate::mapper::*;
use crate::bus::Bus;

const LAYOUT_WITH_SWITCHABLE_CHR_ROM: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

const LAYOUT_WITH_FIXED_CHR_RAM: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM.fixed_index(0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

const IRQ_COUNTER: ReloadDrivenCounter = CounterBuilder::new()
    .step(1)
    .wraps(true)
    .full_range(0, 0x7FFF)
    .initial_count(0)
    .auto_trigger_when(AutoTriggerWhen::EndingOn(0x6000))
    // Todo: Verify timing.
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_reload_driven_counter();

pub const MAPPER042_WITH_SWITCHABLE_CHR_ROM: Mapper042 = Mapper042 {
    layout: LAYOUT_WITH_SWITCHABLE_CHR_ROM,
    irq_counter: IRQ_COUNTER,
};

pub const MAPPER042_WITH_FIXED_CHR_RAM: Mapper042 = Mapper042 {
    layout: LAYOUT_WITH_FIXED_CHR_RAM,
    irq_counter: IRQ_COUNTER,
};

// FDS games hacked into cartridge form.
// Unknown if subject to bus conflicts.
// FIXME: Pixel flickering during first level. Need joypad input to capture test frame.
pub struct Mapper042 {
    layout: Layout,
    irq_counter: ReloadDrivenCounter,
}

impl Mapper for Mapper042 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr & 0xE003 {
            0x8000 => bus.set_chr_register(C0, value & 0b1111),
            0xE000 => bus.set_prg_register(P0, value & 0b1111),
            0xE001 => {
                let mirroring = splitbits_named!(value, "....m...");
                bus.set_name_table_mirroring(mirroring as u8);
            }
            0xE002 => {
                if value & 0b0000_0010 == 0 {
                    self.irq_counter.disable();
                    self.irq_counter.force_reload();
                    bus.cpu_pinout.acknowledge_mapper_irq();
                } else {
                    self.irq_counter.enable();
                }
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, bus: &mut Bus) {
        let tick_result = self.irq_counter.tick();
        if tick_result.triggered {
            bus.cpu_pinout.assert_mapper_irq();
        }

        if tick_result.wrapped {
            bus.cpu_pinout.acknowledge_mapper_irq();
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(self.irq_counter.to_irq_counter_info())
    }

    fn layout(&self) -> Layout {
        self.layout.clone()
    }
}

pub fn chr_board(metadata: &ResolvedMetadata) -> ChrBoard {
    const CHR_RAM_SIZE: u32 = 8 * KIBIBYTE;

    match metadata.chr_work_ram_size {
        0 => ChrBoard::SwitchableRom,
        CHR_RAM_SIZE => ChrBoard::FixedRam,
        _ => panic!("Bad CHR RAM size for mapper 42: {}", metadata.chr_work_ram_size),
    }
}

#[derive(Clone, Copy)]
pub enum ChrBoard {
    // Ai Senshi Nicol, for example
    SwitchableRom,
    // Bio Miracle Bokutte Upa, for example.
    FixedRam,
}