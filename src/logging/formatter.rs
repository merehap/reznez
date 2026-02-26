#![allow(dead_code)]

use std::fmt::Write;

use crate::cpu::dmc_dma::DmcDmaState;
use crate::cpu::instruction::{Instruction, OpCode, AccessMode};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::bus::AddressBusType;
use crate::nes::Nes;

pub trait Formatter {
    fn format_instruction(
        &self,
        nes: &Nes,
        instruction: Instruction,
        start_address: CpuAddress,
        interrupt_text: String,
    ) -> String;
}

pub struct MinimalFormatter;

impl Formatter for MinimalFormatter {
    fn format_instruction(
        &self,
        nes: &Nes,
        instruction: Instruction,
        start_address: CpuAddress,
        _interrupt_text: String,
    ) -> String {
        // FIXME: This isn't the correct bus value.
        let peek = |address| nes.bus().cpu_peek(nes.mapper(), AddressBusType::Cpu, address);

        let cpu = nes.cpu();

        // FIXME: These aren't the correct bus values.
        let low = peek(start_address.offset(1));
        let high = peek(start_address.offset(2));

        let mut argument_string = String::new();
        use AccessMode::*;
        match instruction.access_mode() {
            Imp => {}
            Imm => {
                write!(argument_string, "#${low:02X}").unwrap();
            }
            ZP => {
                write!(argument_string, "${low:02X}").unwrap();
            }
            ZPX => {
                let address = low.wrapping_add(cpu.x_index());
                write!(argument_string, "${address:02X}").unwrap();
            }
            ZPY => {
                let address = low.wrapping_add(cpu.y_index());
                write!(argument_string, "${address:02X}").unwrap();
            }
            Abs => {
                write!(argument_string, "${high:02X}{low:02X}").unwrap();
            }
            AbX => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.x_index());
                write!(argument_string, "${:04X}", *address).unwrap();
            }
            AbY => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.y_index());
                write!(argument_string, "${:04X}", *address).unwrap();
            }
            Rel => {
                let address = start_address
                    .offset(low as i8)
                    .advance(instruction.access_mode().instruction_length());
                write!(argument_string, "${:04X}", *address).unwrap();
            }
            Ind => {
                let first = CpuAddress::from_low_high(low, high);
                let second = CpuAddress::from_low_high(low.wrapping_add(1), high);
                let address = CpuAddress::from_low_high(
                    peek(first),
                    peek(second),
                );
                write!(argument_string, "${:04X}", *address).unwrap();
            }
            IzX => {
                let low = low.wrapping_add(cpu.x_index());
                let address = CpuAddress::from_low_high(
                    peek(CpuAddress::zero_page(low)),
                    peek(CpuAddress::zero_page(low.wrapping_add(1))),
                );
                write!(argument_string, "${:04X}", *address).unwrap();
            }
            IzY => {
                let start_address = CpuAddress::from_low_high(
                    peek(CpuAddress::zero_page(low)),
                    peek(CpuAddress::zero_page(low.wrapping_add(1))),
                );
                let address = start_address.advance(cpu.y_index());
                write!(argument_string, "${:04X}", *address).unwrap();
            }
        }

        format!("{:?} {}", instruction.op_code(), argument_string)
    }
}

pub struct Nintendulator0980Formatter;

