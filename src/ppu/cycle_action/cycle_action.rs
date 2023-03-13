#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    GetPatternIndex,
    GetPaletteIndex,
    GetPatternLowByte,
    GetPatternHighByte,
    GetSpritePatternLowByte,
    GetSpritePatternHighByte,

    GotoNextTileColumn,
    GotoNextPixelRow,
    PrepareForNextTile,
    ResetTileColumn,

    SetPixel,
    PrepareForNextPixel,

    ResetOamAddress,

    ReadOamByte,
    WriteSecondaryOamByte,

    ReadSpriteY,
    ReadSpritePatternIndex,
    ReadSpriteAttributes,
    ReadSpriteX,
    DummyReadSpriteX,

    StartVisibleScanlines,
    StartPostRenderScanline,
    StartVblankScanlines,
    StartPreRenderScanline,

    StartReadingBackgroundTiles,
    StopReadingBackgroundTiles,
    StartClearingSecondaryOam,
    StartSpriteEvaluation,
    StartLoadingOamRegisters,
    StopLoadingOamRegisters,

    StartVblank,
    RequestNmi,
    SetInitialScrollOffsets,
    SetInitialYScroll,
    ClearFlags,
}
