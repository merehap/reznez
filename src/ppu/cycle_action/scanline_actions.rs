use arr_macro::arr;
use lazy_static::lazy_static;

use crate::ppu::cycle_action::cycle_action::CycleAction;

lazy_static! {
    pub static ref VISIBLE_SCANLINE_ACTIONS: ScanlineActions = visible_scanline_actions();
    pub static ref EMPTY_SCANLINE_ACTIONS: ScanlineActions = empty_scanline_actions();
    pub static ref START_VBLANK_SCANLINE_ACTIONS: ScanlineActions = start_vblank_scanline_actions();
    pub static ref VBLANK_SCANLINE_ACTIONS: ScanlineActions = vblank_scanline_actions();
    pub static ref PRE_RENDER_SCANLINE_ACTIONS: ScanlineActions = pre_render_scanline_actions();
}

fn visible_scanline_actions() -> ScanlineActions {
    use CycleAction::*;

    let mut line = ScanlineActions::new();
    //           ||CYCLE||       ||---------BACKGROUND-TILE-ACTIONS---------||  ||-SPRITE--ACTIONS-||  ||-----DISPLAY-ACTIONS-----||
    line.add(          001, vec![                                               ResetForOamClear                                    ]);
    line.add(          065, vec![                                               ResetForSpriteEvaluation                            ]);

    // Fetch the remaining 31 usable background tiles for the current scanline.
    // Secondary OAM clearing then sprite evaluation, transfering OAM to secondary OAM.
    // Cycles 1 through 249.
    for tile in 0..31 {
        let cycle = 8 * tile + 1;
        line.add(cycle + 0, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
        line.add(cycle + 1, vec![GetPatternIndex          ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
        line.add(cycle + 2, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
        line.add(cycle + 3, vec![GetPaletteIndex          ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
        line.add(cycle + 4, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
        line.add(cycle + 5, vec![GetBackgroundTileLowByte ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
        line.add(cycle + 6, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
        line.add(cycle + 7, vec![GetBackgroundTileHighByte, GotoNextTileColumn, WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
        line.add(cycle + 8, vec![PrepareForNextTile                                                                                 ]);
    }

    // Fetch a final unused background tile and get ready for the next ROW of tiles.
    // Complete the sprite evaluation.
    line.add(          249, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
    line.add(          250, vec![GetPatternIndex          ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    line.add(          251, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
    line.add(          252, vec![GetPaletteIndex          ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    line.add(          253, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
    line.add(          254, vec![GetBackgroundTileLowByte ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    line.add(          255, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
    line.add(          256, vec![GetBackgroundTileHighByte, GotoNextPixelRow  , WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    line.add(          257, vec![PrepareForNextTile       , ResetTileColumn   , ResetForTransferToOamRegisters                      ]);

    // Transfer secondary OAM to OAM registers.
    // Cycles 257 through 320
    for sprite in 0..8 {
        let cycle = 8 * sprite + 257;
        line.add(cycle + 0, vec![                                               ReadSpriteY                                         ]);
        line.add(cycle + 1, vec![                                               ReadSpritePatternIndex                              ]);
        line.add(cycle + 2, vec![                                               ReadSpriteAttributes                                ]);
        line.add(cycle + 3, vec![                                               ReadSpriteX                                         ]);
        line.add(cycle + 4, vec![                                               DummyReadSpriteX                                    ]);
        line.add(cycle + 5, vec![                                               DummyReadSpriteX                                    ]);
        line.add(cycle + 6, vec![                                               DummyReadSpriteX                                    ]);
        line.add(cycle + 7, vec![                                               DummyReadSpriteX                                    ]);
    }

    // Fetch the first background tile for the next scanline.
    line.add(          321, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          322, vec![GetPatternIndex          ,                     ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          323, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          324, vec![GetPaletteIndex          ,                     ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          325, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          326, vec![GetBackgroundTileLowByte ,                     ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          327, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          328, vec![GetBackgroundTileHighByte, GotoNextTileColumn, ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          329, vec![PrepareForNextTile       ,                     ReadSpriteY                    , PrepareForNextPixel]);
    // Fetch the second background tile for the next scanline.
    line.add(          330, vec![GetPatternIndex          ,                     ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          331, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          332, vec![GetPaletteIndex          ,                     ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          333, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          334, vec![GetBackgroundTileLowByte ,                     ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          335, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          336, vec![GetBackgroundTileHighByte, GotoNextTileColumn, ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          337, vec![PrepareForNextTile       ,                     ReadSpriteY                                         ]);

    // Unused name table fetches.
    line.add(          338, vec![GetPatternIndex          ,                     ReadSpriteY                                         ]);
    line.add(          339, vec![                                               ReadSpriteY                                         ]);
    line.add(          340, vec![GetPatternIndex          ,                     ReadSpriteY                                         ]);

    line
}

// TODO: Does UpdateOamData occur here despite 'vblank == false' ?
fn empty_scanline_actions() -> ScanlineActions {
    ScanlineActions::new()
}

// TODO: Determine if UpdateOamData actually occurs on cycle 0 and 1.
fn start_vblank_scanline_actions() -> ScanlineActions {
    use CycleAction::*;

    let mut scanline = vblank_scanline_actions();
    scanline.add(1, vec![StartVblank]);
    scanline.add(3, vec![RequestNmi]);
    scanline
}

fn vblank_scanline_actions() -> ScanlineActions {
    use CycleAction::*;

    let mut scanline = ScanlineActions::new();
    // Every cycle.
    for cycle in 0..=340 {
        scanline.add(cycle, vec![UpdateOamData]);
    }

    scanline
}

fn pre_render_scanline_actions() -> ScanlineActions {
    use CycleAction::*;

    let mut scanline = ScanlineActions::new();
    // Clear vblank, sprite 0 hit, and sprite overflow.
    scanline.add(            1, vec![ClearFlags                                                        ]);

    //               ||CYCLE||       ||---------BACKGROUND-TILE-ACTIONS---------|| ||-DISPLAY-ACTIONS-||
    // Fetch the remaining 31 used background tiles for the current scanline.
    // Cycles 1 through 249.
    for tile in 0..31 {
        let cycle = 8 * tile + 1;
        scanline.add(cycle + 1, vec![GetPatternIndex                                                   ]);
        scanline.add(cycle + 3, vec![GetPaletteIndex                                                   ]);
        scanline.add(cycle + 5, vec![GetBackgroundTileLowByte                                          ]);
        scanline.add(cycle + 7, vec![GetBackgroundTileHighByte, GotoNextTileColumn                     ]);
        scanline.add(cycle + 8, vec![PrepareForNextTile                                                ]);
    }

    // Fetch a final unused background tile and get ready for the next ROW of tiles.
    scanline.add(          250, vec![GetPatternIndex                                                   ]);
    scanline.add(          252, vec![GetPaletteIndex                                                   ]);
    scanline.add(          254, vec![GetBackgroundTileLowByte                                          ]);
    scanline.add(          256, vec![GetBackgroundTileHighByte, GotoNextPixelRow                       ]);
    scanline.add(          257, vec![PrepareForNextTile       , ResetTileColumn                        ]);

    for cycle in 280..=304 {
        scanline.add(    cycle, vec![SetInitialYScroll                                                 ]);
    }

    scanline.add(          320, vec![SetInitialScrollOffsets                                           ]);

    // Fetch the first background tile for the next scanline.
    scanline.add(          321, vec![                                               PrepareForNextPixel]);
    scanline.add(          322, vec![GetPatternIndex          ,                     PrepareForNextPixel]);
    scanline.add(          323, vec![                                               PrepareForNextPixel]);
    scanline.add(          324, vec![GetPaletteIndex          ,                     PrepareForNextPixel]);
    scanline.add(          325, vec![                                               PrepareForNextPixel]);
    scanline.add(          326, vec![GetBackgroundTileLowByte ,                     PrepareForNextPixel]);
    scanline.add(          327, vec![                                               PrepareForNextPixel]);
    scanline.add(          328, vec![GetBackgroundTileHighByte, GotoNextTileColumn, PrepareForNextPixel]);
    scanline.add(          329, vec![PrepareForNextTile       ,                     PrepareForNextPixel]);
    // Fetch the second background tile for the next scanline.
    scanline.add(          330, vec![GetPatternIndex          ,                     PrepareForNextPixel]);
    scanline.add(          331, vec![                                               PrepareForNextPixel]);
    scanline.add(          332, vec![GetPaletteIndex          ,                     PrepareForNextPixel]);
    scanline.add(          333, vec![                                               PrepareForNextPixel]);
    scanline.add(          334, vec![GetBackgroundTileLowByte ,                     PrepareForNextPixel]);
    scanline.add(          335, vec![                                               PrepareForNextPixel]);
    scanline.add(          336, vec![GetBackgroundTileHighByte, GotoNextTileColumn, PrepareForNextPixel]);
    scanline.add(          337, vec![PrepareForNextTile                                                ]);

    // Unused name table fetches.
    scanline.add(          338, vec![GetPatternIndex                                                   ]);
    scanline.add(          340, vec![GetPatternIndex                                                   ]);

    scanline
}

#[derive(Clone)]
pub struct ScanlineActions {
    all_cycle_actions: Box<[Vec<CycleAction>; 341]>,
}

impl ScanlineActions {
    pub fn actions_at_cycle(&self, cycle: u16) -> &Vec<CycleAction> {
        &self.all_cycle_actions[usize::from(cycle)]
    }

    fn new() -> ScanlineActions {
        ScanlineActions {
            all_cycle_actions: Box::new(arr![Vec::new(); 341]),
        }
    }

    fn add(&mut self, cycle: usize, mut actions: Vec<CycleAction>) {
        self.all_cycle_actions[cycle].append(&mut actions);
    }
}