impl Formatter for Nintendulator0980Formatter {
    fn format_instruction(
        &self,
        nes: &Nes,
        instruction: Instruction,
        start_address: CpuAddress,
        _interrupt_text: String,
    ) -> String {
        let cpu_cycle = nes.bus().cpu_cycle();
        let peek = |address| nes.bus().cpu_peek(nes.mapper(), AddressBusType::Cpu, address);

        let cpu = nes.cpu();

        let low = peek(start_address.offset(1));
        let high = peek(start_address.offset(2));

        let mut argument_string = String::new();
        use AccessMode::*;
        match instruction.access_mode() {
            Imp => {}
            Imm => {
                write!(argument_string, "#${low:02X}").unwrap();
            }
            ZP => {
                let address = CpuAddress::zero_page(low);
                let value = peek(address);
                write!(argument_string, "${low:02X} = {value:02X}").unwrap();
            }
            ZPX => {
                write!(argument_string, "${low:02X},X").unwrap();
                let _address = CpuAddress::zero_page(low.wrapping_add(cpu.x_index()));
            }
            ZPY => {
                write!(argument_string, "${low:02X},Y").unwrap();
                let _address = CpuAddress::zero_page(low.wrapping_add(cpu.y_index()));
            }
            Abs => {
                let address = CpuAddress::from_low_high(low, high);
                write!(argument_string, "${high:02X}{low:02X}").unwrap();
                if instruction.op_code() != OpCode::JMP {
                    let value = peek(address);
                    write!(argument_string, " = {value:02X}").unwrap();
                }
            }
            AbX => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.x_index());
                let value = peek(address);
                write!(argument_string, "${high:02X}{low:02X},X = {value:02X}").unwrap();
            }
            AbY => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.y_index());
                let value = peek(address);
                write!(argument_string, "${high:02X}{low:02X},Y = {value:02X}").unwrap();
            }
            Rel => {
                let address = start_address
                    .offset(low as i8)
                    .advance(instruction.access_mode().instruction_length());
                write!(argument_string, "${:04X}", *address).unwrap();
            }
            Ind => {
                let first = CpuAddress::from_low_high(low, high);
                let second = CpuAddress::from_low_high(low.wrapping_add(1), high);
                let _address = CpuAddress::from_low_high(
                    peek(first),
                    peek(second),
                );
            }
            IzX => {
                write!(argument_string, "(${low:02X}),X =").unwrap();
                let low = low.wrapping_add(cpu.x_index());
                let _address = CpuAddress::from_low_high(
                    peek(CpuAddress::zero_page(low)),
                    peek(CpuAddress::zero_page(low.wrapping_add(1))),
                );
            }
            IzY => {
                write!(argument_string, "(${low:02X}),Y =").unwrap();
                let start_address = CpuAddress::from_low_high(
                    peek(CpuAddress::zero_page(low)),
                    peek(CpuAddress::zero_page(low.wrapping_add(1))),
                );
                // TODO: Should this wrap around just the current page?
                let _address = start_address.advance(cpu.y_index());
            }
        }

        let instr_bytes = match instruction.access_mode().instruction_length() {
            1 => format!("{:02X}      ", instruction.code_point()),
            2 => format!("{:02X} {:02X}    ", instruction.code_point(), low),
            3 => format!("{:02X} {:02X} {:02X} ", instruction.code_point(), low, high),
            _ => unreachable!(),
        };

        format!(
            "{:04X}  {:<9} {:?} {:28}{} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:>3},{:>3} CYC:{}",
            *start_address,
            instr_bytes,
            instruction.op_code(),
            argument_string,
            interrupts(nes),
            cpu.accumulator(),
            cpu.x_index(),
            cpu.y_index(),
            cpu.status().to_register_byte() | 0b0010_0000,
            cpu.stack_pointer(),
            nes.bus().ppu_clock().cycle(),
            nes.bus().ppu_clock().scanline(),
            cpu_cycle,
        )
    }
}

pub struct MesenFormatter;

