#![allow(dead_code)]

use crate::cpu::dmc_dma::DmcDmaState;
use crate::cpu::instruction::{Instruction, OpCode, AccessMode};
use crate::memory::cpu::cpu_address::CpuAddress;
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
        let peek = |address| nes.mapper().cpu_peek(nes.memory(), address).resolve(nes.memory().cpu_data_bus).0;

        let cpu = nes.cpu();

        // FIXME: These aren't the correct bus values.
        let low = peek(start_address.offset(1));
        let high = peek(start_address.offset(2));

        let mut argument_string = String::new();
        use AccessMode::*;
        match instruction.access_mode() {
            Imp => {}
            Imm => {
                argument_string.push_str(&format!("#${low:02X}"));
            }
            ZP => {
                argument_string.push_str(&format!("${low:02X}"));
            }
            ZPX => {
                let address = low.wrapping_add(cpu.x_index());
                argument_string.push_str(&format!("${address:02X}"));
            }
            ZPY => {
                let address = low.wrapping_add(cpu.y_index());
                argument_string.push_str(&format!("${address:02X}"));
            }
            Abs => {
                argument_string.push_str(&format!("${high:02X}{low:02X}"));
            }
            AbX => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.x_index());
                argument_string.push_str(&format!("${:04X}", address.to_raw()));
            }
            AbY => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.y_index());
                argument_string.push_str(&format!("${:04X}", address.to_raw()));
            }
            Rel => {
                let address = start_address
                    .offset(low as i8)
                    .advance(instruction.access_mode().instruction_length());
                argument_string.push_str(&format!("${:04X}", address.to_raw()));
            }
            Ind => {
                let first = CpuAddress::from_low_high(low, high);
                let second = CpuAddress::from_low_high(low.wrapping_add(1), high);
                let address = CpuAddress::from_low_high(
                    peek(first),
                    peek(second),
                );
                argument_string.push_str(&format!("${:04X}", address.to_raw()));
            }
            IzX => {
                let low = low.wrapping_add(cpu.x_index());
                let address = CpuAddress::from_low_high(
                    peek(CpuAddress::zero_page(low)),
                    peek(CpuAddress::zero_page(low.wrapping_add(1))),
                );
                argument_string.push_str(&format!("${:04X}", address.to_raw()));
            }
            IzY => {
                let start_address = CpuAddress::from_low_high(
                    peek(CpuAddress::zero_page(low)),
                    peek(CpuAddress::zero_page(low.wrapping_add(1))),
                );
                let address = start_address.advance(cpu.y_index());
                argument_string.push_str(&format!("${:04X}", address.to_raw()));
            }
        };

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
        let cpu_cycle = nes.memory().cpu_cycle();
        let peek = |address| nes.mapper().cpu_peek(nes.memory(), address).resolve(nes.memory().cpu_data_bus).0;

        let cpu = nes.cpu();

        let low = peek(start_address.offset(1));
        let high = peek(start_address.offset(2));

        let mut argument_string = String::new();
        use AccessMode::*;
        match instruction.access_mode() {
            Imp => {}
            Imm => {
                argument_string.push_str(&format!("#${low:02X}"));
            }
            ZP => {
                let address = CpuAddress::zero_page(low);
                let value = peek(address);
                argument_string.push_str(&format!("${low:02X} = {value:02X}"));
            }
            ZPX => {
                argument_string.push_str(&format!("${low:02X},X"));
                let _address = CpuAddress::zero_page(low.wrapping_add(cpu.x_index()));
            }
            ZPY => {
                argument_string.push_str(&format!("${low:02X},Y"));
                let _address = CpuAddress::zero_page(low.wrapping_add(cpu.y_index()));
            }
            Abs => {
                let address = CpuAddress::from_low_high(low, high);
                argument_string.push_str(&format!("${high:02X}{low:02X}"));
                if instruction.op_code() != OpCode::JMP {
                    let value = peek(address);
                    argument_string.push_str(&format!(" = {value:02X}"));
                }
            }
            AbX => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.x_index());
                let value = peek(address);
                argument_string.push_str(&format!("${high:02X}{low:02X},X = {value:02X}"));
            }
            AbY => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.y_index());
                let value = peek(address);
                argument_string.push_str(&format!("${high:02X}{low:02X},Y = {value:02X}"));
            }
            Rel => {
                let address = start_address
                    .offset(low as i8)
                    .advance(instruction.access_mode().instruction_length());
                argument_string.push_str(&format!("${:04X}", address.to_raw()));
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
                argument_string.push_str(&format!("(${low:02X}),X ="));
                let low = low.wrapping_add(cpu.x_index());
                let _address = CpuAddress::from_low_high(
                    peek(CpuAddress::zero_page(low)),
                    peek(CpuAddress::zero_page(low.wrapping_add(1))),
                );
            }
            IzY => {
                argument_string.push_str(&format!("(${low:02X}),Y ="));
                let start_address = CpuAddress::from_low_high(
                    peek(CpuAddress::zero_page(low)),
                    peek(CpuAddress::zero_page(low.wrapping_add(1))),
                );
                // TODO: Should this wrap around just the current page?
                let _address = start_address.advance(cpu.y_index());
            }
        };

        let instr_bytes = match instruction.access_mode().instruction_length() {
            1 => format!("{:02X}      ", instruction.code_point()),
            2 => format!("{:02X} {:02X}    ", instruction.code_point(), low),
            3 => format!("{:02X} {:02X} {:02X} ", instruction.code_point(), low, high),
            _ => unreachable!(),
        };

        format!(
            "{:04X}  {:<9} {:?} {:28}{} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:>3},{:>3} CYC:{}",
            start_address.to_raw(),
            instr_bytes,
            instruction.op_code(),
            argument_string,
            interrupts(nes),
            cpu.accumulator(),
            cpu.x_index(),
            cpu.y_index(),
            cpu.status().to_register_byte() | 0b0010_0000,
            nes.memory().stack_pointer(),
            nes.memory().ppu_regs().clock().cycle(),
            nes.memory().ppu_regs().clock().scanline(),
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
        let maybe_peek = |address| nes.mapper().cpu_peek(nes.memory(), address);
        let peek = |address| nes.mapper().cpu_peek(nes.memory(), address).resolve(nes.memory().cpu_data_bus).0;

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
                argument_string.push_str(&format!("#${low:02X}"));
            }
            ZP => {
                let address = CpuAddress::zero_page(low);
                let value = maybe_peek(address).resolve(low).0;
                argument_string.push_str(&format!("${low:02X} = ${value:02X}"));
            }
            ZPX => {
                argument_string.push_str(&format!("${low:02X},X"));
                let address = CpuAddress::zero_page(low.wrapping_add(cpu.x_index()));
                argument_string.push_str(&format!(" [{}] = ${:02X}", address.to_mesen_string(), peek(address)));
            }
            ZPY => {
                argument_string.push_str(&format!("${low:02X},Y"));
                let address = CpuAddress::zero_page(low.wrapping_add(cpu.y_index()));
                argument_string.push_str(&format!(" [{}] = ${:02X}", address.to_mesen_string(), peek(address)));
            }
            Abs => {
                let address = CpuAddress::from_low_high(low, high);
                argument_string.push_str(&address.to_mesen_string());
                match instruction.op_code() {
                    OpCode::JMP | OpCode::JSR => {}
                    _ => argument_string.push_str(&format!(" = ${:02X}", maybe_peek(address).resolve(high).0)),
                }
            }
            AbX => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.x_index());
                let value = maybe_peek(address).resolve(high).0;
                argument_string.push_str(&format!(
                        "{},X [{}] = ${value:02X}",
                        start_address.to_mesen_string(),
                        address.to_mesen_string()));
            }
            AbY => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.y_index());
                let value = peek(address);
                argument_string.push_str(&format!(
                        "{},Y [{}] = ${value:02X}",
                        start_address.to_mesen_string(),
                        address.to_mesen_string()));
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

                argument_string.push_str(&format!("({})", &first.to_mesen_string()));
            }
            IzX => {
                argument_string.push_str(&format!("(${low:02X}),X ="));
                let low = low.wrapping_add(cpu.x_index());
                let _address = CpuAddress::from_low_high(
                    peek(CpuAddress::zero_page(low)),
                    peek(CpuAddress::zero_page(low.wrapping_add(1))),
                );
            }
            IzY => {
                argument_string.push_str(&format!("(${low:02X}),Y "));
                let start_address = CpuAddress::from_low_high(
                    peek(CpuAddress::zero_page(low)),
                    peek(CpuAddress::zero_page(low.wrapping_add(1))),
                );
                // TODO: Should this wrap around just the current page?
                let address = start_address.advance(cpu.y_index());
                argument_string.push_str(&format!("[{}] = ${:02X}", address.to_mesen_string(), peek(address)));
            }
        };

        let mut scanline = nes.memory().ppu_regs().clock().scanline() as i16;
        if scanline == 261 {
            scanline = -1;
        }

        format!(
            "{:04X}  {:?} {:28} A:{:02X} X:{:02X} Y:{:02X} S:{:02X} P:{} V:{:<3} H:{:<3} Cycle:{}",
            start_address.to_raw(),
            instruction.op_code(),
            argument_string,
            cpu.accumulator(),
            cpu.x_index(),
            cpu.y_index(),
            nes.memory().stack_pointer(),
            cpu.status().to_mesen_string(),
            scanline,
            nes.memory().ppu_regs().clock().cycle(),
            nes.memory().cpu_cycle(),
        )
    }
}

pub fn interrupts(nes: &Nes) -> String {
    let mut interrupts = String::new();
    interrupts.push(if nes.memory().apu_regs().frame_irq_pending() { 'F' } else {'-'});
    interrupts.push(if nes.memory().apu_regs().dmc_irq_pending() { 'D' } else {'-'});
    interrupts.push(if nes.memory().mapper_params().irq_pending() { 'M' } else {'-'});
    interrupts.push(if nes.cpu().nmi_pending() { 'N' } else {'-'});
    interrupts.push(if nes.memory().oam_dma.dma_pending() { 'O' } else {'-'});
    interrupts.push(if nes.memory().dmc_dma.state() != DmcDmaState::Idle { 'D' } else {'-'});

    interrupts
}
