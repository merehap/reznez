use crate::ppu::sprite::Sprite;

const ATTRIBUTE_BYTE_INDEX: u8 = 2;

pub struct Oam([u8; 256]);

impl Oam {
    pub fn new() -> Oam {
        Oam([0; 256])
    }

    pub fn sprites(&self) -> [Sprite; 64] {
        let mut iter = self.0.array_chunks::<4>();
        [(); 64].map(|_| {
            let raw = u32::from_be_bytes(*iter.next().unwrap());
            Sprite::from_u32(raw)
        })
    }

    pub fn sprite_0(&self) -> Sprite {
        Sprite::from_u32(u32::from_be_bytes(self.0[0..4].try_into().unwrap()))
    }

    pub fn read(&self, index: u8) -> u8 {
        self.0[index as usize]
    }

    pub fn write(&mut self, index: u8, value: u8) {
        // The three unimplemented attribute bits should never be set.
        let value =
            if index % 4 == ATTRIBUTE_BYTE_INDEX {
                value & 0b1110_0011
            } else {
                value
            };
        self.0[index as usize] = value;
    }
}