impl Formatter for MesenFormatter {
    fn format_instruction(
        &self,
        nes: &Nes,
        instruction: Instruction,
        start_address: CpuAddress,
        _interrupt_text: String,
    ) -> String {
        let maybe_peek = |address| nes.bus().cpu_peek_unresolved(nes.mapper(), AddressBusType::Cpu, address);
        let peek = |address| nes.bus().cpu_peek(nes.mapper(), AddressBusType::Cpu, address);

        let cpu = nes.cpu();

        let low = peek(start_address.offset(1));
        let high = peek(start_address.offset(2));

        let mut argument_string = String::new();
        use AccessMode::*;
        match instruction.access_mode() {
            Imp => {
                if matches!(instruction.op_code(), OpCode::LSR | OpCode::ROR | OpCode::ROL | OpCode::ASL) {
                    argument_string.push('A');
                }
            }
            Imm => {
                write!(argument_string, "#${low:02X}").unwrap();
            }
            ZP => {
                let address = CpuAddress::zero_page(low);
                let value = maybe_peek(address).resolve(low);
                write!(argument_string, "${low:02X} = ${value:02X}").unwrap();
            }
            ZPX => {
                write!(argument_string, "${low:02X},X").unwrap();
                let address = CpuAddress::zero_page(low.wrapping_add(cpu.x_index()));
                write!(argument_string, " [{}] = ${:02X}", address.to_mesen_string(), peek(address)).unwrap();
            }
            ZPY => {
                write!(argument_string, "${low:02X},Y").unwrap();
                let address = CpuAddress::zero_page(low.wrapping_add(cpu.y_index()));
                write!(argument_string, " [{}] = ${:02X}", address.to_mesen_string(), peek(address)).unwrap();
            }
            Abs => {
                let address = CpuAddress::from_low_high(low, high);
                argument_string.push_str(&address.to_mesen_string());
                match instruction.op_code() {
                    OpCode::JMP | OpCode::JSR => {}
                    _ => write!(argument_string, " = ${:02X}", maybe_peek(address).resolve(high)).unwrap(),
                }
            }
            AbX => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.x_index());
                let value = maybe_peek(address).resolve(high);
                write!(
                    argument_string,
                    "{},X [{}] = ${value:02X}",
                    start_address.to_mesen_string(),
                    address.to_mesen_string(),
                ).unwrap();
            }
            AbY => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.y_index());
                let value = peek(address);
                write!(
                    argument_string,
                    "{},Y [{}] = ${value:02X}",
                    start_address.to_mesen_string(),
                    address.to_mesen_string(),
                ).unwrap();
            }
            Rel => {
                let address = start_address
                    .offset(low as i8)
                    .advance(instruction.access_mode().instruction_length());
                argument_string.push_str(&address.to_mesen_string());
            }
            Ind => {
                let first = CpuAddress::from_low_high(low, high);
                let second = CpuAddress::from_low_high(low.wrapping_add(1), high);
                let _address = CpuAddress::from_low_high(
                    peek(first),
                    peek(second),
                );

                write!(argument_string, "({})", first.to_mesen_string()).unwrap();
            }
            IzX => {
                write!(argument_string, "(${low:02X}),X =").unwrap();
                let low = low.wrapping_add(cpu.x_index());
                let _address = CpuAddress::from_low_high(
                    peek(CpuAddress::zero_page(low)),
                    peek(CpuAddress::zero_page(low.wrapping_add(1))),
                );
            }
            IzY => {
                write!(argument_string, "(${low:02X}),Y ").unwrap();
                let start_address = CpuAddress::from_low_high(
                    peek(CpuAddress::zero_page(low)),
                    peek(CpuAddress::zero_page(low.wrapping_add(1))),
                );
                // TODO: Should this wrap around just the current page?
                let address = start_address.advance(cpu.y_index());
                write!(argument_string, "[{}] = ${:02X}", address.to_mesen_string(), peek(address)).unwrap();
            }
        }

        let mut scanline = nes.bus().ppu_clock().scanline() as i16;
        if scanline == 261 {
            scanline = -1;
        }

        format!(
            "{:04X}  {:?} {:28} A:{:02X} X:{:02X} Y:{:02X} S:{:02X} P:{} V:{:<3} H:{:<3} Cycle:{}",
            *start_address,
            instruction.op_code(),
            argument_string,
            cpu.accumulator(),
            cpu.x_index(),
            cpu.y_index(),
            cpu.stack_pointer(),
            cpu.status().to_mesen_string(),
            scanline,
            nes.bus().ppu_clock().cycle(),
            nes.bus().cpu_cycle(),
        )
    }
}

pub fn interrupts(nes: &Nes) -> String {
    let mut interrupts = String::new();
    interrupts.push(if nes.bus().cpu_pinout.frame_irq_asserted() { 'F' } else {'-'});
    interrupts.push(if nes.bus().cpu_pinout.dmc_irq_asserted() { 'D' } else {'-'});
    interrupts.push(if nes.bus().cpu_pinout.mapper_irq_asserted() { 'M' } else {'-'});
    interrupts.push(if nes.cpu().nmi_pending() { 'N' } else {'-'});
    interrupts.push(if nes.bus().oam_dma.dma_pending() { 'O' } else {'-'});
    interrupts.push(if nes.bus().dmc_dma.state() == DmcDmaState::Idle { '-' } else {'D'});

    interrupts
}
