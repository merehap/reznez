use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM.fixed_index(0).status_register(S0)),
    ])
    .read_write_statuses(&[
        ReadWriteStatus::Disabled,
        ReadWriteStatus::ReadWrite,
    ])
    .build();

// Sunsoft-2 IC on the Sunsoft-3R board
pub struct Mapper093;

impl Mapper for Mapper093 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, value, ".ppp...r");
                mem.set_prg_register(P0, fields.p);
                mem.set_read_write_status(S0, fields.r);
            }
        }
    }

    fn has_bus_conflicts(&self) -> HasBusConflicts {
        HasBusConflicts::Yes
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
