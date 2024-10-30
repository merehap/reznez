use crate::memory::bank::bank::{Bank, Location};
use crate::memory::bank::bank_index::{BankRegisters, BankRegisterId};

// A Window is a range within addressable memory.
// If the specified bank cannot fill the window, adjacent banks will be included too.
#[derive(Clone, Copy, Debug)]
pub struct Window {
    start: u16,
    end: u16,
    bank: Bank,
}

impl Window {
    pub const fn new(start: u16, end: u16, size: u32, bank: Bank) -> Window {
        assert!(end > start);
        assert!(end as u32 - start as u32 + 1 == size);

        Window { start, end, bank }
    }

    pub fn bank_string(
        &self,
        registers: &BankRegisters,
        bank_size: u16,
        bank_count: u16,
        align_large_layouts: bool,
    ) -> String {
        match self.bank {
            Bank::Empty => "E".into(),
            Bank::WorkRam(_) => "W".into(),
            Bank::Rom(location) | Bank::Ram(location, _) =>
                self.resolved_bank_index(
                    registers,
                    location,
                    bank_size,
                    bank_count,
                    align_large_layouts,
                ).to_string(),
            Bank::MirrorOf(_) => "M".into(),
        }
    }

    pub fn resolved_bank_index(
        &self,
        registers: &BankRegisters,
        location: Location,
        bank_size: u16,
        bank_count: u16,
        align_large_layouts: bool,
    ) -> u16 {
        let stored_bank_index = match location {
            Location::Fixed(bank_index) => bank_index,
            Location::Switchable(register_id) => registers.get(register_id),
            Location::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id),
        };

        let mut resolved_bank_index = stored_bank_index.to_u16(bank_count);
        if align_large_layouts {
            let window_multiple = self.size() / bank_size;
            // Clear low bits for large windows.
            resolved_bank_index &= !(window_multiple - 1);
        }

        resolved_bank_index
    }

    pub const fn start(self) -> u16 {
        self.start
    }

    pub const fn end(self) -> u16 {
        self.end
    }

    pub const fn size(self) -> u16 {
        self.end - self.start + 1
    }

    pub const fn bank(self) -> Bank {
        self.bank
    }

    pub fn location(self) -> Result<Location, String> {
        match self.bank {
            Bank::Rom(location) | Bank::Ram(location, _) => Ok(location),
            Bank::Empty | Bank::WorkRam(_) | Bank::MirrorOf(_) =>
                Err(format!("Bank type {:?} does not have a bank location.", self.bank)),
        }
    }

    pub const fn register_id(self) -> Option<BankRegisterId> {
        if let Bank::Rom(Location::Switchable(id)) | Bank::Ram(Location::Switchable(id), _) = self.bank {
            Some(id)
        } else {
            None
        }
    }

    pub fn offset(self, address: u16) -> Option<u16> {
        if self.start <= address && address <= self.end {
            Some(address - self.start)
        } else {
            None
        }
    }

    pub fn is_writable(self, registers: &BankRegisters) -> bool {
        self.bank.is_writable(registers)
    }
}

/*
impl Window {
    pub fn bank_string(
        &self,
        registers: &BankRegisters,
        bank_size: u16,
        bank_count: u16,
        align_large_layouts: bool,
    ) -> String {
        match self.bank {
            Bank::Empty => "E".into(),
            Bank::WorkRam(_) => "W".into(),
            Bank::Rom(_) | Bank::Ram(_, _) =>
                self.resolved_bank_index(registers, bank_size, bank_count, align_large_layouts).to_string(),
            Bank::MirrorOf(_) => "M".into(),
        }
    }

    fn resolved_bank_index(
        &self,
        registers: &BankRegisters,
        bank_size: u16,
        bank_count: u16,
        align_large_layouts: bool,
    ) -> u16 {
        let stored_bank_index = self.bank_index(registers);

        let mut resolved_bank_index = stored_bank_index.to_u16(bank_count);
        if align_large_layouts {
            let window_multiple = self.size() / bank_size;
            resolved_bank_index &= !(window_multiple - 1);
        }

        resolved_bank_index
    }

    pub const fn size(self) -> u16  {
        self.end.to_u16() - self.start.to_u16() + 1
    }

    fn offset(self, address: u16) -> Option<u16> {
        if self.start.to_u16() <= address && address <= self.end.to_u16() {
            Some(address - self.start.to_u16())
        } else {
            None
        }
    }

    fn bank_index(self, registers: &BankRegisters) -> BankIndex {
        match self.bank {
            Bank::Rom(Location::Fixed(bank_index)) | Bank::Ram(Location::Fixed(bank_index), _) =>
                bank_index,
            Bank::Rom(Location::Switchable(id)) | Bank::Ram(Location::Switchable(id), _) =>
                registers.get(id),
            Bank::Rom(Location::MetaSwitchable(meta_id)) | Bank::Ram(Location::MetaSwitchable(meta_id), _) =>
                registers.get_from_meta(meta_id),
            Bank::Empty | Bank::WorkRam(_) | Bank::MirrorOf(_) =>
                panic!("Bank type {:?} is not allowed for CHR Windows.", self.bank),
        }
    }

    fn is_writable(self, registers: &BankRegisters) -> bool {
        self.bank.is_writable(registers)
    }

    pub fn register_id(self) -> Option<BankRegisterId> {
        if let Bank::Rom(Location::Switchable(id)) | Bank::Ram(Location::Switchable(id), _) = self.bank {
            Some(id)
        } else {
            None
        }
    }
}
*/
