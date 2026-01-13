use log::info;
use splitbits::splitbits;

use crate::mapper::PatternTableSide;
use crate::memory::ppu::ppu_address::{PpuAddress, XScroll, YScroll};
use crate::ppu::ppu_clock::PpuClock;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::pixel_index::ColumnInTile;
use crate::ppu::register::ppu_io_bus::PpuIoBus;
use crate::ppu::register::registers::status::Status;
use crate::ppu::sprite::oam::Oam;
use crate::ppu::sprite::oam_address::OamAddress;
use crate::ppu::sprite::sprite_height::SpriteHeight;
use crate::ppu::tile_number::TileNumber;

pub struct PpuRegisters {
    // PPUCTRL sub-registers
    pub nmi_enabled: bool,
    pub ext_pin_role: ExtPinRole,
    pub sprite_height: SpriteHeight,
    pub background_table_side: PatternTableSide,
    pub sprite_table_side: PatternTableSide,
    pub current_address_increment: AddressIncrement,
    pub base_name_table_quadrant: NameTableQuadrant,

    mask: Mask,
    status: Status,
    pub oam_addr: OamAddress,
    ppu_read_buffer: u8,

    // "v"
    pub current_address: PpuAddress,
    // "x"
    pub fine_x_scroll: ColumnInTile,
    // "t"
    pub(in crate::ppu) next_address: PpuAddress,

    ppu_io_bus: PpuIoBus,

    pub suppress_vblank_active: bool,

    rendering_enabled: bool,
    // "w"
    write_toggle: WriteToggle,
    rendering_toggle_state: RenderingToggleState,
    reset_recently: bool,
}

impl PpuRegisters {
    pub fn new() -> Self {
        Self {
            // PPUCTRL sub-registers
            nmi_enabled: false,
            ext_pin_role: ExtPinRole::Read,
            sprite_height: SpriteHeight::Normal,
            background_table_side: PatternTableSide::Left,
            sprite_table_side: PatternTableSide::Left,
            current_address_increment: AddressIncrement::Right,
            base_name_table_quadrant: NameTableQuadrant::TopLeft,

            mask: Mask::all_disabled(),
            status: Status::new(),
            oam_addr: OamAddress::new(),
            ppu_read_buffer: 0,

            current_address: PpuAddress::ZERO,
            fine_x_scroll: ColumnInTile::Zero,
            next_address: PpuAddress::ZERO,

            ppu_io_bus: PpuIoBus::new(),

            write_toggle: WriteToggle::FirstByte,
            suppress_vblank_active: false,
            rendering_enabled: false,
            rendering_toggle_state: RenderingToggleState::Inactive,
            reset_recently: true,
        }
    }

    // PPUCTRL sub-registers
    pub fn nmi_enabled(&self) -> bool { self.nmi_enabled }
    pub fn ext_pin_role(&self) -> ExtPinRole { self.ext_pin_role }
    pub fn sprite_height(&self) -> SpriteHeight { self.sprite_height }
    pub fn background_table_side(&self) -> PatternTableSide { self.background_table_side }
    pub fn sprite_table_side(&self) -> PatternTableSide { self.sprite_table_side }
    pub fn current_address_increment(&self) -> AddressIncrement { self.current_address_increment }
    pub fn base_name_table_quadrant(&self) -> NameTableQuadrant { self.base_name_table_quadrant }

    pub fn mask(&self) -> Mask {
        self.mask
    }

    pub fn background_enabled(&self) -> bool {
        self.mask.background_enabled()
    }

    pub fn sprites_enabled(&self) -> bool {
        self.mask.sprites_enabled()
    }

    pub fn rendering_enabled(&self) -> bool {
        self.rendering_enabled
    }

    pub fn active_name_table_quadrant(&self) -> NameTableQuadrant {
        self.next_address.name_table_quadrant()
    }

    pub fn x_scroll(&self) -> XScroll {
        XScroll {
            coarse: self.next_address.coarse_x_scroll(),
            fine: self.fine_x_scroll,
        }
    }

    pub fn y_scroll(&self) -> YScroll {
        self.next_address.y_scroll()
    }

    pub fn write_toggle(&self) -> WriteToggle {
        self.write_toggle
    }

