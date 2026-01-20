use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::new(
            CiramSide::Left.to_source(), CiramSide::Left.to_source(),
            CiramSide::Left.to_source(), CiramSide::Right.to_source(),
        ),
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// Sachen SA-020A
// FIXME: The wiki documentation is incorrect on this, leading to a garbled title page. Research the correct behavior.
#[derive(Default)]
pub struct Mapper243 {
    reg_number: usize,
    regs: [u8; 8],
}

impl Mapper for Mapper243 {
    fn peek_register(&self, _bus: &Bus, addr: CpuAddress) -> ReadResult {
        if *addr & 0xC101 == 0x4101 {
            ReadResult::partial(self.regs[self.reg_number], 0b0000_0111)
        } else {
            ReadResult::OPEN_BUS
        }
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        let value = value & 0b111;
        match *addr & 0xC101 {
            0x4100 => {
                self.reg_number = value.into();
            }
            0x4101 => {
                self.regs[self.reg_number] = value;

                match self.reg_number {
                    0 | 1 | 3 => { /* These regs store values, but do nothing else. */ }
                    2 => bus.set_chr_bank_register_bits(C0, u16::from(value)     , 0b0000_0001),
                    4 => bus.set_chr_bank_register_bits(C0, u16::from(value << 1), 0b0000_0010),
                    6 => bus.set_chr_bank_register_bits(C0, u16::from(value << 2), 0b0000_1100),
                    5 => bus.set_prg_register(P0, value & 0b11),
                    7 => bus.set_name_table_mirroring(value >> 1),
                    _ => unreachable!(),
                }
            }
            _ => { /* No registers here. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    } 
}