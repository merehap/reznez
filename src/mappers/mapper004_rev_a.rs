use crate::mappers::mmc3::irq_state::IrqState;
use crate::mappers::mmc3::mmc3::Mapper004Mmc3;

// MMC3 with Sharp Rev A IRQs. There's no submapper assigned to it for some reason.
pub fn mapper004_rev_a() -> Mapper004Mmc3 {
    Mapper004Mmc3::new(IrqState::REV_A_IRQ_STATE)
}
