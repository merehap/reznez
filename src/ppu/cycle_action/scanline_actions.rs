use arr_macro::arr;
use lazy_static::lazy_static;

use crate::ppu::cycle_action::cycle_action::CycleAction;

lazy_static! {
    pub static ref FIRST_VISIBLE_SCANLINE_ACTIONS: ScanlineActions = first_visible_scanline_actions();
    pub static ref VISIBLE_SCANLINE_ACTIONS: ScanlineActions = visible_scanline_actions();
    pub static ref POST_RENDER_SCANLINE_ACTIONS: ScanlineActions = post_render_scanline_actions();
    pub static ref START_VBLANK_SCANLINE_ACTIONS: ScanlineActions = start_vblank_scanline_actions();
    pub static ref EMPTY_SCANLINE_ACTIONS: ScanlineActions = empty_scanline_actions();
    pub static ref PRE_RENDER_SCANLINE_ACTIONS: ScanlineActions = pre_render_scanline_actions();
}

#[allow(clippy::identity_op)]
fn visible_scanline_actions() -> ScanlineActions {
    use CycleAction::*;

    let mut line = ScanlineActions::new();
    //           ||CYCLE||       ||---------BACKGROUND-TILE-ACTIONS---------||  ||-SPRITE--ACTIONS-||  ||-----DISPLAY-ACTIONS-----||
    line.add(            1, vec![                                               StartClearingSecondaryOam                           ]);
    line.add(           65, vec![                                               StartSpriteEvaluation                               ]);
    line.add(          257, vec![StopReadingBackgroundTiles                                                                         ]);
    line.add(          257, vec![                                               StartLoadingOamRegisters                            ]);
    line.add(          321, vec![                                               StopLoadingOamRegisters                             ]);
    line.add(          321, vec![StartReadingBackgroundTiles                                                                        ]);

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
        line.add(cycle + 5, vec![GetPatternLowByte        ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
        line.add(cycle + 6, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
        line.add(cycle + 7, vec![GetPatternHighByte       , GotoNextTileColumn, WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
        line.add(cycle + 8, vec![PrepareForNextTile                                                                                 ]);
    }

    // Fetch a final unused background tile and get ready for the next ROW of tiles.
    // Complete the sprite evaluation.
    line.add(          249, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
    line.add(          250, vec![GetPatternIndex          ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    line.add(          251, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
    line.add(          252, vec![GetPaletteIndex          ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    line.add(          253, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
    line.add(          254, vec![GetPatternLowByte        ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    line.add(          255, vec![                                               ReadOamByte          , SetPixel, PrepareForNextPixel]);
    line.add(          256, vec![GetPatternHighByte       , GotoNextPixelRow  , WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    line.add(          257, vec![PrepareForNextTile       , ResetTileColumn                                                         ]);

    // Transfer secondary OAM to OAM registers and fetch sprite pattern data.
    // Cycles 257 through 320
    for sprite in 0..8 {
        let cycle = 8 * sprite + 257;
        line.add(cycle + 0, vec![                                               ReadSpriteY           , ResetOamAddress             ]);
        line.add(cycle + 1, vec![GetPatternIndex          ,                     ReadSpritePatternIndex, ResetOamAddress             ]);
        line.add(cycle + 2, vec![                                               ReadSpriteAttributes  , ResetOamAddress             ]);
        line.add(cycle + 3, vec![GetPatternIndex          ,                     ReadSpriteX           , ResetOamAddress             ]);
        line.add(cycle + 4, vec![                                               DummyReadSpriteX      , ResetOamAddress             ]);
        line.add(cycle + 5, vec![GetSpritePatternLowByte  ,                     DummyReadSpriteX      , ResetOamAddress             ]);
        line.add(cycle + 6, vec![                                               DummyReadSpriteX      , ResetOamAddress             ]);
        line.add(cycle + 7, vec![GetSpritePatternHighByte ,                     DummyReadSpriteX      , ResetOamAddress, IncrementOamRegisterIndex]);
    }

    // Fetch the first background tile for the next scanline.
    line.add(          321, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          322, vec![GetPatternIndex          ,                     ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          323, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          324, vec![GetPaletteIndex          ,                     ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          325, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          326, vec![GetPatternLowByte        ,                     ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          327, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          328, vec![GetPatternHighByte       , GotoNextTileColumn, ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          329, vec![PrepareForNextTile       ,                     ReadSpriteY                    , PrepareForNextPixel]);
    // Fetch the second background tile for the next scanline.
    line.add(          330, vec![GetPatternIndex          ,                     ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          331, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          332, vec![GetPaletteIndex          ,                     ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          333, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          334, vec![GetPatternLowByte        ,                     ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          335, vec![                                               ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          336, vec![GetPatternHighByte       , GotoNextTileColumn, ReadSpriteY                    , PrepareForNextPixel]);
    line.add(          337, vec![PrepareForNextTile       ,                     ReadSpriteY                                         ]);

    // Unused name table fetches.
    line.add(          338, vec![GetPatternIndex          ,                     ReadSpriteY                                         ]);
    line.add(          339, vec![                                               ReadSpriteY                                         ]);
    line.add(          340, vec![GetPatternIndex          ,                     ReadSpriteY                                         ]);

    line
}

fn post_render_scanline_actions() -> ScanlineActions {
    let mut post_render_scanline = ScanlineActions::new();
    post_render_scanline.prepend(0, CycleAction::StartPostRenderScanline);
    post_render_scanline
}

fn start_vblank_scanline_actions() -> ScanlineActions {
    use CycleAction::*;

    let mut scanline = ScanlineActions::new();
    scanline.add(0, vec![StartVblankScanlines]);
    scanline.add(1, vec![StartVblank]);
    scanline.add(3, vec![RequestNmi]);
    scanline
}

fn empty_scanline_actions() -> ScanlineActions {
    ScanlineActions::new()
}

#[allow(clippy::identity_op)]
fn pre_render_scanline_actions() -> ScanlineActions {
    use CycleAction::*;

    let mut scanline = ScanlineActions::new();
    // Clear vblank, sprite 0 hit, and sprite overflow.
    scanline.add(            1, vec![ClearFlags                                                        ]);
    scanline.add(          321, vec![StartReadingBackgroundTiles                                       ]);

    //               ||CYCLE||       ||---------BACKGROUND-TILE-ACTIONS---------|| ||-DISPLAY-ACTIONS-||
    // Fetch the remaining 31 used background tiles for the current scanline.
    // Cycles 1 through 249.
    for tile in 0..31 {
        let cycle = 8 * tile + 1;
        scanline.add(cycle + 1, vec![GetPatternIndex                                                   ]);
        scanline.add(cycle + 3, vec![GetPaletteIndex                                                   ]);
        scanline.add(cycle + 5, vec![GetPatternLowByte                                                 ]);
        scanline.add(cycle + 7, vec![GetPatternHighByte       , GotoNextTileColumn                     ]);
        scanline.add(cycle + 8, vec![PrepareForNextTile                                                ]);
    }

    // Fetch a final unused background tile and get ready for the next ROW of tiles.
    scanline.add(          250, vec![GetPatternIndex                                                   ]);
    scanline.add(          252, vec![GetPaletteIndex                                                   ]);
    scanline.add(          254, vec![GetPatternLowByte                                                 ]);
    scanline.add(          256, vec![GetPatternHighByte       , GotoNextPixelRow                       ]);
    scanline.add(          257, vec![PrepareForNextTile       , ResetTileColumn                        ]);

    // Dummy sprite pattern fetches.
    // Cycles 257 through 320
    for sprite in 0..8 {
        let cycle = 8 * sprite + 257;
        scanline.add(cycle + 0, vec![                           ResetOamAddress                        ]);
        scanline.add(cycle + 1, vec![                           ResetOamAddress                        ]);
        scanline.add(cycle + 2, vec![                           ResetOamAddress                        ]);
        scanline.add(cycle + 3, vec![                           ResetOamAddress                        ]);
        scanline.add(cycle + 4, vec![                           ResetOamAddress                        ]);
        scanline.add(cycle + 5, vec![GetSpritePatternLowByte  , ResetOamAddress                        ]);
        scanline.add(cycle + 6, vec![                           ResetOamAddress                        ]);
        scanline.add(cycle + 7, vec![GetSpritePatternHighByte , ResetOamAddress                        ]);
    }

    // Overlaps the above dummy sprite pattern fetches.
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
    scanline.add(          326, vec![GetPatternLowByte        ,                     PrepareForNextPixel]);
    scanline.add(          327, vec![                                               PrepareForNextPixel]);
    scanline.add(          328, vec![GetPatternHighByte       , GotoNextTileColumn, PrepareForNextPixel]);
    scanline.add(          329, vec![PrepareForNextTile       ,                     PrepareForNextPixel]);
    // Fetch the second background tile for the next scanline.
    scanline.add(          330, vec![GetPatternIndex          ,                     PrepareForNextPixel]);
    scanline.add(          331, vec![                                               PrepareForNextPixel]);
    scanline.add(          332, vec![GetPaletteIndex          ,                     PrepareForNextPixel]);
    scanline.add(          333, vec![                                               PrepareForNextPixel]);
    scanline.add(          334, vec![GetPatternLowByte        ,                     PrepareForNextPixel]);
    scanline.add(          335, vec![                                               PrepareForNextPixel]);
    scanline.add(          336, vec![GetPatternHighByte       , GotoNextTileColumn, PrepareForNextPixel]);
    scanline.add(          337, vec![PrepareForNextTile                                                ]);

    // Dummy name table fetches.
    scanline.add(          338, vec![GetPatternIndex                                                   ]);
    scanline.add(          340, vec![GetPatternIndex                                                   ]);

    scanline
}

fn first_visible_scanline_actions() -> ScanlineActions {
    let mut first_visible_scanline = VISIBLE_SCANLINE_ACTIONS.clone();
    first_visible_scanline.prepend(1, CycleAction::StartVisibleScanlines);
    first_visible_scanline
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

    pub(in crate::ppu::cycle_action) fn prepend(&mut self, cycle: usize, action: CycleAction) {
        self.all_cycle_actions[cycle].insert(0, action);
    }
}
