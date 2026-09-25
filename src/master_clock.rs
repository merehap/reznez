use crate::apu::apu_clock::ApuClock;
use crate::ppu::ppu_clock::PpuClock;
use crate::signals::Signals;

// Master half-cycles
const PERIOD: u64 = 24;

pub struct MasterClock {
    master_cycle: u64,

    cpu_cycle: i64,
    ppu_clock: PpuClock,
    pub apu_clock: ApuClock,

    pub cpu_first_half: bool,
    pub cpu_second_half: bool,
    pub ppu_first_half: bool,
    pub ppu_second_half: bool,
    pub apu: bool,

    is_last_dot_of_frame: bool,
}

impl MasterClock {
    pub fn new(starting_cpu_cycle: i64, ppu_clock: PpuClock) -> Self {
        Self {
            master_cycle: 0,

            cpu_cycle: starting_cpu_cycle,
            ppu_clock,
            apu_clock: ApuClock::new(),

            cpu_first_half: false,
            cpu_second_half: false,
            ppu_first_half: false,
            ppu_second_half: false,
            apu: false,

            is_last_dot_of_frame: false,
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

    pub fn tick(&mut self, signals: &mut Signals) -> bool {
        self.cpu_first_half = false;
        self.cpu_second_half = false;
        self.ppu_first_half = false;
        self.ppu_second_half = false;
        self.apu = false;
        self.is_last_dot_of_frame = false;

        // FIXME: Remove this hack.
        match self.master_cycle % PERIOD {
            0 => {
                self.cpu_cycle += 1;
                self.cpu_first_half = true;
                self.apu = true;
            }
            8 => {
                self.cpu_second_half = true;
            }
            _ => {}
        }

        match self.master_cycle % 8 {
            0 => {
                self.is_last_dot_of_frame = self.ppu_clock.tick(signals).is_some();
                self.ppu_first_half = true;
            }
            4 => {
                self.ppu_second_half = true;
            }
            _ => {}
        }

        self.master_cycle += 1;
        let end_reached = self.master_cycle.is_multiple_of(PERIOD);
        end_reached
    }
}