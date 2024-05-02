use crate::memory::mapper::HasBusConflicts;
use crate::memory::mappers::common::uxrom::Uxrom;

pub const MAPPER002_2: Uxrom = Uxrom::new(HasBusConflicts::Yes);
