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

    chr_bank_mode: ChrBankMode,
    selected_chr_bank0: u8,
    selected_chr_bank1: u8,

    prg_bank_mode: PrgBankMode,
    selected_prg_bank: u8,

    params: MapperParams,
}

impl Mapper for Mapper1 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        if matches!(address.to_raw(), 0x6000..=0x7FFF) {
            self.params.prg_memory.write(address, value);
            return;
        }

        if get_bit(value, 0) {
            self.shift = EMPTY_SHIFT_REGISTER;
            self.prg_bank_mode = PrgBankMode::FixedLast;
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
                0x8000..=0x9FFF => self.set_controls(self.shift),
                // FIXME: Handle cases for special boards.
                0xA000..=0xBFFF => self.selected_chr_bank0 = self.shift,
                // FIXME: Handle cases for special boards.
                0xC000..=0xDFFF => self.selected_chr_bank1 = self.shift,
                0xE000..=0xFFFF => {
                    self.selected_prg_bank = self.shift & 0b0_1111;
                    if self.shift & 0b1_0000 == 0 {
                        self.params.prg_memory.enable_work_ram(0x6000);
                    } else {
                        self.params.prg_memory.disable_work_ram(0x6000);
                    }
                }
            }

            self.shift = EMPTY_SHIFT_REGISTER;
        }

        match self.chr_bank_mode {
            ChrBankMode::Large => {
                self.params.chr_memory.set_layout(CHR_LAYOUT_8KIB_WINDOW.clone());
                self.params.chr_memory.window_at(0x0000).switch_bank_to(self.selected_chr_bank0 >> 1);
            }
            ChrBankMode::TwoSmall => {
                self.params.chr_memory.set_layout(CHR_LAYOUT_4KIB_WINDOWS.clone());
                self.params.chr_memory.window_at(0x0000).switch_bank_to(self.selected_chr_bank0);
                self.params.chr_memory.window_at(0x1000).switch_bank_to(self.selected_chr_bank1);
            }
        }

        match self.prg_bank_mode {
            PrgBankMode::Large => {
                self.params.prg_memory.set_layout(PRG_LAYOUT_32KIB_WINDOW.clone());
                self.params.prg_memory.window_at(0x8000).switch_bank_to(self.selected_prg_bank >> 1);
            }
            PrgBankMode::FixedFirst => {
                self.params.prg_memory.set_layout(PRG_LAYOUT_16KIB_WINDOWS.clone());
                self.params.prg_memory.window_at(0x8000).switch_bank_to(BankIndex::FIRST);
                self.params.prg_memory.window_at(0xC000).switch_bank_to(self.selected_prg_bank);
            }
            PrgBankMode::FixedLast => {
                self.params.prg_memory.set_layout(PRG_LAYOUT_16KIB_WINDOWS.clone());
                self.params.prg_memory.window_at(0x8000).switch_bank_to(self.selected_prg_bank);
                self.params.prg_memory.window_at(0xC000).switch_bank_to(BankIndex::LAST);
            }
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper1 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper1, String> {
        let params = MapperParams::new(
            cartridge,
            PRG_LAYOUT_16KIB_WINDOWS.clone(),
            CHR_LAYOUT_4KIB_WINDOWS.clone(),
            NameTableMirroring::OneScreenRightBank,
        );
        Ok(Mapper1 {
            shift: EMPTY_SHIFT_REGISTER,

            chr_bank_mode: ChrBankMode::Large,
            selected_chr_bank0: 0,
            selected_chr_bank1: 0,

            prg_bank_mode: PrgBankMode::FixedLast,
            selected_prg_bank: 0,

            params,
        })
    }

    #[rustfmt::skip]
    fn set_controls(&mut self, value: u8) {
        self.chr_bank_mode = if get_bit(value, 3) {
            ChrBankMode::TwoSmall
        } else {
            ChrBankMode::Large
        };
        self.prg_bank_mode =
            match (get_bit(value, 4), get_bit(value, 5)) {
                (false, _    ) => PrgBankMode::Large,
                (true , false) => PrgBankMode::FixedFirst,
                (true , true ) => PrgBankMode::FixedLast,
            };
        self.params.name_table_mirroring =
            match (get_bit(value, 6), get_bit(value, 7)) {
                (false, false) => NameTableMirroring::OneScreenRightBank,
                (false, true ) => NameTableMirroring::OneScreenLeftBank,
                (true , false) => NameTableMirroring::Vertical,
                (true , true ) => NameTableMirroring::Horizontal,
            };
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
