use arr_macro::arr;
use lazy_static::lazy_static;

use crate::ppu::clock::Clock;

lazy_static! {
    pub static ref NTSC_FRAME_ACTIONS: FrameActions = ntsc_frame_actions();
}

fn ntsc_frame_actions() -> FrameActions {
    let mut ntsc_frame = FrameActions::new();

    let empty_scanline = ScanlineActions::new();

    let start_vblank = ScanlineActions::new();

    // VISIBLE SCANLINES
    for scanline in 0..=239 {
        ntsc_frame.set_scanline_actions_at(scanline, visible_scanline_actions());
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

fn visible_scanline_actions() -> ScanlineActions {
    use CycleAction::*;

    let mut line = ScanlineActions::new();
    // Fetch the remaining 31 used background tiles for the current scanline.
    // Cycles 1 through 249.
    for tile in 0..31 {
        let cycle = 8 * tile + 1;
        line.add(cycle + 0, vec![                                               SetBackgroundPixel, PrepareNextBackgroundPixel]);
        line.add(cycle + 1, vec![GetPatternIndex          ,                     SetBackgroundPixel, PrepareNextBackgroundPixel]);
        line.add(cycle + 2, vec![                                               SetBackgroundPixel, PrepareNextBackgroundPixel]);
        line.add(cycle + 3, vec![GetPaletteIndex          ,                     SetBackgroundPixel, PrepareNextBackgroundPixel]);
        line.add(cycle + 4, vec![                                               SetBackgroundPixel, PrepareNextBackgroundPixel]);
        line.add(cycle + 5, vec![GetBackgroundTileLowByte ,                     SetBackgroundPixel, PrepareNextBackgroundPixel]);
        line.add(cycle + 6, vec![                                               SetBackgroundPixel, PrepareNextBackgroundPixel]);
        line.add(cycle + 7, vec![GetBackgroundTileHighByte, GotoNextTileColumn, SetBackgroundPixel, PrepareNextBackgroundPixel]);
        line.add(cycle + 8, vec![PrepareForNextTile                                                                           ]);
    }

    // Fetch a final unused background tile and get ready for the next ROW of tiles.
    line.add(          249, vec![                                               SetBackgroundPixel, PrepareNextBackgroundPixel]);
    line.add(          250, vec![GetPatternIndex          ,                     SetBackgroundPixel, PrepareNextBackgroundPixel]);
    line.add(          251, vec![                                               SetBackgroundPixel, PrepareNextBackgroundPixel]);
    line.add(          252, vec![GetPaletteIndex          ,                     SetBackgroundPixel, PrepareNextBackgroundPixel]);
    line.add(          253, vec![                                               SetBackgroundPixel, PrepareNextBackgroundPixel]);
    line.add(          254, vec![GetBackgroundTileLowByte ,                     SetBackgroundPixel, PrepareNextBackgroundPixel]);
    line.add(          255, vec![                                               SetBackgroundPixel, PrepareNextBackgroundPixel]);
    line.add(          256, vec![GetBackgroundTileHighByte, GotoNextPixelRow  , SetBackgroundPixel, PrepareNextBackgroundPixel]);
    line.add(          257, vec![PrepareForNextTile       , ResetTileColumn                                                   ]);

    // Cycles 258 through 320: No background rendering occurs, only sprite rendering.

    // Fetch the first background tile for the next scanline.
    line.add(          321, vec![                                                                   PrepareNextBackgroundPixel]);
    line.add(          322, vec![GetPatternIndex          ,                                         PrepareNextBackgroundPixel]);
    line.add(          323, vec![                                                                   PrepareNextBackgroundPixel]);
    line.add(          324, vec![GetPaletteIndex          ,                                         PrepareNextBackgroundPixel]);
    line.add(          325, vec![                                                                   PrepareNextBackgroundPixel]);
    line.add(          326, vec![GetBackgroundTileLowByte ,                                         PrepareNextBackgroundPixel]);
    line.add(          327, vec![                                                                   PrepareNextBackgroundPixel]);
    line.add(          328, vec![GetBackgroundTileHighByte, GotoNextTileColumn,                     PrepareNextBackgroundPixel]);
    line.add(          329, vec![PrepareForNextTile       ,                                         PrepareNextBackgroundPixel]);
    // Fetch the second background tile for the next scanline.
    line.add(          330, vec![GetPatternIndex          ,                                         PrepareNextBackgroundPixel]);
    line.add(          331, vec![                                                                   PrepareNextBackgroundPixel]);
    line.add(          332, vec![GetPaletteIndex          ,                                         PrepareNextBackgroundPixel]);
    line.add(          333, vec![                                                                   PrepareNextBackgroundPixel]);
    line.add(          334, vec![GetBackgroundTileLowByte ,                                         PrepareNextBackgroundPixel]);
    line.add(          335, vec![                                                                   PrepareNextBackgroundPixel]);
    line.add(          336, vec![GetBackgroundTileHighByte, GotoNextTileColumn,                     PrepareNextBackgroundPixel]);
    line.add(          337, vec![PrepareForNextTile                                                                           ]);

    // Unused name table fetches.
    line.add(          338, vec![GetPatternIndex                                                                              ]);
    line.add(          340, vec![GetPatternIndex                                                                              ]);

    // SPRITE CALCULATIONS

    // Sprite evaluation (including clearing secondary OAM), transfering OAM to secondary OAM.
    // Cycles 1 through 256.
    for read_write in 0..128 {
        let cycle = 2 * read_write + 1;
        line.add(cycle + 0, vec![ReadOamByte]);
        line.add(cycle + 1, vec![WriteSecondaryOamByte]);
    }

    // Transfer secondary OAM to OAM registers.
    for sprite in 0..8 {
        let cycle = 8 * sprite + 257;
        line.add(cycle + 0, vec![ReadSpriteY]);
        line.add(cycle + 1, vec![ReadSpritePatternIndex]);
        line.add(cycle + 2, vec![ReadSpriteAttributes]);
        line.add(cycle + 3, vec![ReadSpriteX]);
        line.add(cycle + 4, vec![DummyReadSpriteX]);
        line.add(cycle + 5, vec![DummyReadSpriteX]);
        line.add(cycle + 6, vec![DummyReadSpriteX]);
        line.add(cycle + 7, vec![DummyReadSpriteX]);
    }

    for cycle in 321..=340 {
        // TODO: Verify that this is reading the first byte of secondary OAM.
        line.add(cycle, vec![ReadSpriteY]);
    }

    line
}

fn pre_render_scanline_actions() -> ScanlineActions {
    use CycleAction::*;

    let mut scanline = ScanlineActions::new();
    // Fetch the remaining 31 used background tiles for the current scanline.
    // Cycles 1 through 249.
    for tile in 0..31 {
        let cycle = 8 * tile + 1;
        scanline.add(cycle + 1, vec![GetPatternIndex                                                          ]);
        scanline.add(cycle + 3, vec![GetPaletteIndex                                                          ]);
        scanline.add(cycle + 5, vec![GetBackgroundTileLowByte                                                 ]);
        scanline.add(cycle + 7, vec![GetBackgroundTileHighByte, GotoNextTileColumn                            ]);
        scanline.add(cycle + 8, vec![PrepareForNextTile                                                       ]);
    }

    // Fetch a final unused background tile and get ready for the next ROW of tiles.
    scanline.add(          250, vec![GetPatternIndex                                                          ]);
    scanline.add(          252, vec![GetPaletteIndex                                                          ]);
    scanline.add(          254, vec![GetBackgroundTileLowByte                                                 ]);
    scanline.add(          256, vec![GetBackgroundTileHighByte, GotoNextPixelRow                              ]);
    scanline.add(          257, vec![PrepareForNextTile       , ResetTileColumn                               ]);

    // Cycles 258 through 320: No background rendering occurs, only sprite rendering.

    // Fetch the first background tile for the next scanline.
    scanline.add(          321, vec![                                               PrepareNextBackgroundPixel]);
    scanline.add(          322, vec![GetPatternIndex          ,                     PrepareNextBackgroundPixel]);
    scanline.add(          323, vec![                                               PrepareNextBackgroundPixel]);
    scanline.add(          324, vec![GetPaletteIndex          ,                     PrepareNextBackgroundPixel]);
    scanline.add(          325, vec![                                               PrepareNextBackgroundPixel]);
    scanline.add(          326, vec![GetBackgroundTileLowByte ,                     PrepareNextBackgroundPixel]);
    scanline.add(          327, vec![                                               PrepareNextBackgroundPixel]);
    scanline.add(          328, vec![GetBackgroundTileHighByte, GotoNextTileColumn, PrepareNextBackgroundPixel]);
    scanline.add(          329, vec![PrepareForNextTile       ,                     PrepareNextBackgroundPixel]);
    // Fetch the second background tile for the next scanline.
    scanline.add(          330, vec![GetPatternIndex          ,                     PrepareNextBackgroundPixel]);
    scanline.add(          331, vec![                                               PrepareNextBackgroundPixel]);
    scanline.add(          332, vec![GetPaletteIndex          ,                     PrepareNextBackgroundPixel]);
    scanline.add(          333, vec![                                               PrepareNextBackgroundPixel]);
    scanline.add(          334, vec![GetBackgroundTileLowByte ,                     PrepareNextBackgroundPixel]);
    scanline.add(          335, vec![                                               PrepareNextBackgroundPixel]);
    scanline.add(          336, vec![GetBackgroundTileHighByte, GotoNextTileColumn, PrepareNextBackgroundPixel]);
    scanline.add(          337, vec![PrepareForNextTile                                                       ]);

    // Unused name table fetches.
    scanline.add(          338, vec![GetPatternIndex                                                          ]);
    scanline.add(          340, vec![GetPatternIndex                                                          ]);

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

    fn add(&mut self, cycle: usize, mut actions: Vec<CycleAction>) {
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

    ReadOamByte,
    WriteSecondaryOamByte,

    ReadSpriteY,
    ReadSpritePatternIndex,
    ReadSpriteAttributes,
    ReadSpriteX,
    DummyReadSpriteX,
}
