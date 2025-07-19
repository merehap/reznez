use crate::mapper::*;
use crate::memory::memory::Memory;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM.fixed_index(0)),
    ])
    .build();

// VRC3
// FIXME: Status bar shouldn't scroll off the screen in Salamander.
#[derive(Default)]
pub struct Mapper073 {
    irq_enabled: bool,
    irq_enabled_on_acknowledgement: bool,
    irq_mode: IrqMode,
    irq_counter: u16,
    irq_counter_reload_value: u16,
}

impl Mapper for Mapper073 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x8FFF => {
                self.irq_counter_reload_value &= 0x000F;
                self.irq_counter_reload_value |= u16::from(value) & 0xF;
            }
            0x9000..=0x9FFF => {
                self.irq_counter_reload_value &= 0x00F0;
                self.irq_counter_reload_value |= (u16::from(value) & 0xF) << 4;
            }
            0xA000..=0xAFFF => {
                self.irq_counter_reload_value &= 0x0F00;
                self.irq_counter_reload_value |= (u16::from(value) & 0xF) << 8;
            }
            0xB000..=0xBFFF => {
                self.irq_counter_reload_value &= 0xF000;
                self.irq_counter_reload_value |= (u16::from(value) & 0xF) << 12;
            }
            0xC000..=0xCFFF => {
                params.set_irq_pending(false);

                let fields = splitbits!(value, ".....mea");
                self.irq_mode = if fields.m { IrqMode::EightBit } else { IrqMode::SixteenBit };
                self.irq_enabled = fields.e;
                self.irq_enabled_on_acknowledgement = fields.a;

                if self.irq_enabled {
                    self.irq_counter = self.irq_counter_reload_value;
                }
            }
            0xD000..=0xDFFF => {
                params.set_irq_pending(false);
                self.irq_enabled = self.irq_enabled_on_acknowledgement;
            }
            0xE000..=0xEFFF => { /* Do nothing. */ }
            0xF000..=0xFFFF => params.set_prg_register(P0, value & 0b111),
        }
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        if !self.irq_enabled {
            return;
        }

        if self.irq_mode == IrqMode::SixteenBit && self.irq_counter == 0xFFFF {
            mem.mapper_params.set_irq_pending(true);
            self.irq_counter = self.irq_counter_reload_value;
        } else if self.irq_mode == IrqMode::EightBit && self.irq_counter & 0xFF == 0xFF {
            mem.mapper_params.set_irq_pending(true);
            self.irq_counter &= 0xFF00;
            self.irq_counter |= self.irq_counter_reload_value & 0x00FF;
        } else {
            self.irq_counter += 1;
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

#[derive(PartialEq, Eq, Default)]
enum IrqMode {
    #[default]
    SixteenBit,
    EightBit,
}
