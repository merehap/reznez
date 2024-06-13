extern crate proc_macro;

use std::fmt;

use itertools::Itertools;
use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident};
use syn::{Token, Expr, Lit};
use syn::parse::Parser;
use syn::punctuated::Punctuated;

// TODO:
// * Enable emitting precise-sized ux crate types.
// * Implement combinebits.
// * Implement splitbits_then_combine
// * combinebits_hexadecimal
// * Allow const variable templates.
// * Allow non-const variable templates (as a separate macro?).
#[proc_macro]
pub fn splitbits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Binary)
}

#[proc_macro]
pub fn splithex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Hexadecimal)
}

#[proc_macro]
pub fn splitbits_tuple(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_tuple_base(input, Base::Binary)
}

#[proc_macro]
pub fn splithex_tuple(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_tuple_base(input, Base::Hexadecimal)
}

#[proc_macro]
pub fn splitbits_tuple_into(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_tuple_into_base(input, Base::Binary)
}

#[proc_macro]
pub fn splithex_tuple_into(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_tuple_into_base(input, Base::Hexadecimal)
}

#[proc_macro]
pub fn onefield(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    onefield_base(input, Base::Binary)
}

#[proc_macro]
pub fn onehexfield(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    onefield_base(input, Base::Hexadecimal)
}

fn splitbits_base(input: proc_macro::TokenStream, base: Base) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into());
    let (template, input_type) = apply_base(&template, base);
    let fields = fields(template.clone());

    let struct_name_suffix: String = template.iter()
        // Underscores work in struct names, periods do not.
        .map(|&c| if c == '.' { '_' } else { c })
        .collect();
    let struct_ident = format_ident!("{}", format!("Fields·{}", struct_name_suffix));

    let names: Vec<_> = fields.iter().map(|field| field.name()).collect();
    let types: Vec<_> = fields.iter().map(|field| field.t()).collect();
    let values: Vec<TokenStream> = fields.iter().map(|field| field.value(input_type, &value)).collect();
    let result = quote! {
        {
            struct #struct_ident {
                #(#names: #types,)*
            }

            #struct_ident {
                #(#names: #values,)*
            }
        }
    };

    result.into()
}

fn splitbits_tuple_base(input: proc_macro::TokenStream, base: Base) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into());
    let (template, input_type) = apply_base(&template, base);
    let fields = fields(template.clone());
    let values: Vec<TokenStream> = fields.iter().map(|field| field.value(input_type, &value)).collect();

    let result = quote! {
        (#(#values,)*)
    };

    result.into()
}

fn splitbits_tuple_into_base(input: proc_macro::TokenStream, base: Base) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into());
    let (template, input_type) = apply_base(&template, base);
    let fields = fields(template.clone());
    let values: Vec<TokenStream> = fields.iter().map(|field| field.value(input_type, &value)).collect();

    let result = quote! {
        (#((#values).into(),)*)
    };

    result.into()
}

fn onefield_base(input: proc_macro::TokenStream, base: Base) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into());
    let (template, input_type) = apply_base(&template, base);
    let fields = fields(template.clone());
    assert_eq!(fields.len(), 1);
    fields[0].value(input_type, &value).into()
}

fn apply_base(template: &[char], base: Base) -> (Vec<char>, Type) {
    let input_type = Type::from_template(template, base);
    // Each template char needs to be repeated if we aren't working in base 2.
    let bit_template = template.iter()
        .cloned()
        .flat_map(|n| std::iter::repeat(n).take(base.bits_per_digit()))
        .collect();

    (bit_template, input_type)
}

fn parse_input(item: TokenStream) -> (Expr, Vec<char>) {
    let parts: Punctuated::<Expr, Token![,]> = Parser::parse2(
        Punctuated::<Expr, Token![,]>::parse_terminated,
        item.clone().into(),
    ).unwrap();
    let parts: Vec<Expr> = parts.into_iter().collect();
    assert_eq!(parts.len(), 2);

    let value = parts[0].clone();
    let Expr::Lit(template) = parts[1].clone() else { panic!() };
    let Lit::Str(template) = template.lit else { panic!() };
    let template: String = template.value();

    let template: Vec<char> = template.chars()
        // Spaces are only for human-readability.
        .filter(|&c| c != ' ')
        .collect();

    (value, template)
}

