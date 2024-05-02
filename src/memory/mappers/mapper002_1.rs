use crate::memory::mapper::HasBusConflicts;
use crate::memory::mappers::common::uxrom::Uxrom;

pub const MAPPER002_1: Uxrom = Uxrom::new(HasBusConflicts::No);
