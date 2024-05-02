use crate::memory::mapper::HasBusConflicts;
use crate::memory::mappers::common::axrom::Axrom;

pub const MAPPER007_1: Axrom = Axrom::new(HasBusConflicts::No);
