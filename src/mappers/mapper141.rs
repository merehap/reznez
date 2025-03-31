use crate::mappers::common::sachen8259::{Sachen8259, Sachen8259Board};

// TODO: Support Q Boy once a suitable ROM is found.
pub const MAPPER141: Sachen8259 = Sachen8259::new(Sachen8259Board::A);