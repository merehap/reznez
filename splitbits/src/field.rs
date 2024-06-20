use std::collections::BTreeMap;

use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident};
use syn::Expr;

use crate::segment::{Segment, Location};
use crate::r#type::{Type, Precision};

#[derive(Clone)]
pub struct Field {
    name: Name,
    segments: Vec<Segment>,
    t: Type,
}

impl Field {
    pub fn new(name: Name, input_type: Type, input: &Expr, precision: Precision, locations: &[Location]) -> Field {
        let mut segment_offset = 0;
        let mut segments = Vec::new();
        for location in locations {
            let mut mask: u128 = 2u128.pow(location.len() as u32) - 1;
            mask <<= location.mask_offset();

            let segment = Segment::new(input.clone(), input_type, location.len(), mask)
                .shift_right(location.mask_offset())
                .shift_left(segment_offset);
            segment_offset += location.len();
            segments.push(segment);
        }

        let bit_count = locations.iter().map(|location| location.len()).sum();
        let t = Type::for_field(bit_count, precision);
        Field { name, segments, t }
    }

    pub fn to_token_stream(&self) -> TokenStream {
        let t = self.t.to_ident();
        let mut segments = self.segments.iter().map(Segment::to_value);
        if self.t == Type::BOOL {
            let segment = segments.next().unwrap();
            quote! { (#segment) != 0 }
        } else {
            quote! { #t::try_from(#(#segments)|*).unwrap() }
        }
    }

    pub fn merge(upper: Vec<Field>, lower: Vec<Field>) -> Vec<Field> {
        let lower_map: BTreeMap<_, _> = lower.iter()
            .map(|field| (field.name, field))
            .collect();
        let mut result = Vec::new();
        for u in &upper {
            if let Some(l) = lower_map.get(&u.name) {
                result.push(u.concat(&l));
            } else {
                result.push(u.clone());
            }
        }

        let upper_map: BTreeMap<_, _> = upper.iter()
            .map(|field| (field.name, field))
            .collect();
        for l in lower {
            if !upper_map.contains_key(&l.name) {
                result.push(l);
            }
        }

        result
    }

    pub fn concat(&self, lower: &Field) -> Field {
        assert_eq!(self.name, lower.name);

        let mut new_segments = Vec::new();
        for segment in &self.segments {
            let new_segment = segment.clone().shift_left(lower.len());
            new_segments.push(new_segment);
        }

        for segment in &lower.segments {
            new_segments.push(segment.clone());
        }

        Field {
            name: self.name,
            segments: new_segments,
            t: self.t.concat(lower.t),
        }
    }

    // TODO: Fail on overflow.
    pub fn shift_left(mut self, shift: u8) -> Field {
        for segment in &mut self.segments {
            segment.shift_left(shift);
        }

        self
    }

    pub fn widen(mut self, new_type: Type) -> Field {
        self.t = new_type;
        for segment in &mut self.segments {
            segment.widen(new_type);
        }

        self
    }

    pub fn name(&self) -> Name {
        self.name
    }

    pub fn t(&self) -> Type {
        self.t
    }

    pub fn len(&self) -> u8 {
        self.segments.iter()
            .map(|segment| segment.len())
            .sum()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Character {
    Name(Name),
    Placeholder,
    Zero,
    One,
}

impl Character {
    pub fn from_char(c: char) -> Result<Self, String> {
        Ok(match c {
            '.' => Character::Placeholder,
            '0' => Character::Zero,
            '1' => Character::One,
            _   => Character::Name(Name::new(c)?),
        })
    }

    pub fn is_literal(self) -> bool {
        self == Character::Zero || self == Character::One
    }

    pub fn to_name(self) -> Option<Name> {
        if let Character::Name(name) = self {
            Some(name)
        } else {
            None
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Character::Name(name) => name.to_char(),
            Character::Placeholder => '.',
            Character::Zero => '0',
            Character::One  => '1',
        }
    }

    pub fn characters_to_literal(chars: &[Character]) -> Option<u128> {
        if chars.iter().filter(|c| c.is_literal()).next().is_none() {
            return None;
        }

        let literal_string: String = chars.iter()
            .map(|&c| if c == Character::One { '1' } else { '0' })
            .collect();
        Some(u128::from_str_radix(&literal_string, 2).unwrap())
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Name(char);

impl Name {
    pub fn new(raw_name: char) -> Result<Self, String> {
        if raw_name.is_ascii_lowercase() {
            Ok(Name(raw_name))
        } else {
            Err(format!("'{raw_name}' is not a valid Name."))
        }
    }

    pub fn to_char(self) -> char {
        self.0
    }

    pub fn to_ident(self) -> Ident {
        format_ident!("{}", self.to_char())
    }
}
