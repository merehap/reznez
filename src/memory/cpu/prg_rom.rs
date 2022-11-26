// 32 KiB
const PRG_ROM_SIZE: usize = 32 * 0x400;
// 8 KiB
const MINIMUM_BANK_SIZE: usize = 8 * 0x400;

pub struct PrgRom {
    prg_rom: Vec<u8>,
    selected_bank_indexes: Vec<u8>,
}

impl PrgRom {
    pub fn single_bank(bank: Box<[u8; PRG_ROM_SIZE]>) -> PrgRom {
        // Only a single bank and only a single index, which points at it.
        let prg_rom = bank.to_vec();
        let selected_bank_indexes = vec![0];
        PrgRom { prg_rom, selected_bank_indexes }
    }

    pub fn single_bank_mirrored(bank: Box<[u8; PRG_ROM_SIZE / 2]>) -> PrgRom {
        // Only a single bank that is half the size of indexable PRG ROM,
        // so two indexes are necessary, both which point to the only bank.
        let prg_rom = bank.to_vec();
        let selected_bank_indexes = vec![0, 0];
        PrgRom { prg_rom, selected_bank_indexes }
    }

    pub fn multiple_banks(
        raw_bank_bytes: Vec<u8>,
        bank_count: u8,
        selected_bank_indexes: Vec<u8>,
    ) -> PrgRom {
        assert!(bank_count > 0);
        for &bank_index in &selected_bank_indexes {
            assert!(bank_index < bank_count);
        }

        let bank_count = usize::from(bank_count);
        assert_eq!(raw_bank_bytes.len() % bank_count, 0);
        assert_eq!(raw_bank_bytes.len() % MINIMUM_BANK_SIZE, 0);

        PrgRom { prg_rom: raw_bank_bytes, selected_bank_indexes }
    }

    pub fn selected_bank_indexes(&self) -> &[u8] {
        &self.selected_bank_indexes
    }

    pub fn select_new_banks(&mut self, selected_bank_indexes: Vec<u8>) {
        assert_eq!(self.selected_bank_indexes.len(), selected_bank_indexes.len());
        self.selected_bank_indexes = selected_bank_indexes;
    }

    pub fn read(&self, index: usize) -> u8 {
        let index = self.convert_to_internal_index(index);
        self.prg_rom[index]
    }

    pub fn write(&mut self, index: usize, value: u8) {
        let index = self.convert_to_internal_index(index);
        self.prg_rom[index] = value;
    }

    fn convert_to_internal_index(&self, external_index: usize) -> usize {
        assert!(external_index < PRG_ROM_SIZE);

        let placement  = external_index / self.bank_size();
        let bank_index = usize::from(self.selected_bank_indexes[placement]);
        let byte_index = external_index % self.bank_size();
        self.bank_size() * bank_index + byte_index
    }

    fn bank_size(&self) -> usize {
        PRG_ROM_SIZE / self.selected_bank_indexes.len()
    }
}
