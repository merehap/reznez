use log::{Level, info, log_enabled};
use splitbits::{splitbits, combinebits};

use crate::memory::ppu::ppu_address::{PpuAddress, XScroll};
use crate::memory::read_result::ReadResult;
use crate::memory::regions::palette_ram::PaletteRam;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::ppu::ppu_clock::PpuClock;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::pixel_index::ColumnInTile;
use crate::ppu::register::ppu_io_bus::PpuIoBus;
use crate::memory::regions::oam::Oam;
use crate::ppu::sprite::oam_address::OamAddress;
use crate::ppu::sprite::sprite_height::SpriteHeight;
use crate::signals::{ScanlineType, Signals};

pub struct PpuRegisters {
    // PPUCTRL (0x2000) sub-registers
    nmi_enabled: bool,
    ext_pin_role: ExtPinRole,
    sprite_height: SpriteHeight,
    background_table_side: PatternTableSide,
    sprite_table_side: PatternTableSide,
    current_address_increment: AddressIncrement,
    base_name_table_quadrant: NameTableQuadrant,

    // PPUMASK (0x2001)
    mask: Mask,

    // PPUMASK (0x2001) and PPUCLOCK
    rendering_toggle_state: RenderingToggleState,

    // PPUSTATUS (0x2002) sub-registers
    pub vblank_active: bool,
    pub ppu_status_vblank_active: bool,
    pub suppress_vblank_active: bool,
    pub sprite0_hit: bool,
    pub sprite0_hit_pending: bool,
    pub sprite_overflow: bool,
    pub sprite_overflow_pending: bool,

    // OAMADDR (0x2003) and OAMDATA (0x2004)
    pub oam_addr: OamAddress,

    // PPUSCROLL (0x2005)
    pub fine_x_scroll: ColumnInTile, // "x"

    // PPUSCROLL (0x2005) and PPUADDR (0x2006)
    pub next_address: PpuAddress, // "t"
    write_toggle: WriteToggle, // "w"

    // PPUADDR (0x2006) and PPUDATA (0x2007)
    pub current_address: PpuAddress, // "v"

    // PPUDATA (0x2007)
    pub ppu_read_buffer: u8,

    // Shared between all registers (0x2000 through 0x2007)
    pub ppu_io_bus: PpuIoBus,

    rendering_disabled_outside_of_vblank: bool,
}

