use crate::memory::mapper::*;
use crate::util::bit_util::get_bit;

const EMPTY_SHIFT_REGISTER: u8 = 0b0001_0000;

lazy_static! {
    static ref PRG_LAYOUT_FIXED_LAST_WINDOW: PrgLayout = PrgLayout::builder()
        .max_bank_count(16)
        .bank_size(16 * KIBIBYTE)
        .window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::WorkRam)
        .window(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0)))
        .window(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::LAST))
        .build();
    static ref PRG_LAYOUT_FIXED_FIRST_WINDOW: PrgLayout = PrgLayout::builder()
        .max_bank_count(16)
        .bank_size(16 * KIBIBYTE)
        .window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::WorkRam)
        .window(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::FIRST))
        .window(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0)))
        .build();
    static ref PRG_LAYOUT_32KIB_WINDOW: PrgLayout = PrgLayout::builder()
        .max_bank_count(16)
        .bank_size(16 * KIBIBYTE)
        .window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::WorkRam)
        .window(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0)))
        .build();

    // TODO: Not all boards support CHR RAM.
    static ref CHR_LAYOUT_BIG_WINDOW: ChrLayout = ChrLayout::builder()
        .max_bank_count(32)
        .bank_size(4 * KIBIBYTE)
        .window(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType(Ram, BankIndex::Register(C0)))
        .build();
    static ref CHR_LAYOUT_TWO_SMALL_WINDOWS: ChrLayout = ChrLayout::builder()
        .max_bank_count(32)
        .bank_size(4 * KIBIBYTE)
        .window(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrType(Ram, BankIndex::Register(C0)))
        .window(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrType(Ram, BankIndex::Register(C1)))
        .build();
}

// SxROM (MMC1)
pub struct Mapper1 {
    shift: u8,
    params: MapperParams,
}

impl Mapper for Mapper1 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        // Work RAM writes don't trigger any of the shifter logic.
        if matches!(address.to_raw(), 0x6000..=0x7FFF) {
            self.prg_memory_mut().write(address, value);
            return;
        }

        if get_bit(value, 0) {
            self.shift = EMPTY_SHIFT_REGISTER;
            self.prg_memory_mut().set_layout(PRG_LAYOUT_FIXED_LAST_WINDOW.clone());
            return;
        }

        let is_last_shift = get_bit(self.shift, 7);

        self.shift >>= 1;
        self.shift |= u8::from(get_bit(value, 7)) << 4;

        if is_last_shift {
            let shift = self.shift;
            match address.to_raw() {
                0x0000..=0x401F => unreachable!(),
                0x4020..=0x5FFF => { /* Do nothing. */ }
                0x6000..=0x7FFF => unreachable!(),
                0x8000..=0x9FFF => {
                    self.prg_memory_mut().set_layout(Mapper1::next_prg_layout(shift));
                    self.chr_memory_mut().set_layout(Mapper1::next_chr_layout(shift));
                    self.set_name_table_mirroring(Mapper1::next_mirroring(shift));
                }
                // FIXME: Handle cases for special boards.
                0xA000..=0xBFFF => self.chr_memory_mut().set_bank_index_register(C0, shift),
                // FIXME: Handle cases for special boards.
                0xC000..=0xDFFF => self.chr_memory_mut().set_bank_index_register(C1, shift),
                0xE000..=0xFFFF => {
                    self.prg_memory_mut().set_bank_index_register(P0, shift & 0b0_1111);
                    if shift & 0b1_0000 == 0 {
                        self.prg_memory_mut().enable_work_ram(0x6000);
                    } else {
                        self.prg_memory_mut().disable_work_ram(0x6000);
                    }
                }
            }

            self.shift = EMPTY_SHIFT_REGISTER;
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper1 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper1, String> {
        Ok(Mapper1 {
            shift: EMPTY_SHIFT_REGISTER,
            params: MapperParams::new(
                cartridge,
                PRG_LAYOUT_FIXED_LAST_WINDOW.clone(),
                CHR_LAYOUT_BIG_WINDOW.clone(),
                NameTableMirroring::OneScreenRightBank,
            ),
        })
    }

    fn next_prg_layout(value: u8) -> PrgLayout {
        match (value & 0b0000_1100) >> 2 {
            0b00 | 0b01 => PRG_LAYOUT_32KIB_WINDOW.clone(),
            0b10 => PRG_LAYOUT_FIXED_FIRST_WINDOW.clone(),
            0b11 => PRG_LAYOUT_FIXED_LAST_WINDOW.clone(),
            _ => unreachable!(),
        }
    }

    fn next_chr_layout(value: u8) -> ChrLayout {
        match (value & 0b0001_0000) >> 4 {
            0 => CHR_LAYOUT_BIG_WINDOW.clone(),
            1 => CHR_LAYOUT_TWO_SMALL_WINDOWS.clone(),
            _ => unreachable!(),
        }
    }

    fn next_mirroring(value: u8) -> NameTableMirroring {
        match value & 0b0000_0011 {
            0b00 => NameTableMirroring::OneScreenRightBank,
            0b01 => NameTableMirroring::OneScreenLeftBank,
            0b10 => NameTableMirroring::Vertical,
            0b11 => NameTableMirroring::Horizontal,
            _ => unreachable!(),
        }
    }
}
