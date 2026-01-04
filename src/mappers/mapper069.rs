use crate::mapper::*;
use crate::memory::bank::bank::PrgSource;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P0).read_write_status(R0, W0).rom_ram_register(PS0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

const IRQ_COUNTER: DirectlySetCounter = CounterBuilder::new()
    .step(-1)
    .wraps(true)
    .full_range(0, 0xFFFF)
    .initial_count(0)
    .auto_trigger_when(AutoTriggerWhen::Wrapping)
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_directly_set_counter();

const CHR_REGISTER_IDS: [ChrBankRegisterId; 8] = [C0, C1, C2, C3, C4, C5, C6, C7];
// P0 is used by the ROM/RAM window, which gets special treatment.
const PRG_ROM_REGISTER_IDS: [PrgBankRegisterId; 3] = [P1, P2, P3];

// Sunsoft FME-7
pub struct Mapper069 {
    irq_counter: DirectlySetCounter,
    command: Command,
}

impl Mapper for Mapper069 {
    fn on_end_of_cpu_cycle(&mut self, bus: &mut Bus) {
        if self.irq_counter.tick().triggered {
            bus.cpu_pinout.assert_mapper_irq();
        }
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x9FFF => {
                let value = usize::from(value) & 0b0000_1111;
                self.command = match value {
                    0x0..=0x7 => Command::ChrRomBank(CHR_REGISTER_IDS[value]),
                    0x8       => Command::PrgRomRamBank,
                    0x9..=0xB => Command::PrgRomBank(PRG_ROM_REGISTER_IDS[value - 0x9]),
                    0xC       => Command::NameTableMirroring,
                    0xD       => Command::IrqControl,
                    0xE       => Command::IrqCounterLowByte,
                    0xF       => Command::IrqCounterHighByte,
                    _ => unreachable!(),
                }
            }
            0xA000..=0xBFFF => {
                match self.command {
                    Command::ChrRomBank(id) =>
                        bus.set_chr_register(id, value),
                    Command::PrgRomRamBank => {
                        let fields = splitbits!(value, "empppppp");
                        bus.set_reads_enabled(R0, fields.e);
                        bus.set_writes_enabled(W0, fields.e);
                        let rom_ram_mode = [PrgSource::Rom, PrgSource::WorkRamOrRom][fields.m as usize];
                        bus.set_rom_ram_mode(PS0, rom_ram_mode);
                        bus.set_prg_register(P0, fields.p);
                    }
                    Command::PrgRomBank(id) =>
                        bus.set_prg_register(id, value),
                    Command::NameTableMirroring =>
                        bus.set_name_table_mirroring(value & 0b11),
                    Command::IrqControl => {
                        bus.cpu_pinout.acknowledge_mapper_irq();
                        let (counter_counting_enabled, irq_triggering_enabled) = splitbits_named!(value, "c......i");
                        self.irq_counter.set_counting_enabled(counter_counting_enabled);
                        self.irq_counter.set_triggering_enabled(irq_triggering_enabled);
                    }
                    Command::IrqCounterLowByte =>
                        self.irq_counter.set_count_low_byte(value),
                    Command::IrqCounterHighByte =>
                        self.irq_counter.set_count_high_byte(value),
                }
            }
            0xC000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(self.irq_counter.to_irq_counter_info())
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper069 {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER,
            // TODO: Verify that this is the correct startup value.
            command: Command::ChrRomBank(C0),
        }
    }
}

enum Command {
    ChrRomBank(ChrBankRegisterId),
    PrgRomRamBank,
    PrgRomBank(PrgBankRegisterId),
    NameTableMirroring,
    IrqControl,
    IrqCounterLowByte,
    IrqCounterHighByte,
}