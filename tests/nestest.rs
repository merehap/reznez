extern crate reznez;

use std::fmt;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::Path;
use std::str::FromStr;

use reznez::config::Config;
use reznez::cpu::address::Address;
use reznez::cpu::instruction::OpCode;
use reznez::cpu::status::Status;
use reznez::nes::Nes;
use reznez::ppu::render::frame::Frame;

#[test]
fn nestest() {
    let f = File::open("testdata/nestest_expected").expect("Test data not found!");
    let mut expected_states = BufReader::new(f)
        .lines()
        .map(|line| State::from_text(line.unwrap()));

    let config = Config::with_override_program_counter(
        Path::new("testroms/nestest.nes"),
        Address::new(0xC000),
    );
    let mut nes = Nes::new(config);

    let mut frame = Frame::new();
    loop {
        let program_counter = nes.cpu().program_counter();
        let a = nes.cpu().accumulator();
        let x = nes.cpu().x_index();
        let y = nes.cpu().y_index();
        let p = nes.cpu().status();
        let s = nes.cpu().stack_pointer();
        let ppu_cycle = nes.ppu().clock().cycle();
        let ppu_scanline = nes.ppu().clock().scanline();
        let c = nes.cpu().cycle();

        if let Some(instruction) = nes.step(&mut frame) {
            if let Some(expected_state) = expected_states.next() {
                let state = State {
                    program_counter,
                    code_point: instruction.template.code_point,
                    op_code: instruction.template.op_code,
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
                    panic!("State diverged from expected state!\nExpected:\n{}\nActual:\n{}", expected_state, state);
                }
            } else {
                break;
            }
        }

    }
}

#[derive(PartialEq, Eq, Debug)]
struct State {
    program_counter: Address,
    code_point: u8,
    op_code: OpCode,
    a: u8,
    x: u8,
    y: u8,
    p: Status,
    s: u8,
    ppu_cycle: u16,
    ppu_scanline: u16,
    c: u64,
}

impl State {
    fn from_text(line: String) -> State {
        let mut raw_op_code = &line[16..19];
        // nestest uses a diffent moniker for ISC.
        if raw_op_code == "ISB" {
            raw_op_code = "ISC";
        }

        State {
            program_counter: Address::new(u16::from_str_radix(&line[0..4], 16).unwrap()),
            code_point: u8::from_str_radix(&line[6..8], 16).unwrap(),
            op_code: OpCode::from_str(raw_op_code).unwrap(),
            a: u8::from_str_radix(&line[50..52], 16).unwrap(),
            x: u8::from_str_radix(&line[55..57], 16).unwrap(),
            y: u8::from_str_radix(&line[60..62], 16).unwrap(),
            p: Status::from_byte(u8::from_str_radix(&line[65..67], 16).unwrap()),
            s: u8::from_str_radix(&line[71..73], 16).unwrap(),
            ppu_cycle: u16::from_str_radix(&line[78..81].trim(), 10).unwrap(),
            ppu_scanline: u16::from_str_radix(&line[82..85].trim(), 10).unwrap(),
            c: u64::from_str_radix(&line[90..], 10).unwrap(),
        }
    }
}

impl fmt::Display for State {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> fmt::Result {
        write!(f, "State {{PC:{}, CodePoint:0x{:02X}, OpCode:{:?}, A:0x{:02X}, X:0x{:02X}, Y:0x{:02X}, P:{} (0x{:2X}), S:0x{:02X}, C:{:05}, PPUC:{:03}, PPUS:{:03}}}",
               self.program_counter, self.code_point, self.op_code, self.a,
               self.x, self.y, self.p.to_string(), self.p.to_instruction_byte(), self.s, self.c, self.ppu_cycle, self.ppu_scanline)
    }
}
