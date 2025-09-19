use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_rom_outer_bank_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(1024 * KIBIBYTE)
    .chr_rom_outer_bank_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
    ])
    .build();

const IRQ_COUNTER_RELOAD_VALUE: u8 = 64;

// J.Y. Company JY830623C and YY840238C
#[derive(Default)]
pub struct Mapper091_0 {
    irq_enabled: bool,
    irq_counter: u8,
    pattern_table_side: PatternTableSide,
}

impl Mapper for Mapper091_0 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr & 0xF003 {
            0x6000 => mem.set_chr_register(C0, value),
            0x6001 => mem.set_chr_register(C1, value),
            0x6002 => mem.set_chr_register(C2, value),
            0x6003 => mem.set_chr_register(C3, value),
            0x7000 => mem.set_prg_register(P0, value & 0b00001111),
            0x7001 => mem.set_prg_register(P1, value & 0b00001111),
            0x7002 => {
                self.irq_enabled = false;
                mem.cpu_pinout.clear_mapper_irq_pending();
            }
            0x7003 => {
                self.irq_enabled = true;
                self.irq_counter = IRQ_COUNTER_RELOAD_VALUE;
            }
            0x8000..=0x9FFF => {
                let outer_banks = splitbits!(min=u8, *addr, ".... .... .... .pcc");
                mem.set_prg_rom_outer_bank_index(outer_banks.p);
                mem.set_prg_rom_outer_bank_index(outer_banks.c);
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_ppu_address_change(&mut self, mem: &mut Memory, address: PpuAddress) {
        let next_side = address.pattern_table_side();
        let should_tick_irq_counter = self.irq_enabled
            && self.pattern_table_side == PatternTableSide::Left
            && next_side == PatternTableSide::Right;
        self.pattern_table_side = next_side;

        if should_tick_irq_counter {
            self.irq_counter = self.irq_counter
                .checked_sub(1)
                .unwrap_or(IRQ_COUNTER_RELOAD_VALUE);

            if self.irq_counter == 0 {
                mem.cpu_pinout.set_mapper_irq_pending();
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
