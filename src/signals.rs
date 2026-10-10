use crate::circuit::circuit_board::{CircuitBoard, READ_2007_COMPONENTS, Read2007ComponentName, WRITE_2007_COMPONENTS, Write2007ComponentName};
use crate::circuit::node::NodeGraph;
use crate::circuit::source::PrimarySource;
use crate::memory::ppu::ppu_pinout::PpuPinout;

pub struct Signals {
    pub rendering_enabled: bool,

    circuit_board: CircuitBoard,
    read_2007_nodes: NodeGraph<Read2007ComponentName>,
    write_2007_nodes: NodeGraph<Write2007ComponentName>,
    read_2007: bool,
    write_2007: bool,
    // On hardware, the $2007_Read and $2007_Write circuits are always active, but almost always in their default state.
    // But for performance reasons it is important to not run these circuits when they are not progressing.
    read_2007_active: bool,
    write_2007_active: bool,

    scanline_type: ScanlineType,
    sprite_rendering_dot_type: SpriteRenderingDotType,
}

impl Signals {
    pub fn new() -> Self {
        let read_2007_nodes = READ_2007_COMPONENTS.to_node_graph(&[PrimarySource::PclkHigh, PrimarySource::PclkLow, PrimarySource::Read2007]);
        let write_2007_nodes = WRITE_2007_COMPONENTS.to_node_graph(&[PrimarySource::PclkHigh, PrimarySource::PclkLow, PrimarySource::Write2007]);
        Self {
            rendering_enabled: false,

            circuit_board: CircuitBoard::new(),
            read_2007_nodes,
            write_2007_nodes,
            read_2007: false,
            write_2007: false,
            read_2007_active: false,
            write_2007_active: false,

            // FIXME: This needs to come from the initial clock.
            scanline_type: ScanlineType::Visible,
            sprite_rendering_dot_type: SpriteRenderingDotType::Other,
        }
    }

    pub fn start_pclk_high(&mut self, ppu_pinout: &mut PpuPinout) {
        assert!(self.circuit_board.source(PrimarySource::PclkLow));
        self.circuit_board.set_source(PrimarySource::PclkHigh, true);
        self.circuit_board.set_source(PrimarySource::PclkLow, false);

        let read_sources = [true, false, self.read_2007];
        if self.read_2007_active {
            self.read_2007_nodes.evaluate(&read_sources);
            self.circuit_board.calculate_read_2007_outputs(ppu_pinout);
            if self.circuit_board.is_read_2007_circuit_in_base_state() {
                self.read_2007_active = false;
            }
        }

        let write_sources = [true, false, self.write_2007];
        if self.write_2007_active {
            self.write_2007_nodes.evaluate(&write_sources);
            self.circuit_board.calculate_write_2007_outputs(ppu_pinout);
            if self.circuit_board.is_write_2007_circuit_in_base_state() {
                self.write_2007_active = false;
            }
        }

        /*
        assert_eq!(self.read_2007_nodes.state_at(0), ppu_pinout.ppu_data_read, "Read didn't match. {}", self.read_2007_nodes.format_states(&read_sources));
        assert_eq!(self.read_2007_nodes.state_at(1), ppu_pinout.ppu_data_read_ale, "Read ALE didn't match. {}.", self.read_2007_nodes.format_states(&read_sources));
        assert_eq!(self.write_2007_nodes.state_at(0), ppu_pinout.ppu_data_write, "Write didn't match. {}", self.write_2007_nodes.format_states(&write_sources));
        assert_eq!(self.write_2007_nodes.state_at(1), ppu_pinout.ppu_data_write_ale, "Write ALE didn't match. {}.", self.write_2007_nodes.format_states(&write_sources));
        */
    }

    pub fn start_pclk_low(&mut self, ppu_pinout: &mut PpuPinout) {
        assert!(self.circuit_board.source(PrimarySource::PclkHigh));
        self.circuit_board.set_source(PrimarySource::PclkHigh, false);
        self.circuit_board.set_source(PrimarySource::PclkLow, true);

        if self.read_2007_active {
            let read_sources = [false, true, self.read_2007];
            self.read_2007_nodes.evaluate(&read_sources);
            self.circuit_board.calculate_read_2007_outputs(ppu_pinout);
            if self.circuit_board.is_read_2007_circuit_in_base_state() {
                self.read_2007_active = false;
            }
        }

        if self.write_2007_active {
            let write_sources = [false, true, self.write_2007];
            self.write_2007_nodes.evaluate(&write_sources);
            self.circuit_board.calculate_write_2007_outputs(ppu_pinout);
            if self.circuit_board.is_write_2007_circuit_in_base_state() {
                self.write_2007_active = false;
            }
        }

        /*
        assert_eq!(self.read_2007_nodes.state_at(0), ppu_pinout.ppu_data_read, "Read didn't match. {}", self.read_2007_nodes.format_states(&read_sources));
        assert_eq!(self.read_2007_nodes.state_at(1), ppu_pinout.ppu_data_read_ale, "ALE didn't match. {}", self.read_2007_nodes.format_states(&read_sources));
        assert_eq!(self.write_2007_nodes.state_at(0), ppu_pinout.ppu_data_write, "Write didn't match. {}", self.write_2007_nodes.format_states(&write_sources));
        assert_eq!(self.write_2007_nodes.state_at(1), ppu_pinout.ppu_data_write_ale, "Write ALE didn't match. {}.", self.write_2007_nodes.format_states(&write_sources));
        */
    }

    pub fn start_read_2007(&mut self) {
        assert!(!self.circuit_board.source(PrimarySource::Read2007));
        self.circuit_board.set_source(PrimarySource::Read2007, true);
        self.read_2007_active = true;
        self.read_2007 = true;
    }

    pub fn stop_read_2007(&mut self) {
        self.circuit_board.set_source(PrimarySource::Read2007, false);
        self.read_2007 = false;
    }

    pub fn start_write_2007(&mut self) {
        assert!(!self.circuit_board.source(PrimarySource::Write2007));
        self.circuit_board.set_source(PrimarySource::Write2007, true);
        self.write_2007_active = true;
        self.write_2007 = true;
    }

    pub fn stop_write_2007(&mut self) {
        self.circuit_board.set_source(PrimarySource::Write2007, false);
        self.write_2007 = false;
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