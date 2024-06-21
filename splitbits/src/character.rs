use crate::name::Name;

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
