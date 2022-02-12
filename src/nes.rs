use std::cell::RefCell;
use std::ops::Add;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use log::{info, warn, log_enabled};
use log::Level::Info;

use crate::config::Config;
use crate::controller::joypad::Joypad;
use crate::cpu::cpu::Cpu;
use crate::cpu::instruction::{Instruction, Argument, AccessMode};
use crate::gui::gui::Gui;
use crate::memory::cpu::ports::Ports;
use crate::memory::memory::Memory;
use crate::memory::mapper::Mapper;
use crate::memory::mappers::mapper0::Mapper0;
use crate::memory::mappers::mapper1::Mapper1;
use crate::memory::mappers::mapper3::Mapper3;
use crate::ppu::ppu::Ppu;
use crate::ppu::render::frame::Frame;
use crate::ppu::render::frame_rate::TargetFrameRate;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    memory: Memory,

    joypad1: Rc<RefCell<Joypad>>,
    joypad2: Rc<RefCell<Joypad>>,
    cycle: u64,

    target_frame_rate: TargetFrameRate,
    stop_frame: Option<u64>,
}

impl Nes {
    pub fn new(config: Config) -> Nes {
        let mapper =
            match config.cartridge.mapper_number() {
                0 => Box::new(Mapper0::new(config.cartridge).unwrap()) as Box<dyn Mapper>,
                1 => Box::new(Mapper1::new(config.cartridge).unwrap()),
                3 => Box::new(Mapper3::new(config.cartridge).unwrap()),
                _ => todo!(),
            };

        let joypad1 = Rc::new(RefCell::new(Joypad::new()));
        let joypad2 = Rc::new(RefCell::new(Joypad::new()));
        let ports = Ports::new(joypad1.clone(), joypad2.clone());
        let mut memory = Memory::new(mapper, ports, config.system_palette);

        Nes {
            cpu: Cpu::new(&mut memory.as_cpu_memory(), config.program_counter_source),
            ppu: Ppu::new(),
            memory,

            joypad1,
            joypad2,
            cycle: 0,

            target_frame_rate: config.target_frame_rate,
            stop_frame: config.stop_frame,
        }
    }

    pub fn cpu(&self) -> &Cpu {
       &self.cpu
    }

    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    pub fn stack_pointer(&self) -> u8 {
        self.memory.stack_pointer()
    }

    pub fn step_frame(&mut self, gui: &mut dyn Gui) {
        let frame_index = self.ppu().clock().frame();
        let start_time = SystemTime::now();
        let intended_frame_end_time = start_time.add(self.frame_duration());

        let events = gui.events();

        for (button, status) in events.joypad1_button_statuses {
            self.joypad1.borrow_mut().set_button_status(button, status);
        }

        for (button, status) in events.joypad2_button_statuses {
            self.joypad2.borrow_mut().set_button_status(button, status);
        }

        loop {
            let step_result = self.step(gui.frame_mut());
            if step_result.is_last_cycle_of_frame {
                break;
            }
        }

        info!("Displaying frame {}.", frame_index);
        gui.display_frame(frame_index);

        let end_time = SystemTime::now();
        if let Ok(duration) = intended_frame_end_time.duration_since(end_time) {
            std::thread::sleep(duration);
        }

        let end_time = SystemTime::now();
        if let Ok(duration) = end_time.duration_since(start_time) {
            info!("Framerate: {}", 1_000_000_000.0 / duration.as_nanos() as f64);
        } else {
            warn!("Unknown framerate. System clock went backwards.");
        }

        if events.should_quit || Some(frame_index) == self.stop_frame {
            std::process::exit(0);
        }
    }

    #[inline(always)]
    fn log_state(&self, instruction: Instruction) {
        if log_enabled!(target: "cpu", Info) {
            /*
            info!(
                target: "cpu",
                "{:010} PC:{}, A:{:02X} X:{:02X} Y:{:02X} P:{:02X} S:{:02X} {} | {instruction}",
                self.cpu.cycle(),
                self.cpu.program_counter(),
                self.cpu.accumulator(),
                self.cpu.x_index(),
                self.cpu.y_index(),
                self.cpu.status().to_register_byte(),
                self.memory.stack_pointer(),
                self.cpu.status(),
            );
            */
            let argument =
                match instruction.argument {
                    Argument::Imp => /* Unused. */ 0,
                    Argument::Imm(value) => value as u16,
                    Argument::Addr(address) => address.to_raw(),
                };
            use AccessMode::*;
            let formatted_argument =
                match instruction.template.access_mode {
                    Imp => "".to_string(),
                    Imm => format!("#${:02X}", argument),
                    ZP  => format!("${:02X}", argument),
                    ZPX => format!("${:02X},X @", argument),
                    ZPY => format!("(${:02X}),Y", argument),
                    Abs => format!("${:04X}", argument),
                    AbX => format!("${:04X},X", argument),
                    AbY => format!("${:04X},Y", argument),
                    Rel => format!("${:04X}", argument),
                    Ind => format!("${:04X}", argument),
                    IzX => format!("${:04X},X", argument),
                    IzY => format!("${:04X},Y", argument),
                };

            info!(
                "{:04X} {:?} {:8}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:<3} SL:{:<3} CPU Cycle:{}",
                self.cpu.program_counter().to_raw(),
                instruction.template.op_code,
                formatted_argument,
                self.cpu.accumulator(),
                self.cpu.x_index(),
                self.cpu.y_index(),
                self.cpu.status().to_register_byte(),
                self.memory.stack_pointer(),
                self.ppu.clock().cycle(),
                self.ppu.clock().scanline(),
                self.cpu.cycle(),
            );
        }
    }

