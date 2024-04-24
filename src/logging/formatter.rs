use crate::cpu::instruction::{Instruction, INSTRUCTIONS, OpCode, AccessMode};
use crate::memory::mapper::CpuAddress;
use crate::nes::Nes;

pub trait Formatter {
    fn format_instruction(&self, nes: &Nes, interrupt_text: String) -> String;
}

pub struct Nintendulator0980Formatter;

impl Formatter for Nintendulator0980Formatter {
    fn format_instruction(&self, nes: &Nes, _interrupt_text: String) -> String {
        let cpu_cycle = nes.memory().cpu_cycle();
        let peek = |address| nes.memory().cpu_peek(address).unwrap_or(0);

        let cpu = nes.cpu();

        let (op_code, start_address) = nes.cpu().next_op_code_and_address().unwrap();
        let instruction: Instruction = INSTRUCTIONS[usize::from(op_code)];
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
            nes.ppu().clock().cycle(),
            nes.ppu().clock().scanline(),
            cpu_cycle,
        )
    }
}

pub struct MesenFormatter;

impl Formatter for MesenFormatter {
    fn format_instruction(&self, nes: &Nes, interrupt_text: String) -> String {
        let peek = |address| nes.memory().cpu_peek(address).unwrap_or(0);

        let cpu = nes.cpu();

        let (op_code, start_address) = nes.cpu().next_op_code_and_address().unwrap();
        let instruction: Instruction = INSTRUCTIONS[usize::from(op_code)];
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

        let mut scanline = nes.ppu().clock().scanline() as i16;
        if scanline == 261 {
            scanline = -1;
        }

        format!(
            "{:04X}  {:?} {:28}{} A:{:02X} X:{:02X} Y:{:02X} S:{:02X} P:{} V:{:<3} H:{:<3} Fr:{} Cycle:{}",
            start_address.to_raw(),
            instruction.op_code(),
            argument_string,
            interrupt_text,
            cpu.accumulator(),
            cpu.x_index(),
            cpu.y_index(),
            nes.memory().stack_pointer(),
            cpu.status().to_mesen_string(),
            scanline,
            nes.ppu().clock().cycle(),
            nes.ppu().clock().frame(),
            nes.memory().cpu_cycle(),
        )
    }
}

pub fn interrupts(nes: &Nes) -> String {
    let mut interrupts = String::new();
    interrupts.push(if nes.memory().apu_regs().frame_irq_pending() { 'F' } else {'-'});
    interrupts.push(if nes.memory().apu_regs().dmc_irq_pending() { 'D' } else {'-'});
    interrupts.push(if nes.memory().mapper().irq_pending() { 'M' } else {'-'});
    interrupts.push(if nes.cpu().nmi_pending() { 'N' } else {'-'});
    interrupts.push(if nes.cpu().oam_dma_pending() { 'O' } else {'-'});
    interrupts.push(if nes.memory().apu_regs().dmc.dma_pending() { 'D' } else {'-'});

    interrupts
}
