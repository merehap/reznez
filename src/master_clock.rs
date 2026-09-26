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

    cpu_offset: u8,
    ppu_offset: u8,

    pub m2_transition: Option<M2Transition>,
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

            cpu_offset: 0,
            ppu_offset: 0,

            m2_transition: None,
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
        self.m2_transition = None;
        self.ppu_first_half = false;
        self.ppu_second_half = false;
        self.apu = false;
        self.is_last_dot_of_frame = false;

        match self.cpu_offset {
            24 => {
                self.cpu_offset = 0;

                self.cpu_cycle += 1;
                self.m2_transition = Some(M2Transition::Rising);
            }
            1 => self.m2_transition = Some(M2Transition::Romsel),
            2 => self.m2_transition = Some(M2Transition::PpuselReadWrite),
            8 => self.m2_transition = Some(M2Transition::MiddleOfHigh),
            14 => self.m2_transition = Some(M2Transition::Falling),
            0..=23 => {}
            25.. => unreachable!(),
        }

        match self.ppu_offset {
            8 => {
                self.ppu_offset = 0;

                self.is_last_dot_of_frame = self.ppu_clock.tick(signals).is_some();
                self.ppu_first_half = true;
            }
            4 => {
                self.ppu_second_half = true;
            }
            0..=7 => {}
            9.. => unreachable!(),
        }

        if self.cpu_offset == 0 {
            self.apu = true;
        }

        self.master_cycle += 1;
        self.cpu_offset += 1;
        self.ppu_offset += 1;

        let end_reached = self.master_cycle.is_multiple_of(PERIOD);
        end_reached
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum M2Transition {
    Rising,
    Romsel,
    PpuselReadWrite,
    MiddleOfHigh,
    Falling,
}