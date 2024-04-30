use crate::memory::mappers::mmc3::mmc3::Mapper004Mmc3;
use crate::memory::mappers::mmc3::sharp_irq_state::SharpIrqState;

// MMC3 with Sharp IRQs
pub fn mapper004_0() -> Mapper004Mmc3 {
    Mapper004Mmc3::new(Box::new(SharpIrqState::new()))
}
