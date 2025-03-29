use crate::mappers::mmc3::mmc3::Mapper004Mmc3;
use crate::mappers::mmc3::rev_a_irq_state::RevAIrqState;

// MMC3 with Sharp Rev A IRQs. There's no submapper assigned to it for some reason.
pub fn mapper004_rev_a() -> Mapper004Mmc3 {
    Mapper004Mmc3::new(Box::new(RevAIrqState::new()))
}
