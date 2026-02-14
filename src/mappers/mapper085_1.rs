use crate::mapper::*;
use crate::mappers::vrc::vrc_irq_state::VrcIrqState;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.write_status(WS0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(Q)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(R)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    // TODO: Support CHR ROM and RAM
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(D)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(E)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(F)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(G)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(H)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(I)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(J)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// Konami VRC7b
#[derive(Default)]
pub struct Mapper085_1 {
    irq_state: VrcIrqState,
}

impl Mapper for Mapper085_1 {
    fn on_end_of_cpu_cycle(&mut self, bus: &mut Bus) {
        self.irq_state.step(bus);
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x8000 => bus.set_prg_register(P, value & 0b0011_1111),
            0x8008 => bus.set_prg_register(Q, value & 0b0011_1111),
            0x9000 => bus.set_prg_register(R, value & 0b0011_1111),
            0xA000 => bus.set_chr_register(C, value),
            0xA008 => bus.set_chr_register(D, value),
            0xB000 => bus.set_chr_register(E, value),
            0xB008 => bus.set_chr_register(F, value),
            0xC000 => bus.set_chr_register(G, value),
            0xC008 => bus.set_chr_register(H, value),
            0xD000 => bus.set_chr_register(I, value),
            0xD008 => bus.set_chr_register(J, value),
            0xE000 => {
                let fields = splitbits!(value, "w.....mm");
                bus.set_writes_enabled(WS0, fields.w);
                bus.set_name_table_mirroring(fields.m);
            }
            0xE008 => self.irq_state.set_reload_value(value),
            0xF000 => self.irq_state.set_mode(bus, value),
            0xF008 => self.irq_state.acknowledge(bus),

            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
