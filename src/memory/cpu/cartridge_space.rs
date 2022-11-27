use crate::util::unit::KIBIBYTE;

const PRG_ROM_SIZE: usize = 32 * KIBIBYTE;
const MINIMUM_BANK_SIZE: usize = 8 * KIBIBYTE;

pub struct CartridgeSpace {
    prg_rom: Vec<u8>,
    bank_count: u8,
    selected_bank_indexes: Vec<u8>,
}

impl CartridgeSpace {
    pub fn single_bank(bank: Box<[u8; PRG_ROM_SIZE]>) -> CartridgeSpace {
        // Only a single bank and only a single index, which points at it.
        CartridgeSpace {
            prg_rom: bank.to_vec(),
            bank_count: 1,
            selected_bank_indexes: vec![0],
        }
    }

    pub fn single_bank_mirrored(bank: Box<[u8; PRG_ROM_SIZE / 2]>) -> CartridgeSpace {
        // Only a single bank that is half the size of indexable PRG ROM,
        // so two indexes are necessary, both which point to the only bank.
        CartridgeSpace {
            prg_rom: bank.to_vec(),
            bank_count: 1,
            selected_bank_indexes: vec![0, 0],
        }
    }

    pub fn multiple_banks(
        raw_bank_bytes: Vec<u8>,
        bank_count: u8,
        selected_bank_indexes: Vec<u8>,
    ) -> CartridgeSpace {
        assert!(bank_count > 0);
        for &bank_index in &selected_bank_indexes {
            assert!(bank_index < bank_count);
        }

        assert_eq!(raw_bank_bytes.len() % usize::from(bank_count), 0);
        assert_eq!(raw_bank_bytes.len() % MINIMUM_BANK_SIZE, 0);

        CartridgeSpace { prg_rom: raw_bank_bytes, bank_count, selected_bank_indexes }
    }

    pub fn selected_bank_indexes(&self) -> &[u8] {
        &self.selected_bank_indexes
    }

    pub fn select_new_banks(&mut self, selected_bank_indexes: Vec<u8>) {
        assert_eq!(self.selected_bank_indexes.len(), selected_bank_indexes.len());
        assert_eq!(PRG_ROM_SIZE % selected_bank_indexes.len(), 0);
        for &bank_index in &selected_bank_indexes {
            assert!(bank_index < self.bank_count);
        }

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
