pub struct NameTable<'a>(&'a [u8; 0x400]);

impl <'a> NameTable<'a> {
    pub fn new(raw: &'a [u8; 0x400]) -> NameTable<'a> {
        NameTable(raw)
    }

    pub fn pixel_at(&self, column: u8, row: u8) -> u8 {
        assert!(row < 240, "Row must be less than 240.");
        self.0[32 * row as usize + column as usize]
    }

    pub fn attribute_table(&self) -> AttributeTable<'a> {
        AttributeTable::new((&self.0[0x3C0..]).try_into().unwrap())
    }
}

pub struct AttributeTable<'a>(&'a [u8; 64]);

impl <'a> AttributeTable<'a> {
    pub fn new(raw: &'a [u8; 64]) -> AttributeTable<'a> {
        AttributeTable(raw)
    }
}
