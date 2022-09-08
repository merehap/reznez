use std::collections::{BTreeMap, BTreeSet, HashMap};

use egui::{ClippedMesh, Context, TexturesDelta};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use lazy_static::lazy_static;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::{LogicalSize, PhysicalPosition, Position};
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::Window;
use winit::window::{WindowBuilder, WindowId};
use winit_input_helper::WinitInputHelper;

use crate::config::Config;
use crate::controller::joypad::{Button, ButtonStatus};
use crate::gui::gui::{execute_frame, Events, Gui};
use crate::nes::Nes;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::pattern_table::{PatternIndex, PatternTable, PatternTableSide, Tile};
use crate::ppu::pixel_index::{PixelColumn, PixelRow};
use crate::ppu::render::frame::{DebugBuffer, Frame};
use crate::util::mapped_array::MappedArray;

const TOP_MENU_BAR_HEIGHT: usize = 24;

lazy_static! {
    #[rustfmt::skip]
    static ref JOY_1_BUTTON_MAPPINGS: HashMap<VirtualKeyCode, Button> = {
        let mut mappings = HashMap::new();
        mappings.insert(VirtualKeyCode::Space,  Button::A);
        mappings.insert(VirtualKeyCode::F,      Button::B);
        mappings.insert(VirtualKeyCode::RShift, Button::Select);
        mappings.insert(VirtualKeyCode::Return, Button::Start);
        mappings.insert(VirtualKeyCode::Up,     Button::Up);
        mappings.insert(VirtualKeyCode::Down,   Button::Down);
        mappings.insert(VirtualKeyCode::Left,   Button::Left);
        mappings.insert(VirtualKeyCode::Right,  Button::Right);
        mappings
    };

    #[rustfmt::skip]
    static ref JOY_2_BUTTON_MAPPINGS: HashMap<VirtualKeyCode, Button> = {
        let mut mappings = HashMap::new();
        mappings.insert(VirtualKeyCode::Numpad0,        Button::A);
        mappings.insert(VirtualKeyCode::NumpadEnter,    Button::B);
        mappings.insert(VirtualKeyCode::NumpadSubtract, Button::Select);
        mappings.insert(VirtualKeyCode::NumpadAdd,      Button::Start);
        mappings.insert(VirtualKeyCode::Numpad8,        Button::Up);
        mappings.insert(VirtualKeyCode::Numpad5,        Button::Down);
        mappings.insert(VirtualKeyCode::Numpad4,        Button::Left);
        mappings.insert(VirtualKeyCode::Numpad6,        Button::Right);
        mappings
    };
}

pub struct EguiGui;

impl EguiGui {
    pub fn new() -> EguiGui {
        EguiGui
    }
}

