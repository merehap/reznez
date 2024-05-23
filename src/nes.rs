use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use log::Level::Info;
use log::{info, log_enabled};

use crate::apu::apu::Apu;
use crate::apu::apu_registers::{ApuRegisters, FrameCounterWriteStatus};
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

                break;
            }
        }
    }

    pub fn step(&mut self) -> StepResult {
        let mut step = None;
        let is_last_cycle_of_frame;
        match self.cycle % 6 {
            0 => {
                self.apu_step();
                step = self.cpu_step();
                is_last_cycle_of_frame = self.ppu_step();
            }
            1 => is_last_cycle_of_frame = self.ppu_step(),
            2 => {
                is_last_cycle_of_frame = self.ppu_step();
                self.snapshots.start_next();
            }
            3 => {
                self.apu_step();
                step = self.cpu_step();
                is_last_cycle_of_frame = self.ppu_step();
            }
            4 => is_last_cycle_of_frame = self.ppu_step(),
            5 => {
                is_last_cycle_of_frame = self.ppu_step();
                self.snapshots.start_next();
            }
            _ => unreachable!(),
        }

        self.cycle += 1;

        StepResult {
            step,
            is_last_cycle_of_frame,
        }
    }

    fn apu_step(&mut self) {
        if log_enabled!(target: "timings", Info) {
            self.snapshots.current().apu_regs(&self.memory.apu_regs());
        }

        self.apu.step(self.memory.apu_regs_mut());

        if log_enabled!(target: "timings", Info) {
            self.snapshots.current().frame_irq(&self.memory.apu_regs(), &self.cpu);
        }

        self.memory.apu_regs_mut().clock_mut().increment();
    }

    fn cpu_step(&mut self) -> Option<Step> {
        self.memory.as_cpu_memory().increment_cpu_cycle();

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

        if log_enabled!(target: "timings", Info) {
            if self.memory.apu_regs().frame_counter_write_status() == FrameCounterWriteStatus::Initialized {
                self.snapshots.start();
            }

            self.snapshots.current().cpu_cycle(self.memory.cpu_cycle());
            self.snapshots.current().irq_status(self.cpu.irq_status());
            self.snapshots.current().nmi_status(self.cpu.nmi_status());
            if self.cpu.next_instruction_starting() {
                let formatted_instruction = self.minimal_formatter.format_instruction(self, "".into());
                self.snapshots.current().instruction(formatted_instruction);
            }
        }

        step
    }

    fn ppu_step(&mut self) -> bool {
        let is_last_cycle_of_frame = self.ppu.clock_mut().tick(self.memory.ppu_regs().rendering_enabled());
        if log_enabled!(target: "timings", Info) {
            self.snapshots.current().add_ppu_position(self.ppu.clock());
        }

        let should_generate_nmi = self
            .ppu
            .step(&mut self.memory.as_ppu_memory(), &mut self.frame);
        if should_generate_nmi {
            self.cpu.schedule_nmi();
        }

        is_last_cycle_of_frame
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
    active: bool,
    snapshots: Vec<Snapshot>,
    builder: SnapshotBuilder,
    max_count: usize,
}

impl Snapshots {
    fn new() -> Snapshots {
        Snapshots {
            active: false,
            snapshots: Vec::new(),
            builder: SnapshotBuilder::new(),
            max_count: 29832 + 6,
        }
    }

    fn start(&mut self) {
        self.snapshots = Vec::new();
        self.active = true;
    }

    fn clear(&mut self) {
        self.snapshots = Vec::new();
        self.builder = SnapshotBuilder::new();
    }

    fn count(&self) -> usize {
        self.snapshots.len()
    }

    fn current(&mut self) -> &mut SnapshotBuilder {
        &mut self.builder
    }

    fn start_next(&mut self) {
        if !self.active {
            return;
        }

        let snapshot = std::mem::take(&mut self.builder).build();
        self.snapshots.push(snapshot);

        if self.count() >= self.max_count {
            self.active = false;
            info!("{}", self.format());
            info!("");
            self.clear();
            return;
        }
    }

