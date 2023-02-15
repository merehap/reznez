use num_traits::FromPrimitive;

use crate::memory::mapper::*;

lazy_static! {
    static ref PRG_LAYOUT_R6_AT_8000: PrgLayout = PrgLayout::builder()
        .max_bank_count(32)
        .bank_size(8 * KIBIBYTE)
        .add_window(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgType::WorkRam)
        .add_window(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(R6)))
        .add_window(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(R7)))
        .add_window(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::SECOND_LAST))
        .add_window(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::LAST))
        .build();

    // Same as PRG_LAYOUT_R6_AT_8000, except the 0x8000 and 0xC000 windows are swapped.
    static ref PRG_LAYOUT_R6_AT_C000: PrgLayout = PrgLayout::builder()
        .max_bank_count(32)
        .bank_size(8 * KIBIBYTE)
        .add_window(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgType::WorkRam)
        .add_window(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::SECOND_LAST))
        .add_window(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(R7)))
        .add_window(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(R6)))
        .add_window(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::LAST))
        .build();

    static ref CHR_LAYOUT_BIG_WINDOWS_FIRST: ChrLayout = ChrLayout::builder()
        .max_bank_count(256)
        .bank_size(1 * KIBIBYTE)
        // Big windows.
        .add_window(0x0000, 0x07FF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::Register(R0)))
        .add_window(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::Register(R1)))
        // Small windows.
        .add_window(0x1000, 0x13FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(R2)))
        .add_window(0x1400, 0x17FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(R3)))
        .add_window(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(R4)))
        .add_window(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(R5)))
        .build();

    static ref CHR_LAYOUT_SMALL_WINDOWS_FIRST: ChrLayout = ChrLayout::builder()
        .max_bank_count(256)
        .bank_size(1 * KIBIBYTE)
        // Small windows.
        .add_window(0x0000, 0x03FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(R2)))
        .add_window(0x0400, 0x07FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(R3)))
        .add_window(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(R4)))
        .add_window(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(R5)))
        // Big windows.
        .add_window(0x1000, 0x17FF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::Register(R0)))
        .add_window(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::Register(R1)))
        .build();
}

// MMC3 and MMC6
pub struct Mapper4 {
    selected_register_id: BankIndexRegisterId,

    irq_pending: bool,
    irq_enabled: bool,
    irq_counter: u8,
    force_reload_irq_counter: bool,
    irq_counter_reload_value: u8,
    pattern_table_side: PatternTableSide,

    prg_memory: PrgMemory,
    chr_memory: ChrMemory,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper4 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper4, String> {
        Ok(Mapper4 {
            selected_register_id: R0,

            irq_pending: false,
            irq_enabled: false,
            irq_counter: 0,
            force_reload_irq_counter: false,
            irq_counter_reload_value: 0,
            pattern_table_side: PatternTableSide::Left,

            prg_memory: PrgMemory::new(PRG_LAYOUT_R6_AT_8000.clone(), cartridge.prg_rom()),
            chr_memory: ChrMemory::new(CHR_LAYOUT_BIG_WINDOWS_FIRST.clone(), cartridge.chr_rom()),
            name_table_mirroring: cartridge.name_table_mirroring(),
        })
    }

    fn bank_select(&mut self, value: u8) {
        let chr_big_windows_first =                             (value & 0b1000_0000) == 0;
        let r6_is_at_0x8000 =                                   (value & 0b0100_0000) == 0;
        self.selected_register_id = BankIndexRegisterId::from_u8(value & 0b0000_0111).unwrap();

        if chr_big_windows_first {
            self.chr_memory.set_layout(CHR_LAYOUT_BIG_WINDOWS_FIRST.clone())
        } else {
            self.chr_memory.set_layout(CHR_LAYOUT_SMALL_WINDOWS_FIRST.clone())
        }

        if r6_is_at_0x8000 {
            self.prg_memory.set_layout(PRG_LAYOUT_R6_AT_8000.clone());
        } else {
            self.prg_memory.set_layout(PRG_LAYOUT_R6_AT_C000.clone());
        }
    }

