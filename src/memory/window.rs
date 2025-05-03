use std::num::NonZeroU16;

use crate::memory::bank::bank::{PrgBank, PrgBankLocation};
use crate::memory::bank::bank_index::{PrgBankRegisters, PrgBankRegisterId};

use crate::mapper::{BankIndex, ReadWriteStatus, ReadWriteStatusRegisterId};

use super::bank::bank::{ChrBank, ChrBankLocation};
use super::bank::bank_index::ChrBankRegisterId;

// A Window is a range within addressable memory.
// If the specified bank cannot fill the window, adjacent banks will be included too.
#[derive(Clone, Copy, Debug)]
pub struct PrgWindow {
    start: u16,
    end: NonZeroU16,
    size: NonZeroU16,
    bank: PrgBank,
}

impl PrgWindow {
    pub const fn new(start: u16, end: u16, size: u32, bank: PrgBank) -> PrgWindow {
        assert!(end > start);
        let actual_size = end - start + 1;

        assert!(size < u16::MAX as u32, "Window size must be small enough to fit inside a u16.");
        let size = NonZeroU16::new(size as u16).expect("Window size to not be zero.");
        assert!(actual_size == size.get());

        let end = NonZeroU16::new(end).expect("Window end index to not be zero.");
        PrgWindow { start, end, size, bank }
    }

    pub const fn start(self) -> u16 {
        self.start
    }

    pub const fn end(self) -> NonZeroU16 {
        self.end
    }

    pub const fn size(self) -> NonZeroU16 {
        self.size
    }

    pub const fn bank(self) -> PrgBank {
        self.bank
    }

    pub fn is_in_bounds(self, address: u16) -> bool {
        self.start <= address && address <= self.end.get()
    }

    pub fn location(self) -> Result<PrgBankLocation, String> {
        match self.bank {
            PrgBank::Rom(location, _) | PrgBank::Ram(location, _)  | PrgBank::RomRam(location, _, _) | PrgBank::WorkRam(location, _) => Ok(location),
            PrgBank::Empty | PrgBank::MirrorOf(_) =>
                Err(format!("Bank type {:?} does not have a bank location.", self.bank)),
        }
    }

    pub const fn register_id(self) -> Option<PrgBankRegisterId> {
        if let PrgBank::Rom(PrgBankLocation::Switchable(id), _) | PrgBank::Ram(PrgBankLocation::Switchable(id), _) = self.bank {
            Some(id)
        } else {
            None
        }
    }
    pub fn read_write_status_info(self) -> ReadWriteStatusInfo {
        match self.bank {
            PrgBank::Ram(_, Some(register_id)) | PrgBank::RomRam(_, register_id, _) =>
                ReadWriteStatusInfo::PossiblyPresent { register_id, status_on_absent: ReadWriteStatus::ReadOnly },
            PrgBank::WorkRam(_, Some(register_id)) =>
                ReadWriteStatusInfo::PossiblyPresent { register_id, status_on_absent: ReadWriteStatus::Disabled },
            PrgBank::Empty | PrgBank::Rom(..) | PrgBank::MirrorOf(..) | PrgBank::Ram(..) | PrgBank::WorkRam(..) =>
                ReadWriteStatusInfo::Absent,
        }
    }

    pub fn offset(self, address: u16) -> Option<u16> {
        if self.start <= address && address <= self.end.get() {
            Some(address - self.start)
        } else {
            None
        }
    }

    pub fn is_writable(self, registers: &PrgBankRegisters) -> bool {
        self.bank.is_writable(registers)
    }
}

pub enum ReadWriteStatusInfo {
    Absent,
    PossiblyPresent { register_id: ReadWriteStatusRegisterId, status_on_absent: ReadWriteStatus },
    MapperCustom { register_id: ReadWriteStatusRegisterId },
}

#[derive(Clone, Copy, Debug)]
pub struct ChrWindow {
    start: u16,
    end: NonZeroU16,
    size: NonZeroU16,
    bank: ChrBank,
}

impl ChrWindow {
    pub const fn new(start: u16, end: u16, size: u32, bank: ChrBank) -> Self {
        assert!(end > start);
        let actual_size = end - start + 1;

        assert!(size < u16::MAX as u32, "Window size must be small enough to fit inside a u16.");
        let size = NonZeroU16::new(size as u16).expect("Window size to not be zero.");
        assert!(actual_size == size.get());

        let end = NonZeroU16::new(end).expect("Window end index to not be zero.");
        Self { start, end, size, bank }
    }

    pub const fn start(self) -> u16 {
        self.start
    }

    pub const fn end(self) -> NonZeroU16 {
        self.end
    }

    pub const fn size(self) -> NonZeroU16 {
        self.size
    }

    pub const fn bank(self) -> ChrBank {
        self.bank
    }

    pub fn is_in_bounds(self, address: u16) -> bool {
        self.start <= address && address <= self.end.get()
    }

    pub fn location(self) -> Result<ChrBankLocation, String> {
        match self.bank {
            ChrBank::Rom(location, _) | ChrBank::Ram(location, _) => Ok(location),
            ChrBank::SaveRam(_) => Ok(ChrBankLocation::Fixed(BankIndex::from_u8(0))),
        }
    }

    pub const fn register_id(self) -> Option<ChrBankRegisterId> {
        if let ChrBank::Rom(ChrBankLocation::Switchable(id), _) | ChrBank::Ram(ChrBankLocation::Switchable(id), _) = self.bank {
            Some(id)
        } else {
            None
        }
    }
    pub fn read_write_status_info(self) -> ReadWriteStatusInfo {
        match self.bank {
            ChrBank::Ram(_, Some(register_id)) =>
                ReadWriteStatusInfo::PossiblyPresent { register_id, status_on_absent: ReadWriteStatus::ReadOnly },
            // TODO: SaveRam will probably need to support status registers.
            ChrBank::SaveRam(..) =>
                ReadWriteStatusInfo::Absent,
            ChrBank::Rom(..) | ChrBank::Ram(..) =>
                ReadWriteStatusInfo::Absent,
        }
    }

    pub fn offset(self, address: u16) -> Option<u16> {
        if self.start <= address && address <= self.end.get() {
            Some(address - self.start)
        } else {
            None
        }
    }

    pub fn is_writable(self, registers: &PrgBankRegisters) -> bool {
        self.bank.is_writable(registers)
    }
}