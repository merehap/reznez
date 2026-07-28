pub struct Signals {
    pub rendering_enabled: bool,
    scanline_type: ScanlineType,
    sprite_rendering_dot_type: SpriteRenderingDotType,
}

impl Signals {
    pub fn new() -> Self {
        Self {
            rendering_enabled: false,
            // FIXME: This needs to come from the initial clock.
            scanline_type: ScanlineType::Visible,
            sprite_rendering_dot_type: SpriteRenderingDotType::Other,
        }
    }

    pub fn scanline_type(&self) -> ScanlineType {
        self.scanline_type
    }

    pub fn sprite_rendering_dot_type(&self) -> SpriteRenderingDotType {
        self.sprite_rendering_dot_type
    }

    pub fn visible_scanlines_starting(&mut self) {
        self.scanline_type = ScanlineType::Visible;
    }

    pub fn vblank_scanlines_starting(&mut self) {
        self.scanline_type = ScanlineType::Vblank;
    }

    pub fn prerender_scanline_starting(&mut self) {
        self.scanline_type = ScanlineType::Prerender;
    }

    pub fn oam_clearing_starting(&mut self) {
        self.sprite_rendering_dot_type = SpriteRenderingDotType::OamClearing;
    }

    pub fn evaluation_starting(&mut self) {
        self.sprite_rendering_dot_type = SpriteRenderingDotType::Evaluation;
    }

    pub fn secondary_oam_transfer_starting(&mut self) {
        self.sprite_rendering_dot_type = SpriteRenderingDotType::SecondaryOamTransfer;
    }

    pub fn other_sprite_rendering_dot_type_starting(&mut self) {
        self.sprite_rendering_dot_type = SpriteRenderingDotType::Other;
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ScanlineType {
    Visible,
    Vblank,
    Prerender,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SpriteRenderingDotType {
    OamClearing,
    Evaluation,
    SecondaryOamTransfer,
    Other,
}