#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    GetPatternIndex,
    GetPaletteIndex,
    LoadPatternLowAddress,
    LoadPatternHighAddress,
    GetPatternLowByte,
    GetPatternHighByte,
    LoadSpritePatternLowAddress,
    LoadSpritePatternHighAddress,
    GetSpritePatternLowByte,
    GetSpritePatternHighByte,
    IncrementOamRegisterIndex,

    GotoNextTileColumn,
    GotoNextPixelRow,
    PrepareForNextTile,
    ResetTileColumn,

    SetPixel,
    PrepareForNextPixel,

    MaybeCorruptOamStart,
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
    SetInitialScrollOffsets,
    SetInitialYScroll,
    ClearFlags,
}
