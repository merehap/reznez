use crate::mappers::mmc3::irq_state::Mmc3IrqState;
use crate::mappers::mmc3::mmc3::Mapper004Mmc3;

// MMC3 with Sharp IRQs
pub fn mapper004_0() -> Mapper004Mmc3 {
    Mapper004Mmc3::new(Mmc3IrqState::SHARP_IRQ_STATE)
}
