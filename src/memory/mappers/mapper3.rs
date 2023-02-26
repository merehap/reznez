use crate::memory::mapper::*;

lazy_static! {
    // Same as NROM's PRG layouts. Only one bank, so not bank-switched.
    static ref PRG_LAYOUT_CNROM_128: PrgLayout = PrgLayout::builder()
        .max_bank_count(1)
        .bank_size(16 * KIBIBYTE)
        .window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
        .window(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::FIRST))
        .window(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Mirror(0x8000))
        .build();
    static ref PRG_LAYOUT_CNROM_256: PrgLayout = PrgLayout::builder()
        .max_bank_count(1)
        .bank_size(32 * KIBIBYTE)
        .window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
        .window(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::FIRST))
        .build();

    static ref CHR_LAYOUT: ChrLayout = ChrLayout::builder()
        .max_bank_count(256)
        .bank_size(8 * KIBIBYTE)
        .window(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C0)))
        .build();
}

// CNROM
pub struct Mapper3 {
    params: MapperParams,
}

impl Mapper for Mapper3 {
    fn write_to_cartridge_space(&mut self, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => self.params.chr_memory.set_bank_index_register(C0, value),
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper3 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper3, String> {
        let prg_layout = match Mapper3::board(cartridge)? {
            Board::Cnrom128 => PRG_LAYOUT_CNROM_128.clone(),
            Board::Cnrom256 => PRG_LAYOUT_CNROM_256.clone(),
        };

        let params = MapperParams::new(
            cartridge,
            prg_layout,
            CHR_LAYOUT.clone(),
            cartridge.name_table_mirroring(),
        );
        Ok(Mapper3 { params })
    }

    fn board(cartridge: &Cartridge) -> Result<Board, String> {
        let prg_rom_len = cartridge.prg_rom().len();
        if prg_rom_len == 16 * KIBIBYTE {
            Ok(Board::Cnrom128)
        } else if prg_rom_len == 32 * KIBIBYTE {
            Ok(Board::Cnrom256)
        } else {
            Err("PRG ROM size must be 16K or 32K for mapper 0.".to_string())
        }
    }
}

enum Board {
    Cnrom128,
    Cnrom256,
}