    pub(in crate::ppu) fn start_vblank(&mut self, clock: &PpuClock) {
        info!(target: "ppuflags", " {clock}\tStarting vblank.");
        self.status.vblank_active = true;
    }

    pub(in crate::ppu) fn stop_vblank(&mut self, clock: &PpuClock) {
        if self.status.vblank_active {
            info!(target: "ppuflags", " {clock}\tStopping vblank.");
        }

        self.status.vblank_active = false;
    }

    pub(in crate::ppu) fn set_sprite0_hit(&mut self) {
        self.status.sprite0_hit = true;
    }

    pub(in crate::ppu) fn clear_sprite0_hit(&mut self) {
        self.status.sprite0_hit = false;
    }

    pub(in crate::ppu) fn set_sprite_overflow(&mut self) {
        self.status.sprite_overflow = true;
    }

    pub(in crate::ppu) fn clear_sprite_overflow(&mut self) {
        self.status.sprite_overflow = false;
    }

    pub(in crate::ppu) fn clear_reset(&mut self) {
        self.reset_recently = false;
    }

    pub fn tick(&mut self, clock: &PpuClock) -> PpuRegistersTickResult {
        self.maybe_decay_ppu_io_bus(clock);
        let rendering_toggled = self.maybe_toggle_rendering_enabled();
        PpuRegistersTickResult { rendering_toggled }
    }

    fn maybe_decay_ppu_io_bus(&mut self, clock: &PpuClock) {
        if clock.cycle() == 1 {
            self.ppu_io_bus.maybe_decay();
        }
    }

    fn maybe_toggle_rendering_enabled(&mut self) -> Option<Toggle> {
        use RenderingToggleState::*;
        match self.rendering_toggle_state {
            Inactive => None,
            Pending => {
                self.rendering_toggle_state = Ready;
                None
            }
            Ready => {
                self.rendering_enabled = !self.rendering_enabled;
                self.rendering_toggle_state = Inactive;
                Some(if self.rendering_enabled { Toggle::Enable } else { Toggle::Disable })
            }
        }
    }

    pub fn can_generate_nmi(&self) -> bool {
        self.status.vblank_active && self.nmi_enabled
    }

    pub fn reset_recently(&self) -> bool {
        self.reset_recently
    }

    pub fn peek_ppu_io_bus(&self) -> u8 {
        self.ppu_io_bus.value()
    }

    pub fn peek_status(&self) -> u8 {
        self.status.to_u8() | (self.ppu_io_bus.value() & 0b0001_1111)
    }

    // 0x2002
    pub fn read_status(&mut self, clock: &PpuClock) -> u8 {
        self.write_toggle = WriteToggle::FirstByte;

        let value = self.peek_status();
        self.ppu_io_bus.update_from_status_read(value);

        self.stop_vblank(clock);
        // https://wiki.nesdev.org/w/index.php?title=NMI#Race_condition
        if clock.scanline() == 241 && clock.cycle() == 0 {
            self.suppress_vblank_active = true;
        }

        self.ppu_io_bus.value()
    }

    pub fn peek_oam_data(&self, oam: &Oam) -> u8 {
        oam.peek(self.oam_addr)
    }

    pub fn read_oam_data(&mut self, oam: &Oam) -> u8 {
        let value = self.peek_oam_data(oam);
        self.ppu_io_bus.update_from_read(value);
        value
    }

    pub fn peek_ppu_data(&self, old_data: u8) -> u8 {
        if self.current_address.is_in_palette_table() {
            // When reading palette data only, read the current data pointed to
            // by self.current_address, not what was previously pointed to.
            // Retain the previous ppu_io_bus values for the unused bits of palette data.
            (self.ppu_io_bus.value() & 0b1100_0000) | (old_data & 0b0011_1111)
        } else {
            self.ppu_read_buffer
        }
    }

    pub fn read_ppu_data(&mut self, old_data: u8) -> u8 {
        let value_read = self.peek_ppu_data(old_data);
        self.ppu_io_bus.update_from_read(value_read);
        value_read
    }

    pub fn set_ppu_read_buffer_and_advance(&mut self, new_buffer_data: u8) {
        self.ppu_read_buffer = new_buffer_data;
        self.current_address.advance(self.current_address_increment);
    }

