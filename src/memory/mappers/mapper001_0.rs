use crate::memory::mapper::*;
use crate::memory::mappers::mmc1::board::Board;
use crate::memory::mappers::mmc1::shift_register::{ShiftRegister, ShiftStatus};

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(256 * KIBIBYTE)
    .prg_layout_index(3)
    .prg_layout(PRG_WINDOWS_ONE_BIG)
    .prg_layout(PRG_WINDOWS_ONE_BIG)
    .prg_layout(PRG_WINDOWS_FIXED_FIRST)
    .prg_layout(PRG_WINDOWS_FIXED_LAST)
    .chr_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::RAM.switchable(C0)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::RAM.switchable(C0)),
        Window::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::RAM.switchable(C1)),
    ])
    .override_initial_name_table_mirroring(NameTableMirroring::ONE_SCREEN_LEFT_BANK)
    .name_table_mirrorings(&[
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .ram_statuses(&[
        RamStatus::ReadWrite,
        RamStatus::Disabled,
    ])
    .build();

const PRG_WINDOWS_ONE_BIG: &[Window] = &[
    Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM.switchable(P1).status_register(S0)),
    Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
];
const PRG_WINDOWS_FIXED_FIRST: &[Window] = &[
    Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM.switchable(P1).status_register(S0)),
    Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(0)),
    Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
];
const PRG_WINDOWS_FIXED_LAST: &[Window] = &[
    Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM.switchable(P1).status_register(S0)),
    Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
    Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
];

// SxROM (MMC1, MMC1B)
pub struct Mapper001_0 {
    _board: Board,
    shift_register: ShiftRegister,
}

impl Mapper for Mapper001_0 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        // Only writes of 0x8000 to 0xFFFF trigger shifter logic.
        if cpu_address < 0x8000 {
            return;
        }

        match self.shift_register.shift(value) {
            ShiftStatus::Clear => params.set_prg_layout(3),
            ShiftStatus::Continue => { /* Do nothing additional. */ }
            ShiftStatus::Done { finished_value } => match cpu_address {
                0x0000..=0x7FFF => unreachable!(),
                0x8000..=0x9FFF => {
                    let fields = splitbits!(min=u8, finished_value, "...cppmm");
                    params.set_chr_layout(fields.c);
                    params.set_prg_layout(fields.p);
                    params.set_name_table_mirroring(fields.m);
                }
                // FIXME: Handle cases for special boards.
                0xA000..=0xBFFF => params.set_bank_register(C0, finished_value),
                // FIXME: Handle cases for special boards.
                0xC000..=0xDFFF => params.set_bank_register(C1, finished_value),
                0xE000..=0xFFFF => {
                    let fields = splitbits!(min=u8, finished_value, "...rbbbb");
                    params.set_ram_status(S0, fields.r);
                    params.set_bank_register(P0, fields.b);
                }
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper001_0 {
    pub fn new(cartridge: &Cartridge) -> Self {
        Self {
            _board: Board::from_cartridge(cartridge),
            shift_register: ShiftRegister::default(),
        }
    }
}