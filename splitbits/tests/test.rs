extern crate splitbits;

use splitbits::splitbits;

#[test]
fn u8() {
    let fields = splitbits!(0b11011101, "aaabbccc");
    assert_eq!(fields.a, 0b110);
    assert_eq!(fields.b, 0b11);
    assert_eq!(fields.c, 0b101);
}

// Decimal constants should work.
#[test]
fn decimal() {
    let fields = splitbits!(221, "aaabbccc");
    assert_eq!(fields.a, 0b110);
    assert_eq!(fields.b, 0b11);
    assert_eq!(fields.c, 0b101);
}

// Passing in a variable is the most common use case for the macro.
#[test]
fn variable() {
    let value: u8 = 221;
    let fields = splitbits!(value, "aaabbccc");
    assert_eq!(fields.a, 0b110);
    assert_eq!(fields.b, 0b11);
    assert_eq!(fields.c, 0b101);
}

// The caller shouldn't have to specify the type of the value passed in.
#[test]
fn variable_type_inferred() {
    let value = 221;
    let fields = splitbits!(value, "aaabbccc");
    assert_eq!(fields.a, 0b110);
    assert_eq!(fields.b, 0b11);
    assert_eq!(fields.c, 0b101);
}

// Single bit fields should result in bools, not u8s.
#[test]
fn bools() {
    let fields = splitbits!(0b11010101, "abbbcdee");
    assert_eq!(fields.a, true);
    assert_eq!(fields.b, 0b101);
    assert_eq!(fields.c, false);
    assert_eq!(fields.d, true);
    assert_eq!(fields.e, 0b01);
}

// Periods hold their place, but the bits they correspond to are ignored.
#[test]
fn periods() {
    let fields = splitbits!(0b11011101, ".aa.bb..");
    assert_eq!(fields.a, 0b10);
    assert_eq!(fields.b, 0b11);
}

// Spaces are stripped out before processing, whatever place they are in.
#[test]
fn underscores() {
    let fields = splitbits!(0b110_11101, " a aa   b bccc  ");
    assert_eq!(fields.a, 0b110);
    assert_eq!(fields.b, 0b11);
    assert_eq!(fields.c, 0b101);
}

#[test]
fn some_of_everything() {
    let fields = splitbits!(0b1101_1101, ".ab. cc.d");
    assert_eq!(fields.a, true);
    assert_eq!(fields.b, false);
    assert_eq!(fields.c, 0b11);
    assert_eq!(fields.d, true);
}

// Using the same template twice in the same scope should work (i.e. no struct name conflicts)
#[test]
fn duplicate() {
    let fields = splitbits!(0b11011101, "aaabbccc");
    assert_eq!(fields.a, 0b110);
    assert_eq!(fields.b, 0b11);
    assert_eq!(fields.c, 0b101);

    let fields2 = splitbits!(0b01001100, "aaabbccc");
    assert_eq!(fields2.a, 0b010);
    assert_eq!(fields2.b, 0b01);
    assert_eq!(fields2.c, 0b100);
}

// LARGE FIELD TESTS

#[test]
fn u16() {
    let fields = splitbits!(
        0b1101_1101_1111_0001,
         "aaaa aaaa addd efff",
    );
    assert_eq!(fields.a, 0b1_1011_1011);

    assert_eq!(fields.d, 0b111);
    assert_eq!(fields.e, false);
    assert_eq!(fields.f, 0b001);
}

#[test]
fn u32() {
    let fields = splitbits!(
        0b1101_1101_1000_0100_0000_0000_1111_1001,
         "aaaa bbbb bbbb bbbb bbbb bbbi jjjj klll",
    );

    assert_eq!(fields.a, 0b1101);
    assert_eq!(fields.b, 0b110_1100_0010_0000_0000);
    assert_eq!(fields.i, false);
    assert_eq!(fields.j, 0b1111);
    assert_eq!(fields.k, true);
    assert_eq!(fields.l, 0b001);
}

#[test]
fn u64() {
    let fields = splitbits!(
        0b1101_1101_1000_0000_0000_0000_1111_0001_1101_1101_1000_0000_0000_0000_1101_0001,
         "aaaa bbbb bbbb bbbb bbbb bbbi jjjk klll mmmm nnoo pppq qrrr ssst tuuu uuvw xxxx",
    );
    assert_eq!(fields.a, 0b1101);
    assert_eq!(fields.b, 0b110_1100_0000_0000_0000);
    assert_eq!(fields.i, false);
    assert_eq!(fields.j, 0b111);
    assert_eq!(fields.k, 0b10);
    assert_eq!(fields.l, 0b001);
    assert_eq!(fields.m, 0b1101);
    assert_eq!(fields.n, 0b11);
    assert_eq!(fields.o, 0b01);
    assert_eq!(fields.p, 0b100);
    assert_eq!(fields.q, 0b00);
    assert_eq!(fields.r, 0b000);
    assert_eq!(fields.s, 0b000);
    assert_eq!(fields.t, 0b00);
    assert_eq!(fields.u, 0b00011);
    assert_eq!(fields.v, false);
    assert_eq!(fields.w, true);
    assert_eq!(fields.x, 0b0001);
}

#[test]
fn u128() {
    let fields = splitbits!(
        0b1101_1101_1000_0100_0001_0000_1111_0001_1101_1101_1000_0000_0100_0110_1101_0001_1101_1101_1000_0000_1001_0000_1111_0001_1101_1101_1000_0000_0000_0000_1101_0001,
         "aaaa bbcc cdde efff ffff ff.. .... gg.. ...h hiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iii. pppq qrrr ssst tuuu vvvw wxxx",
    );
    assert_eq!(fields.a, 0b1101);
    assert_eq!(fields.b, 0b11);
    assert_eq!(fields.c, 0b011);
    assert_eq!(fields.d, 0b00);
    assert_eq!(fields.e, 0b00);
    assert_eq!(fields.f, 0b1_0000_0100);
    assert_eq!(fields.g, 0b00);
    assert_eq!(fields.h, 0b11);
    assert_eq!(fields.i, 0b10_1100_0000_0010_0011_0110_1000_1110_1110_1100_0000_0100_1000_0111_1000_1110_1110);
    assert_eq!(fields.p, 0b100);
    assert_eq!(fields.q, 0b00);
    assert_eq!(fields.r, 0b000);
    assert_eq!(fields.s, 0b000);
    assert_eq!(fields.t, 0b00);
    assert_eq!(fields.u, 0b000);
    assert_eq!(fields.v, 0b110);
    assert_eq!(fields.w, 0b10);
    assert_eq!(fields.x, 0b0001);
}
