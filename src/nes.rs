use std::cell::RefCell;
use std::rc::Rc;

use log::Level::Info;
use log::{info, log_enabled};

use crate::apu::apu::Apu;
use crate::apu::apu_registers::ApuRegisters;
use crate::cartridge::cartridge::Cartridge;
use crate::config::Config;
use crate::controller::joypad::Joypad;
use crate::cpu::cpu::{Cpu, NmiStatus, IrqStatus};
use crate::cpu::step::Step;
use crate::gui::gui::Events;
use crate::logging::formatter;
use crate::logging::formatter::*;
use crate::memory::cpu::ports::Ports;
use crate::memory::mapper_list;
use crate::memory::memory::Memory;
use crate::ppu::clock::Clock;
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
    minimal_formatter: MinimalFormatter,
    snapshots: Snapshots,
}

impl Nes {
    pub fn new(config: &Config) -> Nes {
        let (mapper, mapper_params) = mapper_list::lookup_mapper_with_params(&config.cartridge);
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
            minimal_formatter: MinimalFormatter,
            snapshots: Snapshots::new(),
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
        self.cpu.reset();
        self.memory.apu_regs_mut().reset();
    }

    pub fn step_frame(&mut self) {
        loop {
            let step_result = self.step();
            if step_result.is_last_cycle_of_frame {
                if self.cpu.jammed() {
                    info!("CPU is jammed!");
                }

                println!("{}", self.snapshots.format());
                println!();

                break;
            }
        }
    }

    pub fn step(&mut self) -> StepResult {
        let mut step = None;
        let ppu_result;
        match self.cycle % 6 {
            0 => {
                self.apu_step();
                step = self.cpu_step();
                ppu_result = self.ppu_step();
            }
            1 => ppu_result = self.ppu_step(),
            2 => {
                ppu_result = self.ppu_step();
                self.snapshots.start_next();
            }
            3 => {
                self.apu_step();
                step = self.cpu_step();
                ppu_result = self.ppu_step();
            }
            4 => ppu_result = self.ppu_step(),
            5 => {
                ppu_result = self.ppu_step();
                self.snapshots.start_next();
            }
            _ => unreachable!(),
        }

        self.cycle += 1;

        StepResult {
            step,
            is_last_cycle_of_frame: ppu_result.is_last_cycle_of_frame,
            nmi_scheduled: ppu_result.should_generate_nmi,
        }
    }

    fn apu_step(&mut self) {
        self.snapshots.current().apu_regs(&self.memory.apu_regs());
        self.apu.step(self.memory.apu_regs_mut());
    }

    fn cpu_step(&mut self) -> Option<Step> {
        self.snapshots.current().cpu_cycle(self.memory.cpu_cycle());

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

        if self.cpu.next_instruction_starting() {
            let formatted_instruction = self.minimal_formatter.format_instruction(self, "".into());
            self.snapshots.current().instruction(formatted_instruction);
        }

        self.snapshots.current().irq_status(self.cpu.irq_status());
        self.snapshots.current().nmi_status(self.cpu.nmi_status());

        step
    }

