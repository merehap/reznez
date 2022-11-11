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
        visible_scanline.add_cycle_action_at(cycle + 1, GetPatternIndex);
        visible_scanline.add_cycle_action_at(cycle + 3, GetPaletteIndex);
        visible_scanline.add_cycle_action_at(cycle + 5, GetBackgroundTileLowByte);
        visible_scanline.add_cycle_action_at(cycle + 7, GetBackgroundTileHighByte);
        visible_scanline.add_cycle_action_at(cycle + 7, GotoNextTileColumn);
        visible_scanline.add_cycle_action_at(cycle + 8, PrepareForNextTile);
    }

    // Fetch a final unused background tile and get ready for the next ROW of tiles.
    visible_scanline.add_cycle_action_at(250, GetPatternIndex);
    visible_scanline.add_cycle_action_at(252, GetPaletteIndex);
    visible_scanline.add_cycle_action_at(254, GetBackgroundTileLowByte);
    visible_scanline.add_cycle_action_at(256, GetBackgroundTileHighByte);
    visible_scanline.add_cycle_action_at(256, GotoNextPixelRow);
    visible_scanline.add_cycle_action_at(257, ResetTileColumn);
    visible_scanline.add_cycle_action_at(257, PrepareForNextTile);

    // Cycles 258 through 321: No background rendering occurs, only sprite rendering.

    // Fetch the first background tile for the next scanline.
    visible_scanline.add_cycle_action_at(322, GetPatternIndex);
    visible_scanline.add_cycle_action_at(324, GetPaletteIndex);
    visible_scanline.add_cycle_action_at(326, GetBackgroundTileLowByte);
    visible_scanline.add_cycle_action_at(328, GetBackgroundTileHighByte);
    visible_scanline.add_cycle_action_at(328, GotoNextTileColumn);
    visible_scanline.add_cycle_action_at(329, PrepareForNextTile);
    // Fetch the second background tile for the next scanline.
    visible_scanline.add_cycle_action_at(330, GetPatternIndex);
    visible_scanline.add_cycle_action_at(332, GetPaletteIndex);
    visible_scanline.add_cycle_action_at(334, GetBackgroundTileLowByte);
    visible_scanline.add_cycle_action_at(336, GetBackgroundTileHighByte);
    visible_scanline.add_cycle_action_at(336, GotoNextTileColumn);
    visible_scanline.add_cycle_action_at(337, PrepareForNextTile);

    // Unused name table fetches.
    visible_scanline.add_cycle_action_at(338, GetPatternIndex);
    visible_scanline.add_cycle_action_at(340, GetPatternIndex);

    // SPRITE CALCULATIONS

    // Clear secondary OAM.
    // Cycles 1 through 64.
    for read_clear in 0..32 {
        let cycle = 2 * read_clear + 1;
        visible_scanline.add_cycle_action_at(cycle + 0, DummyReadOamByte);
        visible_scanline.add_cycle_action_at(cycle + 1, ClearSecondaryOamByte);
    }

    // Sprite evaluation, transfering OAM to secondary OAM.
    // Cycles 65 through 256.
    for read_write in 0..96 {
        let cycle = 2 * read_write + 65;
        visible_scanline.add_cycle_action_at(cycle + 0, ReadOamByte);
        visible_scanline.add_cycle_action_at(cycle + 1, WriteSecondaryOamByte);
    }

    // Transfer secondary OAM to OAM registers.
    for sprite in 0..8 {
        let cycle = 8 * sprite + 257;
        visible_scanline.add_cycle_action_at(cycle + 0, ReadSpriteY);
        visible_scanline.add_cycle_action_at(cycle + 1, ReadSpritePatternIndex);
        visible_scanline.add_cycle_action_at(cycle + 2, ReadSpriteAttributes);
        visible_scanline.add_cycle_action_at(cycle + 3, ReadSpriteX);
        visible_scanline.add_cycle_action_at(cycle + 4, DummyReadSpriteX);
        visible_scanline.add_cycle_action_at(cycle + 5, DummyReadSpriteX);
        visible_scanline.add_cycle_action_at(cycle + 6, DummyReadSpriteX);
        visible_scanline.add_cycle_action_at(cycle + 7, DummyReadSpriteX);
    }

    for cycle in 321..=340 {
        // TODO: Verify that this is reading the first byte of secondary OAM.
        visible_scanline.add_cycle_action_at(cycle, ReadSpriteY);
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
    ntsc_frame.set_scanline_actions_at(261, visible_scanline);

    ntsc_frame
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

    fn add_cycle_action_at(&mut self, cycle: usize, action: CycleAction) {
        self.all_cycle_actions[cycle].push(action);
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
