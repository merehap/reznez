use crate::memory::bank_index::{BankIndex, BankRegisters, BankRegisterId, MetaRegisterId};

use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::writability::Writability;
use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
use crate::util::unit::KIBIBYTE;

pub struct ChrMemory {
    windows: ChrLayout,
    max_bank_count: u16,
    bank_size: usize,
    align_large_chr_windows: bool,
    override_write_protection: bool,
    raw_memory: Vec<u8>,
}

impl ChrMemory {
    pub fn new(
        windows: ChrLayout,
        max_bank_count: u16,
        bank_size: usize,
        align_large_chr_windows: bool,
        mut raw_memory: Vec<u8>,
    ) -> ChrMemory {
        windows.validate_bank_size_multiples(bank_size);
        // If no CHR data is provided, add 8KiB of CHR RAM and allow writing to read-only windows.
        let mut override_write_protection = false;
        if raw_memory.is_empty() {
            raw_memory = vec![0; 8 * KIBIBYTE];
            override_write_protection = true;
        }

        let chr_memory = ChrMemory {
            windows,
            max_bank_count,
            bank_size,
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
        self.windows.0.len().try_into().unwrap()
    }

    pub fn peek(&self, registers: &BankRegisters, address: PpuAddress) -> u8 {
        let (index, _) = self.address_to_chr_index(registers, address.to_u16());
        self.raw_memory[index]
    }

    pub fn write(&mut self, registers: &BankRegisters, address: PpuAddress, value: u8) {
        let (index, writable) = self.address_to_chr_index(registers, address.to_u16());
        if writable || self.override_write_protection {
            self.raw_memory[index] = value;
        }
    }

    pub fn resolve_selected_bank_indexes(&self, registers: &BankRegisters) -> Vec<u16> {
        self.windows.0.iter()
            .map(|window| window.bank_index(registers).to_u16(self.bank_count()))
            .collect()
    }

    pub fn window_at(&self, start: u16) -> &ChrWindow {
        for window in self.windows.0 {
            if window.start.to_u16() == start {
                return window;
            }
        }

        panic!("No window exists at {start:X?}");
    }

    pub fn set_layout(&mut self, windows: ChrLayout) {
        windows.validate_bank_size_multiples(self.bank_size);
        self.windows = windows;
    }

    pub fn pattern_table(&self, registers: &BankRegisters, side: PatternTableSide) -> PatternTable {
        match side {
            PatternTableSide::Left => PatternTable::new(self.left_chunks(registers)),
            PatternTableSide::Right => PatternTable::new(self.right_chunks(registers)),
        }
    }

    fn address_to_chr_index(&self, registers: &BankRegisters, address: u16) -> (usize, bool) {
        assert!(address < 0x2000);

        for window in self.windows.0 {
            if let Some(bank_offset) = window.offset(address) {
                let mut raw_bank_index = window.bank_index(registers).to_usize(self.bank_count());
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
    fn left_chunks(&self, registers: &BankRegisters) -> [&[u8]; 4] {
        self.left_indexes(registers)
            .map(|index| &self.raw_memory[index..index + 0x400])
    }

    #[inline]
    fn right_chunks(&self, registers: &BankRegisters) -> [&[u8]; 4] {
        self.right_indexes(registers)
            .map(|index| &self.raw_memory[index..index + 0x400])
    }

    #[inline]
    fn left_indexes(&self, registers: &BankRegisters) -> [usize; 4] {
        [
            self.address_to_chr_index(registers, 0x0000).0,
            self.address_to_chr_index(registers, 0x0400).0,
            self.address_to_chr_index(registers, 0x0800).0,
            self.address_to_chr_index(registers, 0x0C00).0,
        ]
    }

    #[inline]
    fn right_indexes(&self, registers: &BankRegisters) -> [usize; 4] {
        [
            self.address_to_chr_index(registers, 0x1000).0,
            self.address_to_chr_index(registers, 0x1400).0,
            self.address_to_chr_index(registers, 0x1800).0,
            self.address_to_chr_index(registers, 0x1C00).0,
        ]
    }
}

#[derive(Clone, Copy)]
pub struct ChrLayout(&'static [ChrWindow]);

impl ChrLayout {
    pub const fn new(windows: &'static [ChrWindow]) -> ChrLayout {
        assert!(!windows.is_empty(), "No PRG windows specified.");

        assert!(windows[0].start.to_u16() == 0x0000, "The first CHR window must start at 0x0000.");

        assert!(windows[windows.len() - 1].end.to_u16() == 0x1FFF, "The last CHR window must end at 0x1FFF.");

        let mut i = 1;
        while i < windows.len() {
            assert!(windows[i].start.to_u16() == windows[i - 1].end.to_u16() + 1,
                    "There must be no gaps nor overlap between CHR windows.");

            i += 1;
        }

        ChrLayout(windows)
    }

    pub fn active_register_ids(&self) -> Vec<BankRegisterId> {
        self.0.iter()
            .filter_map(|window| window.register_id())
            .collect()
    }

    const fn validate_bank_size_multiples(&self, bank_size: usize) {
        let mut i = 0;
        while i < self.0.len() {
            let window = self.0[i];
            assert!(window.size() % bank_size == 0, "Window size must be a multiple of bank size.");
            i += 1;
        }
    }
}

// TODO: Switch over to PpuAddress?
#[derive(Clone, Copy, Debug)]
pub struct ChrWindow {
    start: PpuAddress,
    end: PpuAddress,
    chr_type: ChrBank,
    write_status: Option<WriteStatus>,
}

impl ChrWindow {
    #[allow(clippy::identity_op)]
    pub const fn new(start: u16, end: u16, size: usize, chr_type: ChrBank) -> ChrWindow {
        //assert!([1 * KIBIBYTE, 2 * KIBIBYTE, 4 * KIBIBYTE, 8 * KIBIBYTE].contains(&size));
        assert!(end > start);
        assert!(end as usize - start as usize + 1 == size);

        ChrWindow {
            start: PpuAddress::from_u16(start),
            end: PpuAddress::from_u16(end),
            chr_type,
            write_status: None,
        }
    }

    const fn size(self) -> usize {
        (self.end.to_u16() - self.start.to_u16() + 1) as usize
    }

    fn offset(self, address: u16) -> Option<u16> {
        if self.start.to_u16() <= address && address <= self.end.to_u16() {
            Some(address - self.start.to_u16())
        } else {
            None
        }
    }

    fn bank_index(self, registers: &BankRegisters) -> BankIndex {
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

    pub fn register_id(self) -> Option<BankRegisterId> {
        if let ChrBank::Switchable(_, id) = self.chr_type {
            Some(id)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChrBank {
    Fixed(Writability, BankIndex),
    Switchable(Writability, BankRegisterId),
    MetaSwitchable(Writability, MetaRegisterId),
}

impl ChrBank {
    fn writability(self) -> Writability {
        match self {
            ChrBank::Fixed(writability, _) => writability,
            ChrBank::Switchable(writability, _) => writability,
            ChrBank::MetaSwitchable(writability, _) => writability,
        }
    }

    fn bank_index(self, registers: &BankRegisters) -> BankIndex {
        match self {
            ChrBank::Fixed(_, bank_index) => bank_index,
            ChrBank::Switchable(_, register_id) => registers.get(register_id),
            ChrBank::MetaSwitchable(_, meta_id) => registers.get_from_meta(meta_id),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum WriteStatus {
    ReadOnly,
    Writable,
}
