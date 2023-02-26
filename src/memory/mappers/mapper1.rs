use crate::memory::mapper::*;
use crate::util::bit_util::get_bit;

const EMPTY_SHIFT_REGISTER: u8 = 0b0001_0000;

lazy_static! {
    static ref PRG_LAYOUT_16KIB_WINDOWS: PrgLayout = PrgLayout::builder()
        .max_bank_count(16)
        .bank_size(16 * KIBIBYTE)
        .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::WorkRam)
        .add_window(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::FIRST))
        .add_window(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::LAST))
        .build();
    static ref PRG_LAYOUT_32KIB_WINDOW: PrgLayout = PrgLayout::builder()
        .max_bank_count(8)
        .bank_size(32 * KIBIBYTE)
        .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::WorkRam)
        .add_window(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::FIRST))
        .build();

    // TODO: Not all boards support CHR RAM.
    static ref CHR_LAYOUT_4KIB_WINDOWS: ChrLayout = ChrLayout::builder()
        .max_bank_count(32)
        .bank_size(4 * KIBIBYTE)
        .add_window(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrType(Ram, BankIndex::FIRST))
        .add_window(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrType(Ram, BankIndex::FIRST))
        .build();
    static ref CHR_LAYOUT_8KIB_WINDOW: ChrLayout = ChrLayout::builder()
        .max_bank_count(16)
        .bank_size(8 * KIBIBYTE)
        .add_window(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType(Ram, BankIndex::FIRST))
        .build();
}

// SxROM (MMC1)
// TODO: Migrate to using bank index registers?
pub struct Mapper1 {
    shift: u8,
    control: Control,
    selected_chr_bank0: u8,
    selected_chr_bank1: u8,
    selected_prg_bank: u8,

    prg_memory: PrgMemory,
    chr_memory: ChrMemory,
}

impl Mapper for Mapper1 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        if matches!(address.to_raw(), 0x6000..=0x7FFF) {
            self.prg_memory.write(address, value);
            return;
        }

        if get_bit(value, 0) {
            self.shift = EMPTY_SHIFT_REGISTER;
            self.control.prg_bank_mode = PrgBankMode::FixedLast;
            return;
        }

        let is_last_shift = get_bit(self.shift, 7);

        self.shift >>= 1;
        self.shift |= u8::from(get_bit(value, 7)) << 4;

        if is_last_shift {
            match address.to_raw() {
                0x0000..=0x401F => unreachable!(),
                0x4020..=0x5FFF => { /* Do nothing. */ }
                0x6000..=0x7FFF => unreachable!(),
                0x8000..=0x9FFF => self.control = Control::from_u8(self.shift),
                // FIXME: Handle cases for special boards.
                0xA000..=0xBFFF => self.selected_chr_bank0 = self.shift,
                // FIXME: Handle cases for special boards.
                0xC000..=0xDFFF => self.selected_chr_bank1 = self.shift,
                0xE000..=0xFFFF => {
                    self.selected_prg_bank = self.shift & 0b0_1111;
                    if self.shift & 0b1_0000 == 0 {
                        self.prg_memory.enable_work_ram(0x6000);
                    } else {
                        self.prg_memory.disable_work_ram(0x6000);
                    }
                }
            }

            self.shift = EMPTY_SHIFT_REGISTER;
        }

        match self.control.chr_bank_mode {
            ChrBankMode::Large => {
                self.chr_memory.set_layout(CHR_LAYOUT_8KIB_WINDOW.clone());
                self.chr_memory.window_at(0x0000).switch_bank_to(self.selected_chr_bank0 >> 1);
            }
            ChrBankMode::TwoSmall => {
                self.chr_memory.set_layout(CHR_LAYOUT_4KIB_WINDOWS.clone());
                self.chr_memory.window_at(0x0000).switch_bank_to(self.selected_chr_bank0);
                self.chr_memory.window_at(0x1000).switch_bank_to(self.selected_chr_bank1);
            }
        }

        match self.control.prg_bank_mode {
            PrgBankMode::Large => {
                self.prg_memory.set_layout(PRG_LAYOUT_32KIB_WINDOW.clone());
                self.prg_memory.window_at(0x8000).switch_bank_to(self.selected_prg_bank >> 1);
            }
            PrgBankMode::FixedFirst => {
                self.prg_memory.set_layout(PRG_LAYOUT_16KIB_WINDOWS.clone());
                self.prg_memory.window_at(0x8000).switch_bank_to(BankIndex::FIRST);
                self.prg_memory.window_at(0xC000).switch_bank_to(self.selected_prg_bank);
            }
            PrgBankMode::FixedLast => {
                self.prg_memory.set_layout(PRG_LAYOUT_16KIB_WINDOWS.clone());
                self.prg_memory.window_at(0x8000).switch_bank_to(self.selected_prg_bank);
                self.prg_memory.window_at(0xC000).switch_bank_to(BankIndex::LAST);
            }
        }
    }

    fn name_table_mirroring(&self) -> NameTableMirroring {
        self.control.mirroring
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

impl Mapper1 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper1, String> {
        Ok(Mapper1 {
            shift: EMPTY_SHIFT_REGISTER,
            control: Control::new(),
            selected_chr_bank0: 0,
            selected_chr_bank1: 0,
            selected_prg_bank: 0,
            prg_memory: PrgMemory::new(PRG_LAYOUT_16KIB_WINDOWS.clone(), cartridge.prg_rom()),
            chr_memory: ChrMemory::new(CHR_LAYOUT_4KIB_WINDOWS.clone(), cartridge.chr_rom()),
        })
    }
}

#[derive(Debug)]
struct Control {
    chr_bank_mode: ChrBankMode,
    prg_bank_mode: PrgBankMode,
    mirroring: NameTableMirroring,
}

impl Control {
    fn new() -> Control {
        Control {
            chr_bank_mode: ChrBankMode::Large,
            prg_bank_mode: PrgBankMode::FixedLast,
            mirroring: NameTableMirroring::OneScreenRightBank,
        }
    }

    #[rustfmt::skip]
    fn from_u8(value: u8) -> Control {
        Control {
            chr_bank_mode: if get_bit(value, 3) {
                ChrBankMode::TwoSmall
            } else {
                ChrBankMode::Large
            },
            prg_bank_mode:
                match (get_bit(value, 4), get_bit(value, 5)) {
                    (false, _    ) => PrgBankMode::Large,
                    (true , false) => PrgBankMode::FixedFirst,
                    (true , true ) => PrgBankMode::FixedLast,
                },
            mirroring:
                match (get_bit(value, 6), get_bit(value, 7)) {
                    (false, false) => NameTableMirroring::OneScreenRightBank,
                    (false, true ) => NameTableMirroring::OneScreenLeftBank,
                    (true , false) => NameTableMirroring::Vertical,
                    (true , true ) => NameTableMirroring::Horizontal,
                },
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum ChrBankMode {
    Large,
    TwoSmall,
}

#[derive(PartialEq, Eq, Debug)]
enum PrgBankMode {
    Large,
    FixedFirst,
    FixedLast,
}
