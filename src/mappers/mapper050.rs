use crate::mapper::*;
use crate::memory::memory::Memory;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(0xF)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(0x8)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(0x9)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(0xB)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.fixed_index(0)),
    ])
    .build();

// N-32 conversion of Super Mario Bros. 2 (J). PCB code 761214.
#[derive(Default)]
pub struct Mapper050 {
    irq_enabled: bool,
    irq_counter: u16,
}

impl Mapper for Mapper050 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr & 0x4120 {
            0x4020 => {
                //println!("Setting PRG bank. Value: {value:b} . Address: 0x{cpu_address:04X}");
                let prg_bank = splitbits_then_combine!(value, "....hllm",
                                                              "0000hmll");

                let prg_bank2 = (value & 0x08) | ((value & 0x01) << 2) | ((value & 0x06) >> 1);
                assert_eq!(prg_bank, prg_bank2);

                //println!("\tActual value : {prg_bank:b}");
                mem.set_prg_register(P0, prg_bank);
            }
            0x4120 => {
                //println!("Setting IRQ. Value: {value}");
                self.irq_enabled = value & 1 == 1;
                if !self.irq_enabled {
                    self.irq_counter = 0;
                    mem.cpu_pinout.clear_mapper_irq_pending();
                }
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        if !self.irq_enabled {
            return;
        }

        self.irq_counter = self.irq_counter.wrapping_add(1);
        if self.irq_counter == 0x1000 {
            mem.cpu_pinout.set_mapper_irq_pending();
            self.irq_enabled = false;
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(IrqCounterInfo { ticking_enabled: self.irq_enabled, triggering_enabled: self.irq_enabled, count: self.irq_counter })
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