    fn format(&self) -> String {
        let mut cpu_cycle   = "CPU Cycle   ".to_string();
        let mut apu_cycle   = "APU Cycle   ".to_string();
        let mut cycle_count = "Cycle Offset".to_string();
        let mut apu_parity  = "Parity      ".to_string();
        let mut instr       = "CPU         ".to_string();
        let mut fcw_status  = "FRM Count   ".to_string();
        let mut nmi_status  = "NMI Status  ".to_string();
        let mut irq_status  = "IRQ Status  ".to_string();
        let mut frame_irq   = "FRM         ".to_string();
        let mut ppu_vpos    = "PPU VPOS    ".to_string();
        let mut ppu_hpos    = "PPU HPOS    ".to_string();

        let mut append_cycle = |index, skip| {
            let snapshot: &Snapshot = &self.snapshots[index];
            append(&mut cpu_cycle, &center(snapshot.cpu_cycle.to_string()), true, skip);
            append(&mut apu_cycle, &center(snapshot.apu_cycle.to_string()), true, skip);
            append(&mut cycle_count, &center((snapshot.cpu_cycle - self.snapshots[0].cpu_cycle).to_string()), true, skip);
            append(&mut apu_parity, &center(snapshot.apu_parity.clone()), true, skip);

            let mut vpos = String::new();
            let mut hpos = String::new();
            for (v, h) in snapshot.ppu_pos {
                vpos.push_str(&center_n(3, v.to_string()));
                hpos.push_str(&center_n(3, h.to_string()));
            }

            append(&mut ppu_vpos, &vpos, true, skip);
            append(&mut ppu_hpos, &hpos, true, skip);

            append(&mut instr, &center(format!("{}", snapshot.instruction.clone())), true, skip);
            append(&mut fcw_status, &center(format!("{:?}", snapshot.frame_counter_write_status)),
                snapshot.frame_counter_write_status != FrameCounterWriteStatus::Inactive, skip);
            append(&mut nmi_status, &center(format!("{:?}", snapshot.nmi_status)), snapshot.nmi_status != NmiStatus::Inactive, skip);
            append(&mut irq_status, &center(format!("{:?}", snapshot.irq_status)), snapshot.irq_status != IrqStatus::Inactive, skip);
            append(&mut frame_irq, &center("Raise IRQ".to_string()), snapshot.frame_irq, skip);
        };

        append_cycle(0, false);
        append_cycle(1, true);

        let len = self.snapshots.len();
        for index in len - 10..len {
            append_cycle(index, false);
        }

        vec![cpu_cycle, apu_cycle, cycle_count, apu_parity, instr,
             nmi_status, irq_status, frame_irq, /*fcw_status, */ppu_vpos, ppu_hpos].join("\n")
    }
}

fn append(field: &mut String, value: &str, active: bool, skip: bool) {
    let result = if skip {
        "........"
    } else if active {
        value
    } else {
        "               "
    };

    field.push_str(result);
}

fn center(text: String) -> String {
    center_n(13, text)
}

fn center_n(n: usize, text: String) -> String {
    assert!(n >= 2);

    let text: String = text.chars().take(n).collect();
    let back = (n - text.len()) / 2;
    let front = n - text.len() - back;

    let mut result = "[".to_string();
    result.push_str(&String::from_utf8(vec![b' '; front]).unwrap());
    result.push_str(&text);
    result.push_str(&String::from_utf8(vec![b' '; back]).unwrap());
    result.push_str("]");
    result
}

struct Snapshot {
    cpu_cycle: i64,
    apu_cycle: u16,
    apu_parity: String,
    instruction: String,
    frame_counter_write_status: FrameCounterWriteStatus,
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
    frame_counter_write_status: Option<FrameCounterWriteStatus>,
    frame_irq: Option<bool>,
    irq_status: Option<IrqStatus>,
    nmi_status: Option<NmiStatus>,
    ppu_pos: VecDeque<(u16, u16)>,
}

impl SnapshotBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn cpu_cycle(&mut self, value: i64) {
        self.cpu_cycle = Some(value);
    }

    fn apu_regs(&mut self, regs: &ApuRegisters) {
        let clock = regs.clock();
        self.apu_cycle = Some(clock.cycle());
        let on_or_off = if clock.is_off_cycle() { "OFF" } else { "ON" };
        self.apu_parity = Some(on_or_off.to_string());
        self.frame_counter_write_status = Some(regs.frame_counter_write_status());
    }

    fn frame_irq(&mut self, regs: &ApuRegisters, cpu: &Cpu) {
        self.frame_irq = Some(regs.frame_irq_pending() && !cpu.status().interrupts_disabled);
    }

    fn add_ppu_position(&mut self, clock: &Clock) {
        assert!(self.ppu_pos.len() < 4);
        if self.ppu_pos.len() == 3 {
            self.ppu_pos.pop_front();
        }

        self.ppu_pos.push_back((clock.scanline(), clock.cycle()));
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
        assert_eq!(self.ppu_pos.len(), 3);
        Snapshot {
            cpu_cycle: self.cpu_cycle.unwrap(),
            apu_cycle: self.apu_cycle.unwrap(),
            apu_parity: self.apu_parity.unwrap(),
            instruction: self.instruction,
            frame_counter_write_status: self.frame_counter_write_status.unwrap(),
            frame_irq: self.frame_irq.unwrap(),
            irq_status: self.irq_status.unwrap(),
            nmi_status: self.nmi_status.unwrap(),
            ppu_pos: [self.ppu_pos[0], self.ppu_pos[1], self.ppu_pos[2]],
        }
    }
}

pub struct StepResult {
    pub step: Option<Step>,
    pub is_last_cycle_of_frame: bool,
}