    pub fn write_ppu_io_bus(&mut self, register_value: u8) {
        self.ppu_io_bus.update_from_write(register_value);
    }

    // 0x2000
    pub fn set_ctrl(&mut self, value: u8) {
        let fields = splitbits!(value, "nehbsiqq");
        self.nmi_enabled = fields.n;
        self.ext_pin_role = [ExtPinRole::Read, ExtPinRole::Write][fields.e as usize];
        self.sprite_height = [SpriteHeight::Normal, SpriteHeight::Tall][fields.h as usize];
        self.background_table_side = [PatternTableSide::Left, PatternTableSide::Right][fields.b as usize];
        self.sprite_table_side = [PatternTableSide::Left, PatternTableSide::Right][fields.s as usize];
        self.current_address_increment = [AddressIncrement::Right, AddressIncrement::Down][fields.i as usize];
        self.base_name_table_quadrant =  NameTableQuadrant::ALL[fields.q as usize];

        self.next_address.set_name_table_quadrant(self.base_name_table_quadrant);
        self.ppu_io_bus.update_from_write(value);
    }

    // 0x2001
    pub fn write_mask(&mut self, value: u8) {
        self.ppu_io_bus.update_from_write(value);
        let old_mask = self.mask;
        let fields = splitbits!(value, "efgs blmz");
        self.mask.emphasize_blue = fields.e;
        self.mask.emphasize_green = fields.f;
        self.mask.emphasize_red = fields.g;
        self.mask.sprites_enabled = fields.s;
        self.mask.background_enabled = fields.b;
        self.mask.left_sprite_columns_enabled = fields.l;
        self.mask.left_background_columns_enabled = fields.m;
        self.mask.greyscale_enabled = fields.z;

        if self.rendering_enabled != (self.mask.sprites_enabled() || self.mask.background_enabled()) {
            self.rendering_toggle_state = RenderingToggleState::Pending;
        }

        // TODO: Merge this into the standard, higher level change-logging.
        log_change(old_mask.emphasize_blue, self.mask.emphasize_blue, "Blue emphasis");
        log_change(old_mask.emphasize_green, self.mask.emphasize_green, "Green emphasis");
        log_change(old_mask.emphasize_red, self.mask.emphasize_red, "Red emphasis");
        log_change(old_mask.sprites_enabled, self.mask.sprites_enabled, "Sprites");
        log_change(old_mask.background_enabled, self.mask.background_enabled, "Background");

        log_change(
            old_mask.left_sprite_columns_enabled,
            self.mask.left_sprite_columns_enabled,
            "Left sprite columns",
        );
        log_change(
            old_mask.left_background_columns_enabled,
            self.mask.left_background_columns_enabled,
            "Left background columns",
        );
        log_change(old_mask.greyscale_enabled, self.mask.greyscale_enabled, "Greyscale");
    }

    pub fn write_oam_addr(&mut self, value: u8) {
        self.ppu_io_bus.update_from_write(value);
        self.oam_addr = OamAddress::from_u8(value);
    }

    pub fn write_oam_data(&mut self, oam: &mut Oam, value: u8) {
        oam.write(self.oam_addr, value);
        self.ppu_io_bus.update_from_write(value);
        // Advance to next sprite byte to write.
        self.oam_addr.increment();
    }

    // 0x2005
    pub fn write_scroll(&mut self, dimension: u8) {
        match self.write_toggle {
            WriteToggle::FirstByte => self.set_next_address_x_scroll(dimension),
            WriteToggle::SecondByte => self.next_address.set_y_scroll(dimension),
        }

        self.write_toggle.toggle();
        self.ppu_io_bus.update_from_write(dimension);
    }

    // 0x2006
    pub fn write_ppu_addr(&mut self, value: u8) {
        match self.write_toggle {
            WriteToggle::FirstByte => self.next_address.set_high_byte(value),
            WriteToggle::SecondByte => {
                self.next_address.set_low_byte(value);
                self.current_address = self.next_address;
            }
        }

        self.write_toggle.toggle();
        self.write_ppu_io_bus(value);
    }

    pub fn write_ppu_data(&mut self, value: u8) {
        self.ppu_io_bus.update_from_write(value);
        self.current_address.advance(self.current_address_increment);
    }

