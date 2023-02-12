//use num_traits::FromPrimitive;
use num_derive::FromPrimitive;

use crate::memory::mapper::*;

lazy_static! {
    static ref PRG_LAYOUT: PrgLayout = PrgLayout::builder()
        .max_bank_count(32)
        .bank_size(8 * KIBIBYTE)
        .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::WorkRam)
        .add_window(0x8000, 0x9FFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::FIRST))
        .add_window(0xA000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::FIRST))
        .add_window(0xC000, 0xDFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::SECOND_LAST))
        .add_window(0xE000, 0xFFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::LAST))
        .build();

    static ref CHR_LAYOUT_BIG_WINDOWS_FIRST: ChrLayout = ChrLayout::builder()
        .max_bank_count(256)
        .bank_size(1 * KIBIBYTE)
        .add_window(0x0000, 0x07FF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .add_window(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .add_window(0x1000, 0x13FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .add_window(0x1400, 0x17FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .add_window(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .add_window(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .build();

    static ref CHR_LAYOUT_SMALL_WINDOWS_FIRST: ChrLayout = ChrLayout::builder()
        .max_bank_count(256)
        .bank_size(1 * KIBIBYTE)
        .add_window(0x0000, 0x07FF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .add_window(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .add_window(0x1000, 0x13FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .add_window(0x1400, 0x17FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .add_window(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .add_window(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .build();
}

// MMC3 and MMC6
pub struct Mapper4 {
    //registers: Registers,
    prg_swap_window: PrgSwapWindow,

    prg_memory: PrgMemory,
    chr_memory: ChrMemory,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper4 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper4, String> {
        Ok(Mapper4 {
            //registers: Registers::new(),
            prg_swap_window: PrgSwapWindow::Ox8000,

            prg_memory: PrgMemory::new(PRG_LAYOUT.clone(), cartridge.prg_rom()),
            chr_memory: ChrMemory::new(CHR_LAYOUT_BIG_WINDOWS_FIRST.clone(), cartridge.chr_rom()),
            name_table_mirroring: cartridge.name_table_mirroring(),
        })
    }

    fn bank_select(&mut self, value: u8) {
        /*
        let chr_big_windows_first =               (value & 0b1000_0000) == 0;
        let prg_8000_swappable =                  (value & 0b0100_0000) == 0;
        self.selected_bank = SelectedBank::from_u8(value & 0b0000_0111).unwrap();
        if chr_big_windows_first {
            self.chr_memory.set_layout(CHR_LAYOUT_BIG_WINDOWS_FIRST.clone())
        } else {
            self.chr_memory.set_layout(CHR_LAYOUT_SMALL_WINDOWS_FIRST.clone())
        }

        if prg_8000_swappable {
            self.prg_swap_window = PrgSwapWindow::Ox8000;
        } else {
            self.prg_swap_window = PrgSwapWindow::OxC000;
        };

        self.prg_memory
            .window_at(self.prg_swap_window as u16)
            .switch_bank_to(self.registers.selected_bank_index());
        self.prg_memory
            .window_at(self.prg_swap_window.opposing_fixed_window() as u16)
            .switch_bank_to(BankIndex::SecondLast);

        */
    }

    fn set_bank_index(&mut self, value: u8) {

    }

    fn set_mirroring(&mut self, value: u8) {

    }

    fn prg_ram_protect(&mut self, value: u8) {

    }

    fn set_irq_reload_value(&mut self, value: u8) {

    }

    fn reload_irq(&mut self) {

    }

    fn disable_irq(&mut self) {

    }

    fn enable_irq(&mut self) {

    }
}

impl Mapper for Mapper4 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        let is_even_address = address.to_raw() % 2 == 0;
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ },
            0x6000..=0x7FFF => self.prg_memory.write(address, value),
            0x8000..=0x9FFF if is_even_address => self.bank_select(value),
            0x8000..=0x9FFF => self.set_bank_index(value),
            0xA000..=0xBFFF if is_even_address => self.set_mirroring(value),
            0xA000..=0xBFFF => self.prg_ram_protect(value),
            0xC000..=0xDFFF if is_even_address => self.set_irq_reload_value(value),
            0xC000..=0xDFFF => self.reload_irq(),
            0xE000..=0xFFFF if is_even_address => self.disable_irq(),
            0xE000..=0xFFFF => self.enable_irq(),
        }
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

/*
struct Registers {
    selected_register_id: RegisterId,
}

impl Registers {
    fn new() -> Registers {
        Registers {
            selected_register: SelectedRegister::Chr0,
        }
    }

    fn selected_bank_index(&self) -> BankIndex {
        self.registers[self.selected_register_id as usize]
    }

    fn set_selected_bank_index(&mut self, bank_index: BankIndex) {
        self.registers[self.selected_register_id as usize] = bank_index;
    }

    fn set_selected_register_id(&mut self, register_id: RegisterId) {
        self.selected_register_id = register_id;
    }
}
*/

#[derive(FromPrimitive)]
enum RegisterId {
    Chr0,
    Chr1,
    Chr2,
    Chr3,
    Chr4,
    Chr5,
    Prg0,
    Prg1,
}

#[derive(Clone, Copy)]
enum PrgSwapWindow {
    Ox8000 = 0x8000,
    OxC000 = 0xC000,
}

impl PrgSwapWindow {
    fn opposing_fixed_window(self) -> PrgSwapWindow {
        use PrgSwapWindow::*;
        match self {
            Ox8000 => OxC000,
            OxC000 => Ox8000,
        }
    }
}
