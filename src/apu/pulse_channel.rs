use splitbits::{splitbits_ux, splitbits_named_ux};
use ux::{u2, u4};

use crate::apu::envelope::Envelope;
use crate::apu::length_counter::LengthCounter;
use crate::apu::sweep::{NegateBehavior, Sweep};
use crate::util::bit_util;

//                  Sweep -----> Timer
//                    |            |
//                    |            |
//                    |            v
//                    |        Sequencer   Length Counter
//                    |            |             |
//                    |            |             |
//                    v            v             v
// Envelope -------> Gate -----> Gate -------> Gate ---> (to mixer)
#[derive(Default)]
pub struct PulseChannel<const N: NegateBehavior> {
    pub(super) length_counter: LengthCounter,

    enabled: bool,

    sweep: Sweep<N>,
    envelope: Envelope,
    sequencer: Sequencer,
}

impl <const N: NegateBehavior> PulseChannel<N> {
    // Write $4000 or $4004
    pub fn set_control(&mut self, value: u8) {
        let fields = splitbits_ux!(value, "ddhc eeee");
        self.sequencer.set_duty(fields.d.into());
        self.length_counter.start_halt(fields.h);
        self.envelope.set_control(fields.c, fields.e);

        self.sweep.update_target_period();
    }

    // Write $4001 or $4005
    pub fn set_sweep(&mut self, value: u8) {
        let (enabled, period, negate, shift_count) = splitbits_named_ux!(value, "eppp nsss");
        self.sweep.set(enabled, period, negate, shift_count);
    }

    // Write $4002 or $4006
    pub fn set_period_low(&mut self, value: u8) {
        self.sweep.set_current_period_low(value);
    }

    // Write $4003 or $4007
    pub fn set_length_and_period_high(&mut self, value: u8) {
        let fields = splitbits_ux!(value, "llll lppp");
        if self.enabled {
            self.length_counter.start_reload(fields.l);
            // TODO: Does the envelope restart even if !self.enabled?
            self.envelope.start();
        }

        self.sequencer.reset();
        self.sweep.set_current_period_high_and_reset_index(fields.p);
    }

    // Write 0x4015
    pub(super) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !self.enabled {
            self.length_counter.set_to_zero();
        }
    }

    pub(super) fn active(&self) -> bool {
        !self.length_counter.is_zero()
    }

    pub(super) fn tick_put(&mut self) {
        let triggered = self.sweep.tick_frequency_timer();
        if triggered {
            self.sequencer.step();
        }
    }

    pub(super) fn tick_envelope(&mut self) {
        self.envelope.step();
    }

    pub(super) fn tick_sweep(&mut self) {
        self.sweep.tick();
    }

    pub(super) fn sample_volume(&self) -> u4 {
        if self.muted() {
            u4::new(0)
        } else {
            self.envelope.volume()
        }
    }

    fn muted(&self) -> bool {
        !self.enabled
            || self.sweep.muting()
            || self.length_counter.is_zero()
            || !self.sequencer.on_duty()
    }
}

#[derive(Default)]
pub struct Sequencer {
    index: u32,
    duty: Duty,
}

impl Sequencer {
    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn step(&mut self) {
        self.index += 1;
        self.index %= 8;
    }

    pub fn on_duty(&self) -> bool {
        bit_util::get_bit(self.duty as u8, self.index)
    }

    pub fn set_duty(&mut self, duty: Duty) {
        self.duty = duty;
    }
}

#[derive(Clone, Copy, Default)]
pub enum Duty {
    #[default]
    Low     = 0b0100_0000,
    Medium  = 0b0110_0000,
    High    = 0b0111_1000,
    Negated = 0b1001_1111,
}

impl From<u2> for Duty {
    fn from(value: u2) -> Self {
        match u8::from(value) {
            0 => Duty::Low,
            1 => Duty::Medium,
            2 => Duty::High,
            3 => Duty::Negated,
            _ => unreachable!(),
        }
    }
}