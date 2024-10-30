use crate::memory::bank::bank_index::{BankIndex, BankRegisters, BankRegisterId, MetaRegisterId, RamStatus};

#[derive(Clone, Copy, Debug)]
pub enum Bank {
    Empty,
    WorkRam(Option<RamStatusRegisterId>),
    Rom(Location),
    Ram(Location, Option<RamStatusRegisterId>),
    MirrorOf(u16),
}

impl Bank {
    pub const EMPTY: Bank = Bank::Empty;
    pub const WORK_RAM: Bank = Bank::WorkRam(None);

    pub const fn fixed_rom(bank_index: BankIndex) -> Bank {
        Bank::Rom(Location::Fixed(bank_index))
    }

    pub const fn switchable_rom(id: BankRegisterId) -> Bank {
        Bank::Rom(Location::Switchable(id))
    }

    pub const fn meta_switchable_rom(id: MetaRegisterId) -> Bank {
        Bank::Rom(Location::MetaSwitchable(id))
    }

    pub const fn fixed_ram(bank_index: BankIndex) -> Bank {
        Bank::Ram(Location::Fixed(bank_index), None)
    }

    pub const fn switchable_ram(id: BankRegisterId) -> Bank {
        Bank::Ram(Location::Switchable(id), None)
    }

    pub const fn mirror_of(window_address: u16) -> Bank {
        Bank::MirrorOf(window_address)
    }

    pub const fn status_register(self, id: RamStatusRegisterId) -> Bank {
        match self {
            Bank::WorkRam(None) => Bank::WorkRam(Some(id)),
            Bank::Ram(location, None) => Bank::Ram(location, Some(id)),
            _ => panic!("Only RAM and Work RAM support status registers."),
        }
    }

    pub fn is_work_ram(self) -> bool {
        matches!(self, Bank::WorkRam(_))
    }

    pub fn bank_index(self, registers: &BankRegisters) -> Option<BankIndex> {
        if let Bank::Rom(location) | Bank::Ram(location, _) = self {
            Some(location.bank_index(registers))
        } else {
            None
        }
    }

    pub fn is_writable(self, registers: &BankRegisters) -> bool {
        match self {
            Bank::Empty => false,
            Bank::Rom(_) => false,
            Bank::MirrorOf(_) => todo!("Writability of MirrorOf"),
            // RAM with no status register is always writable.
            Bank::Ram(_, None) | Bank::WorkRam(None) => true,
            Bank::Ram(_, Some(status_register_id)) | Bank::WorkRam(Some(status_register_id)) =>
                registers.ram_status(status_register_id) == RamStatus::ReadWrite,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Location {
    Fixed(BankIndex),
    Switchable(BankRegisterId),
    MetaSwitchable(MetaRegisterId),
}

impl Location {
    pub fn bank_index(self, registers: &BankRegisters) -> BankIndex {
        match self {
            Self::Fixed(bank_index) => bank_index,
            Self::Switchable(register_id) => registers.get(register_id),
            Self::MetaSwitchable(_) => todo!(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RamStatusRegisterId {
    S0,
    S1,
}
