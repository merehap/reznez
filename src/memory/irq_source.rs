// An edge-detector for IRQs so we can print when an IRQ first goes pending.
#[derive(Default)]
pub struct IrqSource {
    pending: bool,
    just_went_pending: bool,
}

impl IrqSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pending(&self) -> bool {
        self.pending
    }

    pub fn take_just_went_pending(&mut self) -> bool {
        let value = self.just_went_pending;
        self.just_went_pending = false;
        value
    }

    pub fn set_pending(&mut self, pending: bool) {
        self.just_went_pending = !self.pending && pending;
        self.pending = pending;
    }
}
