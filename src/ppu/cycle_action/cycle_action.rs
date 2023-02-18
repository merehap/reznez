#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    StartBackgroundRendering,
    StartSpriteRendering,

    GetPatternIndex,
    GetPaletteIndex,
    GetBackgroundTileLowByte,
    GetBackgroundTileHighByte,

    GotoNextTileColumn,
    GotoNextPixelRow,
    PrepareForNextTile,
    ResetTileColumn,

    SetPixel,
    PrepareForNextPixel,

    ReadOamByte,
    WriteSecondaryOamByte,

    ReadSpriteY,
    ReadSpritePatternIndex,
    ReadSpriteAttributes,
    ReadSpriteX,
    DummyReadSpriteX,

    ResetForOamClear,
    ResetForSpriteEvaluation,
    ResetForTransferToOamRegisters,

    StartVblank,
    RequestNmi,
    SetInitialScrollOffsets,
    SetInitialYScroll,
    ClearFlags,

    UpdateOamData,
}