    fn set_bank_index(&mut self, value: u8) {
        match self.selected_register_id {
            // Double-width windows can only use even banks.
            R0 | R1 => {
                let bank_index = u16::from(value & 0b1111_1110);
                self.chr_memory.set_bank_index_register(self.selected_register_id, bank_index);
            }
            R2 | R3 | R4 | R5 => {
                let bank_index = u16::from(value);
                self.chr_memory.set_bank_index_register(self.selected_register_id, bank_index);
            }
            // There can only be up to 64 PRG banks, though some ROM hacks use more.
            R6 | R7 => {
                assert_eq!(value & 0b1100_0000, 0, "ROM hack.");
                let bank_index = u16::from(value & 0b0011_1111);
                self.prg_memory.set_bank_index_register(self.selected_register_id, bank_index);
            }
        };
    }

    fn set_mirroring(&mut self, value: u8) {
        use NameTableMirroring::*;
        match (self.name_table_mirroring, value & 0b0000_0001) {
            (Vertical, 1) => self.name_table_mirroring = Horizontal,
            (Horizontal, 0) => self.name_table_mirroring = Vertical,
            _ => { /* Other mirrorings cannot be changed. */ },
        }
    }

    fn prg_ram_protect(&mut self, _value: u8) {
        // TODO: See if this can be implemented while remaining compatible with MMC6.
        // https://www.nesdev.org/wiki/MMC3#PRG_RAM_protect_($A001-$BFFF,_odd)
    }

    fn set_irq_reload_value(&mut self, value: u8) {
        self.irq_counter_reload_value = value;
    }

    fn reload_irq_counter(&mut self) {
        self.irq_counter = 0;
        // FIXME: This needs to be delayed until the next "rising edge of the PPU address".
        self.irq_counter = self.irq_counter_reload_value;
    }

    fn disable_irq(&mut self) {
        self.irq_enabled = false;
        self.irq_pending = false;
    }

    fn enable_irq(&mut self) {
        self.irq_enabled = true;
    }
}

impl Mapper for Mapper4 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        let is_even_address = address.to_raw() % 2 == 0;
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ },
            0x6000..=0x7FFF =>                    self.prg_memory.write(address, value),
            0x8000..=0x9FFF if is_even_address => self.bank_select(value),
            0x8000..=0x9FFF =>                    self.set_bank_index(value),
            0xA000..=0xBFFF if is_even_address => self.set_mirroring(value),
            0xA000..=0xBFFF =>                    self.prg_ram_protect(value),
            0xC000..=0xDFFF if is_even_address => self.set_irq_reload_value(value),
            0xC000..=0xDFFF =>                    self.reload_irq_counter(),
            0xE000..=0xFFFF if is_even_address => self.disable_irq(),
            0xE000..=0xFFFF =>                    self.enable_irq(),
        }
    }

    fn process_current_ppu_address(&mut self, address: PpuAddress) {
        let next_side = address.pattern_table_side();
        let should_tick_irq_counter =
            self.pattern_table_side == PatternTableSide::Left
            && next_side == PatternTableSide::Right;

        if should_tick_irq_counter {
            if self.irq_counter == 0 || self.force_reload_irq_counter {
                self.irq_counter = self.irq_counter_reload_value;
                if self.irq_enabled {
                    self.irq_pending = true;
                }
            } else {
                self.irq_counter -= 1;
            }
        }

        self.pattern_table_side = next_side;
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    fn prg_memory(&self) -> &PrgMemory {
        &self.prg_memory
    }

    fn chr_memory(&self) -> &ChrMemory {
        &self.chr_memory
    }

    fn chr_memory_mut(&mut self) -> &mut ChrMemory {
        &mut self.chr_memory
    }
}
