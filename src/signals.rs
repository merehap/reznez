use crate::circuit::circuit_board::CircuitBoard;
use crate::circuit::source::PrimarySource;
use crate::memory::ppu::ppu_pinout::PpuPinout;

pub struct Signals {
    pub rendering_enabled: bool,

    circuit_board: CircuitBoard,
    // On hardware, the $2007_Read and $2007_Write circuits are always active, but almost always in their default state.
    // But for performance reasons it is important to not run these circuits when they are not progressing.
    read_2007_active: bool,
    write_2007_active: bool,

    scanline_type: ScanlineType,
    sprite_rendering_dot_type: SpriteRenderingDotType,
}

impl Signals {
    pub fn new() -> Self {
        Self {
            rendering_enabled: false,

            circuit_board: CircuitBoard::new(),
            read_2007_active: false,
            write_2007_active: false,

            // FIXME: This needs to come from the initial clock.
            scanline_type: ScanlineType::Visible,
            sprite_rendering_dot_type: SpriteRenderingDotType::Other,
        }
    }

    pub fn start_ppu_clock_first_half(&mut self, ppu_pinout: &mut PpuPinout) {
        assert!(self.circuit_board.source(PrimarySource::PpuClockSecondHalf));
        self.circuit_board.set_source(PrimarySource::PpuClockFirstHalf, true);
        self.circuit_board.set_source(PrimarySource::PpuClockSecondHalf, false);

        if self.read_2007_active {
            self.circuit_board.calculate_read_2007_outputs(ppu_pinout);
            if self.circuit_board.is_read_2007_circuit_in_base_state() {
                self.read_2007_active = false;
            }
        }

        if self.write_2007_active {
            self.circuit_board.calculate_write_2007_outputs(ppu_pinout);
            if self.circuit_board.is_write_2007_circuit_in_base_state() {
                self.write_2007_active = false;
            }
        }
    }

    pub fn start_ppu_clock_second_half(&mut self, ppu_pinout: &mut PpuPinout) {
        assert!(self.circuit_board.source(PrimarySource::PpuClockFirstHalf));
        self.circuit_board.set_source(PrimarySource::PpuClockFirstHalf, false);
        self.circuit_board.set_source(PrimarySource::PpuClockSecondHalf, true);

        if self.read_2007_active {
            self.circuit_board.calculate_read_2007_outputs(ppu_pinout);
            if self.circuit_board.is_read_2007_circuit_in_base_state() {
                self.read_2007_active = false;
            }
        }

        if self.write_2007_active {
            self.circuit_board.calculate_write_2007_outputs(ppu_pinout);
            if self.circuit_board.is_write_2007_circuit_in_base_state() {
                self.write_2007_active = false;
            }
        }
    }

    pub fn start_read_2007(&mut self) {
        assert!(!self.circuit_board.source(PrimarySource::Read2007));
        self.circuit_board.set_source(PrimarySource::Read2007, true);
        self.read_2007_active = true;
    }

    pub fn stop_read_2007(&mut self) {
        self.circuit_board.set_source(PrimarySource::Read2007, false);
    }

    pub fn start_write_2007(&mut self) {
        assert!(!self.circuit_board.source(PrimarySource::Write2007));
        self.circuit_board.set_source(PrimarySource::Write2007, true);
        self.write_2007_active = true;
    }

    pub fn stop_write_2007(&mut self) {
        self.circuit_board.set_source(PrimarySource::Write2007, false);
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