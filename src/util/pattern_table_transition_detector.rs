use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table_side::PatternTableSide;

// A specialized EdgeDetector for PatternTableSide transitions.
pub struct PatternTableTransitionDetector {
    prev_side: PatternTableSide,
    allowed_addresses: AllowedAddresses,
}

impl PatternTableTransitionDetector {
    pub const fn new(allowed_addresses: AllowedAddresses) -> Self {
        Self {
            prev_side: PatternTableSide::Left,
            allowed_addresses,
        }
    }

    pub fn detect(&mut self, addr: PpuAddress) -> Option<PatternTableSide> {
        if self.allowed_addresses == AllowedAddresses::PatternTableOnly && !addr.is_in_pattern_table() {
            return None;
        }

        let curr_side = addr.pattern_table_side();
        if curr_side == self.prev_side {
            return None;
        }

        self.prev_side = curr_side;
        Some(curr_side)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AllowedAddresses {
    All,
    PatternTableOnly,
}