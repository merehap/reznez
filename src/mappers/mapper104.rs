use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_rom_outer_bank_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM),
    ])
    .fixed_name_table_mirroring()
    .build();

// PEGASUS 5 IN 1 (Golden Five)
#[derive(Default)]
pub struct Mapper104 {
    lock_outer_bank: bool,
}

impl Mapper for Mapper104 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* No regs here. */ }
            0x8000..=0xBFFF => {
                let fields = splitbits!(value, ".... looo");
                if !self.lock_outer_bank {
                    bus.set_prg_rom_outer_bank_number(fields.o);
                }

                self.lock_outer_bank = fields.l;
            }
            0xC000..=0xFFFF => {
                let fields = splitbits!(value, ".... pppp");
                bus.set_prg_register(P, fields.p);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}