    pub fn step(&mut self, frame: &mut Frame) -> StepResult {
        let mut instruction = None;
        if self.cycle % 3 == 0 {
            instruction = self.cpu.step(&mut self.memory.as_cpu_memory());
            if let Some(instruction) = instruction {
                self.log_state(instruction);
            }
        }

        let ppu_result = self.ppu.step(&mut self.memory.as_ppu_memory(), frame);

        if ppu_result.should_generate_nmi {
            self.cpu.schedule_nmi();
        }

        self.cycle += 1;

        StepResult {
            instruction,
            is_last_cycle_of_frame: ppu_result.is_last_cycle_of_frame,
            nmi_scheduled: ppu_result.should_generate_nmi,
        }
    }

    fn frame_duration(&self) -> Duration {
        match self.target_frame_rate {
            TargetFrameRate::Value(frame_rate) => frame_rate.to_frame_duration(),
            TargetFrameRate::Unbounded => Duration::ZERO,
        }
    }
}

pub struct StepResult {
    pub instruction: Option<Instruction>,
    pub is_last_cycle_of_frame: bool,
    pub nmi_scheduled: bool,
}

#[cfg(test)]
mod tests {
    use crate::cpu::cpu::ProgramCounterSource;
    use crate::memory::cpu::cpu_address::CpuAddress;
    use crate::memory::memory::Memory;
    use crate::ppu::palette::system_palette;
    use crate::ppu::register::registers::ctrl::Ctrl;
    use crate::ppu::render::frame::Frame;
    use crate::ppu::render::frame_rate::TargetFrameRate;

    use crate::cartridge::test_data;

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

        let mut frame = Frame::new();
        while nes.step(&mut frame).instruction.is_none() {}
        nes.step(&mut frame);

        assert!(
            !nes.cpu.nmi_pending(),
            "nmi_pending should have been cleared after one instruction."
        );

        // Disable vblank_nmi.
        write_ppuctrl_through_opcode_injection(&mut nes, 0b0000_0000);

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

        let mut frame = Frame::new();
        while nes.step(&mut frame).instruction.is_none() {}
        nes.step(&mut frame);

        assert!(
            !nes.cpu.nmi_pending(),
            "nmi_pending should have been cleared after one instruction."
        );

        // Disable vblank_nmi.
        write_ppuctrl_through_opcode_injection(&mut nes, 0b0000_0000);
        // Enable vblank_nmi.
        write_ppuctrl_through_opcode_injection(&mut nes, 0b1000_0000);

        assert!(
            nes.cpu.nmi_pending(),
            "A second NMI should have been allowed after toggling CTRL.0 .",
        );
    }

    fn sample_nes() -> Nes {
        let mapper = Box::new(Mapper0::new(test_data::cartridge()).unwrap());
        let system_palette = system_palette::test_data::system_palette();
        let joypad1 = Rc::new(RefCell::new(Joypad::new()));
        let joypad2 = Rc::new(RefCell::new(Joypad::new()));
        let ports = Ports::new(joypad1.clone(), joypad2.clone());
        let mut memory = Memory::new(mapper, ports, system_palette);
        // Write NOPs to where the RESET_VECTOR starts the program.
        for i in 0x0200..0x0800 {
            memory.as_cpu_memory().write(CpuAddress::new(i), 0xEA);
        }

        Nes {
            cpu: Cpu::new(&mut memory.as_cpu_memory(), ProgramCounterSource::Override(CpuAddress::new(0x0000))),
            ppu: Ppu::new(),
            memory,
            joypad1,
            joypad2,
            cycle: 0,

            target_frame_rate: TargetFrameRate::Unbounded,
            stop_frame: None,
        }
    }

    fn step_until_vblank_nmi_enabled(nes: &mut Nes) {
        let mut ctrl = Ctrl::new();
        ctrl.nmi_enabled = true;
        nes.memory.as_cpu_memory().write(CpuAddress::new(0x2000), ctrl.to_u8());

        let mut frame = Frame::new();
        loop {
            assert!(!nes.cpu.nmi_pending(), "NMI must not be pending before one is scheduled.");
            let nmi_scheduled = nes.step(&mut frame).nmi_scheduled;
            if nmi_scheduled {
                break;
            }

            if nes.ppu.clock().total_cycles() > 200_000 {
                panic!("It took too long for the PPU to enable NMI.");
            }
        }

        println!("Should generate NMI");
    }

    fn write_ppuctrl_through_opcode_injection(nes: &mut Nes, ctrl: u8) {
        let mut memory = nes.memory.as_cpu_memory();
        // STA: Store to the accumulator.
        memory.write(nes.cpu.program_counter().advance(0), 0xA9);
        // Store VBLANK_NMI DISABLED to the accumulator.
        memory.write(nes.cpu.program_counter().advance(1), ctrl);

        // LDA: Load the accumulator into a memory location.
        memory.write(nes.cpu.program_counter().advance(2), 0x8D);
        // Low byte of PPUCTRL, the address to be set.
        memory.write(nes.cpu.program_counter().advance(3), 0x00);
        // High byte of PPUCTRL, the address to be set.
        memory.write(nes.cpu.program_counter().advance(4), 0x20);

        // Execute the two op codes we just injected.
        let mut frame = Frame::new();
        while nes.step(&mut frame).instruction.is_none() {}
        while nes.step(&mut frame).instruction.is_none() {}
    }
}