impl PpuRegisters {
    pub fn new() -> Self {
        Self {
            // PPUCTRL (0x2000)
            nmi_enabled: false,
            ext_pin_role: ExtPinRole::Read,
            sprite_height: SpriteHeight::Normal,
            background_table_side: PatternTableSide::Left,
            sprite_table_side: PatternTableSide::Left,
            current_address_increment: AddressIncrement::Right,
            base_name_table_quadrant: NameTableQuadrant::TopLeft,

            // PPUMASK (0x2001)
            mask: Mask::all_disabled(),

            // PPUMASK (0x2001) and PPUCLOCK
            rendering_toggle_state: RenderingToggleState::Inactive,

            // PPUSTATUS (0x2002)
            vblank_active: false,
            ppu_status_vblank_active: false,
            suppress_vblank_active: false,
            sprite0_hit: false,
            sprite0_hit_pending: false,
            sprite_overflow: false,
            sprite_overflow_pending: false,

            // OAMADDR (0x2003) and OAMDATA (0x2004)
            oam_addr: OamAddress::new(),

            // PPUSCROLL (0x2005)
            fine_x_scroll: ColumnInTile::Zero,

            // PPUSCROLL (0x2005) and PPUADDR (0x2006)
            next_address: PpuAddress::ZERO,
            write_toggle: WriteToggle::FirstByte,

            // PPUADDR (0x2006) and PPUDATA (0x2007)
            current_address: PpuAddress::ZERO,

            ppu_read_buffer: 0,

            // Shared between all registers (0x2000 through 0x2007)
            ppu_io_bus: PpuIoBus::new(),

            rendering_disabled_outside_of_vblank: false,
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

    // Write 0x2000
    pub fn write_ctrl(&mut self, value: u8) {
        self.ppu_io_bus.update(value);

        let fields = splitbits!(value, "nehb siqq");
        self.nmi_enabled = fields.n;
        self.ext_pin_role              = [ExtPinRole::Read       , ExtPinRole::Write      ][fields.e as usize];
        self.sprite_height             = [SpriteHeight::Normal   , SpriteHeight::Tall     ][fields.h as usize];
        self.background_table_side     = [PatternTableSide::Left , PatternTableSide::Right][fields.b as usize];
        self.sprite_table_side         = [PatternTableSide::Left , PatternTableSide::Right][fields.s as usize];
        self.current_address_increment = [AddressIncrement::Right, AddressIncrement::Down ][fields.i as usize];
        self.base_name_table_quadrant =  NameTableQuadrant::ALL[fields.q as usize];

        self.next_address.set_name_table_quadrant(self.base_name_table_quadrant);
    }

    // PPUMASK sub-registers
    pub fn mask(&self) -> Mask { self.mask }
    pub fn background_enabled(&self) -> bool { self.mask.background_enabled() }
    pub fn sprites_enabled(&self) -> bool { self.mask.sprites_enabled() }

    pub fn take_rendering_disabled_outside_of_vblank(&mut self) -> bool {
        let value = self.rendering_disabled_outside_of_vblank;
        self.rendering_disabled_outside_of_vblank = false;
        value
    }

    // Write 0x2001
    pub fn write_mask(&mut self, signals: &Signals, value: u8) {
        self.ppu_io_bus.update(value);

        let rendering_enabled_before = self.mask.background_enabled || self.mask.sprites_enabled;

        let fields = splitbits!(value, "efgs blmz");
        self.mask.emphasis.blue = fields.e;
        self.mask.emphasis.green = fields.f;
        self.mask.emphasis.red = fields.g;
        self.mask.sprites_enabled = fields.s;
        self.mask.background_enabled = fields.b;
        self.mask.left_sprite_columns_enabled = fields.l;
        self.mask.left_background_columns_enabled = fields.m;
        self.mask.greyscale_enabled = fields.z;

        let rendering_enabled_after = self.mask.background_enabled || self.mask.sprites_enabled;
        if signals.scanline_type() != ScanlineType::Vblank
            && rendering_enabled_before
            && !rendering_enabled_after {

            self.rendering_disabled_outside_of_vblank = true;
        }

        if signals.rendering_enabled != (self.mask.sprites_enabled() || self.mask.background_enabled()) {
            self.rendering_toggle_state = RenderingToggleState::Pending;
        }
    }

    // Peek 0x2002
    pub fn peek_status(&self) -> ReadResult {
        let v = self.ppu_status_vblank_active;
        let h = self.sprite0_hit;
        let o = self.sprite_overflow;
        let b = self.ppu_io_bus.value() & 0b0001_1111;
        ReadResult::full(combinebits!("vhobbbbb"))
    }

    // Read 0x2002
    pub fn read_status(&mut self) -> ReadResult {
        let value = self.peek_status();
        self.ppu_io_bus.update_from_status_read(value.unmasked_value());

        self.write_toggle = WriteToggle::FirstByte;

        ReadResult::full(self.ppu_io_bus.value())
    }

    // Write 0x2002 (PPUSTATUS is readonly)
    pub fn write_status(&mut self, register_value: u8) {
        self.ppu_io_bus.update(register_value);
    }

    // Write 0x2003
    pub fn write_oam_addr(&mut self, value: u8) {
        self.ppu_io_bus.update(value);
        self.oam_addr = OamAddress::from_u8(value);
    }

    // Peek 0x2004
    // TODO: Sync this up with read_oam_data
    pub fn peek_oam_data(&self, oam: &Oam) -> ReadResult {
        ReadResult::full(oam.peek(self.oam_addr))
    }

    // Read 0x2004
    pub fn read_oam_data(&mut self, oam: &mut Oam, signals: &Signals, oam_buffer: u8) -> ReadResult {
        let rendering_enabled = self.background_enabled() || self.sprites_enabled();
        let value = if rendering_enabled && signals.scanline_type() == ScanlineType::Visible {
            oam_buffer
        } else {
            oam.read(self.oam_addr)
        };
        self.ppu_io_bus.update(value);
        ReadResult::full(value)
    }

    // Write 0x2004
    pub fn write_oam_data(&mut self, oam: &mut Oam, signals: &Signals, value: u8) {
        self.ppu_io_bus.update(value);

        use ScanlineType::*;
        if signals.rendering_enabled && matches!(signals.scanline_type(), Visible | Prerender) {
            // TODO: Test ROM needed. Can this set all_sprites_evaluated?
            self.oam_addr.corrupt_by_write();
        } else {
            oam.write(self.oam_addr, value);
            self.oam_addr.next_field();
        }
    }

    // Write 0x2005
    pub fn write_scroll(&mut self, dimension: u8) {
        self.ppu_io_bus.update(dimension);

        match self.write_toggle {
            WriteToggle::FirstByte => {
                let value = XScroll::from_u8(dimension);
                self.fine_x_scroll = value.fine();
                self.next_address.set_coarse_x_scroll(value.coarse());
            }
,
            WriteToggle::SecondByte => {
                self.next_address.set_y_scroll(dimension);
            }
        }

        self.write_toggle.toggle();
    }

    // Write 0x2006
    pub fn write_ppu_addr(&mut self, value: u8) {
        self.ppu_io_bus.update(value);
        match self.write_toggle {
            WriteToggle::FirstByte => self.next_address.set_high_byte(value),
            WriteToggle::SecondByte => {
                self.next_address.set_low_byte(value);
                self.current_address = self.next_address;
            }
        }

        self.write_toggle.toggle();
    }

    // Peek 0x2007
    pub fn peek_ppu_data(&self, palette_ram: &PaletteRam) -> ReadResult {
        let data = if let Some(palette_ram_index) = self.current_address.to_palette_ram_index() {
            // This doesn't count as a PPU read for mapper purposes: /RD hasn't been asserted yet!
            let color = palette_ram.peek_color(self.mask(), palette_ram_index);
            color.to_read_result().resolve_with_bus(self.ppu_io_bus.value())
        } else {
            self.ppu_read_buffer
        };

        ReadResult::full(data)
    }

    // Read 0x2007
    pub fn read_ppu_data(&mut self, palette_ram: &PaletteRam) -> ReadResult {
        let data = self.peek_ppu_data(palette_ram);
        self.ppu_io_bus.update(data.unmasked_value());
        data
    }

    // Write 0x2007
    pub fn write_ppu_data(&mut self, value: u8) {
        self.ppu_io_bus.update(value);
        self.current_address.advance(self.current_address_increment);
    }

    // Write 0x2000 (PPUCTRL), 0x2001 (PPUMASK), 0x2003 (OAMADDR), 0x2005 (PPUSCROLL), 0x2006 (PPUADDR)
    pub fn peek_from_write_only_register(&self) -> ReadResult {
        ReadResult::full(self.ppu_io_bus.value())
    }

    // Every PPU dot
    pub fn tick(&mut self, signals: &mut Signals, clock: &PpuClock) -> PpuRegistersTickResult {
        if clock.cycle() == 1 {
            self.ppu_io_bus.maybe_decay();
        }

        if self.sprite0_hit_pending {
            self.sprite0_hit = true;
            self.sprite0_hit_pending = false;
        }

        if self.sprite_overflow_pending {
            self.sprite_overflow = true;
            self.sprite_overflow_pending = false;
        }

        use RenderingToggleState::*;
        let rendering_toggled = match self.rendering_toggle_state {
            Inactive => None,
            Pending => {
                self.rendering_toggle_state = Ready;
                None
            }
            Ready => {
                signals.rendering_enabled = !signals.rendering_enabled;
                if log_enabled!(target: "ppuflags", Level::Info) {
                    let state = if signals.rendering_enabled { "enabled" } else { "disabled" };
                    info!("Rendering {state} on {clock}");
                }

                self.rendering_toggle_state = Inactive;
                Some(if signals.rendering_enabled { Toggle::Enable } else { Toggle::Disable })
            }
        };

        PpuRegistersTickResult { rendering_toggled }
    }

    pub fn x_scroll(&self) -> XScroll {
        XScroll {
            coarse: self.next_address.coarse_x_scroll(),
            fine: self.fine_x_scroll,
        }
    }

    pub fn write_toggle(&self) -> WriteToggle {
        self.write_toggle
    }

    pub fn ppu_io_bus(&self) -> &PpuIoBus {
        &self.ppu_io_bus
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
    emphasis: Emphasis,
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

    pub fn greyscale_enabled(self) -> bool { self.greyscale_enabled }
    pub fn left_background_columns_enabled(self) -> bool { self.left_background_columns_enabled }
    pub fn left_sprite_columns_enabled(self) -> bool { self.left_sprite_columns_enabled }
    pub fn background_enabled(self) -> bool { self.background_enabled }
    pub fn sprites_enabled(self) -> bool { self.sprites_enabled }
    pub fn emphasis(self) -> Emphasis { self.emphasis }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Emphasis {
    red: bool,
    green: bool,
    blue: bool,
}

impl Emphasis {
    pub const OFF: Self = Self { red: false, green: false, blue: false };
    pub const RED: Self = Self { red: true, green: false, blue: false };
    pub const GREEN: Self = Self { red: false, green: true, blue: false };
    pub const BLUE: Self = Self { red: false, green: false, blue: true };
    pub const GREEN_BLUE: Self = Self { red: false, green: true, blue: true };
    pub const RED_BLUE: Self = Self { red: true, green: false, blue: true };
    pub const RED_GREEN: Self = Self { red: true, green: true, blue: false };
    pub const ALL: Self = Self { red: true, green: true, blue: true };

    pub fn red(self) -> bool { self.red }
    pub fn green(self) -> bool { self.green }
    pub fn blue(self) -> bool { self.blue }

    pub fn index(self) -> usize {
        ((self.blue as usize) << 2)
            | ((self.green as usize) << 1)
            | (self.red as usize)
    }
}