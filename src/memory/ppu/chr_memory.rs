use crate::memory::bank_index::{BankIndex, BankIndexRegisters, BankIndexRegisterId};

use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::writability::Writability;
use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
use crate::util::unit::KIBIBYTE;

pub struct ChrMemory {
    windows: &'static [ChrWindow],
    max_bank_count: u16,
    bank_size: usize,
    align_large_chr_windows: bool,
    override_write_protection: bool,
    bank_index_registers: BankIndexRegisters,
    raw_memory: Vec<u8>,
}

impl ChrMemory {
    pub fn new(
        windows: &'static [ChrWindow],
        max_bank_count: u16,
        bank_size: usize,
        align_large_chr_windows: bool,
        bank_index_registers: BankIndexRegisters,
        mut raw_memory: Vec<u8>,
    ) -> ChrMemory {
        // If no CHR data is provided, add 8KiB of CHR RAM and allow writing to read-only windows.
        let mut override_write_protection = false;
        if raw_memory.is_empty() {
            raw_memory = vec![0; 8 * KIBIBYTE];
            override_write_protection = true;
        }

        assert!(!windows.is_empty());

        assert_eq!(windows[0].start, 0x0000);
        assert_eq!(windows[windows.len() - 1].end, 0x1FFF);

        for i in 0..windows.len() - 1 {
            assert_eq!(
                windows[i + 1].start,
                windows[i].end + 1,
            );
        }

        let chr_memory = ChrMemory {
            windows,
            max_bank_count,
            bank_size,
            bank_index_registers,
            align_large_chr_windows,
            override_write_protection,
            raw_memory,
        };

        let bank_count = chr_memory.bank_count();
        assert_eq!(usize::from(bank_count) * chr_memory.bank_size, chr_memory.raw_memory.len());
        // Power of 2.
        assert_eq!(bank_count & (bank_count - 1), 0);
        assert!(bank_count <= chr_memory.max_bank_count);

        chr_memory
    }

    #[inline]
    pub fn bank_count(&self) -> u16 {
        (self.raw_memory.len() / self.bank_size)
            .try_into()
            .expect("Way too many CHR banks.")
    }

    pub fn window_count(&self) -> u8 {
        self.windows.len().try_into().unwrap()
    }

    pub fn peek(&self, address: PpuAddress) -> u8 {
        let (index, _) = self.address_to_chr_index(address.to_u16());
        self.raw_memory[index]
    }

    pub fn write(&mut self, address: PpuAddress, value: u8) {
        let (index, writable) = self.address_to_chr_index(address.to_u16());
        if writable || self.override_write_protection {
            self.raw_memory[index] = value;
        }
    }

    pub fn resolve_selected_bank_indexes(&self) -> Vec<u16> {
        self.windows.iter()
            .map(|window| window.bank_index(&self.bank_index_registers).to_u16(self.bank_count()))
            .collect()
    }

    pub fn window_at(&self, start: u16) -> &ChrWindow {
        for window in self.windows {
            if window.start == start {
                return window;
            }
        }

        panic!("No window exists at {start:X?}");
    }

    pub fn set_windows(&mut self, windows: &'static [ChrWindow]) {
        let reg_ids: Vec<_> = windows.iter()
            .filter_map(|window| window.register_id())
            .collect();
        let new_bank_index_registers = BankIndexRegisters::new(&reg_ids);
        self.bank_index_registers.merge(&new_bank_index_registers);
        self.windows = windows;
    }

    pub fn set_bank_index_register<INDEX: Into<u16>>(
        &mut self,
        id: BankIndexRegisterId,
        raw_bank_index: INDEX,
    ) {
        let mut raw_bank_index = raw_bank_index.into();
        raw_bank_index %= self.bank_count();
        self.bank_index_registers.set(id, BankIndex::from_u16(raw_bank_index));
    }

    pub fn pattern_table(&self, side: PatternTableSide) -> PatternTable {
        match side {
            PatternTableSide::Left => PatternTable::new(self.left_chunks()),
            PatternTableSide::Right => PatternTable::new(self.right_chunks()),
        }
    }

    fn address_to_chr_index(&self, address: u16) -> (usize, bool) {
        assert!(address < 0x2000);

        for window in self.windows {
            if let Some(bank_offset) = window.offset(address) {
                let mut raw_bank_index = window.bank_index(&self.bank_index_registers)
                    .to_usize(self.bank_count());
                if self.align_large_chr_windows {
                    let window_multiple = window.size() / self.bank_size;
                    raw_bank_index &= !(window_multiple >> 1);
                }

                let index = raw_bank_index *
                    self.bank_size +
                    usize::from(bank_offset);
                return (index, window.is_writable());
            }
        }

        unreachable!();
    }

