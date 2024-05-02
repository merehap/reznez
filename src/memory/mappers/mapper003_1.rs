use crate::memory::mapper::HasBusConflicts;
use crate::memory::mappers::common::cnrom::Cnrom;

pub const MAPPER003_1: Cnrom = Cnrom::new(HasBusConflicts::No);