impl Gui for EguiGui {
    fn run(&mut self, nes: Nes, config: Config) {
        let input = WinitInputHelper::new();
        let mut world = World { nes, config, input };
        let event_loop = EventLoop::new();

        let mut window_manager =
            WindowManager::new(&event_loop, Box::new(PrimaryRenderer::new()));

        let mut pause = false;
        event_loop.run(move |event, event_loop_window_target, control_flow| {
            if world.input.update(&event) {
                if world.input.key_pressed(VirtualKeyCode::Escape) {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if world.input.key_pressed(VirtualKeyCode::Pause)
                    || world.input.key_pressed(VirtualKeyCode::P)
                {
                    pause = !pause;
                }

                if !pause {
                    window_manager.request_redraws();
                }
            }

            match event {
                Event::WindowEvent { event, window_id } => match event {
                    WindowEvent::CloseRequested => {
                        let primary_removed = window_manager.remove_window(&window_id);
                        if primary_removed {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    _ => {
                        if let Some(window) = window_manager.window_mut(&window_id) {
                            window.handle_event(&event);
                        }
                    }
                },
                Event::RedrawRequested(window_id) => {
                    let window = window_manager.window_mut(&window_id).unwrap();
                    match window.draw(&mut world) {
                        Ok(Some((renderer, position, scale))) => window_manager
                            .create_window_from_renderer(
                                event_loop_window_target,
                                renderer,
                                position,
                                scale,
                            ),
                        Ok(None) => {}
                        Err(e) => {
                            error!("pixels.render() failed: {}", e);
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                _ => (),
            }
        });
    }
}

type WindowArgs = (Box<dyn Renderer>, Position, u64);

/// Manages all state required for rendering egui over `Pixels`.
struct EguiWindow {
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    rpass: RenderPass,
    paint_jobs: Vec<ClippedMesh>,
    textures: TexturesDelta,

    // State for the GUI
    window: Window,
    pixels: Pixels,
    renderer: Box<dyn Renderer>,
}

impl EguiWindow {
    fn from_event_loop(
        event_loop: &EventLoopWindowTarget<()>,
        scale_factor: u64,
        initial_position: Position,
        renderer: Box<dyn Renderer>,
    ) -> Self {
        let window = {
            let size = LogicalSize::new(
                scale_factor as f64 * renderer.width() as f64,
                scale_factor as f64 * renderer.height() as f64,
            );
            WindowBuilder::new()
                .with_title(renderer.name())
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_resizable(false)
                .with_position(initial_position)
                .build(event_loop)
                .unwrap()
        };

        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(
            renderer.width() as u32,
            renderer.height() as u32,
            surface_texture,
        )
        .unwrap();

        EguiWindow::new(
            window_size.width,
            window_size.height,
            scale_factor,
            window,
            pixels,
            renderer,
        )
    }

    fn new(
        width: u32,
        height: u32,
        scale_factor: f32,
        window: Window,
        pixels: pixels::Pixels,
        renderer: Box<dyn Renderer>,
    ) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = Context::default();
        let egui_state =
            egui_winit::State::from_pixels_per_point(max_texture_size, scale_factor);
        let screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
        };
        let rpass = RenderPass::new(pixels.device(), pixels.render_texture_format(), 1);
        let textures = TexturesDelta::default();

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            rpass,
            paint_jobs: Vec::new(),
            textures,
            window,
            pixels,
            renderer,
        }
    }

    /// Handle input events from the window manager.
    fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        self.egui_state.on_event(&self.egui_ctx, event);
    }

    fn draw(&mut self, world: &mut World) -> Result<Option<WindowArgs>, String> {
        self.renderer.render(world, &mut self.pixels);

        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let mut result = None;
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            result = self.renderer.ui(egui_ctx, world);
        });

        self.textures.append(output.textures_delta);
        self.egui_state.handle_platform_output(
            &self.window,
            &self.egui_ctx,
            output.platform_output,
        );
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);

        self.pixels
            .render_with(|encoder, render_target, context| {
                context.scaling_renderer.render(encoder, render_target);
                self.rpass
                    .add_textures(&context.device, &context.queue, &self.textures)
                    .map_err(|err| err.to_string())?;
                self.rpass.update_buffers(
                    &context.device,
                    &context.queue,
                    &self.paint_jobs,
                    &self.screen_descriptor,
                );

                // Record all render passes.
                self.rpass
                    .execute(
                        encoder,
                        render_target,
                        &self.paint_jobs,
                        &self.screen_descriptor,
                        None,
                    )
                    .map_err(|err| err.to_string())?;

                // Cleanup
                let textures = std::mem::take(&mut self.textures);
                Ok(self
                    .rpass
                    .remove_textures(textures)
                    .map_err(|err| err.to_string())?)
            })
            .map_err(|err| err.to_string())?;

        Ok(result)
    }
}

struct World {
    nes: Nes,
    config: Config,
    input: WinitInputHelper,
}

struct WindowManager {
    primary_window_id: WindowId,
    windows_by_id: BTreeMap<WindowId, (String, EguiWindow)>,
    window_names: BTreeSet<String>,
}

