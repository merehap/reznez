use std::collections::BTreeMap;

use itertools::Itertools;
use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident};
use syn::{Expr, Lit};

use crate::character::Character;
use crate::field::Field;
use crate::location::Location;
use crate::name::Name;
use crate::r#type::{Type, Precision};

pub struct Template {
    input_type: Type,
    precision: Precision,
    characters: Vec<Character>,
    locations_by_name: Vec<(Name, Vec<Location>)>,
    has_placeholders: bool,
    literal: Option<u128>,
}

impl Template {
    pub fn from_expr(template: &Expr, base: Base, precision: Precision) -> Template {
        let template: Vec<char> = Template::template_string(template).chars()
            // Spaces are only for human-readability.
            .filter(|&c| c != ' ')
            .collect();

        let characters: Result<Vec<Character>, String> = template.into_iter()
            .map(Character::from_char)
            .collect();
        let characters = characters.unwrap();

        let literal: Option<u128> = Character::characters_to_literal(&characters);

        let has_placeholders = characters.contains(&Character::Placeholder);
        let names: Vec<Name> = characters.clone().into_iter()
            .filter_map(Character::to_name)
            .unique()
            .collect();

        // Each template char needs to be repeated if we aren't working in base 2.
        let characters: Vec<_> = characters.iter()
            .cloned()
            .flat_map(|n| std::iter::repeat(n).take(base.bits_per_digit()))
            .collect();

        assert!(characters.len() <= 128);

        let input_type = Type::for_template(characters.len() as u8);
        let mut locations_by_name: Vec<(Name, Vec<Location>)> = Vec::new();
        for &name in &names {
            let locations: Vec<Location> = characters.iter()
                .rev()
                .enumerate()
                .chunk_by(|(_, &n)| n)
                .into_iter()
                .filter_map(|(c, segment)| {
                    if c == Character::Name(name) {
                        let segment: Vec<_> = segment.collect();
                        let len = segment.len() as u8;
                        let mask_offset = segment[0].0 as u8;
                        Some(Location::new(len, mask_offset))
                    } else {
                        None
                    }
                })
                .collect();
            locations_by_name.push((name, locations));
        }

        Template { input_type, precision, characters, locations_by_name, has_placeholders, literal }
    }

    pub fn template_string(template: &Expr) -> String {
        let Expr::Lit(template) = template.clone() else { panic!() };
        let Lit::Str(template) = template.lit else { panic!() };
        template.value()
    }

    fn characters(&self) -> &[Character] {
        &self.characters
    }

    pub fn has_placeholders(&self) -> bool {
        self.has_placeholders
    }

    pub fn extract_fields(&self, input: &Expr) -> Vec<Field> {
        self.locations_by_name.iter()
            .map(|(name, locations)| Field::new(*name, self.input_type, input, self.precision, &locations))
            .collect()
    }

    // WRONG ASSUMPTIONS:
    // * Each name only has a single segment.
    pub fn substitute_fields(&self, fields: Vec<Field>) -> TokenStream {
        let fields: BTreeMap<Name, Field> = fields.into_iter()
            .map(|field| (field.name(), field))
            .collect();
        let mut field_streams = Vec::new();
        for (name, locations) in &self.locations_by_name {
            assert_eq!(locations.len(), 1);
            let location = locations[0];
            let field = fields[name].clone()
                .shift_left(location.mask_offset())
                .widen(self.input_type);
            assert_eq!(location.len(), field.len());
            field_streams.push(field.to_token_stream());
        }

        let mut literal_quote = quote! {};
        if let Some(literal) = self.literal {
            let t = self.input_type.to_ident();
            literal_quote = quote! { | (#literal as #t) };
        }

        quote! { (#(#field_streams)|*) #literal_quote }
    }

    pub fn to_struct_name(&self) -> Ident {
        let struct_name_suffix: String = self.characters().iter()
            // Underscores work in struct names, periods do not.
            .map(|&c| if c == Character::Placeholder { '_' } else { c.to_char() })
            .collect();
        format_ident!("{}", format!("Fields·{}", struct_name_suffix))
    }
}

#[derive(Clone, Copy)]
pub enum Base {
    Binary,
    Hexadecimal,
}

impl Base {
    pub fn bits_per_digit(self) -> usize {
        match self {
            Base::Binary => 1,
            Base::Hexadecimal => 4,
        }
    }
}
