use crate::memory::mapper::*;
use crate::memory::mappers::common::mmc1::{ShiftRegister, ShiftStatus};

const PRG_LAYOUT_FIXED_LAST: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);
const PRG_LAYOUT_FIXED_FIRST: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::switchable_rom(P0)),
]);
const PRG_LAYOUT_ONE_BIG: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_rom(P0)),
]);

// TODO: Not all boards support CHR RAM.
const CHR_LAYOUT_ONE_BIG: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_ram(C0)),
]);
const CHR_LAYOUT_TWO_SMALL: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::switchable_ram(C0)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::switchable_ram(C1)),
]);

const PRG_LAYOUTS: [PrgLayout; 4] =
    [PRG_LAYOUT_ONE_BIG, PRG_LAYOUT_ONE_BIG, PRG_LAYOUT_FIXED_FIRST, PRG_LAYOUT_FIXED_LAST];

const CHR_LAYOUTS: [ChrLayout; 2] =
    [CHR_LAYOUT_ONE_BIG, CHR_LAYOUT_TWO_SMALL];

const MIRRORINGS: [NameTableMirroring; 4] = [
    NameTableMirroring::OneScreenRightBank,
    NameTableMirroring::OneScreenLeftBank,
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

const RAM_STATUSES: [RamStatus; 2] = [
    RamStatus::ReadWrite,
    RamStatus::Disabled,
];

// SxROM (MMC1, MMC1B)
pub struct Mapper001_0 {
    shift_register: ShiftRegister,
}

impl Mapper for Mapper001_0 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(16)
            .prg_bank_size(16 * KIBIBYTE)
            .prg_layout(PRG_LAYOUT_FIXED_LAST)
            .chr_max_bank_count(32)
            .chr_bank_size(4 * KIBIBYTE)
            .chr_layout(CHR_LAYOUT_ONE_BIG)
            .name_table_mirroring_source(NameTableMirroring::OneScreenRightBank.to_source())
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        // Work RAM writes don't trigger any of the shifter logic.
        if matches!(address.to_raw(), 0x6000..=0x7FFF) {
            params.write_prg(address, value);
            return;
        }

        match self.shift_register.shift(value) {
            ShiftStatus::Clear => params.set_prg_layout(PRG_LAYOUT_FIXED_LAST),
            ShiftStatus::Continue => { /* Do nothing additional. */ }
            ShiftStatus::Done { finished_value } => match address.to_raw() {
                0x0000..=0x401F => unreachable!(),
                0x4020..=0x5FFF => { /* Do nothing. */ }
                0x6000..=0x7FFF => unreachable!(),
                0x8000..=0x9FFF => {
                    let fields = splitbits!(finished_value, "...cppmm");
                    params.set_chr_layout(CHR_LAYOUTS[fields.c as usize]);
                    params.set_prg_layout(PRG_LAYOUTS[fields.p as usize]);
                    params.set_name_table_mirroring(MIRRORINGS[fields.m as usize]);
                }
                // FIXME: Handle cases for special boards.
                0xA000..=0xBFFF => params.set_bank_register(C0, finished_value),
                // FIXME: Handle cases for special boards.
                0xC000..=0xDFFF => params.set_bank_register(C1, finished_value),
                0xE000..=0xFFFF => {
                    let fields = splitbits!(finished_value, "...rbbbb");
                    params.set_ram_status(S0, RAM_STATUSES[fields.r as usize]);
                    params.set_bank_register(P0, fields.b);
                }
            }
        }
    }
}

impl Mapper001_0 {
    pub fn new(cartridge: &Cartridge) -> Self {
        let prg_rom_size = cartridge.prg_rom_size() / KIBIBYTE;
        let prg_ram_size = cartridge.prg_ram_size() / KIBIBYTE;
        let chr_ram_size = cartridge.chr_ram_size() / KIBIBYTE;

        let board = match (prg_rom_size, prg_ram_size, chr_ram_size) {
            (128,  8, 8) => Board::Snrom,
            (256,  8, 8) => Board::Snrom,
            (  _, 16, _) => Board::Sorom,
            (  _, 32, _) => Board::Sxrom,
            _ => Board::Standard,
        };

        assert_eq!(board, Board::Standard, "MMC1 {board:?} is not yet supported.");

        Self { shift_register: ShiftRegister::new() }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Board {
    Standard,
    // PRG ROM <= 256k, CHR RAM = 8k, PRG RAM = 8k
    Snrom,
    // PRG RAM = 16k
    Sorom,
    // PRG RAM = 32k
    Sxrom
}