    #[inline]
    fn left_chunks(&self) -> [&[u8]; 4] {
        self.left_indexes()
            .map(|index| &self.raw_memory[index..index + 0x400])
    }

    #[inline]
    fn right_chunks(&self) -> [&[u8]; 4] {
        self.right_indexes()
            .map(|index| &self.raw_memory[index..index + 0x400])
    }

    #[inline]
    fn left_indexes(&self) -> [usize; 4] {
        [
            self.address_to_chr_index(0x0000).0,
            self.address_to_chr_index(0x0400).0,
            self.address_to_chr_index(0x0800).0,
            self.address_to_chr_index(0x0C00).0,
        ]
    }

    #[inline]
    fn right_indexes(&self) -> [usize; 4] {
        [
            self.address_to_chr_index(0x1000).0,
            self.address_to_chr_index(0x1400).0,
            self.address_to_chr_index(0x1800).0,
            self.address_to_chr_index(0x1C00).0,
        ]
    }
}

/*
#[derive(Clone)]
pub struct ChrLayout {
    max_bank_count: u16,
    bank_size: usize,
    windows: Vec<ChrWindow>,
}

impl ChrLayout {
    pub fn new(
        max_bank_count: u16,
        bank_size: usize,
        windows: Vec<ChrWindow>,
    ) -> ChrLayout {
        assert!(!windows.is_empty());

        assert_eq!(windows[0].start, 0x0000);
        assert_eq!(windows[windows.len() - 1].end, 0x1FFF);

        for i in 0..windows.len() - 1 {
            assert_eq!(
                windows[i + 1].start,
                windows[i].end + 1,
            );
        }

        assert_eq!(max_bank_count & (max_bank_count - 1), 0);
        ChrLayout { max_bank_count, bank_size, windows }
    }

    fn active_register_ids(&self) -> Vec<BankIndexRegisterId> {
        self.windows.iter()
            .filter_map(|window| window.register_id())
            .collect()
    }
}
*/

// TODO: Switch over to PpuAddress?
#[derive(Clone, Copy, Debug)]
pub struct ChrWindow {
    start: u16,
    end: u16,
    chr_type: ChrType,
    write_status: Option<WriteStatus>,
}

impl ChrWindow {
    #[allow(clippy::identity_op)]
    pub const fn new(start: u16, end: u16, size: usize, chr_type: ChrType) -> ChrWindow {
        //assert!([1 * KIBIBYTE, 2 * KIBIBYTE, 4 * KIBIBYTE, 8 * KIBIBYTE].contains(&size));
        assert!(end > start);
        if end as usize - start as usize + 1 != size {
            panic!("CHR window 'end - start != size'");
        }

        ChrWindow { start, end, chr_type, write_status: None }
    }

    fn size(self) -> usize {
        usize::from(self.end - self.start + 1)
    }

    fn offset(self, address: u16) -> Option<u16> {
        if self.start <= address && address <= self.end {
            Some(address - self.start)
        } else {
            None
        }
    }

    fn bank_index(self, registers: &BankIndexRegisters) -> BankIndex {
        self.chr_type.bank_index(registers)
    }

    fn is_writable(self) -> bool {
        match (self.chr_type.writability(), self.write_status) {
            (Writability::Rom   , None) => false,
            (Writability::Ram   , None) => true,
            (Writability::RomRam, Some(WriteStatus::ReadOnly)) => false,
            (Writability::RomRam, Some(WriteStatus::Writable)) => true,
            _ => unreachable!(),
        }
    }

    pub fn register_id(self) -> Option<BankIndexRegisterId> {
        if let ChrType::VariableBank(_, id) = self.chr_type {
            Some(id)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn make_writable(&mut self) {
        assert!(self.write_status.is_some(), "Only RamRom can have its WriteStatus changed.");
        self.write_status = Some(WriteStatus::Writable);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChrType {
    ConstantBank(Writability, BankIndex),
    VariableBank(Writability, BankIndexRegisterId),
}

impl ChrType {
    fn writability(self) -> Writability {
        match self {
            ChrType::ConstantBank(writability, _) => writability,
            ChrType::VariableBank(writability, _) => writability,
        }
    }

    fn bank_index(self, registers: &BankIndexRegisters) -> BankIndex {
        match self {
            ChrType::ConstantBank(_, bank_index) => bank_index,
            ChrType::VariableBank(_, register_id) => registers.get(register_id),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum WriteStatus {
    ReadOnly,
    Writable,
}
