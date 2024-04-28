use std::cell::RefCell;
use std::rc::Rc;

use log::Level::Info;
use log::{info, log_enabled};

use crate::apu::apu::Apu;
use crate::cartridge::cartridge::Cartridge;
use crate::config::Config;
use crate::controller::joypad::Joypad;
use crate::cpu::cpu::Cpu;
use crate::cpu::step::Step;
use crate::gui::gui::Events;
use crate::logging::formatter;
use crate::logging::formatter::*;
use crate::memory::cpu::ports::Ports;
use crate::memory::mapper_list;
use crate::memory::memory::Memory;
use crate::ppu::ppu;
use crate::ppu::ppu::Ppu;
use crate::ppu::render::frame::Frame;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    apu: Apu,
    memory: Memory,
    cartridge: Cartridge,
    frame: Frame,

    joypad1: Rc<RefCell<Joypad>>,
    joypad2: Rc<RefCell<Joypad>>,
    cycle: u64,

    log_formatter: Box<dyn Formatter>,
}

impl Nes {
    pub fn new(config: &Config) -> Nes {
        let (mapper, mapper_params) = mapper_list::lookup_mapper(&config.cartridge);
        let (joypad1, joypad2) =
        if config.joypad_enabled {
            (Rc::new(RefCell::new(Joypad::new())), Rc::new(RefCell::new(Joypad::new())))
        } else {
            (Rc::new(RefCell::new(Joypad::disabled())), Rc::new(RefCell::new(Joypad::disabled())))
        };

        let ports = Ports::new(joypad1.clone(), joypad2.clone());
        let mut memory = Memory::new(mapper, mapper_params, ports, config.system_palette.clone());

        Nes {
            cpu: Cpu::new(&mut memory.as_cpu_memory(), config.starting_cpu_cycle),
            ppu: Ppu::new(config.ppu_clock),
            apu: Apu::new(config.disable_audio),
            memory,
            cartridge: config.cartridge.clone(),
            frame: Frame::new(),

            joypad1,
            joypad2,
            cycle: 0,

            log_formatter: Box::new(MesenFormatter),
        }
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }

    pub fn ppu_and_memory_mut(&mut self) -> (&Ppu, &mut Memory) {
        (&self.ppu, &mut self.memory)
    }

    pub fn cartridge(&self) -> &Cartridge {
        &self.cartridge
    }

    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    pub fn stack_pointer(&self) -> u8 {
        self.memory.stack_pointer()
    }

    pub fn mute(&mut self) {
        self.apu.mute();
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.memory.as_cpu_memory());
    }

    pub fn step_frame(&mut self) {
        loop {
            let step_result = self.step();
            if step_result.is_last_cycle_of_frame {
                if self.cpu.jammed() {
                    info!("CPU is jammed!");
                }

                break;
            }
        }
    }

    pub fn step(&mut self) -> StepResult {
        let mut step = None;
        let ppu_result;
        match self.cycle % 6 {
            0 => {
                self.apu.step(self.memory.apu_regs_mut());
                step = self.cpu_step();
                ppu_result = self.ppu_step();
            }
            1 => ppu_result = self.ppu_step(),
            2 => ppu_result = self.ppu_step(),
            3 => {
                self.apu.step(self.memory.apu_regs_mut());
                step = self.cpu_step();
                ppu_result = self.ppu_step();
            }
            4 => ppu_result = self.ppu_step(),
            5 => ppu_result = self.ppu_step(),
            _ => unreachable!(),
        }

        self.cycle += 1;

        StepResult {
            step,
            is_last_cycle_of_frame: ppu_result.is_last_cycle_of_frame,
            nmi_scheduled: ppu_result.should_generate_nmi,
        }
    }

    fn cpu_step(&mut self) -> Option<Step> {
        let irq_pending =
            self.memory.apu_regs().frame_irq_pending()
            || self.memory.apu_regs().dmc_irq_pending()
            || self.memory.mapper().irq_pending();
        let mut interrupt_text = String::new();
        if log_enabled!(target: "cpuinstructions", Info) {
            interrupt_text = formatter::interrupts(self);
        }

        let step = self.cpu.step(&mut self.memory.as_cpu_memory(), irq_pending);
        if log_enabled!(target: "cpuinstructions", Info) && self.cpu.next_instruction_starting() {
            info!("{}", self.log_formatter.format_instruction(self, interrupt_text));
        }

        step
    }

    fn ppu_step(&mut self) -> ppu::StepResult {
        let ppu_result = self
            .ppu
            .step(&mut self.memory.as_ppu_memory(), &mut self.frame);
        if ppu_result.should_generate_nmi {
            self.cpu.schedule_nmi();
        }

        ppu_result
    }

    #[inline]
    pub fn process_gui_events(&mut self, events: &Events) {
        for (button, status) in &events.joypad1_button_statuses {
            info!("Joypad 1: button {:?} status is {:?}", button, status);
            self.joypad1
                .borrow_mut()
                .set_button_status(*button, *status);
        }

        for (button, status) in &events.joypad2_button_statuses {
            self.joypad2
                .borrow_mut()
                .set_button_status(*button, *status);
        }
    }
}

pub struct StepResult {
    pub step: Option<Step>,
    pub is_last_cycle_of_frame: bool,
    pub nmi_scheduled: bool,
}