    fn ppu_step(&mut self) -> ppu::StepResult {
        self.snapshots.current().add_ppu_position(self.ppu.clock());

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

struct Snapshots {
    snapshots: Vec<Snapshot>,
    builder: SnapshotBuilder,
}

impl Snapshots {
    fn new() -> Snapshots {
        Snapshots {
            snapshots: Vec::new(),
            builder: SnapshotBuilder::new(),
        }
    }

    fn current(&mut self) -> &mut SnapshotBuilder {
        &mut self.builder
    }

    fn start_next(&mut self) {
        let snapshot = std::mem::take(&mut self.builder).build();
        self.snapshots.push(snapshot);
    }

    fn format(&self) -> String {
        let mut cpu_cycle  = "CPU Cycle ".to_string();
        let mut apu_cycle  = "APU Cycle ".to_string();
        let mut apu_parity = "Parity    ".to_string();
        let mut instr      = "CPU       ".to_string();
        let mut nmi_status = "NMI Status".to_string();
        let mut irq_status = "IRQ Status".to_string();
        let mut frame_irq  = "FRM       ".to_string();
        let mut ppu_vpos   = "PPU VPOS  ".to_string();
        let mut ppu_hpos   = "PPU HPOS  ".to_string();
        let mut indexes = vec![0];
        indexes.append(&mut (self.snapshots.len() - 10..self.snapshots.len() - 1).collect());
        for index in indexes {
            let snapshot = &self.snapshots[index];
            cpu_cycle.push_str(&Snapshots::center(snapshot.cpu_cycle.to_string()));
            apu_cycle.push_str(&Snapshots::center(snapshot.apu_cycle.to_string()));
            apu_parity.push_str(&Snapshots::center(snapshot.apu_parity.clone()));

            for (vpos, hpos) in snapshot.ppu_pos {
                ppu_vpos.push_str(&format!("[{:03}]", vpos));
                ppu_hpos.push_str(&format!("[{:03}]", hpos));
            }

            instr.push_str(&Snapshots::center(snapshot.instruction.clone()));
            if snapshot.nmi_status != NmiStatus::Inactive {
                nmi_status.push_str(&Snapshots::center(format!("{:?}", snapshot.nmi_status)));
            }

            if snapshot.irq_status != IrqStatus::Inactive {
                irq_status.push_str(&Snapshots::center(format!("{:?}", snapshot.irq_status)));
            }

            if snapshot.frame_irq {
                frame_irq.push_str(&Snapshots::center("Raise IRQ".to_string()));
            }
        }

        vec![cpu_cycle, apu_cycle, apu_parity, ppu_vpos, ppu_hpos, instr, nmi_status, irq_status, frame_irq].join("\n")
    }

    fn center(text: String) -> String {
       let back = (13 - text.len()) / 2;
       let front = 13 - text.len() - back;

       let mut result = "[".to_string();
       result.push_str(&String::from_utf8(vec![b' '; front]).unwrap());
       result.push_str(&text);
       result.push_str(&String::from_utf8(vec![b' '; back]).unwrap());
       result.push_str("]");
       result
    }
}

struct Snapshot {
    cpu_cycle: i64,
    apu_cycle: u16,
    apu_parity: String,
    instruction: String,
    frame_irq: bool,
    irq_status: IrqStatus,
    nmi_status: NmiStatus,
    ppu_pos: [(u16, u16); 3],
}

#[derive(Default)]
struct SnapshotBuilder {
    cpu_cycle: Option<i64>,
    apu_cycle: Option<u16>,
    apu_parity: Option<String>,
    instruction: String,
    frame_irq: Option<bool>,
    irq_status: Option<IrqStatus>,
    nmi_status: Option<NmiStatus>,
    ppu_pos: Vec<(u16, u16)>,
}

impl SnapshotBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn cpu_cycle(&mut self, value: i64) {
        self.cpu_cycle = Some(value);
    }

    fn apu_regs(&mut self, regs: &ApuRegisters) {
        self.frame_irq = Some(regs.frame_irq_pending());
        let clock = regs.clock();
        self.apu_cycle = Some(clock.cycle());
        let on_or_off = if clock.is_off_cycle() { "OFF" } else { "ON" };
        self.apu_parity = Some(on_or_off.to_string());
    }

    fn add_ppu_position(&mut self, clock: &Clock) {
        self.ppu_pos.push((clock.scanline(), clock.cycle()));
    }

    fn instruction(&mut self, value: String) {
        self.instruction = value;
    }

    fn irq_status(&mut self, irq_status: IrqStatus) {
        self.irq_status = Some(irq_status);
    }

    fn nmi_status(&mut self, nmi_status: NmiStatus) {
        self.nmi_status = Some(nmi_status);
    }

    fn build(self) -> Snapshot {
        Snapshot {
            cpu_cycle: self.cpu_cycle.unwrap(),
            apu_cycle: self.apu_cycle.unwrap(),
            apu_parity: self.apu_parity.unwrap(),
            frame_irq: self.frame_irq.unwrap(),
            irq_status: self.irq_status.unwrap(),
            nmi_status: self.nmi_status.unwrap(),
            ppu_pos: self.ppu_pos.try_into().unwrap(),
            instruction: self.instruction,
        }
    }
}

pub struct StepResult {
    pub step: Option<Step>,
    pub is_last_cycle_of_frame: bool,
    pub nmi_scheduled: bool,
}
