use std::num::NonZeroU16;

use crate::memory::bank::bank::{PrgBank, PrgBankLocation};
use crate::memory::bank::bank_index::{PrgBankRegisters, PrgBankRegisterId};

use crate::mapper::{BankIndex, ReadWriteStatus, ReadWriteStatusRegisterId, KIBIBYTE};

use super::bank::bank::{ChrBank, ChrBankLocation};
use super::bank::bank_index::ChrBankRegisterId;

// A Window is a range within addressable memory.
// If the specified bank cannot fill the window, adjacent banks will be included too.
#[derive(Clone, Copy, Debug)]
pub struct PrgWindow {
    start: PrgWindowStart,
    end: PrgWindowEnd,
    size: PrgWindowSize,
    bank: PrgBank,
}

impl PrgWindow {
    pub const fn new(start: u16, end: u16, size: u32, bank: PrgBank) -> PrgWindow {
        let start = PrgWindowStart::new(start);
        let end = PrgWindowEnd::new(end);
        let size = PrgWindowSize::new(size, start, end);
        PrgWindow { start, end, size, bank }
    }

    pub fn force_rom(self) -> Self {
        Self {
            bank: self.bank.as_rom(),
            .. self
        }
    }

    pub const fn start(self) -> u16 {
        self.start.0
    }

    pub const fn end(self) -> NonZeroU16 {
        self.end.0
    }

    pub const fn size(self) -> PrgWindowSize {
        self.size
    }

    pub const fn bank(self) -> PrgBank {
        self.bank
    }

    pub fn location(self) -> Result<PrgBankLocation, String> {
        use PrgBank::*;
        match self.bank {
            Rom(location, _) | Ram(location, _) | RomRam(location, _, _) | WorkRam(location, _) | SaveRam(location, _) => Ok(location),
            Empty => Err(format!("Empty banks {:?} don't have a bank location.", self.bank)),
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
            PrgBank::WorkRam(_, Some(register_id)) | PrgBank::SaveRam(_, Some(register_id)) =>
                ReadWriteStatusInfo::PossiblyPresent { register_id, status_on_absent: ReadWriteStatus::Disabled },
            PrgBank::Empty | PrgBank::Rom(..) | PrgBank::Ram(..) | PrgBank::WorkRam(..) | PrgBank::SaveRam(..) =>
                ReadWriteStatusInfo::Absent,
        }
    }

