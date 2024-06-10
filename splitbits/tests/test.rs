extern crate splitbits;

use splitbits::splitbits;

#[test]
fn basic() {
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
