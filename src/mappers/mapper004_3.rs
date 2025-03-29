use crate::mappers::mmc3::mmc3::Mapper004Mmc3;
use crate::mappers::mmc3::mc_acc_irq_state::McAccIrqState;

// MMC3. Identical to submapper 0, except MC-ACC's IRQ behavior is used instead of Sharp's.
pub fn mapper004_3() -> Mapper004Mmc3 {
    Mapper004Mmc3::new(Box::new(McAccIrqState::new()))
}
