use arr_macro::arr;
use lazy_static::lazy_static;

use crate::ppu::clock::Clock;

lazy_static! {
    pub static ref NTSC_FRAME_ACTIONS: FrameActions = ntsc_frame_actions();
}

fn ntsc_frame_actions() -> FrameActions {
    use CycleAction::*;

    let mut ntsc_frame = FrameActions::new();

    let empty_scanline = ScanlineActions::new();

    let mut visible_scanline = ScanlineActions::new();
    // Fetch the remaining 31 used background tiles for the current scanline.
    // Cycles 1 through 249.
    for tile in 0..31 {
        let cycle = 8 * tile + 1;
        visible_scanline.add_cycle_actions_at(cycle + 0, vec![                           SetBackgroundPixel, PrepareNextBackgroundPixel]);
        visible_scanline.add_cycle_actions_at(cycle + 1, vec![GetPatternIndex          , SetBackgroundPixel, PrepareNextBackgroundPixel]);
        visible_scanline.add_cycle_actions_at(cycle + 2, vec![                           SetBackgroundPixel, PrepareNextBackgroundPixel]);
        visible_scanline.add_cycle_actions_at(cycle + 3, vec![GetPaletteIndex          , SetBackgroundPixel, PrepareNextBackgroundPixel]);
        visible_scanline.add_cycle_actions_at(cycle + 4, vec![                           SetBackgroundPixel, PrepareNextBackgroundPixel]);
        visible_scanline.add_cycle_actions_at(cycle + 5, vec![GetBackgroundTileLowByte , SetBackgroundPixel, PrepareNextBackgroundPixel]);
        visible_scanline.add_cycle_actions_at(cycle + 6, vec![                           SetBackgroundPixel, PrepareNextBackgroundPixel]);
        visible_scanline.add_cycle_actions_at(cycle + 7, vec![GetBackgroundTileHighByte, GotoNextTileColumn, SetBackgroundPixel, PrepareNextBackgroundPixel]);
        visible_scanline.add_cycle_actions_at(cycle + 8, vec![PrepareForNextTile]);
    }

    visible_scanline.add_cycle_actions_at(249, vec![                           SetBackgroundPixel, PrepareNextBackgroundPixel]);

    // Fetch a final unused background tile and get ready for the next ROW of tiles.
    visible_scanline.add_cycle_actions_at(250, vec![GetPatternIndex          , SetBackgroundPixel, PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(251, vec![                           SetBackgroundPixel, PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(252, vec![GetPaletteIndex          , SetBackgroundPixel, PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(253, vec![                           SetBackgroundPixel, PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(254, vec![GetBackgroundTileLowByte , SetBackgroundPixel, PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(255, vec![                           SetBackgroundPixel, PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(256, vec![GetBackgroundTileHighByte, GotoNextPixelRow  , SetBackgroundPixel, PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(257, vec![ResetTileColumn          , PrepareForNextTile]);

    // Cycles 258 through 320: No background rendering occurs, only sprite rendering.

    visible_scanline.add_cycle_actions_at(321, vec![                           PrepareNextBackgroundPixel]);
    // Fetch the first background tile for the next scanline.
    visible_scanline.add_cycle_actions_at(322, vec![GetPatternIndex          , PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(323, vec![                           PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(324, vec![GetPaletteIndex          , PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(325, vec![                           PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(326, vec![GetBackgroundTileLowByte , PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(327, vec![                           PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(328, vec![GetBackgroundTileHighByte, GotoNextTileColumn, PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(329, vec![PrepareForNextTile       , PrepareNextBackgroundPixel]);
    // Fetch the second background tile for the next scanline.
    visible_scanline.add_cycle_actions_at(330, vec![GetPatternIndex          , PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(331, vec![                           PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(332, vec![GetPaletteIndex          , PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(333, vec![                           PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(334, vec![GetBackgroundTileLowByte , PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(335, vec![                           PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(336, vec![GetBackgroundTileHighByte, GotoNextTileColumn, PrepareNextBackgroundPixel]);
    visible_scanline.add_cycle_actions_at(337, vec![PrepareForNextTile]);

    // Unused name table fetches.
    visible_scanline.add_cycle_actions_at(338, vec![GetPatternIndex]);
    visible_scanline.add_cycle_actions_at(340, vec![GetPatternIndex]);

    // SPRITE CALCULATIONS

    // Clear secondary OAM.
    // Cycles 1 through 64.
    for read_clear in 0..32 {
        let cycle = 2 * read_clear + 1;
        visible_scanline.add_cycle_actions_at(cycle + 0, vec![DummyReadOamByte]);
        visible_scanline.add_cycle_actions_at(cycle + 1, vec![ClearSecondaryOamByte]);
    }

    // Sprite evaluation, transfering OAM to secondary OAM.
    // Cycles 65 through 256.
    for read_write in 0..96 {
        let cycle = 2 * read_write + 65;
        visible_scanline.add_cycle_actions_at(cycle + 0, vec![ReadOamByte]);
        visible_scanline.add_cycle_actions_at(cycle + 1, vec![WriteSecondaryOamByte]);
    }

    // Transfer secondary OAM to OAM registers.
    for sprite in 0..8 {
        let cycle = 8 * sprite + 257;
        visible_scanline.add_cycle_actions_at(cycle + 0, vec![ReadSpriteY]);
        visible_scanline.add_cycle_actions_at(cycle + 1, vec![ReadSpritePatternIndex]);
        visible_scanline.add_cycle_actions_at(cycle + 2, vec![ReadSpriteAttributes]);
        visible_scanline.add_cycle_actions_at(cycle + 3, vec![ReadSpriteX]);
        visible_scanline.add_cycle_actions_at(cycle + 4, vec![DummyReadSpriteX]);
        visible_scanline.add_cycle_actions_at(cycle + 5, vec![DummyReadSpriteX]);
        visible_scanline.add_cycle_actions_at(cycle + 6, vec![DummyReadSpriteX]);
        visible_scanline.add_cycle_actions_at(cycle + 7, vec![DummyReadSpriteX]);
    }

    for cycle in 321..=340 {
        // TODO: Verify that this is reading the first byte of secondary OAM.
        visible_scanline.add_cycle_actions_at(cycle, vec![ReadSpriteY]);
    }

    let start_vblank = ScanlineActions::new();
    let _pre_render = ScanlineActions::new();

    // VISIBLE SCANLINES
    for scanline in 0..=239 {
        ntsc_frame.set_scanline_actions_at(scanline, visible_scanline.clone());
    }

    // POST-RENDER SCANLINES
    ntsc_frame.set_scanline_actions_at(240, empty_scanline.clone());
    ntsc_frame.set_scanline_actions_at(241, start_vblank);
    for scanline in 242..=260 {
        ntsc_frame.set_scanline_actions_at(scanline, empty_scanline.clone());
    }

    // PRE-RENDER SCANLINE
    ntsc_frame.set_scanline_actions_at(261, pre_render_scanline_actions());

    ntsc_frame
}

fn pre_render_scanline_actions() -> ScanlineActions {
    use CycleAction::*;

    let mut scanline = ScanlineActions::new();
    // Fetch the remaining 31 used background tiles for the current scanline.
    // Cycles 1 through 249.
    for tile in 0..31 {
        let cycle = 8 * tile + 1;
        scanline.add_cycle_actions_at(cycle + 1, vec![GetPatternIndex]);
        scanline.add_cycle_actions_at(cycle + 3, vec![GetPaletteIndex]);
        scanline.add_cycle_actions_at(cycle + 5, vec![GetBackgroundTileLowByte]);
        scanline.add_cycle_actions_at(cycle + 7, vec![GetBackgroundTileHighByte, GotoNextTileColumn]);
        scanline.add_cycle_actions_at(cycle + 8, vec![PrepareForNextTile]);
    }

    // Fetch a final unused background tile and get ready for the next ROW of tiles.
    scanline.add_cycle_actions_at(250, vec![GetPatternIndex]);
    scanline.add_cycle_actions_at(252, vec![GetPaletteIndex]);
    scanline.add_cycle_actions_at(254, vec![GetBackgroundTileLowByte]);
    scanline.add_cycle_actions_at(256, vec![GetBackgroundTileHighByte, GotoNextPixelRow]);
    scanline.add_cycle_actions_at(257, vec![ResetTileColumn, PrepareForNextTile]);

    // Cycles 258 through 320: No background rendering occurs, only sprite rendering.

    scanline.add_cycle_actions_at(321, vec![                          PrepareNextBackgroundPixel]);
    // Fetch the first background tile for the next scanline.
    scanline.add_cycle_actions_at(322, vec![GetPatternIndex         , PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(323, vec![                          PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(324, vec![GetPaletteIndex         , PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(325, vec![                          PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(326, vec![GetBackgroundTileLowByte, PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(327, vec![                          PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(328, vec![GetBackgroundTileHighByte, GotoNextTileColumn, PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(329, vec![PrepareForNextTile      , PrepareNextBackgroundPixel]);
    // Fetch the second background tile for the next scanline.
    scanline.add_cycle_actions_at(330, vec![GetPatternIndex         , PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(331, vec![                          PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(332, vec![GetPaletteIndex         , PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(333, vec![                          PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(334, vec![GetBackgroundTileLowByte, PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(335, vec![                          PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(336, vec![GetBackgroundTileHighByte, GotoNextTileColumn, PrepareNextBackgroundPixel]);
    scanline.add_cycle_actions_at(337, vec![PrepareForNextTile]);

    // Unused name table fetches.
    scanline.add_cycle_actions_at(338, vec![GetPatternIndex]);
    scanline.add_cycle_actions_at(340, vec![GetPatternIndex]);

    scanline
}

#[derive(Clone)]
pub struct FrameActions {
    all_scanline_actions: Box<[ScanlineActions; 262]>,
}

impl FrameActions {
    pub fn current_cycle_actions(&self, clock: &Clock) -> &[CycleAction] {
        &self.all_scanline_actions[clock.scanline() as usize].all_cycle_actions[clock.cycle() as usize]
    }

    fn new() -> FrameActions {
        FrameActions {
            all_scanline_actions: Box::new(arr![ScanlineActions::new(); 262]),
        }
    }

    fn set_scanline_actions_at(&mut self, scanline: usize, scanline_actions: ScanlineActions) {
        self.all_scanline_actions[scanline] = scanline_actions;
    }
}

#[derive(Clone)]
pub struct ScanlineActions {
    all_cycle_actions: Box<[Vec<CycleAction>; 341]>,
}

impl ScanlineActions {
    fn new() -> ScanlineActions {
        ScanlineActions {
            all_cycle_actions: Box::new(arr![Vec::new(); 341]),
        }
    }

    fn add_cycle_actions_at(&mut self, cycle: usize, mut actions: Vec<CycleAction>) {
        self.all_cycle_actions[cycle].append(&mut actions);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    GetPatternIndex,
    GetPaletteIndex,
    GetBackgroundTileLowByte,
    GetBackgroundTileHighByte,

    GotoNextTileColumn,
    GotoNextPixelRow,
    PrepareForNextTile,
    ResetTileColumn,

    SetBackgroundPixel,
    PrepareNextBackgroundPixel,

    DummyReadOamByte,
    ClearSecondaryOamByte,
    ReadOamByte,
    WriteSecondaryOamByte,

    ReadSpriteY,
    ReadSpritePatternIndex,
    ReadSpriteAttributes,
    ReadSpriteX,
    DummyReadSpriteX,
}