use crate::mapper::*;
use crate::mappers::mmc3::mmc3;
use crate::mappers::mmc3::irq_state::Mmc3IrqState;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(mmc3::PRG_WINDOWS_8000_SWITCHABLE)
    .prg_layout(mmc3::PRG_WINDOWS_C000_SWITCHABLE)
    // NROM-128
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.read_write_status(RS0, WS0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(Z)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(Z)),
    ])
    // NROM-256
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.read_write_status(RS0, WS0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(Z)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(mmc3::CHR_BIG_WINDOWS_FIRST)
    .chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST)
    .name_table_mirrorings(mmc3::NAME_TABLE_MIRRORINGS)
    .build();

const SCRAMBLE: [u8; 8] = [0, 3, 1, 5, 6, 7, 2, 4];

// Kǎshèng H2288
pub struct Mapper123 {
    mmc3: mmc3::Mapper004Mmc3,
    mode: Mode,
}

impl Mapper for Mapper123 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, mut value: u8) {
        if *addr & 0xF800 == 0x5800 {
            let fields = splitbits!(min=u8, value, ".mac .bld");
            bus.set_prg_register(Z, (fields.a << 3) | (fields.b << 2) | (fields.c << 1) | fields.d);
            self.mode = if fields.m == 0 {
                Mode::Mmc3
            } else {
                Mode::Nrom { layout_index: fields.l + 2 }
            };
        }

        // Scramble the PRG and CHR registers.
        if 0x8000 <= *addr && *addr <= 0x9FFF && addr.is_multiple_of(2) {
            value = (value & 0b1100_0000) | (SCRAMBLE[(value & 0b0000_0111) as usize]);
        }

        self.mmc3.write_register(bus, addr, value);

        bus.modify_base_prg_layout_index(|base_index| {
            match self.mode {
                Mode::Mmc3 => base_index,
                Mode::Nrom { layout_index } => layout_index,
            }
        });
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper123 {
    pub fn new() -> Self {
        Self {
            mmc3: mmc3::Mapper004Mmc3::new(Mmc3IrqState::SHARP_IRQ_STATE),
            mode: Mode::Mmc3,
        }
    }
}

enum Mode {
    Mmc3,
    Nrom { layout_index: u8 },
}