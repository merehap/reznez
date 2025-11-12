pub struct RawMem {
    raw: Vec<Option<u8>>,
    fill_byte: u8,
}


impl RawMem {
    pub fn new(size: u32) -> Self {
        Self {
            raw: vec![None; size as usize],
            fill_byte: 0,
        }
    }

    pub fn size(&self) -> u32 {
        self.raw.len() as u32
    }

    pub fn set_at(&mut self, index: u32, value: u8) {
        let index = index as usize;
        assert!(self.raw[index].is_none(), "Memory at 0x{index:X} already set (to 0x{value:02X}).");
        self.raw[index] = Some(value);
    }

    pub fn resolve(&self) -> Vec<u8> {
        self.raw.iter()
            .map(|value| value.unwrap_or(self.fill_byte))
            .collect()
    }
}