impl WindowManager {
    pub fn new(
        event_loop: &EventLoopWindowTarget<()>,
        primary_renderer: Box<dyn Renderer>,
    ) -> WindowManager {
        let name = primary_renderer.name();
        let primary_window = EguiWindow::from_event_loop(
            event_loop,
            3,
            Position::Physical(PhysicalPosition { x: 50, y: 50 }),
            primary_renderer,
        );
        let mut manager = WindowManager {
            primary_window_id: primary_window.window.id(),
            windows_by_id: BTreeMap::new(),
            window_names: BTreeSet::new(),
        };
        manager.window_names.insert(name.clone());
        manager
            .windows_by_id
            .insert(primary_window.window.id(), (name, primary_window));
        manager
    }

    pub fn create_window_from_renderer(
        &mut self,
        event_loop: &EventLoopWindowTarget<()>,
        renderer: Box<dyn Renderer>,
        position: Position,
        scale: u64,
    ) {
        let name = renderer.name();
        if self.window_names.contains(&name) {
            return;
        }

        self.window_names.insert(name.clone());

        let window = EguiWindow::from_event_loop(event_loop, scale, position, renderer);
        self.windows_by_id
            .insert(window.window.id(), (name, window));
    }

    pub fn remove_window(&mut self, window_id: &WindowId) -> bool {
        let primary_removed = *window_id == self.primary_window_id;
        if let Some((name, _)) = self.windows_by_id.remove(window_id) {
            self.window_names.remove(&name);
        }

        primary_removed
    }

    pub fn request_redraws(&self) {
        for (_id, window) in self.windows_by_id.values() {
            window.window.request_redraw();
        }
    }

    pub fn window_mut(&mut self, window_id: &WindowId) -> Option<&mut EguiWindow> {
        self.windows_by_id
            .get_mut(window_id)
            .map(|(_, window)| window)
    }
}

trait Renderer {
    fn name(&self) -> String;
    fn ui(&mut self, ctx: &Context, world: &World) -> Option<WindowArgs>;
    fn render(&mut self, world: &mut World, pixels: &mut Pixels);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

struct PrimaryRenderer {}

impl PrimaryRenderer {
    fn new() -> Self {
        PrimaryRenderer {}
    }
}

impl Renderer for PrimaryRenderer {
    fn name(&self) -> String {
        "REZNEZ".to_string()
    }

    fn ui(&mut self, ctx: &Context, _world: &World) -> Option<WindowArgs> {
        let mut result = None;
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Debug Windows", |ui| {
                    if ui.button("Status").clicked() {
                        ui.close_menu();
                        result = Some((
                            Box::new(StatusRenderer::new()) as Box<dyn Renderer>,
                            Position::Physical(PhysicalPosition { x: 850, y: 360 }),
                            2,
                        ));
                    }
                    if ui.button("Layers").clicked() {
                        ui.close_menu();
                        result = Some((
                            Box::new(LayersRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 850, y: 50 }),
                            1,
                        ));
                    }
                    if ui.button("Name Tables").clicked() {
                        ui.close_menu();
                        result = Some((
                            Box::new(NameTableRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 1400, y: 50 }),
                            1,
                        ));
                    }
                    if ui.button("Sprites").clicked() {
                        ui.close_menu();
                        result = Some((
                            Box::new(SpritesRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 1400, y: 660 }),
                            6,
                        ));
                    }
                    if ui.button("Pattern Tables").clicked() {
                        ui.close_menu();
                        result = Some((
                            Box::new(PatternTableRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 850, y: 660 }),
                            3,
                        ));
                    }
                    if ui.button("CHR Banks").clicked() {
                        ui.close_menu();
                        result = Some((
                            Box::new(ChrBanksRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 50, y: 50 }),
                            2,
                        ));
                    }
                })
            });
        });

        result
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let display_frame = |frame: &Frame, mask, _frame_index| {
            frame.copy_to_rgba_buffer(mask, pixels.get_frame().try_into().unwrap());
        };
        execute_frame(
            &mut world.nes,
            &world.config,
            &events(&world.input),
            display_frame,
        );
    }

    fn width(&self) -> usize {
        PixelColumn::COLUMN_COUNT
    }

    fn height(&self) -> usize {
        PixelRow::ROW_COUNT
    }
}

