use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(8 * KIBIBYTE * KIBIBYTE)
    // TODO: Infer this from the templates, failing if they aren't all equal.
    .prg_rom_outer_bank_size(32 * KIBIBYTE)
    // "At power on, the last 16 KiB of the ROM is mapped into $C000-$FFFF. The rest of the state is unspecified."
    .prg_layout_index(3)
    // Mode 0x00-0x03
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x04-0x07 (Same as above)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x08-0x0B
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀0₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀p₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x0C-0x0F
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀p₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀1₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x10-0x13
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁p₀₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x14-0x17 (Same as above)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁p₀₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x18-0x1B
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀0₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁p₀₁p₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x1C-0x1F
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁p₀₁p₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀1₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x20-0x23
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂p₀₁p₀₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x24-0x27 (Same as above)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂p₀₁p₀₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x28-0x2B
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀0₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂p₀₂p₀₁p₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x2C-0x2F
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂p₀₂p₀₁p₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀1₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x30-0x33
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃p₀₂p₀₁p₀₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x34-0x37 (Same as above)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃p₀₂p₀₁p₀₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x38-0x3B
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀0₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃p₀₃p₀₂p₀₁p₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Mode 0x3C-0x3F
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃p₀₃p₀₂p₀₁p₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀1₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    .chr_rom_max_size(32 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// Action 53
#[derive(Default)]
pub struct Mapper028 {
    selected_register: Register,
}

impl Mapper for Mapper028 {
    fn init_mapper_params(&self, bus: &mut Bus) {
        // "At power on, the last 16 KiB of the ROM is mapped into $C000-$FFFF. The rest of the state is unspecified."
        bus.set_prg_rom_outer_bank_number(0b1111_1111);
        bus.set_prg_register(P, 0b1111_1111_1111_1111u16);
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x4FFF => { /* Do nothing. */ }
            0x5000..=0x5FFF => {
                self.selected_register = match value & 0b1000_0001 {
                    0b0000_0000 => Register::ChrBank,
                    0b0000_0001 => Register::InnerPrgBank,
                    0b1000_0000 => Register::Mode,
                    0b1000_0001 => Register::OuterPrgBank,
                    _ => unreachable!(),
                };
            }
            0x6000..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                match self.selected_register {
                    Register::ChrBank => {
                        let (mirroring, chr_bank) = splitbits_named!(min=u8, value, "...m ..cc");
                        bus.set_chr_register(C, chr_bank);
                        if bus.name_table_mirroring().is_regular_one_screen() {
                            bus.set_name_table_mirroring(mirroring);
                        }
                    }
                    Register::InnerPrgBank => {
                        let (mirroring, inner_bank_bits) = splitbits_named!(min=u8, value, "...m pppp");
                        bus.set_prg_register(P, inner_bank_bits);
                        if bus.name_table_mirroring().is_regular_one_screen() {
                            bus.set_name_table_mirroring(mirroring);
                        }
                    }
                    Register::Mode => {
                        let (layout, name_table_mirroring) = splitbits_named!(value, "..ll llnn");
                        bus.set_prg_layout(layout);
                        bus.set_name_table_mirroring(name_table_mirroring);
                    }
                    Register::OuterPrgBank => {
                        bus.set_prg_rom_outer_bank_number(value);
                    }
                }
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

#[derive(Debug, Default)]
enum Register {
    #[default]
    ChrBank,
    InnerPrgBank,
    Mode,
    OuterPrgBank,
}