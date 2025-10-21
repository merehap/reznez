use crate::cartridge::resolved_metadata::ResolvedMetadata;
use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    // NTDEC 0324
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    // GS-2017
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM.fixed_index(0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// NTDEC 0324 and GS-2017
pub struct Mapper061 {
    chr_board: ChrBoard,
}

impl Mapper for Mapper061 {
    fn init_mapper_params(&self, params: &mut Memory) {
        params.set_chr_layout(self.chr_board as u8);
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, _value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, *addr, ".... cccc m.ql pppp");
                let prg_index = combinebits!(fields.p, fields.q, "000ppppq");

                mem.set_prg_register(P0, prg_index);
                mem.set_chr_register(C0, fields.c);
                mem.set_name_table_mirroring(fields.m);
                mem.set_prg_layout(fields.l);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper061 {
    pub fn new(metadata: &ResolvedMetadata) -> Self {
        const CHR_RAM_SIZE: u32 = 8 * KIBIBYTE;

        let chr_board = match metadata.chr_work_ram_size {
            0 => ChrBoard::SwitchableRom,
            CHR_RAM_SIZE => ChrBoard::FixedRam,
            _ => panic!("Bad CHR RAM size for mapper 64: {}", metadata.chr_work_ram_size),
        };

        Self { chr_board }
    }
}

#[derive(Clone, Copy)]
enum ChrBoard {
    // NTDEC 0324
    SwitchableRom = 0,
    // GS-2017
    FixedRam = 1,
}