struct StatusRenderer {}

impl StatusRenderer {
    const WIDTH: usize = 300;
    const HEIGHT: usize = 300;

    pub fn new() -> StatusRenderer {
        StatusRenderer {}
    }
}

impl Renderer for StatusRenderer {
    fn name(&self) -> String {
        "Status".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &World) -> Option<WindowArgs> {
        let nes = &world.nes;
        let clock = nes.ppu().clock();
        let ppu_regs = nes.memory().ppu_regs();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Frame");
                    ui.label(format!("{:?}", clock.frame()));
                    ui.end_row();
                    /*
                    ui.label("Scanline");
                    ui.label(format!("{:?}", clock.scanline()));
                    ui.end_row();
                    ui.label("PPU Cycle");
                    ui.label(format!("{:?}", clock.cycle()));
                    ui.end_row();
                    ui.label("CPU Cycle");
                    ui.label(format!("{:?}", nes.cpu().cycle()));
                    ui.end_row();
                    */
                    ui.label("X Scroll");
                    ui.label(format!("{}", nes.ppu().x_scroll().to_u8()));
                    ui.end_row();
                    ui.label("Y Scroll");
                    ui.label(format!("{}", nes.ppu().y_scroll().to_u8()));
                    ui.end_row();
                    ui.label("NMI Enabled");
                    ui.label(format!("{}", ppu_regs.nmi_enabled()));
                    ui.end_row();
                    ui.label("Sprite Height");
                    ui.label(format!("{:?}", ppu_regs.sprite_height()));
                    ui.end_row();
                    ui.label("Base Name Table");
                    ui.label(format!("{:?}", ppu_regs.base_name_table_quadrant()));
                    ui.end_row();
                    ui.label("Active Name Table");
                    ui.label(format!("{:?}", nes.ppu().active_name_table_quadrant()));
                    ui.end_row();
                    ui.label("Background");
                    ui.label(format!(
                        "Enabled: {}, Pattern Table: {:?} side",
                        ppu_regs.background_enabled(),
                        ppu_regs.background_table_side(),
                    ));
                    ui.end_row();
                    ui.label("Sprites");
                    ui.label(format!(
                        "Enabled: {}, Pattern Table: {:?} side",
                        ppu_regs.sprites_enabled(),
                        ppu_regs.sprite_table_side(),
                    ));
                    ui.end_row();
                    ui.label("");
                    ui.label("");
                    ui.end_row();
                    ui.label("Mapper");
                    ui.label(format!("{:?}", nes.cartridge().mapper_number()));
                    ui.end_row();
                    ui.label("Name Table Mirroring");
                    ui.label(format!(
                        "{:?}",
                        nes.memory().mapper().name_table_mirroring()
                    ));
                    ui.end_row();
                    ui.label("PRG ROM banks");
                    ui.label(&nes.memory().mapper().prg_rom_bank_string());
                    ui.end_row();
                    ui.label("CHR ROM banks");
                    ui.label(&nes.memory().mapper().chr_rom_bank_string());
                });
        });

        None
    }

    fn render(&mut self, _world: &mut World, _pixels: &mut Pixels) {
        // Do nothing yet.
    }

    fn width(&self) -> usize {
        StatusRenderer::WIDTH
    }

    fn height(&self) -> usize {
        StatusRenderer::HEIGHT
    }
}

