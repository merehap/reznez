use crate::mapper::*;
use crate::mappers::mmc3::irq_state::Mmc3IrqState;
use crate::mappers::mmc3::mmc3;
use crate::util::bit_util;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_rom_outer_bank_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.read_write_status(RS0, WS0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(X)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(Y)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(Z)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.read_write_status(RS0, WS0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(Y)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(X)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(Z)),
    ])
    .override_prg_bank_register(Y, -2)
    .override_prg_bank_register(Z, -1)
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_rom_outer_bank_size(256 * KIBIBYTE)
    .chr_layout(mmc3::CHR_BIG_WINDOWS_FIRST)
    .chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST)
    .name_table_mirrorings(mmc3::NAME_TABLE_MIRRORINGS)
    .build();

const PROTECTION_VALUES: [u8; 4] = [0x83, 0x83, 0x40, 0x00];

// Kǎshèng A9711 and A9713
// FIXME: Broken intro scene in Sonic & Knuckles
// FIXME: Coins can't be collected in Sonic & Knuckles
// TODO: Support A9713
// TODO: Support special banking for 512KiB CHR games.
pub struct Mapper121 {
    mmc3: mmc3::Mapper004Mmc3,
    board: Board,

    protection_index: u8,

    prg_overrides_active: bool,
    next_override_bank_reg: Option<PrgBankRegisterId>,
    next_override_bank_number: u8,
}

impl Mapper for Mapper121 {
    fn peek_register(&self, _bus: &Bus, addr: CpuAddress) -> ReadResult {
        match *addr {
            0x5000..=0x5FFF => ReadResult::full(PROTECTION_VALUES[self.protection_index as usize]),
            _ => ReadResult::OPEN_BUS,
        }
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        if matches!(*addr, 0x5000..=0x5FFF) {
            self.protection_index = value & 0b11;
        }

        if self.board == Board::A9713 && *addr & 0xF180 == 0x5180 {
            bus.set_prg_rom_outer_bank_number(value >> 7);
            bus.set_chr_rom_outer_bank_number(value >> 7);
        }

        if matches!(*addr, 0x8000..=0x9FFF) {
            match *addr & 0b11 {
                0 | 2 => {
                    self.mmc3.write_register(bus, addr, value);
                }
                1 => {
                    // Reverse the last 6 bits to get the next bank number.
                    self.next_override_bank_number = bit_util::reverse_bits(value) >> 2;
                    if let Some(reg_id) = self.next_override_bank_reg {
                        bus.set_prg_register(reg_id, self.next_override_bank_number);
                    }

                    self.mmc3.write_register(bus, CpuAddress::new(0x8001), value);
                }
                3 => {
                    match value {
                        0x00..=0x1F | 0x21..=0x25 | 0x27 | 0x2D..=0x2E | 0x30..=0x3B | 0x3D..=0x3E | 0x40..=0xFF => {
                            // "Invalid" index provided: return to normal MMC3 banking.
                            self.prg_overrides_active = false;
                        }
                        0x20 | 0x29 | 0x2B | 0x3C | 0x3F => {
                            self.prg_overrides_active = true;
                            bus.set_prg_register(Z, self.next_override_bank_number);
                        }
                        0x26 => {
                            self.prg_overrides_active = true;
                            self.next_override_bank_reg = Some(Z);
                            bus.set_prg_register(Z, self.next_override_bank_number);
                        }
                        0x28 => {
                            self.prg_overrides_active = true;
                            self.next_override_bank_reg = Some(Y);
                            bus.set_prg_register(Y, self.next_override_bank_number);
                        }
                        0x2A => {
                            self.prg_overrides_active = true;
                            self.next_override_bank_reg = Some(X);
                            bus.set_prg_register(X, self.next_override_bank_number);
                        }
                        0x2C => {
                            self.prg_overrides_active = true;
                            self.next_override_bank_reg = None;
                            if self.next_override_bank_number != 0 {
                                bus.set_prg_register(Z, self.next_override_bank_number);
                            }
                        }
                        0x2F => {
                            // Continue all current overrides.
                        }
                    }

                    self.mmc3.write_register(bus, CpuAddress::new(0x8000), value);
                }
                _ => unreachable!(),
            }
        }

        if *addr >= 0xA000 {
            self.mmc3.write_register(bus, addr, value);
        }

        if !self.prg_overrides_active {
            bus.set_prg_register(X, bus.prg_memory.bank_registers().get(Q).index().unwrap().to_raw());
            bus.set_prg_register(Y, BankNumber::from_i16(-2).to_raw());
            bus.set_prg_register(Z, BankNumber::from_i16(-1).to_raw());
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper121 {
    pub fn new(board: Board) -> Self {
        Self {
            // TODO: Verify if this is actually Sharp.
            mmc3: mmc3::Mapper004Mmc3::new(Mmc3IrqState::SHARP_IRQ_STATE),
            board,

            protection_index: 0,

            prg_overrides_active: false,
            next_override_bank_reg: None,
            next_override_bank_number: 0,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum Board {
    A9711,
    A9713,
}