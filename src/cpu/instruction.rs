use lazy_static::lazy_static;
use strum_macros::EnumString;

use crate::cpu::step::*;

lazy_static! {
    pub static ref INSTRUCTIONS: [Instruction; 256] = instructions();
}

#[rustfmt::skip]
fn instructions() -> [Instruction; 256] {
    use OpCode::*;
    use AccessMode::*;

    let jam = (JAM, Imp);
    let codes: [[(OpCode, AccessMode); 8]; 32] = [
        /*00*/      /*20*/     /*40*/     /*60*/     /*80*/     /*A0*/     /*C0*/     /*E0*/
/*+00*/ [(BRK,Imp), (JSR,Abs), (RTI,Imp), (RTS,Imp), (NOP,Imm), (LDY,Imm), (CPY,Imm), (CPX,Imm)],
/*+01*/ [(ORA,IzX), (AND,IzX), (EOR,IzX), (ADC,IzX), (STA,IzX), (LDA,IzX), (CMP,IzX), (SBC,IzX)],
/*+02*/ [jam      , jam      , jam      , jam      , (NOP,Imm), (LDX,Imm), (NOP,Imm), (NOP,Imm)],
/*+03*/ [(SLO,IzX), (RLA,IzX), (SRE,IzX), (RRA,IzX), (SAX,IzX), (LAX,IzX), (DCP,IzX), (ISC,IzX)],
/*+04*/ [(NOP,ZP ), (BIT,ZP ), (NOP,ZP ), (NOP,ZP ), (STY,ZP ), (LDY,ZP ), (CPY,ZP ), (CPX,ZP )],
/*+05*/ [(ORA,ZP ), (AND,ZP ), (EOR,ZP ), (ADC,ZP ), (STA,ZP ), (LDA,ZP ), (CMP,ZP ), (SBC,ZP )],
/*+06*/ [(ASL,ZP ), (ROL,ZP ), (LSR,ZP ), (ROR,ZP ), (STX,ZP ), (LDX,ZP ), (DEC,ZP ), (INC,ZP )],
/*+07*/ [(SLO,ZP ), (RLA,ZP ), (SRE,ZP ), (RRA,ZP ), (SAX,ZP ), (LAX,ZP ), (DCP,ZP ), (ISC,ZP )],
/*+08*/ [(PHP,Imp), (PLP,Imp), (PHA,Imp), (PLA,Imp), (DEY,Imp), (TAY,Imp), (INY,Imp), (INX,Imp)],
/*+09*/ [(ORA,Imm), (AND,Imm), (EOR,Imm), (ADC,Imm), (NOP,Imm), (LDA,Imm), (CMP,Imm), (SBC,Imm)],
/*+0A*/ [(ASL,Imp), (ROL,Imp), (LSR,Imp), (ROR,Imp), (TXA,Imp), (TAX,Imp), (DEX,Imp), (NOP,Imp)],
/*+0B*/ [(ANC,Imm), (ANC,Imm), (ALR,Imm), (ARR,Imm), (XAA,Imm), (LAX,Imm), (AXS,Imm), (SBC,Imm)],
/*+0C*/ [(NOP,Abs), (BIT,Abs), (JMP,Abs), (JMP,Ind), (STY,Abs), (LDY,Abs), (CPY,Abs), (CPX,Abs)],
/*+0D*/ [(ORA,Abs), (AND,Abs), (EOR,Abs), (ADC,Abs), (STA,Abs), (LDA,Abs), (CMP,Abs), (SBC,Abs)],
/*+0E*/ [(ASL,Abs), (ROL,Abs), (LSR,Abs), (ROR,Abs), (STX,Abs), (LDX,Abs), (DEC,Abs), (INC,Abs)],
/*+0F*/ [(SLO,Abs), (RLA,Abs), (SRE,Abs), (RRA,Abs), (SAX,Abs), (LAX,Abs), (DCP,Abs), (ISC,Abs)],

/*+10*/ [(BPL,Rel), (BMI,Rel), (BVC,Rel), (BVS,Rel), (BCC,Rel), (BCS,Rel), (BNE,Rel), (BEQ,Rel)],
/*+11*/ [(ORA,IzY), (AND,IzY), (EOR,IzY), (ADC,IzY), (STA,IzY), (LDA,IzY), (CMP,IzY), (SBC,IzY)],
/*+12*/ [jam      , jam      , jam      , jam      , jam      , jam      , jam      , jam      ],
/*+13*/ [(SLO,IzY), (RLA,IzY), (SRE,IzY), (RRA,IzY), (AHX,IzY), (LAX,IzY), (DCP,IzY), (ISC,IzY)],
/*+14*/ [(NOP,ZPX), (NOP,ZPX), (NOP,ZPX), (NOP,ZPX), (STY,ZPX), (LDY,ZPX), (NOP,ZPX), (NOP,ZPX)],
/*+15*/ [(ORA,ZPX), (AND,ZPX), (EOR,ZPX), (ADC,ZPX), (STA,ZPX), (LDA,ZPX), (CMP,ZPX), (SBC,ZPX)],
/*+16*/ [(ASL,ZPX), (ROL,ZPX), (LSR,ZPX), (ROR,ZPX), (STX,ZPY), (LDX,ZPY), (DEC,ZPX), (INC,ZPX)],
/*+17*/ [(SLO,ZPX), (RLA,ZPX), (SRE,ZPX), (RRA,ZPX), (SAX,ZPY), (LAX,ZPY), (DCP,ZPX), (ISC,ZPX)],
/*+18*/ [(CLC,Imp), (SEC,Imp), (CLI,Imp), (SEI,Imp), (TYA,Imp), (CLV,Imp), (CLD,Imp), (SED,Imp)],
/*+19*/ [(ORA,AbY), (AND,AbY), (EOR,AbY), (ADC,AbY), (STA,AbY), (LDA,AbY), (CMP,AbY), (SBC,AbY)],
/*+1A*/ [(NOP,Imp), (NOP,Imp), (NOP,Imp), (NOP,Imp), (TXS,Imp), (TSX,Imp), (NOP,Imp), (NOP,Imp)],
/*+1B*/ [(SLO,AbY), (RLA,AbY), (SRE,AbY), (RRA,AbY), (TAS,AbY), (LAS,AbY), (DCP,AbY), (ISC,AbY)],
/*+1C*/ [(NOP,AbX), (NOP,AbX), (NOP,AbX), (NOP,AbX), (SHY,AbX), (LDY,AbX), (NOP,AbX), (NOP,AbX)],
/*+1D*/ [(ORA,AbX), (AND,AbX), (EOR,AbX), (ADC,AbX), (STA,AbX), (LDA,AbX), (CMP,AbX), (SBC,AbX)],
/*+1E*/ [(ASL,AbX), (ROL,AbX), (LSR,AbX), (ROR,AbX), (SHX,AbY), (LDX,AbY), (DEC,AbX), (INC,AbX)],
/*+1F*/ [(SLO,AbX), (RLA,AbX), (SRE,AbX), (RRA,AbX), (AHX,AbY), (LAX,AbY), (DCP,AbX), (ISC,AbX)],
    ];

    let mut result = [Instruction::from_tuple(0x2, JAM, Imp); 256];
    for (index, template) in result.iter_mut().enumerate() {
        let i = index % 0x20;
        let j = index / 0x20;
        let (op_code, access_mode) = codes[i][j];
        *template = Instruction::from_tuple(index as u8, op_code, access_mode);
    }

    result
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub code_point: u8,
    pub op_code: OpCode,
    pub access_mode: AccessMode,
    steps: &'static [Step],
}

