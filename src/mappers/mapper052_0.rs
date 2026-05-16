use crate::mapper::*;
use crate::mappers::mmc3::mmc3;
use crate::mappers::mmc3::irq_state::Mmc3IrqState;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(1024 * KIBIBYTE)
    // PRG_WINDOWS_8000_SWITCHABLE, but with an outer bank
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.read_write_status(RS0, WS0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁p₀₄p₀₃p₀₂p₀₁p₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁q₀₄q₀₃q₀₂q₀₁q₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁1₀₄1₀₃1₀₂1₀₁0₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁1₀₄1₀₃1₀₂1₀₁1₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // PRG_WINDOWS_C000_SWITCHABLE, but with an outer bank
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.read_write_status(RS0, WS0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁1₀₄1₀₃1₀₂1₀₁0₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁q₀₄q₀₃q₀₂q₀₁q₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁p₀₄p₀₃p₀₂p₀₁p₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁1₀₄1₀₃1₀₂1₀₁1₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Same as layout 0, but with the lowest outer bank bit exposed.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.read_write_status(RS0, WS0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁o₀₀p₀₃p₀₂p₀₁p₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁o₀₀q₀₃q₀₂q₀₁q₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁o₀₀1₀₃1₀₂1₀₁0₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁o₀₀1₀₃1₀₂1₀₁1₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Same as layout 1, but with the lowest outer bank bit exposed.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.read_write_status(RS0, WS0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁o₀₀1₀₃1₀₂1₀₁0₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁o₀₀q₀₃q₀₂q₀₁q₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁o₀₀p₀₃p₀₂p₀₁p₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.rom_address_template("o₀₂o₀₁o₀₀1₀₃1₀₂1₀₁1₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    .chr_rom_max_size(1024 * KIBIBYTE)
    .chr_layout(&[
        // Big windows.
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁c₀₇c₀₆c₀₅c₀₄c₀₃c₀₂c₀₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁d₀₇d₀₆d₀₅d₀₄d₀₃d₀₂d₀₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        // Small windows.
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁e₀₇e₀₆e₀₅e₀₄e₀₃e₀₂e₀₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁f₀₇f₀₆f₀₅f₀₄f₀₃f₀₂f₀₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁g₀₇g₀₆g₀₅g₀₄g₀₃g₀₂g₀₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁h₀₇h₀₆h₀₅h₀₄h₀₃h₀₂h₀₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    .chr_layout(&[
        // Small windows.
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁e₀₇e₀₆e₀₅e₀₄e₀₃e₀₂e₀₁e₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁f₀₇f₀₆f₀₅f₀₄f₀₃f₀₂f₀₁f₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁g₀₇g₀₆g₀₅g₀₄g₀₃g₀₂g₀₁g₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁h₀₇h₀₆h₀₅h₀₄h₀₃h₀₂h₀₁h₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        // Big windows.
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁c₀₇c₀₆c₀₅c₀₄c₀₃c₀₂c₀₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁d₀₇d₀₆d₀₅d₀₄d₀₃d₀₂d₀₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    .chr_layout(&[
        // Big windows.
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁o₀₀c₀₆c₀₅c₀₄c₀₃c₀₂c₀₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁o₀₀d₀₆d₀₅d₀₄d₀₃d₀₂d₀₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        // Small windows.
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁o₀₀e₀₆e₀₅e₀₄e₀₃e₀₂e₀₁e₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁o₀₀f₀₆f₀₅f₀₄f₀₃f₀₂f₀₁f₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁o₀₀g₀₆g₀₅g₀₄g₀₃g₀₂g₀₁g₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁o₀₀h₀₆h₀₅h₀₄h₀₃h₀₂h₀₁h₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    .chr_layout(&[
        // Small windows.
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁o₀₀e₀₆e₀₅e₀₄e₀₃e₀₂e₀₁e₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁o₀₀f₀₆f₀₅f₀₄f₀₃f₀₂f₀₁f₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁o₀₀g₀₆g₀₅g₀₄g₀₃g₀₂g₀₁g₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁o₀₀h₀₆h₀₅h₀₄h₀₃h₀₂h₀₁h₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        // Big windows.
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁o₀₀c₀₆c₀₅c₀₄c₀₃c₀₂c₀₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.rom_address_template("o₀₂o₀₁o₀₀d₀₆d₀₅d₀₄d₀₃d₀₂d₀₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    .name_table_mirrorings(mmc3::NAME_TABLE_MIRRORINGS)
    .build();

// Realtec 8213 (Mario 7-in-1)
// FIXME: Mario 5, 10, and 7 have corrupted CHR rendering.
pub struct Mapper052_0 {
    mmc3: mmc3::Mapper004Mmc3,
    // TODO: Unlock on reset
    lock_outer_bank_register: bool,
    expanded_outer_prg_bank: bool,
    expanded_outer_chr_bank: bool,
}

impl Mapper for Mapper052_0 {
    fn reset(&mut self, bus: &mut Bus) {
        self.lock_outer_bank_register = false;
        self.expanded_outer_chr_bank = false;
        self.expanded_outer_prg_bank = false;
        bus.reset_bank_registers();
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        // "The MMC3's WRAM interface must be enabled and writeable in MMC3 register $A001.
        // The Outer Bank Register overlaps any actual PRG RAM that may be present."
        let ram_writes_enabled = bus.prg_memory.bank_registers().write_status(WS0) == WriteStatus::Enabled;
        if matches!(*addr, 0x6000..=0x7FFF) && !self.lock_outer_bank_register && ram_writes_enabled {
            let fields = splitbits!(value, "lxcc yhpp");
            self.lock_outer_bank_register = fields.l;
            self.expanded_outer_chr_bank = fields.x;
            bus.set_chr_rom_outer_bank_number((u8::from(fields.h) << 2) | fields.c);
            self.expanded_outer_prg_bank = fields.y;
            bus.set_prg_rom_outer_bank_number((u8::from(fields.h) << 2) | fields.p);
        }

        self.mmc3.write_register(bus, addr, value);
        bus.modify_base_prg_layout_index(|base_index| {
            if self.expanded_outer_prg_bank {
                base_index | 0b10
            } else {
                base_index
            }
        });
        bus.modify_base_chr_layout_index(|base_index| {
            if self.expanded_outer_chr_bank {
                base_index | 0b10
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

impl Mapper052_0 {
    pub fn new() -> Self {
        Self {
            // TODO: Verify if Sharp is correct.
            mmc3: mmc3::Mapper004Mmc3::new(Mmc3IrqState::SHARP_IRQ_STATE),
            lock_outer_bank_register: false,
            expanded_outer_prg_bank: false,
            expanded_outer_chr_bank: false,
        }
    }
}