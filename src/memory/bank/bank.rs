use crate::memory::bank::bank_index::{BankIndex, BankRegisters, BankRegisterId, MetaRegisterId};

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
        use Bank::*;
        use Location::*;
        match self {
            Rom(Fixed(bank_index)) | Ram(Fixed(bank_index), _) => Some(bank_index),
            Rom(Switchable(register_id)) | Ram(Switchable(register_id), _) => Some(registers.get(register_id)),
            Rom(MetaSwitchable(_)) | Ram(MetaSwitchable(_), _) => todo!(),
            Empty | WorkRam(_) | MirrorOf(_) => None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Location {
    Fixed(BankIndex),
    Switchable(BankRegisterId),
    MetaSwitchable(MetaRegisterId),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RamStatusRegisterId {
    S0,
    S1,
}