impl Instruction {
    pub fn steps(&self) -> &'static [Step] {
        self.steps
    }

    pub fn from_code_point(code_point: u8) -> Instruction {
        INSTRUCTIONS[code_point as usize]
    }

    fn from_tuple(code_point: u8, op_code: OpCode, access_mode: AccessMode) -> Instruction {
        use AccessMode::*;
        use OpCode::*;
        let steps = match (access_mode, op_code) {
            (Imp, BRK) => BRK_STEPS,
            (Imp, RTI) => RTI_STEPS,
            (Imp, RTS) => RTS_STEPS,
            (Imp, PHA) => PHA_STEPS,
            (Imp, PHP) => PHP_STEPS,
            (Imp, PLA) => PLA_STEPS,
            (Imp, PLP) => PLP_STEPS,
            (Abs, JSR) => JSR_STEPS,
            (Abs, JMP) => JMP_ABS_STEPS,
            (Ind, JMP) => JMP_IND_STEPS,

            (Imp,   _) => IMPLICIT_ADDRESSING_STEPS,
            (Imm,   _) => IMMEDIATE_ADDRESSING_STEPS,
            (Rel,   _) => RELATIVE_ADDRESSING_STEPS,

            // Read operations.
            (Abs, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP | CPX | CPY | BIT | LAX | NOP) => ABSOLUTE_READ_STEPS,
            // TODO: Remove the unused combos.
            (AbX, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP |                   LAX | NOP) => ABSOLUTE_X_READ_STEPS,
            (AbY, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP |                   LAX | NOP | LAS | TAS | AHX) => ABSOLUTE_Y_READ_STEPS,
            (ZP , LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP | CPX | CPY | BIT | LAX | NOP) => ZERO_PAGE_READ_STEPS,
            (ZPX, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP |                   LAX | NOP) => ZERO_PAGE_X_READ_STEPS,
            (ZPY, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP |                   LAX | NOP) => ZERO_PAGE_Y_READ_STEPS,
            (IzX, LDA |             EOR | AND | ORA | ADC | SBC | CMP |                   LAX) => INDEXED_INDIRECT_READ_STEPS,
            (IzY, LDA |             EOR | AND | ORA | ADC | SBC | CMP |                   LAX | AHX) => INDIRECT_INDEXED_READ_STEPS,

            // Write operations.
            (Abs, STA | STX | STY | SAX) => ABSOLUTE_WRITE_STEPS,
            // TODO: Remove the unused combos.
            (AbX, STA | STX | STY |     SHY) => ABSOLUTE_X_WRITE_STEPS,
            (AbY, STA | STX | STY |     SHX) => ABSOLUTE_Y_WRITE_STEPS,
            (ZP , STA | STX | STY | SAX) => ZERO_PAGE_WRITE_STEPS,
            (ZPX, STA | STX | STY | SAX) => ZERO_PAGE_X_WRITE_STEPS,
            (ZPY, STA | STX | STY | SAX) => ZERO_PAGE_Y_WRITE_STEPS,
            (IzX, STA |             SAX) => INDEXED_INDIRECT_WRITE_STEPS,
            (IzY, STA) => INDIRECT_INDEXED_WRITE_STEPS,

            // Read-modify-write operations.
            (Abs, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP) => ABSOLUTE_READ_MODIFY_WRITE_STEPS,
            // TODO: Remove the unused combos.
            (AbX, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP) => ABSOLUTE_X_READ_MODIFY_WRITE_STEPS,
            (AbY, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP) => ABSOLUTE_Y_READ_MODIFY_WRITE_STEPS,
            (ZP , ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP) => ZERO_PAGE_READ_MODIFY_WRITE_STEPS,
            (ZPX, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP) => ZERO_PAGE_X_READ_MODIFY_WRITE_STEPS,
            (ZPY, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP) => ZERO_PAGE_Y_READ_MODIFY_WRITE_STEPS,
            (IzX,                                     SLO | SRE | RLA | RRA | ISC | DCP) => INDEXED_INDIRECT_READ_MODIFY_WRITE_STEPS,
            (IzY,                                     SLO | SRE | RLA | RRA | ISC | DCP) => INDIRECT_INDEXED_READ_MODIFY_WRITE_STEPS,

            template => unreachable!("{:X?}", template),
        };

        Instruction {
            code_point,
            op_code,
            access_mode,
            steps,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, EnumString)]