fn fields(template: Vec<char>) -> Vec<Field> {
    let mut names = template.clone();
    names.sort();
    names.dedup();
    // Periods aren't names, they are placeholders for ignored bits.
    names.retain(|&n| n != '.');

    names.iter()
        .map(|&name| {
            assert!(name.is_ascii_lowercase());
            Field::new(name, &template)
        })
        .collect()
}

#[derive(Clone)]
struct Field {
    name: char,
    mask: u128,
    t: Type,
}

impl Field {
    fn new(name: char, template: &[char]) -> Field {
        let segments: Vec<_> = template.iter()
            .rev()
            .enumerate()
            .chunk_by(|(_, &n)| n)
            .into_iter()
            .filter_map(|(n, p)| if n == name { Some(p) } else { None })
            .map(|p| {
                let p: Vec<_> = p.collect();
                Segment { length: p.len(), offset: p[0].0 }
            })
            .collect();

        let segment = segments[0];
        let mut mask: u128 = 2u128.pow(segment.length as u32) - 1;
        mask <<= segment.offset;

        let bit_count = u128::BITS - mask.leading_zeros() - mask.trailing_zeros();
        let t = match bit_count {
            0 => panic!(),
            1        => Type::Bool,
            2..=8    => Type::U8,
            9..=16   => Type::U16,
            17..=32  => Type::U32,
            33..=64  => Type::U64,
            65..=128 => Type::U128,
            129..=u32::MAX => panic!("Integers larger than u128 are not supported."),
        };

        Field { name, mask, t }
    }

    fn name(&self) -> Ident {
        format_ident!("{}", self.name)
    }

    fn t(&self) -> Ident {
        format_ident!("{}", format!("{}", self.t))
    }

    fn value(&self, input_type: Type, input: &Expr) -> TokenStream {
        let input_type = format_ident!("{}", input_type.to_string());
        let mask = self.mask;
        let shift = mask.trailing_zeros();
        // There's no need to shift if the shift is 0.
        let shifter = if shift == 0 {
            quote! { }
        } else {
            quote! { >> #shift }
        };

        match self.t {
            Type::Bool => quote! {   #input as #input_type & #mask as #input_type != 0 },
            Type::U8   => quote! { ((#input as #input_type & #mask as #input_type) #shifter) as u8 },
            Type::U16  => quote! { ((#input as #input_type & #mask as #input_type) #shifter) as u16 },
            Type::U32  => quote! { ((#input as #input_type & #mask as #input_type) #shifter) as u32 },
            Type::U64  => quote! { ((#input as #input_type & #mask as #input_type) #shifter) as u64 },
            Type::U128 => quote! { ((#input as #input_type & #mask as #input_type) #shifter) as u128 },
        }
    }
}

#[derive(Clone, Copy)]
struct Segment {
    length: usize,
    offset: usize,
}

#[derive(Clone, Copy, Debug)]
enum Type {
    Bool =   1,
    U8   =   8,
    U16  =  16,
    U32  =  32,
    U64  =  64,
    U128 = 128,
}

impl Type {
    fn from_template(template: &[char], base: Base) -> Type {
        match base.bits_per_digit() * template.len() {
            8 => Type::U8,
            16 => Type::U16,
            32 => Type::U32,
            64 => Type::U64,
            128 => Type::U128,
            len => panic!("Template length must be 8, 16, 32, 64, or 128, but was {len}."),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = format!("{:?}", *self).to_lowercase();
        write!(f, "{}", result)
    }
}

#[derive(Clone, Copy)]
enum Base {
    Binary,
    Hexadecimal,
}

impl Base {
    fn bits_per_digit(self) -> usize {
        match self {
            Base::Binary => 1,
            Base::Hexadecimal => 4,
        }
    }
}
