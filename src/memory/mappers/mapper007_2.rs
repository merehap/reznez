use crate::memory::mapper::HasBusConflicts;
use crate::memory::mappers::common::axrom::Axrom;

pub const MAPPER007_2: Axrom = Axrom::new(HasBusConflicts::Yes);
