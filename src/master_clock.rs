use crate::apu::apu_clock::ApuClock;
use crate::ppu::ppu_clock::PpuClock;
use crate::signals::Signals;

use CycleType::*;
const NTSC_SCHEDULE: [&[CycleType]; 24] = [
    &[Apu, CpuFirstHalf, PpuFirstHalf],
    &[],
    &[],
    &[],
    &[PpuSecondHalf],
    &[],
    &[],
    &[],
    &[CpuSecondHalf, PpuFirstHalf],
    &[],
    &[],
    &[],
    &[PpuSecondHalf],
    &[],
    &[],
    &[],
    &[PpuFirstHalf],
    &[],
    &[],
    &[],
    &[PpuSecondHalf],
    &[],
    &[],
    &[],
];

const NTSC_SCHEDULE_WITH_LOGGING: [&[CycleType]; 24] = [
    &[ApuWithLogging, CpuFirstHalfWithLogging, PpuFirstHalfWithLogging],
    &[],
    &[],
    &[],
    &[PpuSecondHalf],
    &[],
    &[],
    &[],
    &[CpuSecondHalfWithLogging, PpuFirstHalfWithLogging],
    &[],
    &[],
    &[],
    &[PpuSecondHalf],
    &[],
    &[],
    &[],
    &[PpuFirstHalfWithLogging],
    &[],
    &[],
    &[],
    &[PpuSecondHalf],
    &[],
    &[],
    &[],
];

pub struct MasterClock {
    master_cycle: u64,

    cpu_cycle: i64,
    ppu_clock: PpuClock,
    pub apu_clock: ApuClock,

    is_last_dot_of_frame: bool,

    schedule: [&'static [CycleType]; 24],
}

impl MasterClock {
    pub fn new(starting_cpu_cycle: i64, ppu_clock: PpuClock) -> Self {
        Self {
            master_cycle: 0,

            cpu_cycle: starting_cpu_cycle,
            ppu_clock,
            apu_clock: ApuClock::new(),

            is_last_dot_of_frame: false,

            schedule: NTSC_SCHEDULE,
        }
    }

    pub fn new_with_diff_logging(starting_cpu_cycle: i64, ppu_clock: PpuClock) -> Self {
        Self {
            master_cycle: 0,

            cpu_cycle: starting_cpu_cycle,
            ppu_clock,
            apu_clock: ApuClock::new(),

            is_last_dot_of_frame: false,

            schedule: NTSC_SCHEDULE_WITH_LOGGING,
        }
    }

    pub fn master_cycle(&self) -> u64 {
        self.master_cycle
    }

    pub fn cpu_cycle(&self) -> i64 {
        self.cpu_cycle
    }

    pub fn ppu_clock(&self) -> &PpuClock {
        &self.ppu_clock
    }

    pub fn apu_clock(&self) -> &ApuClock {
        &self.apu_clock
    }

    pub fn apu_clock_mut(&mut self) -> &mut ApuClock {
        &mut self.apu_clock
    }

    pub fn is_last_dot_of_frame(&self) -> bool {
        self.is_last_dot_of_frame
    }

    pub fn tick(&mut self, signals: &mut Signals) -> (&'static [CycleType], bool) {
        let period: u64 = self.schedule.len().try_into()
            .expect("Sane schedule period.");
        let index: usize = (self.master_cycle % period).try_into()
            .expect("Sane index.");

        // FIXME: Remove this hack.
        if self.master_cycle.is_multiple_of(period) {
            self.cpu_cycle += 1;
        }

        // FIXME: Remove this hack.
        if self.master_cycle.is_multiple_of(period / 3) {
            self.is_last_dot_of_frame = self.ppu_clock.tick(signals).is_some();
        } else {
            self.is_last_dot_of_frame = false;
        }

        self.master_cycle += 1;
        let end_reached = self.master_cycle.is_multiple_of(period);
        (self.schedule[index], end_reached)
    }
}

pub enum CycleType {
    Apu,
    ApuWithLogging,
    CpuFirstHalf,
    CpuFirstHalfWithLogging,
    CpuSecondHalf,
    CpuSecondHalfWithLogging,
    PpuFirstHalf,
    PpuFirstHalfWithLogging,
    PpuSecondHalf,
}
