#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    Read,
    SetPatternIndexAddress,
    SetPaletteIndexAddress,
    SetPatternLowAddress,
    SetPatternHighAddress,
    SetSpritePatternLowAddress,
    SetSpritePatternHighAddress,

    GotoNextPixelRow,
    ResetTileColumn,

    SetPixel,
    PrepareForNextPixel,

    MaybeCorruptOamStart,
    ResetOamAddress,

    ReadOamByte,
    WriteSecondaryOamByte,

    IncrementOamRegisterIndex,
    AdvanceSecondaryOam,
    ReadSpriteY,
    ReadSpritePatternIndex,
    ReadSpriteAttributes,
    ReadSpriteX,
    DummyReadSpriteX,
    MaybeClearSpriteX,

    StartVisibleScanlines,
    StartPostRenderScanline,
    StartVblankScanlines,
    StartPreRenderScanline,

    StartReadingBackgroundTiles,
    StopReadingBackgroundTiles,
    ResetSecondaryOamIndex,
    ClearOamRegisterIndex,
    StartSpriteEvaluation,
    StartLoadingOamRegisters,
    StopLoadingOamRegisters,

    StartVblank,
    SetInitialScrollOffsets,
    SetInitialYScroll,
    ClearFlags,
}
