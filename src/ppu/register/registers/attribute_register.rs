use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::pixel_index::ColumnInTile;
use crate::ppu::register::registers::shift_array::ShiftArray;

pub struct AttributeRegister {
    pending_index: PaletteTableIndex,
    next_index: PaletteTableIndex,
    current_indexes: ShiftArray<PaletteTableIndex, 8>,
}

impl AttributeRegister {
    pub fn new() -> AttributeRegister {
        AttributeRegister {
            pending_index: PaletteTableIndex::Zero,
            next_index: PaletteTableIndex::Zero,
            current_indexes: ShiftArray::new(),
        }
    }

    pub fn set_pending_palette_table_index(&mut self, index: PaletteTableIndex) {
        self.pending_index = index;
    }

    pub fn prepare_next_palette_table_index(&mut self) {
        self.next_index = self.pending_index;
    }

    pub fn push_next_palette_table_index(&mut self) {
        self.current_indexes.push(self.next_index);
    }

    pub fn palette_table_index(&self, column_in_tile: ColumnInTile) -> PaletteTableIndex {
        self.current_indexes[column_in_tile]
    }
}
