use std::cell::RefCell;
use std::rc::Rc;

use log::Level::Info;
use log::{info, log_enabled};

use crate::apu::apu::Apu;
use crate::cartridge::Cartridge;
use crate::config::Config;
use crate::controller::joypad::Joypad;
use crate::cpu::cpu::Cpu;
use crate::cpu::step::Step;
use crate::cpu::instruction::{AccessMode, Argument, Instruction};
use crate::gui::gui::Events;
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
}

impl Nes {
    pub fn new(config: &Config) -> Nes {
        let mapper = mapper_list::lookup_mapper(&config.cartridge);
        let joypad1 = Rc::new(RefCell::new(Joypad::new()));
        let joypad2 = Rc::new(RefCell::new(Joypad::new()));
        let ports = Ports::new(joypad1.clone(), joypad2.clone());
        let mut memory = Memory::new(mapper, ports, config.system_palette.clone());

        Nes {
            cpu: Cpu::new(&mut memory.as_cpu_memory(), config.program_counter_source),
            ppu: Ppu::new(),
            apu: Apu::new(),
            memory,
            cartridge: config.cartridge.clone(),
            frame: Frame::new(),

            joypad1,
            joypad2,
            cycle: 0,
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
        self.apu.mute()
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
                self.apu.half_step(self.memory.apu_regs());
                step = self.cpu_step();
                ppu_result = self.ppu_step();
            }
            1 => ppu_result = self.ppu_step(),
            2 => ppu_result = self.ppu_step(),
            3 => {
                self.apu_step();
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
            || self.memory.mapper().irq_pending();
        let step = self.cpu.step(&mut self.memory.as_cpu_memory(), irq_pending);
        if let Some(ref step) = step && step.has_interpret_op_code() {
            if let Some(instruction) = self.cpu.current_instruction() {
                if log_enabled!(target: "cpuoperation", Info) {
                    self.log_state(instruction);
                }
            }
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

    fn apu_step(&mut self) {
        self.apu.step(self.memory.apu_regs());
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

    #[inline]
    fn log_state(&mut self, instruction: Instruction) {
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


        let (address, value) = match instruction.argument {
            // No argument for Imp, so this value is unused.
            Argument::Imp => (0, "".to_string()),
            Argument::Imm(value) => (0, format!("{value:02X}")),
            Argument::Addr(address) => {
                let value = self.memory.as_cpu_memory().peek(address);
                let value = value.map(|v| format!("#{v:02X}")).unwrap_or("OB".to_string());
                (address.to_raw(), value)
            }
        };
        use AccessMode::*;
        let formatted_argument = match instruction.template.access_mode {
            Imp => "".to_string(),
            Imm => format!("#${value}"),
            ZP => format!("[${address:02X}]={value}"),
            ZPX => format!("[${address:02X},X @]={value}"),
            ZPY => format!("[(${address:02X}),Y]={value}"),
            Abs => format!("[${address:04X}]={value}"),
            AbX => format!("[${address:04X},X]={value}"),
            AbY => format!("[${address:04X},Y]={value}"),
            Rel => format!("${address:04X}"),
            Ind => format!("${address:04X}"),
            IzX => format!("[${address:04X},X]={value}"),
            IzY => format!("[${address:04X},Y]={value}"),
        };

        info!(
            "{:04X} {:?} {:14}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:<3} SL:{:<3} CPU Cycle:{}",
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

pub struct StepResult {
    pub step: Option<Step>,
    pub is_last_cycle_of_frame: bool,
    pub nmi_scheduled: bool,
}

#[cfg(test)]
mod tests {
    use crate::cartridge;
    use crate::cpu::cpu::ProgramCounterSource;
    use crate::memory::cpu::cpu_address::CpuAddress;
    use crate::memory::mappers::mapper000::Mapper000;
    use crate::memory::memory::Memory;
    use crate::ppu::palette::system_palette;
    use crate::ppu::register::registers::ctrl::Ctrl;

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
        let mapper = Box::new(Mapper000::new(&test_data::cartridge()).unwrap());
        let system_palette = system_palette::test_data::system_palette();
        let joypad1 = Rc::new(RefCell::new(Joypad::new()));
        let joypad2 = Rc::new(RefCell::new(Joypad::new()));
        let ports = Ports::new(joypad1.clone(), joypad2.clone());
        let mut memory = Memory::new(mapper, ports, system_palette);
        // Write NOPs to where the RESET_VECTOR starts the program.
        for i in 0x0200..0x0800 {
            memory.as_cpu_memory().write(CpuAddress::new(i), 0xEA);
        }

        let cartridge = cartridge::test_data::cartridge();

        Nes {
            cpu: Cpu::new(
                &mut memory.as_cpu_memory(),
                ProgramCounterSource::Override(CpuAddress::new(0x0000)),
            ),
            ppu: Ppu::new(),
            apu: Apu::new(),
            memory,
            cartridge,
            frame: Frame::new(),
            joypad1,
            joypad2,
            cycle: 0,
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
