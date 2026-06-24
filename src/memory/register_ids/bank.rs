use num_derive::FromPrimitive;

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive)]
pub enum PrgBankRegisterId {
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl const RegisterId for PrgBankRegisterId {
    fn from_char(c: char) -> Option<Self> {
        use PrgBankRegisterId::*;
        Some(match c {
            'p' => P,
            'q' => Q,
            'r' => R,
            's' => S,
            't' => T,
            'u' => U,
            'v' => V,
            'w' => W,
            'x' => X,
            'y' => Y,
            'z' => Z,
            ..'p' => return None,
            'z'.. => return None,
        })
    }

    fn to_char(self) -> char {
        use PrgBankRegisterId::*;
        match self {
            P => 'p',
            Q => 'q',
            R => 'r',
            S => 's',
            T => 't',
            U => 'u',
            V => 'v',
            W => 'w',
            X => 'x',
            Y => 'y',
            Z => 'z',
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive)]
pub enum ChrBankRegisterId {
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,

    NT0,
    NT1,
    NT2,
    NT3,
}

impl ChrBankRegisterId {
    pub const ALL_NAME_TABLE_IDS: [Self; 4] = [Self::NT0, Self::NT1, Self::NT2, Self::NT3];

    pub fn to_raw_chr_id(self) -> u8 {
        self as u8
    }
}

impl const RegisterId for ChrBankRegisterId {
    fn from_char(c: char) -> Option<Self> {
        use ChrBankRegisterId::*;
        Some(match c {
            'c' => C,
            'd' => D,
            'e' => E,
            'f' => F,
            'g' => G,
            'h' => H,
            'i' => I,
            'j' => J,
            'k' => K,
            'l' => L,
            'm' => M,
            'n' => N,

            // Gotta give them something...
            'α' => NT0,
            'β' => NT1,
            'γ' => NT2,
            'δ' => NT3,

            _ => return None,
        })
    }

    fn to_char(self) -> char {
        use ChrBankRegisterId::*;
        match self {
            C => 'c',
            D => 'd',
            E => 'e',
            F => 'f',
            G => 'g',
            H => 'h',
            I => 'i',
            J => 'j',
            K => 'k',
            L => 'l',
            M => 'm',
            N => 'n',

            // Gotta give them something...
            NT0 => 'α',
            NT1 => 'β',
            NT2 => 'γ',
            NT3 => 'δ',
        }
    }
}

pub const trait RegisterId: Sized + PartialEq + Eq + Copy {
    fn from_char(c: char) -> Option<Self>;
    fn to_char(self) -> char;
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MetaRegisterId {
    MR0,
    MR1,
    MR2,
    MR3,
}
