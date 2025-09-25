use crate::mappers::mmc3::irq_state::IrqState;
use crate::mappers::mmc3::mmc3::Mapper004Mmc3;

// MMC3 with Sharp IRQs
pub fn mapper004_0() -> Mapper004Mmc3 {
    Mapper004Mmc3::new(IrqState::SHARP_IRQ_STATE)
}
