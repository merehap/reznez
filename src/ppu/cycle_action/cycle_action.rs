#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    SetPatternIndexAddress,
    GetPatternIndex,
    SetPaletteIndexAddress,
    GetPaletteIndex,
    SetPatternLowAddress,
    SetPatternHighAddress,
    GetPatternLowByte,
    GetPatternHighByte,
    SetSpritePatternLowAddress,
    SetSpritePatternHighAddress,
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
