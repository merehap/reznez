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
        let joypad1 = Rc::new(RefCell::new(Joypad::new()));
        let joypad2 = Rc::new(RefCell::new(Joypad::new()));
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
                self.apu.off_cycle_step(self.memory.apu_regs_mut());
                step = self.cpu_step();
                ppu_result = self.ppu_step();
            }
            1 => ppu_result = self.ppu_step(),
            2 => ppu_result = self.ppu_step(),
            3 => {
                self.apu.on_cycle_step(self.memory.apu_regs_mut());
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

        let address = self.cpu.address_for_next_step(&self.memory.as_cpu_memory());
        let step = self.cpu.step(&mut self.memory.as_cpu_memory(), irq_pending);
        if log_enabled!(target: "cpuinstructions", Info) && self.cpu.next_instruction_starting() {
            info!("{}", self.log_formatter.format_instruction(self, address, interrupt_text));
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

#[cfg(test)]
mod tests {
    use crate::cartridge::cartridge;
    use crate::memory::cpu::cpu_address::CpuAddress;
    use crate::memory::mapper::Mapper;
    use crate::memory::mappers::mapper000::Mapper000;
    use crate::memory::memory::Memory;
    use crate::ppu::clock::Clock;
    use crate::ppu::palette::system_palette;
    use crate::ppu::register::registers::ctrl::Ctrl;

    use crate::cartridge::cartridge::test_data;

    use super::*;

    #[test]
    fn nmi_enabled_upon_vblank() {
        let mut nes = sample_nes();
        step_until_vblank_nmi_enabled(&mut nes);
        assert!(nes.cpu.nmi_pending());
    }

    #[test]
    fn second_nmi_fails_without_ctrl_toggle() {
        let mut nes = sample_nes();
        step_until_vblank_nmi_enabled(&mut nes);
        assert!(nes.cpu.nmi_pending());

        loop {
            if let Some(step) = nes.step().step && step.has_interpret_op_code(){
                break;
            }
        }

        loop {
            if let Some(step) = nes.step().step && step.has_interpret_op_code(){
                break;
            }
        }

        // Disable vblank_nmi.
        assert!(
            !nes.cpu.nmi_pending(),
            "nmi_pending should have been cleared after one instruction."
        );

        assert!(
            !nes.cpu.nmi_pending(),
            "A second NMI should not have been allowed without toggling CTRL.0 .",
        );
    }

    #[test]
    fn second_nmi_succeeds_after_ctrl_toggle() {
        let mut nes = sample_nes();
        step_until_vblank_nmi_enabled(&mut nes);
        assert!(nes.cpu.nmi_pending());

        loop {
            if let Some(step) = nes.step().step && step.has_interpret_op_code(){
                break;
            }
        }
        loop {
            if let Some(step) = nes.step().step && step.has_interpret_op_code(){
                break;
            }
        }

        assert!(
            !nes.cpu.nmi_pending(),
            "nmi_pending should have been cleared after one instruction."
        );

        let ppu_ctrl = CpuAddress::new(0x2000);
        // Disable vblank_nmi.
        nes.memory.as_cpu_memory().write(ppu_ctrl, 0b0000_0000);
        nes.step();
        // Enable vblank_nmi.
        nes.memory.as_cpu_memory().write(ppu_ctrl, 0b1000_0000);
        nes.step();

        assert!(
            nes.cpu.nmi_pending(),
            "A second NMI should have been allowed after toggling CTRL.0 .",
        );
    }

    fn sample_nes() -> Nes {
        let mapper = Mapper000;
        let mapper_params = mapper.initial_layout().make_mapper_params(&test_data::cartridge());
        let system_palette = system_palette::test_data::system_palette();
        let joypad1 = Rc::new(RefCell::new(Joypad::new()));
        let joypad2 = Rc::new(RefCell::new(Joypad::new()));
        let ports = Ports::new(joypad1.clone(), joypad2.clone());
        let mut memory = Memory::new(
            Box::new(mapper),
            mapper_params,
            ports,
            system_palette,
        );
        // Write NOPs to where the RESET_VECTOR starts the program.
        for i in 0x0200..0x0800 {
            memory.as_cpu_memory().write(CpuAddress::new(i), 0xEA);
        }

        let cartridge = cartridge::test_data::cartridge();

        Nes {
            cpu: Cpu::new(&mut memory.as_cpu_memory(), 0),
            ppu: Ppu::new(Clock::mesen_compatible()),
            apu: Apu::new(true),
            memory,
            cartridge,
            frame: Frame::new(),
            joypad1,
            joypad2,
            cycle: 0,
            log_formatter: Box::new(MesenFormatter),
        }
    }

    fn step_until_vblank_nmi_enabled(nes: &mut Nes) {
        let mut ctrl = Ctrl::new();
        ctrl.nmi_enabled = true;
        nes.memory
            .as_cpu_memory()
            .write(CpuAddress::new(0x2000), ctrl.to_u8());

        loop {
            assert!(
                !nes.cpu.nmi_pending(),
                "NMI must not be pending before one is scheduled.",
            );
            let nmi_scheduled = nes.step().nmi_scheduled;
            if nmi_scheduled {
                break;
            }

            if nes.ppu.clock().total_cycles() > 200_000 {
                panic!("It took too long for the PPU to enable NMI.");
            }
        }

        println!("Should generate NMI");
    }
}