struct LayersRenderer {
    frame: Frame,
    buffer: DebugBuffer<{ LayersRenderer::WIDTH }, { LayersRenderer::HEIGHT }>,
}

impl LayersRenderer {
    const WIDTH: usize = 517;
    const HEIGHT: usize = 485 + TOP_MENU_BAR_HEIGHT;

    fn new() -> LayersRenderer {
        LayersRenderer {
            frame: Frame::new(),
            buffer: DebugBuffer::new(Rgb::WHITE),
        }
    }
}

impl Renderer for LayersRenderer {
    fn name(&self) -> String {
        "Layers".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _world: &World) -> Option<WindowArgs> {
        None
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        self.buffer
            .place_frame(0, TOP_MENU_BAR_HEIGHT, world.nes.frame());
        self.buffer.place_frame(
            261,
            TOP_MENU_BAR_HEIGHT,
            &world.nes.frame().to_background_only(),
        );

        let (ppu, mem) = world.nes.ppu_and_memory_mut();
        let mem = mem.as_ppu_memory();

        self.frame.clear();
        ppu.oam().only_front_sprites().render(&mem, &mut self.frame);
        self.buffer
            .place_frame(0, 245 + TOP_MENU_BAR_HEIGHT, &self.frame);

        self.frame.clear();
        ppu.oam().only_back_sprites().render(&mem, &mut self.frame);
        self.buffer
            .place_frame(261, 245 + TOP_MENU_BAR_HEIGHT, &self.frame);

        self.buffer.copy_to_rgba_buffer(pixels.get_frame());
    }

    fn width(&self) -> usize {
        LayersRenderer::WIDTH
    }

    fn height(&self) -> usize {
        LayersRenderer::HEIGHT
    }
}

struct NameTableRenderer {
    frame: Frame,
    buffer: DebugBuffer<{ NameTableRenderer::WIDTH }, { NameTableRenderer::HEIGHT }>,
}

impl NameTableRenderer {
    const WIDTH: usize = 2 * 256 + 2;
    const HEIGHT: usize = 2 * 240 + 2;

    fn new() -> NameTableRenderer {
        NameTableRenderer {
            frame: Frame::new(),
            buffer: DebugBuffer::new(Rgb::WHITE),
        }
    }
}

impl Renderer for NameTableRenderer {
    fn name(&self) -> String {
        "Name Tables".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _world: &World) -> Option<WindowArgs> {
        None
    }

    #[rustfmt::skip]
    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let x = usize::from(world.nes.ppu().x_scroll().to_u8());
        let y = usize::from(world.nes.ppu().y_scroll().to_u8());
        let mem = world
            .nes
            .memory_mut()
            .as_ppu_memory();

        let width = NameTableRenderer::WIDTH;
        let height = NameTableRenderer::HEIGHT;
        // Clear any junk out of the outer border.
        self.buffer.place_wrapping_horizontal_line(0, 0, width, Rgb::new(255, 255, 255));
        self.buffer.place_wrapping_horizontal_line(height, 0, width, Rgb::new(255, 255, 255));
        self.buffer.place_wrapping_vertical_line(0, 0, height, Rgb::new(255, 255, 255));
        self.buffer.place_wrapping_vertical_line(width, 0, height, Rgb::new(255, 255, 255));

