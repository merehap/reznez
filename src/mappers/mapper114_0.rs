use crate::mapper::*;
use crate::mappers::mmc3::mmc3;
use crate::mappers::mmc3::irq_state::Mmc3IrqState;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(mmc3::PRG_WINDOWS_8000_SWITCHABLE)
    .prg_layout(mmc3::PRG_WINDOWS_C000_SWITCHABLE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.read_write_status(RS0, WS0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(Z)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(Z)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.read_write_status(RS0, WS0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(Z)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_rom_outer_bank_size(256 * KIBIBYTE)
    .chr_layout(mmc3::CHR_BIG_WINDOWS_FIRST)
    .chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST)
    .name_table_mirrorings(mmc3::NAME_TABLE_MIRRORINGS)
    .build();

// MMC3 for scrambled register addresses and indices
pub struct Mapper114_0 {
    mmc3: mmc3::Mapper004Mmc3,
    nrom_layout_index: Option<u8>,
}

impl Mapper for Mapper114_0 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr & 0xE001 {
            0x6000 => {
                let fields = splitbits!(value, "n.l. pppp");
                self.nrom_layout_index = if fields.n {
                    Some(fields.l as u8 + 2)
                } else {
                    None
                };
            }
            0x6001 => bus.set_chr_rom_outer_bank_number(value & 1),
            0x8000 => self.mmc3.write_register(bus, 0xA001.into(), value),
            0x8001 => self.mmc3.write_register(bus, 0xA000.into(), value),
            0xA000 => {
                let reg_indexes = [0, 3, 1, 5, 6, 7, 2, 4];
                let reg_index = reg_indexes[usize::from(value) & 0b111];
                let value = (value & 0b1111_1000) | reg_index;
                self.mmc3.write_register(bus, 0x8000.into(), value);
            }
            0xA001 => self.mmc3.write_register(bus, 0xC000.into(), value),
            0xC000 => self.mmc3.write_register(bus, 0x8001.into(), value),
            0xC001 => self.mmc3.write_register(bus, 0xC001.into(), value),
            0xE000 => self.mmc3.write_register(bus, 0xE000.into(), value),
            0xE001 => self.mmc3.write_register(bus, 0xE001.into(), value),
            _ => { /* No regs here. */ }
        }

        bus.modify_base_prg_layout_index(|base_index| {
            if let Some(nrom_layout_index) = self.nrom_layout_index {
                nrom_layout_index
            } else {
                base_index
            }
        });
    }

    fn on_end_of_ppu_cycle(&mut self) {
        self.mmc3.on_end_of_ppu_cycle();
    }

    fn on_ppu_address_change(&mut self, bus: &mut Bus, address: PpuAddress) {
        self.mmc3.on_ppu_address_change(bus, address);
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        self.mmc3.irq_counter_info()
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper114_0 {
    pub fn new() -> Self {
        Self {
            mmc3: mmc3::Mapper004Mmc3::new(Mmc3IrqState::REV_A_IRQ_STATE),
            nrom_layout_index: None,
        }
    }
}