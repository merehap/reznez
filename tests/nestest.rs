#![feature(panic_update_hook)]

extern crate reznez;

use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;

use reznez::config::{Config, GuiType, Opt};
use reznez::cpu::instruction::{Instruction, OpCode};
use reznez::cpu::status::Status;
use reznez::memory::cpu::cpu_address::CpuAddress;
use reznez::nes::Nes;
use reznez::ppu::clock::{Clock, MAX_SCANLINE, MAX_CYCLE};
use reznez::ppu::render::frame_rate::TargetFrameRate;
use reznez::logging::logger;
use reznez::logging::logger::Logger;

#[test]
fn nestest() {
    let f = File::open("tests/data/nestest_expected").expect("Test data not found!");
    let expected_states = BufReader::new(f)
        .lines()
        .map(|line| State::from_text(line.unwrap()));

    let opt = Opt {
        gui: GuiType::NoGui,
        target_frame_rate: TargetFrameRate::Unbounded,
        disable_audio: true,
        log_cpu_all: true,
        prevent_saving: true,
        // This ROM has the RESET vector overriddden to point to where the headless nestest starts.
        ..Opt::new(PathBuf::from("tests/roms/nestest#ignored.nes"))
    };

    logger::init(Logger {
        log_cpu_instructions: true,
        log_cpu_flow_control: true,
        log_cpu_steps: true,
        ..Logger::default()
    }).unwrap();

    std::panic::update_hook(|prev, info| {
        log::logger().flush();
        prev(info);
    });

    let mut config = Config::new(&opt);
    // Nestest starts the first instruction a cycle early compared to the NES Manual and Mesen.
    config.starting_cpu_cycle = -1;
    // Nestest starts the first instruction on cycle 0, but PPU stuff happens before that.
    config.ppu_clock = Clock::starting_at(-1, MAX_SCANLINE, MAX_CYCLE - 21);

    let mut nes = Nes::new(&config, &opt.rom_path);

    // Step past the Start sequence.
    for _ in 0..21 {
        nes.step();
    }

    for expected_state in expected_states {
        let c;
        let ppu_cycle;
        let ppu_scanline;

        let current_instruction: Instruction;
        loop {
            if nes.step().step.is_some()
                    && let Some((instruction, _)) = nes.cpu().mode_state().new_instruction_with_address() {
                current_instruction = instruction;
                c = nes.memory().cpu_cycle();
                ppu_cycle = nes.memory().ppu_regs().clock().cycle();
                ppu_scanline = nes.memory().ppu_regs().clock().scanline();
                break;
            }
        }

        let program_counter = nes.cpu().address_bus();

        let mut a;
        let mut x;
        let mut y;
        let mut p;
        let mut s;
        loop {
            a = nes.cpu().accumulator();
            x = nes.cpu().x_index();
            y = nes.cpu().y_index();
            p = nes.cpu().status();
            s = nes.stack_pointer();

            if let Some(step) = nes.step().step && step.has_interpret_op_code() {
                break;
            }
        }

        let state = State {
            program_counter,
            code_point: current_instruction.code_point(),
            op_code: current_instruction.op_code(),
            a,
            x,
            y,
            p,
            s,
            ppu_cycle,
            ppu_scanline,
            c,
        };

        if state != expected_state {
            panic!("State diverged from expected state!\nExpected:\n{expected_state}\nActual:\n{state}");
        }
    }

    log::logger().flush();
}

#[derive(PartialEq, Eq, Debug)]
struct State {
    program_counter: CpuAddress,
    code_point: u8,
    op_code: OpCode,
    a: u8,
    x: u8,
    y: u8,
    p: Status,
    s: u8,
    ppu_cycle: u16,
    ppu_scanline: u16,
    c: i64,
}

impl State {
    fn from_text(line: String) -> State {
        let mut raw_op_code = &line[16..19];
        // nestest uses a different moniker for ISC.
        if raw_op_code == "ISB" {
            raw_op_code = "ISC";
        }

        State {
            program_counter: CpuAddress::new(
                u16::from_str_radix(&line[0..4], 16).unwrap(),
            ),
            code_point: u8::from_str_radix(&line[6..8], 16).unwrap(),
            op_code: OpCode::from_str(raw_op_code).unwrap(),
            a: u8::from_str_radix(&line[50..52], 16).unwrap(),
            x: u8::from_str_radix(&line[55..57], 16).unwrap(),
            y: u8::from_str_radix(&line[60..62], 16).unwrap(),
            p: Status::from_byte(u8::from_str_radix(&line[65..67], 16).unwrap()),
            s: u8::from_str_radix(&line[71..73], 16).unwrap(),
            ppu_cycle: str::parse(line[78..81].trim()).unwrap(),
            ppu_scanline: str::parse(line[82..85].trim()).unwrap(),
            c: str::parse(&line[90..]).unwrap(),
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "State {{PC:{}, CodePoint:0x{:02X}, OpCode:{:?}, A:0x{:02X}, X:0x{:02X}, Y:0x{:02X}, P:{} (0x{:02X}), S:0x{:02X}, C:{:05}, PPUC:{:03}, PPUS:{:03}}}",
               self.program_counter, self.code_point, self.op_code, self.a,
               self.x, self.y, self.p, self.p.to_register_byte(), self.s, self.c, self.ppu_cycle, self.ppu_scanline)
    }
}
