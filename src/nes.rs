use std::cell::RefCell;
use std::ops::Add;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use log::{info, warn};

use crate::config::Config;
use crate::controller::joypad::Joypad;
use crate::cpu::cpu::{Cpu, StepResult};
use crate::cpu::instruction::Instruction;
use crate::gui::gui::Gui;
use crate::memory::cpu_internal_ram::*;
use crate::memory::memory::Memory;
use crate::memory::mapper::Mapper;
use crate::memory::mappers::mapper0::Mapper0;
use crate::memory::mappers::mapper1::Mapper1;
use crate::memory::mappers::mapper3::Mapper3;
use crate::memory::port_access::{PortAccess, AccessMode};
use crate::ppu::ppu::Ppu;
use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::ppu::render::frame::Frame;
use crate::ppu::render::frame_rate::TargetFrameRate;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    memory: Memory,

    pub joypad_1: Joypad,
    pub joypad_2: Joypad,
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

        let ppu_registers = Rc::new(RefCell::new(PpuRegisters::new()));
        let mut memory = Memory::new(mapper, ppu_registers.clone(), config.system_palette);

        Nes {
            cpu: Cpu::new(&mut memory, config.program_counter_source),
            ppu: Ppu::new(ppu_registers),
            memory,

            joypad_1: Joypad::new(),
            joypad_2: Joypad::new(),
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

        for (button, status) in events.joypad_1_button_statuses {
            self.joypad_1.set_button_status(button, status);
        }

        for (button, status) in events.joypad_2_button_statuses {
            self.joypad_2.set_button_status(button, status);
        }

        loop {
            let is_last_cycle = self.ppu().clock().is_last_cycle_of_frame();
            self.step(gui.frame_mut());

            if is_last_cycle {
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

    pub fn step(&mut self, frame: &mut Frame) -> Option<Instruction> {
        let mut instruction = None;
        if self.cycle % 3 == 0 {
            match self.cpu.step(&mut self.memory) {
                StepResult::Nop => {},
                StepResult::InstructionComplete(inst) => instruction = Some(inst),
                StepResult::DmaWrite {bytes_written, current_byte: value} =>
                    self.ppu.write_oam_at_offset(bytes_written, value),
            }

            if let Some(port_access) = self.memory.latch() {
                self.execute_port_action(port_access);
            }
        }

        self.ppu.step(&mut self.memory, frame);

        if self.ppu.should_generate_nmi() {
            self.cpu.schedule_nmi();
        }

        let status = self.joypad_1.selected_button_status() as u8;
        self.memory.ports_mut().bus_access_write(JOYSTICK_1_PORT, status);
        let status = self.joypad_2.selected_button_status() as u8;
        self.memory.ports_mut().bus_access_write(JOYSTICK_2_PORT, status);

        self.cycle += 1;

        instruction
    }

    fn execute_port_action(&mut self, port_access: PortAccess) {
        let value = port_access.value;

        use AccessMode::{Read, Write};
        match (port_access.address, port_access.access_mode) {
            (OAM_DMA, Write) => self.cpu.initiate_dma_transfer(value),

            // Now that the ROM has read a button status, advance to the next status.
            (JOYSTICK_1_PORT, Read) => self.joypad_1.select_next_button(),
            (JOYSTICK_2_PORT, Read) => self.joypad_2.select_next_button(),
            (JOYSTICK_1_PORT, Write) => {
                if value & 1 == 1 {
                    self.joypad_1.strobe_on();
                    self.joypad_2.strobe_on();
                } else {
                    self.joypad_1.strobe_off();
                    self.joypad_2.strobe_off();
                }
            },

            (_, _) => unreachable!(),
        }
    }

    fn frame_duration(&self) -> Duration {
        match self.target_frame_rate {
            TargetFrameRate::Value(frame_rate) => frame_rate.to_frame_duration(),
            TargetFrameRate::Unbounded => Duration::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::cpu_address::CpuAddress;
    use crate::cpu::cpu::ProgramCounterSource;
    use crate::memory::memory::Memory;
    use crate::ppu::palette::system_palette::SystemPalette;
    use crate::ppu::register::registers::ctrl::Ctrl;
    use crate::ppu::render::frame::Frame;
    use crate::ppu::render::frame_rate::TargetFrameRate;

    use crate::cartridge::tests::sample_cartridge;

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
        while nes.step(&mut frame).is_none() {}
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
        while nes.step(&mut frame).is_none() {}
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
        let cartridge = sample_cartridge();
        let mapper = Box::new(Mapper0::new(cartridge).unwrap());
        let system_palette =
            SystemPalette::parse(include_str!("../palettes/2C02.pal")).unwrap();
        let ppu_registers = Rc::new(RefCell::new(PpuRegisters::new()));
        let mut memory = Memory::new(mapper, ppu_registers.clone(), system_palette);
        // Write NOPs to where the RESET_VECTOR starts the program.
        for i in 0x0200..0x0800 {
            memory.cpu_write(CpuAddress::new(i), 0xEA);
        }

        Nes {
            cpu: Cpu::new(&mut memory, ProgramCounterSource::Override(CpuAddress::new(0x0000))),
            ppu: Ppu::new(ppu_registers),
            memory,
            joypad_1: Joypad::new(),
            joypad_2: Joypad::new(),
            cycle: 0,

            target_frame_rate: TargetFrameRate::Unbounded,
            stop_frame: None,
        }
    }

    fn step_until_vblank_nmi_enabled(nes: &mut Nes) {
        let mut ctrl = Ctrl::new();
        ctrl.nmi_enabled = true;
        nes.memory.cpu_write(CpuAddress::new(0x2000), ctrl.to_u8());

        let mut frame = Frame::new();
        while !nes.ppu.should_generate_nmi() {
            assert!(!nes.cpu.nmi_pending(), "NMI must not be pending before one is scheduled.");
            nes.step(&mut frame);
            if nes.ppu.clock().total_cycles() > 200_000 {
                panic!("It took too long for the PPU to enable NMI.");
            }
        }

        println!("Should generate NMI");
    }

    fn write_ppuctrl_through_opcode_injection(nes: &mut Nes, ctrl: u8) {
        // STA: Store to the accumulator.
        nes.memory.cpu_write(nes.cpu.program_counter().advance(0), 0xA9);
        // Store VBLANK_NMI DISABLED to the accumulator.
        nes.memory.cpu_write(nes.cpu.program_counter().advance(1), ctrl);

        // LDA: Load the accumulator into a memory location.
        nes.memory.cpu_write(nes.cpu.program_counter().advance(2), 0x8D);
        // Low byte of PPUCTRL, the address to be set.
        nes.memory.cpu_write(nes.cpu.program_counter().advance(3), 0x00);
        // High byte of PPUCTRL, the address to be set.
        nes.memory.cpu_write(nes.cpu.program_counter().advance(4), 0x20);

        // Execute the two op codes we just injected.
        let mut frame = Frame::new();
        while nes.step(&mut frame).is_none() {}
        while nes.step(&mut frame).is_none() {}
    }
}
