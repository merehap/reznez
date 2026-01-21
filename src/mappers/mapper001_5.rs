use crate::mapper::*;
use crate::mappers::mmc1::shift_register::{ShiftRegister, ShiftStatus};

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(32 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.read_write_status(R0, W0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.fixed_number(0)),
    ])
    .chr_rom_max_size(64 * KIBIBYTE)
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
    .build();

// SEROM. MMC1 that doesn't support PRG bank switching.
#[derive(Default)]
pub struct Mapper001_5 {
    shift_register: ShiftRegister,
}

impl Mapper for Mapper001_5 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        // Only writes of 0x8000 to 0xFFFF trigger shifter logic.
        if *addr < 0x8000 {
            return;
        }

        match self.shift_register.shift(value) {
            ShiftStatus::Clear | ShiftStatus::Continue => { /* Do nothing additional. */ }
            ShiftStatus::Done { finished_value } => match *addr {
                0x0000..=0x7FFF => unreachable!(),
                0x8000..=0x9FFF => {
                    let fields = splitbits!(min=u8, finished_value, "...c..mm");
                    bus.set_chr_layout(fields.c);
                    bus.set_name_table_mirroring(fields.m);
                }
                0xA000..=0xBFFF => bus.set_chr_register(C0, finished_value),
                0xC000..=0xDFFF => bus.set_chr_register(C1, finished_value),
                0xE000..=0xFFFF => {
                    let ram_disabled = splitbits_named!(finished_value, "...d....");
                    bus.set_reads_enabled(R0, !ram_disabled);
                    bus.set_writes_enabled(W0, !ram_disabled);
                }
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
