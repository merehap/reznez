use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .chr_max_size(128 * KIBIBYTE)
    // NTDEC 0324
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    // GS-2017
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::RAM.fixed_index(0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::Vertical,
        NameTableMirroring::Horizontal,
    ])
    .build();

// NTDEC 0324 and GS-2017
pub struct Mapper061 {
    chr_board: ChrBoard,
}

impl Mapper for Mapper061 {
    fn init_mapper_params(&self, params: &mut MapperParams) {
        params.set_chr_layout(self.chr_board as u8);
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, _value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, cpu_address, ".... cccc m.ql pppp");
                let prg_index = combinebits!(fields.p, fields.q, "000ppppq");

                params.set_bank_register(P0, prg_index);
                params.set_bank_register(C0, fields.c);
                params.set_name_table_mirroring(fields.m);
                params.set_prg_layout(fields.l);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper061 {
    pub fn new(chr_ram_size: u32) -> Self {
        const CHR_RAM_SIZE: u32 = 8 * KIBIBYTE;

        let chr_board = match chr_ram_size {
            0 => ChrBoard::SwitchableRom,
            CHR_RAM_SIZE => ChrBoard::FixedRam,
            _ => panic!("Bad CHR RAM size for mapper 64: {chr_ram_size}"),
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
