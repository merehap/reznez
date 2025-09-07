use crate::cartridge::resolved_metadata::ResolvedMetadata;
use crate::mapper::*;
use crate::mappers::mmc1::board::Board;
use crate::mappers::mmc1::shift_register::{ShiftRegister, ShiftStatus};

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_rom_outer_bank_size(256 * KIBIBYTE)
    .prg_layout_index(3)
    .prg_layout(PRG_WINDOWS_ONE_BIG)
    .prg_layout(PRG_WINDOWS_ONE_BIG)
    .prg_layout(PRG_WINDOWS_FIXED_FIRST)
    .prg_layout(PRG_WINDOWS_FIXED_LAST)
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
    ])
    // TODO: Reconcile these values with nes20db.xml
    .cartridge_selection_name_table_mirrorings([
        // Verified against nes20db.xml, but unknown if that has been verified against an actual cartridge.
        Some(NameTableMirroring::HORIZONTAL),
        // Contradicts nes20db.xml.
        Some(NameTableMirroring::VERTICAL),
        // Contradicts nes20db.xml.
        Some(NameTableMirroring::ONE_SCREEN_LEFT_BANK),
        // Contradicts nes20db.xml.
        Some(NameTableMirroring::ONE_SCREEN_LEFT_BANK),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .read_write_statuses(&[
        ReadWriteStatus::ReadWrite,
        ReadWriteStatus::Disabled,
    ])
    .build();

const PRG_WINDOWS_ONE_BIG: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM.switchable(P0).status_register(S0)),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
];
const PRG_WINDOWS_FIXED_FIRST: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM.switchable(P0).status_register(S0)),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(0)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
];
const PRG_WINDOWS_FIXED_LAST: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM.switchable(P0).status_register(S0)),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
];

// SxROM (MMC1, MMC1B)
pub struct Mapper001_0 {
    board: Board,
    shift_register: ShiftRegister,
}

impl Mapper for Mapper001_0 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
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
                0xA000..=0xBFFF => {
                    self.set_chr_bank_and_board_specifics(params, C0, finished_value);
                }
                0xC000..=0xDFFF => {
                    if params.chr_memory().layout_index() == 1 {
                        self.set_chr_bank_and_board_specifics(params, C1, finished_value);
                    }
                }
                0xE000..=0xFFFF => {
                    let fields = splitbits!(min=u8, finished_value, "...spppp");
                    params.set_read_write_status(S0, fields.s);
                    params.set_prg_register(P1, fields.p);
                }
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper001_0 {
    pub fn new(metadata: &ResolvedMetadata) -> Result<Self, String> {
        Ok(Self {
            board: Board::from_cartridge_metadata(metadata)?,
            shift_register: ShiftRegister::default(),
        })
    }

    fn set_chr_bank_and_board_specifics(&self, params: &mut MapperParams, chr_id: ChrBankRegisterId, value: u8) {
        match self.board {
            Board::SNROM => {
                let fields = splitbits!(min=u8, value, "...s...c");
                params.set_read_write_status(S0, fields.s);
                params.set_chr_register(chr_id, fields.c);
            }
            Board::SUROM => {
                let banks = splitbits!(min=u8, value, "...p...c");
                params.set_prg_rom_outer_bank_index(banks.p);
                params.set_chr_register(chr_id, banks.c);
            }
            Board::SOROM => {
                let banks = splitbits!(min=u8, value, "...pr..c");
                params.set_prg_rom_outer_bank_index(banks.p);
                params.set_prg_register(P0, banks.r);
                params.set_chr_register(chr_id, banks.c);
            }
            Board::SXROM => {
                let banks = splitbits!(min=u8, value, "...prr.c");
                params.set_prg_rom_outer_bank_index(banks.p);
                params.set_prg_register(P0, banks.r);
                params.set_chr_register(chr_id, banks.c);
            }
            Board::SZROM => {
                let banks = splitbits!(min=u8, value, "...rcccc");
                params.set_prg_register(P0, banks.r);
                params.set_chr_register(chr_id, banks.c);
            }
            _ => {
                params.set_chr_register(chr_id, value);
            }
        }
    }
}