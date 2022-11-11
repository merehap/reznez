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
        visible_scanline.add_cycle_actions_at(cycle + 1, vec![GetPatternIndex]);
        visible_scanline.add_cycle_actions_at(cycle + 3, vec![GetPaletteIndex]);
        visible_scanline.add_cycle_actions_at(cycle + 5, vec![GetBackgroundTileLowByte]);
        visible_scanline.add_cycle_actions_at(cycle + 7, vec![GetBackgroundTileHighByte]);
        visible_scanline.add_cycle_actions_at(cycle + 7, vec![GotoNextTileColumn]);
        visible_scanline.add_cycle_actions_at(cycle + 8, vec![PrepareForNextTile]);
    }

    // Fetch a final unused background tile and get ready for the next ROW of tiles.
    visible_scanline.add_cycle_actions_at(250, vec![GetPatternIndex]);
    visible_scanline.add_cycle_actions_at(252, vec![GetPaletteIndex]);
    visible_scanline.add_cycle_actions_at(254, vec![GetBackgroundTileLowByte]);
    visible_scanline.add_cycle_actions_at(256, vec![GetBackgroundTileHighByte]);
    visible_scanline.add_cycle_actions_at(256, vec![GotoNextPixelRow]);
    visible_scanline.add_cycle_actions_at(257, vec![ResetTileColumn]);
    visible_scanline.add_cycle_actions_at(257, vec![PrepareForNextTile]);

    // Cycles 258 through 321: No background rendering occurs, only sprite rendering.

    // Fetch the first background tile for the next scanline.
    visible_scanline.add_cycle_actions_at(322, vec![GetPatternIndex]);
    visible_scanline.add_cycle_actions_at(324, vec![GetPaletteIndex]);
    visible_scanline.add_cycle_actions_at(326, vec![GetBackgroundTileLowByte]);
    visible_scanline.add_cycle_actions_at(328, vec![GetBackgroundTileHighByte]);
    visible_scanline.add_cycle_actions_at(328, vec![GotoNextTileColumn]);
    visible_scanline.add_cycle_actions_at(329, vec![PrepareForNextTile]);
    // Fetch the second background tile for the next scanline.
    visible_scanline.add_cycle_actions_at(330, vec![GetPatternIndex]);
    visible_scanline.add_cycle_actions_at(332, vec![GetPaletteIndex]);
    visible_scanline.add_cycle_actions_at(334, vec![GetBackgroundTileLowByte]);
    visible_scanline.add_cycle_actions_at(336, vec![GetBackgroundTileHighByte]);
    visible_scanline.add_cycle_actions_at(336, vec![GotoNextTileColumn]);
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
