use crate::mappers::mmc3::irq_state::Mmc3IrqState;
use crate::mappers::mmc3::mmc3::Mapper004Mmc3;

// MMC3. Identical to submapper 0, except MC-ACC's IRQ behavior is used instead of Sharp's.
pub fn mapper004_3() -> Mapper004Mmc3 {
    Mapper004Mmc3::new(Mmc3IrqState::MC_ACC_IRQ_STATE)
}