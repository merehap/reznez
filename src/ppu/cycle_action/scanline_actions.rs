use std::sync::LazyLock;

use arr_macro::arr;

use crate::ppu::cycle_action::cycle_action::CycleAction;

#[expect(clippy::identity_op)]
pub static VISIBLE_SCANLINE_ACTIONS: LazyLock<ScanlineActions> = LazyLock::new(|| {
    use CycleAction::*;

    let mut line = ScanlineActions::new();
    //           ||CYCLE||       ||---------BACKGROUND-TILE-ACTIONS---------||  ||-SPRITE--ACTIONS---||  ||-----DISPLAY-ACTIONS-----||
    // TODO: Remove this section in favor of using EdgeDetectors.
    line.add(           65, vec![                                               StartSpriteEvaluation                       ]);
    line.add(          257, vec![StopReadingBackgroundTiles                                                                 ]);
    line.add(          257, vec![                                               StartLoadingOamRegisters                    ]);
    line.add(          321, vec![                                               StopLoadingOamRegisters                     ]);
    line.add(          321, vec![StartReadingBackgroundTiles                                                                ]);

    // Fetch the remaining 31 usable background tiles for the current scanline.
    // Secondary OAM clearing then sprite evaluation, transfering OAM to secondary OAM.
    // Cycles 1 through 249.
    for tile in 0..31 {
        let cycle = 8 * tile + 1;
        line.add(cycle + 0, vec![SetPatternIndexAddress     ,                     ReadOamByte          , SetPixel, PrepareForNextPixel]);
        line.add(cycle + 1, vec![Read                       ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
        line.add(cycle + 2, vec![SetPaletteIndexAddress     ,                     ReadOamByte          , SetPixel, PrepareForNextPixel]);
        line.add(cycle + 3, vec![Read                       ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
        line.add(cycle + 4, vec![SetPatternLowAddress       ,                     ReadOamByte          , SetPixel, PrepareForNextPixel]);
        line.add(cycle + 5, vec![Read                       ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
        line.add(cycle + 6, vec![SetPatternHighAddress      ,                     ReadOamByte          , SetPixel, PrepareForNextPixel]);
        line.add(cycle + 7, vec![Read                       ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    }

    // Resetting the Secondary OAM index actually happens just AFTER the first ReadOamByte, so the first read sees the pre-reset index.
    line.add(            1, vec![                                                 ResetSecondaryOamIndex                              ]);

    // Fetch a final unused background tile and get ready for the next ROW of tiles.
    // Complete the sprite evaluation.
    line.add(          249, vec![SetPatternIndexAddress     ,                     ReadOamByte          , SetPixel, PrepareForNextPixel]);
    line.add(          250, vec![Read                       ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    line.add(          251, vec![SetPaletteIndexAddress     ,                     ReadOamByte          , SetPixel, PrepareForNextPixel]);
    line.add(          252, vec![Read                       ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    line.add(          253, vec![SetPatternLowAddress       ,                     ReadOamByte          , SetPixel, PrepareForNextPixel]);
    line.add(          254, vec![Read                       ,                     WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    line.add(          255, vec![SetPatternHighAddress      ,                     ReadOamByte          , SetPixel, PrepareForNextPixel]);
    line.add(          256, vec![Read                       , GotoNextPixelRow  , WriteSecondaryOamByte, SetPixel, PrepareForNextPixel]);
    line.add(          257, vec![                             ResetTileColumn                                                         ]);

    // Transfer secondary OAM to OAM registers and fetch sprite pattern data.
    // Cycles 257 through 320
    for sprite in 0..8 {
        let cycle = 8 * sprite + 257;
        line.add(cycle + 0, vec![SetPatternIndexAddress     ,                     ReadSpriteY           , AdvanceSecondaryOam, ResetOamAddress]);
        line.add(cycle + 1, vec![Read                       ,                     ReadSpritePatternIndex, AdvanceSecondaryOam, ResetOamAddress]);
        line.add(cycle + 2, vec![SetPaletteIndexAddress     ,                     ReadSpriteAttributes  , AdvanceSecondaryOam, ResetOamAddress]);
        line.add(cycle + 3, vec![Read                       ,                     ReadSpriteX                                , ResetOamAddress]);
        line.add(cycle + 4, vec![SetSpritePatternLowAddress ,                     DummyReadSpriteX                           , ResetOamAddress]);
        line.add(cycle + 5, vec![Read                       ,                     DummyReadSpriteX                           , ResetOamAddress]);
        line.add(cycle + 6, vec![SetSpritePatternHighAddress,                     DummyReadSpriteX                           , ResetOamAddress]);
        line.add(cycle + 7, vec![Read                       ,                     DummyReadSpriteX                           , ResetOamAddress]);
        line.add(cycle + 8, vec![                                                                         AdvanceSecondaryOam                 ]);
    }

    // Fetch the first background tile for the next scanline.
    line.add(          321, vec![SetPatternIndexAddress     ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          322, vec![Read                       ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          323, vec![SetPaletteIndexAddress     ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          324, vec![Read                       ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          325, vec![SetPatternLowAddress       ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          326, vec![Read                       ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          327, vec![SetPatternHighAddress      ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          328, vec![Read                       ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          329, vec![SetPatternIndexAddress     ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    // Fetch the second background tile for the next scanline.
    line.add(          330, vec![Read                       ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          331, vec![SetPaletteIndexAddress     ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          332, vec![Read                       ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          333, vec![SetPatternLowAddress       ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          334, vec![Read                       ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          335, vec![SetPatternHighAddress      ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          336, vec![Read                       ,                     ReadSpriteY           , AdvanceSecondaryOam, PrepareForNextPixel]);
    line.add(          337, vec![SetPatternIndexAddress     ,                     ReadSpriteY           , AdvanceSecondaryOam                     ]);

    // Unused name table fetches.
    line.add(          338, vec![Read                       ,                     ReadSpriteY           , AdvanceSecondaryOam                     ]);
    line.add(          339, vec![SetPatternIndexAddress     ,                     ReadSpriteY           , AdvanceSecondaryOam, MaybeClearSpriteX  ]);
    line.add(          340, vec![Read                       ,                     ReadSpriteY           , AdvanceSecondaryOam                     ]);

    line
});

pub static POST_RENDER_SCANLINE_ACTIONS: LazyLock<ScanlineActions> = LazyLock::new(|| {
    let mut post_render_scanline = ScanlineActions::new();
    post_render_scanline.prepend(0, CycleAction::StartPostRenderScanline);
    post_render_scanline
});

pub static START_VBLANK_SCANLINE_ACTIONS: LazyLock<ScanlineActions> = LazyLock::new(|| {
    use CycleAction::*;

    let mut scanline = ScanlineActions::new();
    scanline.add(0, vec![StartVblankScanlines]);
    scanline.add(1, vec![StartVblank]);
    scanline
});

pub static EMPTY_SCANLINE_ACTIONS: LazyLock<ScanlineActions> = LazyLock::new(ScanlineActions::new);

#[expect(clippy::identity_op)]
pub static PRE_RENDER_SCANLINE_ACTIONS: LazyLock<ScanlineActions> = LazyLock::new(|| {
    use CycleAction::*;

    let mut scanline = ScanlineActions::new();
    scanline.add(            0, vec![StartPreRenderScanline                                              ]);
    scanline.add(            1, vec![ClearFlags                                                          ]);
    scanline.add(          321, vec![StartReadingBackgroundTiles                                         ]);

    for cycle in 1..9 {
        scanline.add(cycle    , vec![MaybeCorruptOamStart                                                ]);
    }

    //               ||CYCLE||       ||---------BACKGROUND-TILE-ACTIONS---------|| ||-DISPLAY-ACTIONS-||
    // Fetch the remaining 31 used background tiles for the current scanline.
    // Cycles 1 through 249.
    for tile in 0..31 {
        let cycle = 8 * tile + 1;
        scanline.add(cycle + 0, vec![SetPatternIndexAddress                                              ]);
        scanline.add(cycle + 1, vec![Read                                                                ]);
        scanline.add(cycle + 2, vec![SetPaletteIndexAddress                                              ]);
        scanline.add(cycle + 3, vec![Read                                                                ]);
        scanline.add(cycle + 4, vec![SetPatternLowAddress                                                ]);
        scanline.add(cycle + 5, vec![Read                                                                ]);
        scanline.add(cycle + 6, vec![SetPatternHighAddress                                               ]);
        scanline.add(cycle + 7, vec![Read                     ,                                          ]);
    }

    // Resetting the Secondary OAM index actually happens just AFTER the first ReadOamByte, so the first read sees the pre-reset index.
    scanline.add(            1, vec![ResetSecondaryOamIndex]);

    // Fetch a final unused background tile and get ready for the next ROW of tiles.
    scanline.add(          249, vec![SetPatternIndexAddress                                              ]);
    scanline.add(          250, vec![Read                                                                ]);
    scanline.add(          251, vec![SetPaletteIndexAddress                                              ]);
    scanline.add(          252, vec![Read                                                                ]);
    scanline.add(          253, vec![SetPatternLowAddress                                                ]);
    scanline.add(          254, vec![Read                                                                ]);
    scanline.add(          255, vec![SetPatternHighAddress                                               ]);
    scanline.add(          256, vec![Read                       , GotoNextPixelRow                       ]);
    scanline.add(          257, vec![                             ResetTileColumn                        ]);

    // Dummy sprite pattern fetches.
    // Cycles 257 through 320
    for sprite in 0..8 {
        let cycle = 8 * sprite + 257;
        scanline.add(cycle + 0, vec![SetPatternIndexAddress     , ReadSpriteY           , AdvanceSecondaryOam, ResetOamAddress]);
        scanline.add(cycle + 1, vec![Read                       , ReadSpritePatternIndex, AdvanceSecondaryOam, ResetOamAddress]);
        scanline.add(cycle + 2, vec![SetPaletteIndexAddress     , ReadSpriteAttributes  , AdvanceSecondaryOam, ResetOamAddress]);
        scanline.add(cycle + 3, vec![Read                       , ReadSpriteX                                , ResetOamAddress]);
        scanline.add(cycle + 4, vec![SetSpritePatternLowAddress , DummyReadSpriteX                           , ResetOamAddress]);
        scanline.add(cycle + 5, vec![Read                       , DummyReadSpriteX                           , ResetOamAddress]);
        scanline.add(cycle + 6, vec![SetSpritePatternHighAddress, DummyReadSpriteX                           , ResetOamAddress]);
        scanline.add(cycle + 7, vec![Read                       , DummyReadSpriteX                           , ResetOamAddress]);
        scanline.add(cycle + 8, vec![                                                     AdvanceSecondaryOam                 ]);
    }

    // Overlaps the above dummy sprite pattern fetches.
    for cycle in 280..=304 {
        scanline.add(    cycle, vec![SetInitialYScroll]);
    }

    scanline.add(          320, vec![SetInitialScrollOffsets]);

    // Fetch the first background tile for the next scanline.
    scanline.add(          321, vec![SetPatternIndexAddress   ,                       PrepareForNextPixel]);
    scanline.add(          322, vec![Read                     ,                       PrepareForNextPixel]);
    scanline.add(          323, vec![SetPaletteIndexAddress   ,                       PrepareForNextPixel]);
    scanline.add(          324, vec![Read                     ,                       PrepareForNextPixel]);
    scanline.add(          325, vec![SetPatternLowAddress     ,                       PrepareForNextPixel]);
    scanline.add(          326, vec![Read                     ,                       PrepareForNextPixel]);
    scanline.add(          327, vec![SetPatternHighAddress    ,                       PrepareForNextPixel]);
    scanline.add(          328, vec![Read                     ,                       PrepareForNextPixel]);
    // Fetch the second background tile for the next scanline.
    scanline.add(          329, vec![SetPatternIndexAddress   ,                       PrepareForNextPixel]);
    scanline.add(          330, vec![Read                     ,                       PrepareForNextPixel]);
    scanline.add(          331, vec![SetPaletteIndexAddress   ,                       PrepareForNextPixel]);
    scanline.add(          332, vec![Read                     ,                       PrepareForNextPixel]);
    scanline.add(          333, vec![SetPatternLowAddress     ,                       PrepareForNextPixel]);
    scanline.add(          334, vec![Read                     ,                       PrepareForNextPixel]);
    scanline.add(          335, vec![SetPatternHighAddress    ,                       PrepareForNextPixel]);
    scanline.add(          336, vec![Read                     ,                       PrepareForNextPixel]);

    // Dummy name table fetches.
    scanline.add(          337, vec![SetPatternIndexAddress                                              ]);
    scanline.add(          338, vec![Read                                                                ]);
    scanline.add(          339, vec![SetPatternIndexAddress                                              ]);
    scanline.add(          340, vec![Read                                                                ]);

    scanline
});

pub static FIRST_VISIBLE_SCANLINE_ACTIONS: LazyLock<ScanlineActions> = LazyLock::new(|| {
    let mut first_visible_scanline = VISIBLE_SCANLINE_ACTIONS.clone();
    first_visible_scanline.prepend(1, CycleAction::StartVisibleScanlines);
    first_visible_scanline
});

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