    pub fn set_next_address_x_scroll(&mut self, value: u8) {
        let value = XScroll::from_u8(value);
        self.fine_x_scroll = value.fine();
        self.next_address.set_coarse_x_scroll(value.coarse());
    }

    pub fn address_in_name_table(&self) -> PpuAddress {
        PpuAddress::in_name_table(
            self.current_address.name_table_quadrant(),
            self.current_address.coarse_x_scroll(),
            self.current_address.coarse_y_scroll(),
        )
    }

    pub fn address_in_attribute_table(&self) -> PpuAddress {
        PpuAddress::in_attribute_table(
            self.current_address.name_table_quadrant(),
            self.current_address.coarse_x_scroll(),
            self.current_address.coarse_y_scroll(),
        )
    }

    pub fn address_for_low_pattern_byte(&self, tile_number: TileNumber) -> PpuAddress {
        PpuAddress::in_pattern_table(
            self.background_table_side,
            tile_number,
            self.current_address.fine_y_scroll(),
            false,
        )
    }

    pub fn address_for_high_pattern_byte(&self, tile_number: TileNumber) -> PpuAddress {
        PpuAddress::in_pattern_table(
            self.background_table_side,
            tile_number,
            self.current_address.y_scroll().fine(),
            true,
        )
    }

    pub fn palette_table_index(&self, attribute_byte: u8) -> PaletteTableIndex {
        PaletteTableIndex::from_attribute_byte(
            attribute_byte,
            self.current_address.coarse_x_scroll(),
            self.current_address.coarse_y_scroll(),
        )
    }

    pub fn reset_tile_column(&mut self) {
        // Reset coarse X scroll. For non-scrolling cartridges, this always means setting it to 0.
        self.current_address.set_coarse_x_scroll(self.next_address.coarse_x_scroll());

        // Reset to the selected name table to be one on the left side (0x2000 or 0x2800).
        let mut name_table_quadrant = self.current_address.name_table_quadrant();
        name_table_quadrant.copy_horizontal_side_from(self.next_address.name_table_quadrant());
        self.current_address.set_name_table_quadrant(name_table_quadrant);
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum WriteToggle {
    FirstByte,
    SecondByte,
}

impl WriteToggle {
    pub fn toggle(&mut self) {
        use WriteToggle::*;
        *self = match self {
            FirstByte => SecondByte,
            SecondByte => FirstByte,
        };
    }
}

#[derive(Clone, Copy)]
pub enum RenderingToggleState {
    Inactive,
    Pending,
    Ready,
}

pub struct PpuRegistersTickResult {
    pub rendering_toggled: Option<Toggle>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Toggle {
    Enable,
    Disable,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ExtPinRole {
    Read,
    Write,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AddressIncrement {
    Right,
    Down,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Mask {
    greyscale_enabled: bool,
    left_background_columns_enabled: bool,
    left_sprite_columns_enabled: bool,
    background_enabled: bool,
    sprites_enabled: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,
}

impl Mask {
    pub fn all_disabled() -> Self {
        Self::default()
    }

    pub fn full_screen_enabled() -> Mask {
        Self {
            left_background_columns_enabled: true,
            left_sprite_columns_enabled: true,
            .. Self::all_disabled()
        }
    }

    pub fn emphasis_index(self) -> usize {
        ((self.emphasize_blue as usize) << 2)
            | ((self.emphasize_green as usize) << 1)
            | (self.emphasize_red as usize)
    }

    pub fn greyscale_enabled(self) -> bool {
        self.greyscale_enabled
    }

    pub fn left_background_columns_enabled(self) -> bool {
        self.left_background_columns_enabled
    }

    pub fn left_sprite_columns_enabled(self) -> bool {
        self.left_sprite_columns_enabled
    }

    pub fn background_enabled(self) -> bool {
        self.background_enabled
    }

    pub fn sprites_enabled(self) -> bool {
        self.sprites_enabled
    }
}

fn log_change(old: bool, new: bool, message_prefix: &str) {
    let message = match (old, new) {
        (false, true) => format!("\t{message_prefix} enabled."),
        (true, false) => format!("\t{message_prefix} disabled."),
        _ => return,
    };
    info!(target: "ppuflags", "{message}");
}