    pub fn offset(self, address: u16) -> Option<u16> {
        if self.start.0 <= address && address <= self.end.0.get() {
            Some(address - self.start.0)
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
    start: ChrWindowStart,
    end: ChrWindowEnd,
    size: ChrWindowSize,
    bank: ChrBank,
}

impl ChrWindow {
    pub const fn new(start: u16, end: u16, size: u32, bank: ChrBank) -> Self {
        let start = ChrWindowStart::new(start);
        let end = ChrWindowEnd::new(end);
        let size = ChrWindowSize::new(size, start, end);
        Self { start, end, size, bank }
    }

    pub fn force_rom(self) -> Self {
        Self {
            bank: self.bank.as_rom(),
            .. self
        }
    }

    pub fn force_ram(self) -> Self {
        Self {
            bank: self.bank.as_ram(),
            .. self
        }
    }

    pub const fn start(self) -> u16 {
        self.start.0
    }

    pub const fn end(self) -> NonZeroU16 {
        self.end.0
    }

    pub const fn size(self) -> ChrWindowSize {
        self.size
    }

    pub const fn bank(self) -> ChrBank {
        self.bank
    }

    pub fn is_in_bounds(self, address: u16) -> bool {
        self.start.0 <= address && address <= self.end.0.get()
    }

    pub fn location(self) -> Result<ChrBankLocation, String> {
        match self.bank {
            ChrBank::Rom(location, _) | ChrBank::Ram(location, _) | ChrBank::RomRam(location, _) => Ok(location),
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
            ChrBank::Rom(..) | ChrBank::Ram(..) | ChrBank::RomRam(..) =>
                ReadWriteStatusInfo::Absent,
        }
    }

    pub fn offset(self, address: u16) -> Option<u16> {
        if self.start.0 <= address && address <= self.end.0.get() {
            Some(address - self.start.0)
        } else {
            None
        }
    }

    pub fn is_writable(self, registers: &PrgBankRegisters) -> bool {
        self.bank.is_writable(registers)
    }
}

const PRG_PAGE_SIZE: u16 = 8 * KIBIBYTE as u16;
const PRG_SUB_PAGE_SIZE: u16 = KIBIBYTE as u16 / 8;

#[derive(Clone, Copy, Debug)]
pub struct PrgWindowStart(u16);

impl PrgWindowStart {
    const fn new(address: u16) -> Self {
        assert!(address >= 0x6000,
            "PrgWindow start address must be equal to or greater than 0x6000.");
        assert!(address.is_multiple_of(PRG_SUB_PAGE_SIZE),
            "PrgWindow start address must be a multiple of 0x80 (128).");
        Self(address)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PrgWindowEnd(NonZeroU16);

impl PrgWindowEnd {
    const fn new(address: u16) -> Self {
        assert!(address > 0x6000,
            "PrgWindow end address must be greater than 0x6000.");
        assert!(address.wrapping_add(1).is_multiple_of(PRG_SUB_PAGE_SIZE),
            "PrgWindow end address must be a multiple of 0x80 (128), minus 1.");
        Self(NonZeroU16::new(address).unwrap())
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct PrgWindowSize(u16);

impl PrgWindowSize {
    const fn new(size: u32, start: PrgWindowStart, end: PrgWindowEnd) -> Self {
        assert!(size >= KIBIBYTE / 8, "PrgWindow sizes must be at least 128 (0x80) bytes.");
        assert!(size <= 32 * KIBIBYTE, "PrgWindow sizes must be at most 32 kibibytes.");
        let size = size as u16;

        if size >= PRG_PAGE_SIZE {
            assert!(size.is_multiple_of(PRG_PAGE_SIZE),
                "PrgWindow sizes larger than 8KiB must be multiples of 8 kibibytes.")
        } else {
            assert!(size.is_multiple_of(PRG_SUB_PAGE_SIZE),
                "PrgWindow sizes smaller than 8KiB must be multiples of 128 bytes.")
        }

        assert!(end.0.get() > start.0,
            "PrgWindow end address was less than its start address.");
        assert!(end.0.get() - start.0 + 1 == size,
            "PrgWindow size was must equal the end address minus the start address, plus one.");

        Self(size)
    }

    pub fn page_multiple(self) -> u16 {
        self.0 / PRG_PAGE_SIZE
    }

    pub fn sub_page_multiple(self) -> u8 {
        u8::try_from(self.0 / PRG_SUB_PAGE_SIZE).unwrap()
    }

    pub fn to_raw(self) -> u16 {
        self.0
    }
}

const CHR_PAGE_SIZE: u16 = KIBIBYTE as u16;

#[derive(Clone, Copy, Debug)]
pub struct ChrWindowStart(u16);

impl ChrWindowStart {
    const fn new(address: u16) -> Self {
        assert!(address < 0x4000,
            "ChrWindow start address must be less than 0x4000.");
        assert!(address.is_multiple_of(CHR_PAGE_SIZE),
            "ChrWindow start address must be a multiple of 0x400.");
        Self(address)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChrWindowEnd(NonZeroU16);

impl ChrWindowEnd {
    const fn new(address: u16) -> Self {
        assert!(address < 0x4000,
            "ChrWindow end address must be less than 0x4000.");
        assert!(address.wrapping_add(1).is_multiple_of(CHR_PAGE_SIZE),
            "ChrWindow end address must be a multiple of 0x400, minus 1.");
        Self(NonZeroU16::new(address).expect("ChrWindow end address to be greater than 0."))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ChrWindowSize(u16);

impl ChrWindowSize {
    const fn new(size: u32, start: ChrWindowStart, end: ChrWindowEnd) -> Self {
        assert!(size >= KIBIBYTE, "ChrWindow sizes must be at least 1 kibibyte.");
        assert!(size <= 8 * KIBIBYTE, "ChrWindow sizes must be at most 8 kibibytes.");
        let size = size as u16;

        assert!(end.0.get() > start.0,
            "ChrWindow end address was less than its start address.");
        assert!(end.0.get() - start.0 + 1 == size,
            "ChrWindow size was must equal the end address minus the start address, plus one.");

        Self(size)
    }

    pub fn page_multiple(self) -> u16 {
        self.0 / CHR_PAGE_SIZE
    }

    pub fn to_raw(self) -> u16 {
        self.0
    }
}