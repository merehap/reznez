pub fn op_codes() -> [(OpCode, AccessMode, u8, ExtraCycle); 256] {
    use OpCode::*;
    use AccessMode::*;
    use ExtraCycle::*;

    /* 0x00           0x01            0x02            0x03            0x04            0x05            0x06            0x07        */
    [ (Brk,Imp,7,No), (Ora,IzX,6,No), (Jam,Imp,0,No), (Slo,IzX,8,No), (Nop, ZP,3,No), (Ora, ZP,3,No), (Asl, ZP,5,No), (Slo, ZP,5,No)
    /* 0x08           0x09            0x0A            0x0B            0x0C            0x0D            0x0E            0x0F        */
    , (Php,Imp,3,No), (Ora,Imm,2,No), (Asl,Imp,2,No), (Anc,Imm,2,No), (Nop,Abs,4,No), (Ora,Abs,4,No), (Asl,Abs,6,No), (Slo,Abs,6,No)
    /* 0x10           0x11            0x12            0x13            0x14            0x15            0x16            0x17        */
    , (Bpl,Rel,2,PB), (Ora,IzY,5,PB), (Jam,Imp,0,No), (Slo,IzY,8,No), (Nop,ZPX,4,No), (Ora,ZPX,4,No), (Asl,ZPX,6,No), (Slo,ZPX,6,No)
    /* 0x18           0x19            0x1A            0x1B            0x1C            0x1D            0x1E            0x1F        */
    , (Clc,Imp,2,No), (Ora,AbY,4,PB), (Nop,Imp,2,No), (Slo,AbY,7,No), (Nop,AbX,4,PB), (Ora,AbX,4,PB), (Asl,AbX,7,No), (Slo,AbX,7,No)
    /* 0x20           0x21            0x22            0x23            0x24            0x25            0x26            0x27        */
    , (Jsr,Abs,6,No), (And,IzX,6,No), (Jam,Imp,0,No), (Rla,IzX,8,No), (Bit, ZP,3,No), (And, ZP,3,No), (Rol, ZP,5,No), (Rla, ZP,5,No)
    /* 0x28           0x29            0x2A            0x2B            0x2C            0x2D            0x2E            0x2F        */
    , (Plp,Imp,4,No), (And,Imm,2,No), (Rol,Imp,2,No), (Anc,Imm,2,No), (Bit,Abs,4,No), (And,Abs,4,No), (Rol,Abs,6,No), (Rla,Abs,6,No)
    /* 0x30           0x31            0x32            0x33            0x34            0x35            0x36            0x37        */
    , (Bmi,Rel,2,PB), (And,IzY,5,PB), (Jam,Imp,0,No), (Rla,IzY,8,No), (Nop,ZPX,4,No), (And,ZPX,4,No), (Rol,ZPX,6,No), (Rla,ZPX,6,No)
    /* 0x38           0x39            0x3A            0x3B            0x3C            0x3D            0x3E            0x3F        */
    , (Sec,Imp,2,No), (And,AbY,4,PB), (Nop,Imp,2,No), (Rla,AbY,7,No), (Nop,AbX,4,PB), (And,AbX,4,PB), (Rol,AbX,7,No), (Rla,AbX,7,No)
    /* 0x40           0x41            0x42            0x43            0x44            0x45            0x46            0x47        */
    , (Rti,Imp,6,No), (Eor,IzX,6,No), (Jam,Imp,0,No), (Sre,IzX,8,No), (Nop, ZP,3,No), (Eor, ZP,3,No), (Lsr, ZP,5,No), (Sre, ZP,5,No)
    /* 0x48           0x49            0x4A            0x4B            0x4C            0x4D            0x4E            0x4F        */
    , (Pha,Imp,3,No), (Eor,Imm,2,No), (Lsr,Imp,2,No), (Alr,Imm,2,No), (Jmp,Abs,3,No), (Eor,Abs,4,No), (Lsr,Abs,6,No), (Sre,Abs,6,No)
    /* 0x50           0x51            0x52            0x53            0x54            0x55            0x56            0x57        */
    , (Bvc,Rel,2,PB), (Eor,IzY,5,PB), (Jam,Imp,0,No), (Sre,IzY,8,No), (Nop,ZPX,4,No), (Eor,ZPX,4,No), (Lsr,ZPX,6,No), (Sre,ZPX,6,No)
    /* 0x58           0x59            0x5A            0x5B            0x5C            0x5D            0x5E            0x5F        */
    , (Cli,Imp,2,No), (Eor,AbY,4,PB), (Nop,Imp,2,No), (Sre,AbY,7,No), (Nop,AbX,4,PB), (Eor,AbX,4,PB), (Lsr,AbX,7,No), (Sre,AbX,7,No)
    /* 0x60           0x61            0x62            0x63            0x64            0x65            0x66            0x67        */
    , (Rts,Imp,6,No), (Adc,IzX,6,No), (Jam,Imp,0,No), (Rra,IzX,8,No), (Nop, ZP,3,No), (Adc, ZP,3,No), (Ror, ZP,5,No), (Rra, ZP,5,No)
    /* 0x68           0x69            0x6A            0x6B            0x6C            0x6D            0x6E            0x6F        */
    , (Pla,Imp,4,No), (Adc,Imm,2,No), (Ror,Imp,2,No), (Arr,Imm,2,No), (Jmp,Ind,5,No), (Adc,Abs,4,No), (Ror,Abs,6,No), (Rra,Abs,6,No)
    /* 0x70           0x71            0x72            0x73            0x74            0x75            0x76            0x77        */
    , (Bvs,Rel,2,PB), (Adc,IzY,5,PB), (Jam,Imp,0,No), (Rra,IzY,8,No), (Nop,ZPX,4,No), (Adc,ZPX,4,No), (Ror,ZPX,6,No), (Rra,ZPX,6,No)
    /* 0x78           0x79            0x7A            0x7B            0x7C            0x7D            0x7E            0x7F        */
    , (Sei,Imp,2,No), (Adc,AbY,4,PB), (Nop,Imp,2,No), (Rra,AbY,7,No), (Nop,AbX,4,PB), (Adc,AbX,4,PB), (Ror,AbX,7,No), (Rra,AbX,7,No)
    /* 0x80           0x81            0x82            0x83            0x84            0x85            0x86            0x87        */
    , (Nop,Imm,2,No), (Sta,IzX,6,No), (Nop,Imm,2,No), (Sax,IzX,6,No), (Sty, ZP,3,No), (Sta, ZP,3,No), (Stx, ZP,3,No), (Sax, ZP,3,No)
    /* 0x88           0x89            0x8A            0x8B            0x8C            0x8D            0x8E            0x8F        */
    , (Dey,Imp,2,No), (Nop,Imm,2,No), (Txa,Imp,2,No), (Xaa,Imm,2,No), (Sty,Abs,4,No), (Sta,Abs,4,No), (Stx,Abs,4,No), (Sax,Abs,4,No)
    /* 0x90           0x91            0x92            0x93            0x94            0x95            0x96            0x97        */
    , (Bcc,Rel,2,PB), (Sta,IzY,6,No), (Jam,Imp,0,No), (Ahx,IzY,6,No), (Sty,ZPX,4,No), (Sta,ZPX,4,No), (Stx,ZPY,4,No), (Sax,ZPY,4,No)
    /* 0x98           0x99            0x9A            0x9B            0x9C            0x9D            0x9E            0x9F        */
    , (Tya,Imp,2,No), (Sta,AbY,5,No), (Txs,Imp,2,No), (Tas,AbY,5,No), (Shy,AbX,5,No), (Sta,AbX,5,No), (Shx,AbY,5,No), (Ahx,AbY,5,No)
    /* 0xA0           0xA1            0xA2            0xA3            0xA4            0xA5            0xA6            0xA7        */
    , (Ldy,Imm,2,No), (Lda,IzX,6,No), (Ldx,Imm,2,No), (Lax,IzX,6,No), (Ldy, ZP,3,No), (Lda, ZP,3,No), (Ldx, ZP,3,No), (Lax, ZP,3,No)
    /* 0xA8           0xA9            0xAA            0xAB            0xAC            0xAD            0xAE            0xAF        */
    , (Tay,Imp,2,No), (Lda,Imm,2,No), (Tax,Imp,2,No), (Lax,Imm,2,No), (Ldy,Abs,4,No), (Lda,Abs,4,No), (Ldx,Abs,4,No), (Lax,Abs,4,No)
    /* 0xB0           0xB1            0xB2            0xB3            0xB4            0xB5            0xB6            0xB7        */
    , (Bcs,Rel,2,PB), (Lda,IzY,5,PB), (Jam,Imp,0,No), (Lax,IzY,5,PB), (Ldy,ZPX,4,No), (Lda,ZPX,4,No), (Ldx,ZPY,4,No), (Lax,ZPY,4,No)
    /* 0xB8           0xB9            0xBA            0xBB            0xBC            0xBD            0xBE            0xBF        */
    , (Clv,Imp,2,No), (Lda,AbY,4,PB), (Tsx,Imp,2,No), (Las,AbY,4,PB), (Ldy,AbY,4,PB), (Lda,AbX,4,PB), (Ldx,AbY,4,PB), (Lax,AbY,4,PB)
    /* 0xC0           0xC1            0xC2            0xC3            0xC4            0xC5            0xC6            0xC7        */
    , (Cpy,Imm,2,No), (Cmp,IzX,6,No), (Nop,Imm,2,No), (Dcp,IzX,8,No), (Cpy, ZP,3,No), (Cmp, ZP,3,No), (Dec, ZP,5,No), (Dcp, ZP,5,No)
    /* 0xC8           0xC9            0xCA            0xCB            0xCC            0xCD            0xCE            0xCF        */
    , (Iny,Imp,2,No), (Cmp,Imm,2,No), (Dex,Imp,2,No), (Axs,Imm,2,No), (Cpy,Abs,4,No), (Cmp,Abs,4,No), (Dec,Abs,6,No), (Dcp,Abs,6,No)
    /* 0xD0           0xD1            0xD2            0xD3            0xD4            0xD5            0xD6            0xD7        */
    , (Bne,Rel,2,PB), (Cmp,IzY,5,PB), (Jam,Imp,0,No), (Dcp,IzY,8,No), (Nop,ZPX,4,No), (Cmp,ZPX,4,No), (Dec,ZPX,4,No), (Dcp,ZPX,6,No)
    /* 0xD8           0xD9            0xDA            0xDB            0xDC            0xDD            0xDE            0xDF        */
    , (Cld,Imp,2,No), (Cmp,AbY,4,PB), (Nop,Imp,2,No), (Dcp,AbY,7,No), (Nop,AbX,4,PB), (Cmp,AbX,4,PB), (Dec,AbX,7,No), (Dcp,AbX,7,No)
    /* 0xE0           0xE1            0xE2            0xE3            0xE4            0xE5            0xE6            0xE7        */
    , (Cpx,Imm,2,No), (Sbc,IzX,6,No), (Nop,Imm,2,No), (Isc,IzX,8,No), (Cpx, ZP,3,No), (Sbc, ZP,3,No), (Inc, ZP,5,No), (Isc, ZP,5,No)
    /* 0xE8           0xE9            0xEA            0xEB            0xEC            0xED            0xEE            0xEF        */
    , (Inx,Imp,2,No), (Sbc,Imm,2,No), (Nop,Imp,2,No), (Sbc,Imm,2,No), (Cpx,Abs,4,No), (Sbc,Abs,4,No), (Inc,Abs,6,No), (Isc,Abs,6,No)
    /* 0xF0           0xF1            0xF2            0xF3            0xF4            0xF5            0xF6            0xF7        */
    , (Beq,Rel,2,PB), (Sbc,IzY,5,PB), (Jam,Imp,2,No), (Isc,IzY,8,No), (Nop,ZPX,4,No), (Sbc,ZPX,4,No), (Inc,ZPX,6,No), (Isc,ZPX,6,No)
    /* 0xF8           0xF9            0xFA            0xFB            0xFC            0xFD            0xFE            0xFF        */
    , (Sed,Imp,2,No), (Sbc,AbY,4,PB), (Nop,Imp,2,No), (Isc,AbY,7,No), (Nop,AbX,4,PB), (Sbc,AbX,4,PB), (Inc,AbX,7,No), (Isc,AbX,7,No)
    ]
}

