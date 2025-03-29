use crate::mappers::mmc3::mmc3::Mapper004Mmc3;
use crate::mappers::mmc3::nec_irq_state::NecIrqState;

// MMC3. Identical to submapper 0, except NEC's IRQ behavior is used instead of Sharp's.
pub fn mapper004_4() -> Mapper004Mmc3 {
    Mapper004Mmc3::new(Box::new(NecIrqState::new()))
}
