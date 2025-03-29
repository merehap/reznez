use crate::mapper::HasBusConflicts;
use crate::mappers::common::cnrom::Cnrom;

// Duplicate of mapper 3, CNROM.
pub const MAPPER101: Cnrom = Cnrom::new(HasBusConflicts::No);
