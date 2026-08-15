use crate::memory::ppu::chr_memory::PeekSource;
use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::pixel_index::ColumnInTile;
use crate::ppu::register::registers::shift_array::ShiftArray;
use crate::util::bit_util::unpack_bools;

pub struct PatternRegister {
    pending_low_byte: u8,
    pending_high_byte: u8,
    pending_low_byte_source: PeekSource,
    pending_high_byte_source: PeekSource,
    current_source: PeekSource,
    current_indexes: ShiftArray<Option<PaletteIndex>, 16>,
}

impl PatternRegister {
    pub fn new() -> PatternRegister {
        PatternRegister {
            pending_low_byte: 0,
            pending_high_byte: 0,
            pending_low_byte_source: PeekSource::Void,
            pending_high_byte_source: PeekSource::Void,
            current_source: PeekSource::Void,
            current_indexes: ShiftArray::new(),
        }
    }

    pub fn set_pending_low_byte(&mut self, low_byte: u8, peek_source: PeekSource) {
        self.pending_low_byte = low_byte;
        self.pending_low_byte_source = peek_source;
    }

    pub fn set_pending_high_byte(&mut self, high_byte: u8, peek_source: PeekSource) {
        self.pending_high_byte = high_byte;
        self.pending_high_byte_source = peek_source;
    }

    pub fn load_next_palette_indexes(&mut self) {
        let low_bits = unpack_bools(self.pending_low_byte);
        let high_bits = unpack_bools(self.pending_high_byte);
        for i in 0..8 {
            self.current_indexes[i + 8] = PaletteIndex::from_low_high(low_bits[i], high_bits[i]);
        }

        self.current_source = self.pending_low_byte_source;
    }

    pub fn shift_left(&mut self) {
        self.current_indexes.push(Some(PaletteIndex::Two));
    }

    pub fn palette_index(&self, column_in_tile: ColumnInTile) -> Option<PaletteIndex> {
        self.current_indexes[column_in_tile]
    }

    pub fn current_source(&self) -> PeekSource {
        self.current_source
    }
}