pub enum OpCode {
    // Logical/Arithmetic
    ORA,
    AND,
    EOR,
    ADC,
    SBC,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    INC,
    INX,
    INY,
    ASL,
    ROL,
    LSR,
    ROR,

    /* Move */
    LDA,
    STA,
    LDX,
    STX,
    LDY,
    STY,
    TAX,
    TXA,
    TAY,
    TYA,
    TSX,
    TXS,
    // Pull accumulator from stack.
    PLA,
    // Push accumulator to stack.
    PHA,
    // Pull status from stack.
    PLP,
    // Push status from stack.
    PHP,

    /* Jump/Flag */
    BPL,
    BMI,
    BVC,
    BVS,
    BCC,
    BCS,
    BNE,
    BEQ,
    BRK,
    RTI,
    JSR,
    RTS,
    JMP,
    BIT,
    CLC,
    SEC,
    CLD,
    SED,
    CLI,
    SEI,
    CLV,
    NOP,

    // Undocumented.
    SLO,
    RLA,
    SRE,
    RRA,
    SAX,
    LAX,
    DCP,
    ISC,
    // a.k.a. AAC
    ANC,
    ALR,
    ARR,
    XAA,
    AXS,
    // a.k.a. SHA
    AHX,
    // a.k.a. SYA
    SHY,
    // a.k.a. SXA
    SHX,
    // a.k.a. ANE
    TAS,
    LAS,

    JAM,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AccessMode {
    Imp,
    Imm,
    ZP,
    ZPX,
    ZPY,
    // Absolute addressing.
    Abs,
    AbX,
    AbY,
    Rel,
    Ind,
    IzX,
    IzY,
}

impl AccessMode {
    pub fn instruction_length(self) -> u8 {
        use AccessMode::*;
        match self {
            Imp => 1,
            Imm | ZP | ZPX | ZPY | Rel | IzX | IzY => 2,
            Abs | AbX | AbY | Ind => 3,
        }
    }
}