        self.frame.set_universal_background_rgb(mem.palette_table().universal_background_rgb());
        mem.name_table(NameTableQuadrant::TopLeft)
            .render(&mem.background_pattern_table(), &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(1, 1, &self.frame);
        mem.name_table(NameTableQuadrant::TopRight)
            .render(&mem.background_pattern_table(), &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(257, 1, &self.frame);
        mem.name_table(NameTableQuadrant::BottomLeft)
            .render(&mem.background_pattern_table(), &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(1, 241, &self.frame);
        mem.name_table(NameTableQuadrant::BottomRight)
            .render(&mem.background_pattern_table(), &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(257, 241, &self.frame);

        self.buffer.place_wrapping_horizontal_line(y, x, x + 257, Rgb::new(255, 0, 0));
        self.buffer.place_wrapping_horizontal_line(y + 241, x, x + 257, Rgb::new(255, 0, 0));
        self.buffer.place_wrapping_vertical_line(x, y, y + 241, Rgb::new(255, 0, 0));
        self.buffer.place_wrapping_vertical_line(x + 257, y, y + 241, Rgb::new(255, 0, 0));

        self.buffer.copy_to_rgba_buffer(pixels.get_frame());
    }

    fn width(&self) -> usize {
        NameTableRenderer::WIDTH
    }

    fn height(&self) -> usize {
        NameTableRenderer::HEIGHT
    }
}

struct SpritesRenderer {
    buffer: DebugBuffer<{ SpritesRenderer::WIDTH }, { SpritesRenderer::HEIGHT }>,
}

impl SpritesRenderer {
    const WIDTH: usize = 8 * (8 + 1);
    const HEIGHT: usize = 8 * (8 + 1);

    fn new() -> SpritesRenderer {
        SpritesRenderer { buffer: DebugBuffer::new(Rgb::WHITE) }
    }
}

impl Renderer for SpritesRenderer {
    fn name(&self) -> String {
        "Sprites".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _world: &World) -> Option<WindowArgs> {
        None
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let sprites = world.nes.ppu().oam().sprites();
        let mem = world.nes.memory_mut().as_ppu_memory();

        for (index, sprite) in sprites.iter().enumerate() {
            let tile = sprite
                .render_normal_height(&mem.sprite_pattern_table(), &mem.palette_table());
            self.buffer.place_tile(
                (8 + 1) * (index as usize % 8),
                (8 + 1) * (index as usize / 8),
                &tile,
            );
        }

        self.buffer.copy_to_rgba_buffer(pixels.get_frame());
    }

    fn width(&self) -> usize {
        SpritesRenderer::WIDTH
    }

    fn height(&self) -> usize {
        SpritesRenderer::HEIGHT
    }
}

struct PatternTableRenderer {
    tile: Tile,
    buffer:
        DebugBuffer<{ PatternTableRenderer::WIDTH }, { PatternTableRenderer::HEIGHT }>,
}

impl PatternTableRenderer {
    const WIDTH: usize = 2 * (8 + 1) * 16 + 10;
    const HEIGHT: usize = (8 + 1) * 16 + TOP_MENU_BAR_HEIGHT / 3;

    fn new() -> PatternTableRenderer {
        PatternTableRenderer {
            tile: Tile::new(),
            buffer: DebugBuffer::new(Rgb::WHITE),
        }
    }
}

impl Renderer for PatternTableRenderer {
    fn name(&self) -> String {
        "Pattern Table".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _world: &World) -> Option<WindowArgs> {
        None
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let mem = world.nes.memory_mut().as_ppu_memory();

        let mut offset = 0;
        for side in [PatternTableSide::Left, PatternTableSide::Right] {
            let palette = if mem.regs().sprite_table_side() == side {
                mem.palette_table().sprite_palette(PaletteTableIndex::Zero)
            } else {
                mem.palette_table()
                    .background_palette(PaletteTableIndex::Zero)
            };
            for index in 0..=255 {
                mem.pattern_table(side).render_background_tile(
                    PatternIndex::new(index),
                    palette,
                    &mut self.tile,
                );
                self.buffer.place_tile(
                    (8 + 1) * (index as usize % 16) + offset,
                    (8 + 1) * (index as usize / 16) + TOP_MENU_BAR_HEIGHT / 3,
                    &self.tile,
                );
            }

            offset += (8 + 1) * 16 + 10;
        }

        self.buffer.copy_to_rgba_buffer(pixels.get_frame());
    }

    fn width(&self) -> usize {
        PatternTableRenderer::WIDTH
    }

    fn height(&self) -> usize {
        PatternTableRenderer::HEIGHT
    }
}

struct ChrBanksRenderer {
    tile: Tile,
    buffer: DebugBuffer<{ ChrBanksRenderer::WIDTH }, { ChrBanksRenderer::HEIGHT }>,
}

impl ChrBanksRenderer {
    const WIDTH: usize = (8 + 1) * 256;
    const HEIGHT: usize = (8 + 1) * 32;

    fn new() -> ChrBanksRenderer {
        ChrBanksRenderer {
            tile: Tile::new(),
            buffer: DebugBuffer::new(Rgb::WHITE),
        }
    }
}

impl Renderer for ChrBanksRenderer {
    fn name(&self) -> String {
        "CHR Banks".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _world: &World) -> Option<WindowArgs> {
        None
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let palette = world
            .nes
            .memory_mut()
            .as_ppu_memory()
            .palette_table()
            .sprite_palette(PaletteTableIndex::Zero);
        let chunks = world.nes.memory().mapper().chr_bank_chunks();
        if chunks.is_empty() {
            return;
        }

        assert_eq!(chunks[0].len(), 4);

        let mut y_offset = 0;
        for raw_pattern_table in chunks {
            let mut x_offset = 0;
            let raw_pattern_table: MappedArray<4> =
                MappedArray::from_chunks(raw_pattern_table.try_into().unwrap());
            let pattern_table = PatternTable::new(&raw_pattern_table);
            for index in 0..=255 {
                pattern_table.render_background_tile(
                    PatternIndex::new(index),
                    palette,
                    &mut self.tile,
                );
                self.buffer.place_tile(x_offset, y_offset, &self.tile);
                x_offset += 9;
            }

            y_offset += 9;
        }

        // TODO: Add ability to switch between the normal display and the following one.
        /*
        let odd_offset = 9 * chunks.len() / 2 + 10;
        let mut y_offset = 0;
        for (i, raw_pattern_table) in chunks.into_iter().enumerate() {
            let mut x_offset = 0;
            let odd_offset = if i % 2 == 0 {0} else {odd_offset};
            let raw_pattern_table: MappedArray<4> = MappedArray::from_chunks(raw_pattern_table.try_into().unwrap());
            let pattern_table = PatternTable::new(&raw_pattern_table);
            for index in 0..=255 {
                pattern_table.render_background_tile(
                    PatternIndex::new(index), palette, &mut self.tile);
                self.buffer.place_tile(x_offset, y_offset + odd_offset, &self.tile);
                x_offset += 9;
            }

            if i % 2 == 0 {
                y_offset += 9;
            }
        }
        */

        self.buffer.copy_to_rgba_buffer(pixels.get_frame());
    }

    fn width(&self) -> usize {
        ChrBanksRenderer::WIDTH
    }

    fn height(&self) -> usize {
        ChrBanksRenderer::HEIGHT
    }
}

fn events(input: &WinitInputHelper) -> Events {
    let mut joypad1_button_statuses = BTreeMap::new();
    let mut joypad2_button_statuses = BTreeMap::new();

    for (&key, &button) in JOY_1_BUTTON_MAPPINGS.iter() {
        if input.key_pressed(key) {
            joypad1_button_statuses.insert(button, ButtonStatus::Pressed);
        } else if input.key_released(key) {
            joypad1_button_statuses.insert(button, ButtonStatus::Unpressed);
        };
    }

    for (&key, &button) in JOY_2_BUTTON_MAPPINGS.iter() {
        if input.key_pressed(key) {
            joypad2_button_statuses.insert(button, ButtonStatus::Pressed);
        } else if input.key_released(key) {
            joypad2_button_statuses.insert(button, ButtonStatus::Unpressed);
        };
    }

    Events {
        // Quit-handling is done by winit.
        should_quit: false,
        joypad1_button_statuses,
        joypad2_button_statuses,
    }
}
