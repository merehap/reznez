use crate::memory::mapper::*;
use crate::memory::mappers::mmc3::mmc3;
use crate::memory::mappers::mmc3::nec_irq_state::NecIrqState;

// Identical to mapper 0, except NEC's IRQ behavior is used instead of Sharp's.
pub struct Mapper004_4 {
    selected_register_id: BankIndexRegisterId,
    irq_state: NecIrqState,
}

impl Mapper for Mapper004_4 {
    fn initial_layout(&self) -> InitialLayout {
        mmc3::INITIAL_LAYOUT
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let is_even_address = address.to_raw() % 2 == 0;
        match (address.to_raw(), is_even_address) {
            (0x0000..=0x401F, _) => unreachable!(),
            (0x4020..=0x5FFF, _) => { /* Do nothing. */ }
            (0x6000..=0x7FFF, _) => params.write_prg(address, value),
            (0x8000..=0x9FFF, true ) => mmc3::bank_select(params, &mut self.selected_register_id, value),
            (0x8000..=0x9FFF, false) => mmc3::set_bank_index(params, &mut self.selected_register_id, value),
            (0xA000..=0xBFFF, true ) => mmc3::set_mirroring(params, value),
            (0xA000..=0xBFFF, false) => { /* Do nothing. */ }
            (0xC000..=0xDFFF, true ) => self.irq_state.set_counter_reload_value(value),
            (0xC000..=0xDFFF, false) => self.irq_state.reload_counter(),
            (0xE000..=0xFFFF, true ) => self.irq_state.disable(),
            (0xE000..=0xFFFF, false) => self.irq_state.enable(),
        }
    }

    fn on_end_of_ppu_cycle(&mut self) {
        self.irq_state.decrement_suppression_cycle_count();
    }

    fn process_current_ppu_address(&mut self, address: PpuAddress) {
        self.irq_state.tick_counter(address);
    }

    fn irq_pending(&self) -> bool {
        self.irq_state.pending()
    }
}

impl Mapper004_4 {
    pub fn new() -> Self {
        Self {
            selected_register_id: C0,
            irq_state: NecIrqState::new(),
        }
    }
}
