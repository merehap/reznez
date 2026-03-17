use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(1024 * KIBIBYTE)
    .override_prg_bank_register(W, -1)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x8FFF, 4 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0x9000, 0x9FFF, 4 * KIBIBYTE, PrgBank::ROM.switchable(Q)),
        PrgWindow::new(0xA000, 0xAFFF, 4 * KIBIBYTE, PrgBank::ROM.switchable(R)),
        PrgWindow::new(0xB000, 0xBFFF, 4 * KIBIBYTE, PrgBank::ROM.switchable(S)),
        PrgWindow::new(0xC000, 0xCFFF, 4 * KIBIBYTE, PrgBank::ROM.switchable(T)),
        PrgWindow::new(0xD000, 0xDFFF, 4 * KIBIBYTE, PrgBank::ROM.switchable(U)),
        PrgWindow::new(0xE000, 0xEFFF, 4 * KIBIBYTE, PrgBank::ROM.switchable(V)),
        PrgWindow::new(0xF000, 0xFFFF, 4 * KIBIBYTE, PrgBank::ROM.switchable(W)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM)
    ])
    .fixed_name_table_mirroring()
    .build();

// NSF Music Compilations
pub struct Mapper031;

impl Mapper for Mapper031 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x4FFF | 0x6000..=0xFFFF => { /* No regs here. */ }
            0x5000..=0x5FFF => {
                let reg_id = [P, Q, R, S, T, U, V, W][usize::from(*addr) & 0b111];
                bus.set_prg_register(reg_id, value);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}