use std::collections::BTreeMap;

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
pub struct Mapper114 {
    mmc3: mmc3::Mapper004Mmc3,
    scrambled_addrs: BTreeMap<u16, u16>,
    scrambled_banks: [u8; 8],
    nrom_layout_index: Option<u8>,
}

impl Mapper for Mapper114 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, mut value: u8) {
        let addr = *addr & 0xE001;
        match addr {
            0x6000 => {
                let fields = splitbits!(value, "n.l. pppp");
                self.nrom_layout_index = if fields.n {
                    Some(fields.l as u8 + 2)
                } else {
                    None
                };
            }
            0x6001 => {
                bus.set_chr_rom_outer_bank_number(value & 1);
            }
            0x8000..=0xFFFF => {
                let unscrambled_addr = self.scrambled_addrs[&addr];
                if unscrambled_addr == 0x8000 {
                    let reg_index = self.scrambled_banks[usize::from(value) & 0b111];
                    value = (value & 0b1111_1000) | reg_index;
                }

                self.mmc3.write_register(bus, unscrambled_addr.into(), value);
            }
            _ => { /* No regs here. */ }
        }

        bus.update_effective_prg_layout_index(|base_index| {
            self.nrom_layout_index.unwrap_or(base_index)
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

impl Mapper114 {
    pub fn new(scrambled_addrs: BTreeMap<u16, u16>, scrambled_banks: [u8; 8]) -> Self {
        Self {
            mmc3: mmc3::Mapper004Mmc3::new(Mmc3IrqState::REV_A_IRQ_STATE),
            scrambled_addrs,
            scrambled_banks,
            nrom_layout_index: None,
        }
    }
}