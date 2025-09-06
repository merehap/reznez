use egui::Context;
use pixels::Pixels;

use crate::gui::debug_screens::pattern_table::{PatternTable, Tile};
use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::mapper::PatternTableSide;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::render::frame::DebugBuffer;
use crate::ppu::tile_number::TileNumber;

const TOP_MENU_BAR_HEIGHT: usize = 24;

pub struct PatternTableRenderer {
    tile: Tile,
    buffer:
        DebugBuffer<{ PatternTableRenderer::WIDTH }, { PatternTableRenderer::HEIGHT }>,
}

impl PatternTableRenderer {
    const WIDTH: usize = 2 * (8 + 1) * 16 + 10;
    const HEIGHT: usize = (8 + 1) * 16 + TOP_MENU_BAR_HEIGHT / 3;

    pub fn new() -> PatternTableRenderer {
        PatternTableRenderer {
            tile: Tile::new(),
            buffer: DebugBuffer::new(Rgb::WHITE),
        }
    }
}

impl WindowRenderer for PatternTableRenderer {
    fn name(&self) -> String {
        "Pattern Table".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let Some(nes) = &world.nes else {
            return;
        };

        let mem = nes.memory();

        let mut offset = 0;
        for side in [PatternTableSide::Left, PatternTableSide::Right] {
            let palette = if mem.ppu_regs.sprite_table_side() == side {
                mem.palette_table().sprite_palette(PaletteTableIndex::Zero)
            } else {
                mem.palette_table()
                    .background_palette(PaletteTableIndex::Zero)
            };
            for index in 0..=255 {
                PatternTable::from_mem(mem, side).render_background_tile(
                    TileNumber::new(index),
                    palette,
                    &mut self.tile,
                );
                self.buffer.place_tile(
                    (8 + 1) * (index as usize % 16) + offset,
                    (8 + 1) * (index as usize / 16) + TOP_MENU_BAR_HEIGHT / 3,
                    &self.tile,
                );
            }

            offset += (8 + 1) * 16 + 10;
        }

        self.buffer.copy_to_rgba_buffer(pixels.frame_mut());
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}