use ux::u3;

use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(64 * KIBIBYTE)
    .chr_rom_outer_bank_size(32 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM),
    ])
    .name_table_mirrorings(&[
        // L L
        // L R
        NameTableMirroring::new(
            NameTableSource::Ciram(CiramSide::Left),
            NameTableSource::Ciram(CiramSide::Left),
            NameTableSource::Ciram(CiramSide::Left),
            NameTableSource::Ciram(CiramSide::Right),
        ),
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
    ])
    .build();

// Sachen SA-015 and SA-630
// Uses the dip switch to modify the CPU data bus behavior on register reads and writes.
#[derive(Default)]
pub struct Mapper150 {
    selected_reg_type: RegisterType,
    regs: [u3; 8],
}

impl Mapper for Mapper150 {
    fn peek_cartridge_space(&self, mem: &Memory, addr: CpuAddress) -> ReadResult {
        if *addr & 0xC101 == 0x4101 {
            let reg_value: u8 = self.regs[self.selected_reg_type as usize].into() ;
            let present_bits_mask = if mem.dip_switch & 1 == 1 { 0b0000_0011 } else { 0b0000_0111 };
            return ReadResult::partial_open_bus(reg_value, present_bits_mask);
        }

        if *addr < 0x6000 {
            ReadResult::OPEN_BUS
        } else {
            mem.peek_prg(addr)
        }
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, mut value: u8) {
        if mem.dip_switch & 1 == 1 {
            value |= 0b1000;
        }

        match *addr & 0xC101 {
            0x4100 => self.selected_reg_type = match value & 0b11 {
                0 => RegisterType::Dummy0,
                1 => RegisterType::Dummy1,
                2 => RegisterType::Dummy2,
                3 => RegisterType::Dummy3,
                4 => RegisterType::ChrOuterBank,
                5 => RegisterType::PrgBank,
                6 => RegisterType::ChrInnerBank,
                7 => RegisterType::NameTableMirroring,
                _ => unreachable!(),
            },
            0x4101 => {
                self.regs[self.selected_reg_type as usize] = u3::new(value & 0b111);
                match self.selected_reg_type {
                    RegisterType::Dummy0 | RegisterType::Dummy1 | RegisterType::Dummy2 | RegisterType::Dummy3 => {}
                    RegisterType::ChrOuterBank => mem.set_chr_rom_outer_bank_number(value & 1),
                    RegisterType::PrgBank => mem.set_prg_register(P0, value & 0b11),
                    RegisterType::ChrInnerBank => mem.set_chr_register(C0, value & 0b11),
                    RegisterType::NameTableMirroring => mem.set_name_table_mirroring((value >> 1) & 0b11),
                }
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

#[derive(Clone, Copy, Default)]
pub enum RegisterType {
    #[default]
    Dummy0,
    Dummy1,
    Dummy2,
    Dummy3,
    ChrOuterBank,
    PrgBank,
    ChrInnerBank,
    NameTableMirroring,
}