pub enum OpCode {
    // Logical/Arithmetic
    Ora,
    And,
    Eor,
    Adc,
    Sbc,
    Cmp,
    Cpx,
    Cpy,
    Dec,
    Dex,
    Dey,
    Inc,
    Inx,
    Iny,
    Asl,
    Rol,
    Lsr,
    Ror,

    // Move
    Lda,
    Sta,
    Ldx,
    Stx,
    Ldy,
    Sty,
    Tax,
    Txa,
    Tay,
    Tya,
    Tsx,
    Txs,
    Pla,
    Pha,
    Plp,
    Php,

    // Jump/Flag
    Bpl,
    Bmi,
    Bvc,
    Bvs,
    Bcc,
    Bcs,
    Bne,
    Beq,
    Brk,
    Rti,
    Jsr,
    Rts,
    Jmp,
    Bit,
    Clc,
    Sec,
    Cld,
    Sed,
    Cli,
    Sei,
    Clv,
    Nop,

    // Illegal
    Slo,
    Rla,
    Sre,
    Rra,
    Sax,
    Lax,
    Dcp,
    Isc,
    Anc,
    Alr,
    Arr,
    Xaa,
    Axs,
    Ahx,
    Shy,
    Shx,
    Tas,
    Las,
    Jam,
}

pub enum AccessMode {
    Imp,
    Acc,
    Imm,
    ZP,
    ZPX,
    ZPY,
    Abs,
    AbX,
    AbY,
    Rel,
    Ind,
    IzX,
    IzY,
}

pub enum ExtraCycle {
    No,
    PB,
}
