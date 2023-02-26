use crate::memory::mapper::*;

lazy_static! {
    // Only one bank, so not bank-switched.
    static ref PRG_LAYOUT_NROM_128: PrgLayout = PrgLayout::builder()
        .max_bank_count(1)
        .bank_size(16 * KIBIBYTE)
        .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
        .add_window(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::FIRST))
        .add_window(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Mirror(0x8000))
        .build();
    static ref PRG_LAYOUT_NROM_256: PrgLayout = PrgLayout::builder()
        .max_bank_count(1)
        .bank_size(32 * KIBIBYTE)
        .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
        .add_window(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::FIRST))
        .build();

    // Only one bank, so not bank-switched.
    static ref CHR_LAYOUT: ChrLayout = ChrLayout::builder()
        .max_bank_count(1)
        .bank_size(8 * KIBIBYTE)
        .add_window(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .build();
}

// NROM
pub struct Mapper0 {
    prg_memory: PrgMemory,
    chr_memory: ChrMemory,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper for Mapper0 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, _value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* Only mapper 0 does nothing here. */ },
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

impl Mapper0 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper0, String> {
        let prg_layout = match Mapper0::board(cartridge)? {
            Board::Nrom128 => PRG_LAYOUT_NROM_128.clone(),
            Board::Nrom256 => PRG_LAYOUT_NROM_256.clone(),
        };

        Ok(Mapper0 {
            prg_memory: PrgMemory::new(prg_layout, cartridge.prg_rom()),
            chr_memory: ChrMemory::new(CHR_LAYOUT.clone(), cartridge.chr_rom()),
            name_table_mirroring: cartridge.name_table_mirroring(),
        })
    }

    fn board(cartridge: &Cartridge) -> Result<Board, String> {
        let prg_rom_len = cartridge.prg_rom().len();
        if prg_rom_len == 16 * KIBIBYTE {
            Ok(Board::Nrom128)
        } else if prg_rom_len == 32 * KIBIBYTE {
            Ok(Board::Nrom256)
        } else {
            Err("PRG ROM size must be 16K or 32K for mapper 0.".to_string())
        }
    }
}

enum Board {
    Nrom128,
    Nrom256,
}
