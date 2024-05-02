use crate::memory::mapper::HasBusConflicts;
use crate::memory::mappers::common::cnrom::Cnrom;

pub const MAPPER003_2: Cnrom = Cnrom::new(HasBusConflicts::Yes);
