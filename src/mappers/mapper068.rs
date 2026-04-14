use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.read_write_status(RS0, WS0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        // TODO: The fixed number here may have to change once external ROM is supported.
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_rom_inner_bank_size(2 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(D)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(E)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(F)),
        ChrWindow::new(0x2000, 0x23FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NTS0).meta_switchable(MR0)),
        ChrWindow::new(0x2400, 0x27FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NTS1).meta_switchable(MR1)),
        ChrWindow::new(0x2800, 0x2BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NTS2).meta_switchable(MR2)),
        ChrWindow::new(0x2C00, 0x2FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NTS3).meta_switchable(MR3)),
    ])
    // Vertical mirroring
    .override_chr_meta_register(MR0, NT0)
    .override_chr_meta_register(MR1, NT1)
    .override_chr_meta_register(MR2, NT0)
    .override_chr_meta_register(MR3, NT1)
    .complicated_name_table_mirroring()
    // TODO: Verify that this hooks up properly with NTS0-NTS3
    // TODO: Verify that these values are correct.
    .cartridge_selection_name_table_mirrorings([
        Some(NameTableMirroring::VERTICAL),
        Some(NameTableMirroring::HORIZONTAL),
        Some(NameTableMirroring::ONE_SCREEN_LEFT_BANK),
        Some(NameTableMirroring::ONE_SCREEN_RIGHT_BANK),
    ])
    .build();

// Sunsoft-4
// TODO: Support Nantettatte!! Baseball/external ROM/licensing IC
// FIXME: Broken
pub struct Mapper068;

impl Mapper for Mapper068 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* No regs here. */ }
            0x6000..=0x7FFF => { /* TODO: Licensing IC. */ }
            0x8000..=0x8FFF => bus.set_chr_register(C, value),
            0x9000..=0x9FFF => bus.set_chr_register(D, value),
            0xA000..=0xAFFF => bus.set_chr_register(E, value),
            0xB000..=0xBFFF => bus.set_chr_register(F, value),
            0xC000..=0xCFFF => bus.set_chr_register(NT0, value | 0b1000_0000),
            0xD000..=0xDFFF => bus.set_chr_register(NT1, value | 0b1000_0000),
            0xE000..=0xEFFF => {
                let (use_rom, name_table_mirroring_index) = splitbits_named!(value, "...r ..mm");
                if use_rom {
                    bus.set_chr_source(NTS0, ChrSource::Rom);
                    bus.set_chr_source(NTS1, ChrSource::Rom);
                    bus.set_chr_source(NTS2, ChrSource::Rom);
                    bus.set_chr_source(NTS3, ChrSource::Rom);
                    let (mr0, mr1, mr2, mr3) = match name_table_mirroring_index {
                        0 => (NT0, NT1, NT0, NT1),
                        1 => (NT0, NT0, NT1, NT1),
                        2 => (NT0, NT0, NT0, NT0),
                        3 => (NT1, NT1, NT1, NT1),
                        _ => unreachable!(),
                    };
                    bus.set_chr_meta_register(MR0, mr0);
                    bus.set_chr_meta_register(MR1, mr1);
                    bus.set_chr_meta_register(MR2, mr2);
                    bus.set_chr_meta_register(MR3, mr3);
                } else {
                    let (side0, side1, side2, side3) = match name_table_mirroring_index {
                        0 => (CiramSide::Left , CiramSide::Right, CiramSide::Left , CiramSide::Right),
                        1 => (CiramSide::Left , CiramSide::Left , CiramSide::Right, CiramSide::Right),
                        2 => (CiramSide::Left , CiramSide::Left , CiramSide::Left , CiramSide::Left ),
                        3 => (CiramSide::Right, CiramSide::Right, CiramSide::Right, CiramSide::Right),
                        _ => unreachable!(),
                    };
                    bus.set_chr_source(NTS0, ChrSource::Ciram(side0));
                    bus.set_chr_source(NTS1, ChrSource::Ciram(side1));
                    bus.set_chr_source(NTS2, ChrSource::Ciram(side2));
                    bus.set_chr_source(NTS3, ChrSource::Ciram(side3));
                }
            }
            0xF000..=0xFFFF => {
                let fields = splitbits!(value, "...e pppp");
                bus.set_reads_enabled(RS0, fields.e);
                bus.set_writes_enabled(WS0, fields.e);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}