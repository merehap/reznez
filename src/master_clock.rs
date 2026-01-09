use crate::apu::apu_registers::ApuClock;
use crate::ppu::ppu_clock::PpuClock;

use CycleType::*;
const NTSC_SCHEDULE: [&[CycleType]; 3] = [
    &[Apu, CpuFirstHalf, Ppu],
    &[CpuSecondHalf, Ppu],
    &[Ppu],
];

pub struct MasterClock {
    master_cycle: u64,

    cpu_cycle: i64,
    ppu_clock: PpuClock,
    pub apu_clock: ApuClock,

    schedule: [&'static [CycleType]; 3],
}

impl MasterClock {
    pub fn new(starting_cpu_cycle: i64, ppu_clock: PpuClock) -> Self {
        Self {
            master_cycle: 0,

            cpu_cycle: starting_cpu_cycle,
            ppu_clock,
            apu_clock: ApuClock::new(),

            schedule: NTSC_SCHEDULE,
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

    pub fn tick(&mut self) -> (&'static [CycleType], bool) {
        let period: u64 = self.schedule.len().try_into()
            .expect("Sane schedule period.");
        let index: usize = (self.master_cycle % period).try_into()
            .expect("Sane index.");
        self.master_cycle += 1;
        let end_reached = self.master_cycle.is_multiple_of(period);
        (self.schedule[index], end_reached)
    }

    // TODO: Remove this. Stepping the master clock should do this automatically.
    pub fn increment_master_cycle(&mut self) {
        self.master_cycle += 1;
    }

    // TODO: Remove this. Stepping the master clock should do this automatically.
    pub fn increment_cpu_cycle(&mut self) {
        self.cpu_cycle += 1;
    }

    pub fn tick_ppu_clock(&mut self, skip_odd_frame_cycle: bool) -> bool {
        self.ppu_clock.tick(skip_odd_frame_cycle)
    }
}

pub enum CycleType {
    Apu,
    CpuFirstHalf,
    CpuSecondHalf,
    Ppu,
}