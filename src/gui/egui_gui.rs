use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, LazyLock};

use egui::{ClippedPrimitive, Context, TexturesDelta, ViewportId};
use egui_file::FileDialog;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use gilrs::{self, GamepadId};
use log::info;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::{LogicalSize, PhysicalPosition, Position};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::keyboard::KeyCode;
use winit::window::Window;
use winit::window::{WindowBuilder, WindowId};
use winit_input_helper::WinitInputHelper;

use crate::config::Config;
use crate::controller::joypad::{Button, ButtonStatus};
use crate::gui::debug_screens::name_table::NameTable;
use crate::gui::debug_screens::pattern_table::{PatternTable, Tile};
use crate::gui::gui::{execute_frame, Events, Gui};
use crate::mapper::{CpuAddress, KIBIBYTE};
use crate::nes::Nes;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::ppu::pixel_index::{PixelColumn, PixelRow};
use crate::ppu::render::frame::{DebugBuffer, Frame};
use crate::ppu::tile_number::TileNumber;

const TOP_MENU_BAR_HEIGHT: usize = 24;

#[rustfmt::skip]
static JOY_1_KEYBOARD_MAPPINGS: LazyLock<HashMap<KeyCode, Button>> = LazyLock::new(|| {
    let mut mappings = HashMap::new();
    mappings.insert(KeyCode::KeyJ,   Button::B);
    mappings.insert(KeyCode::KeyK,   Button::A);
    mappings.insert(KeyCode::KeyU,   Button::Select);
    mappings.insert(KeyCode::KeyI,   Button::Start);

    mappings.insert(KeyCode::KeyW,   Button::Up);
    mappings.insert(KeyCode::KeyS,   Button::Down);
    mappings.insert(KeyCode::KeyA,   Button::Left);
    mappings.insert(KeyCode::KeyD,   Button::Right);
    mappings.insert(KeyCode::ArrowUp,    Button::Up);
    mappings.insert(KeyCode::ArrowDown,  Button::Down);
    mappings.insert(KeyCode::ArrowLeft,  Button::Left);
    mappings.insert(KeyCode::ArrowRight, Button::Right);
    mappings
});

#[rustfmt::skip]
static JOY_2_KEYBOARD_MAPPINGS: LazyLock<HashMap<KeyCode, Button>> = LazyLock::new(|| {
    let mut mappings = HashMap::new();
    mappings.insert(KeyCode::Numpad0,        Button::A);
    mappings.insert(KeyCode::NumpadEnter,    Button::B);
    mappings.insert(KeyCode::NumpadSubtract, Button::Select);
    mappings.insert(KeyCode::NumpadAdd,      Button::Start);
    mappings.insert(KeyCode::Numpad8,        Button::Up);
    mappings.insert(KeyCode::Numpad5,        Button::Down);
    mappings.insert(KeyCode::Numpad4,        Button::Left);
    mappings.insert(KeyCode::Numpad6,        Button::Right);
    mappings
});

static JOY_1_JOYPAD_MAPPINGS: LazyLock<HashMap<u32, Button>> = LazyLock::new(|| {
    let mut mappings = HashMap::new();
    mappings.insert(65824, Button::A);
    mappings.insert(65825, Button::B);
    mappings.insert(65830, Button::Select);
    mappings.insert(65831, Button::Start);
    mappings.insert(66080, Button::Up);
    mappings.insert(66081, Button::Down);
    mappings.insert(66082, Button::Left);
    mappings.insert(66083, Button::Right);
    mappings
});

/*
#[rustfmt::skip]
static JOY_2_JOYPAD_MAPPINGS: LazyLock<HashMap<u32, Button>> = LazyLock::new(|| {
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
});
*/

pub struct EguiGui;

impl Gui for EguiGui {
    fn run(&mut self, nes: Nes, config: Config) {
        let input = WinitInputHelper::new();

        let gilrs = gilrs::Gilrs::new().unwrap();
        let gamepads: Vec<(gilrs::GamepadId, gilrs::Gamepad)> = gilrs.gamepads().collect();
        assert!(gamepads.len() < 2, "There must not be more than one gamepad connected at a time.");
        let active_gamepad_id = gamepads.first().map(|(id, _)| *id);

        let mut world = World { nes, config, input, gilrs, active_gamepad_id };
        let event_loop = EventLoop::new().unwrap();

        let primary_renderer = Box::new(PrimaryRenderer::new());
        let primary_window_name = primary_renderer.name();
        let primary_window = EguiWindow::from_event_loop(
            &event_loop,
            3,
            Position::Physical(PhysicalPosition { x: 50, y: 50 }),
            primary_renderer,
        );
        let mut window_manager = WindowManager::new(primary_window, primary_window_name);

        event_loop.run(move |event, event_loop_window_target| {
            if world.input.update(&event) {
                if world.input.key_pressed(KeyCode::F12) {
                    world.nes.reset();
                }

                if world.input.key_pressed(KeyCode::Pause)
                    || world.input.key_pressed(KeyCode::KeyP)
                    || world.input.key_pressed(KeyCode::Escape)
                {
                    window_manager.toggle_pause();
                }

                window_manager.request_redraws();
            }

            if let Event::WindowEvent { event, window_id } = event {
                match event {
                    WindowEvent::CloseRequested => {
                        let primary_removed = window_manager.remove_window(&window_id);
                        if primary_removed {
                            log::logger().flush();
                            event_loop_window_target.exit();
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        match window_manager.draw(&mut world, &window_id) {
                            Ok(FlowControl { window_args, should_close_window }) => {
                                if let Some((renderer, position, scale)) = window_args {
                                    window_manager.create_window_from_renderer(
                                        event_loop_window_target,
                                        renderer,
                                        position,
                                        scale,
                                    );
                                }

                                if should_close_window {
                                    window_manager.remove_window(&window_id);
                                }
                            }
                            Err(e) => {
                                if window_id == window_manager.primary_window_id {
                                    info!("Closing REZNEZ due to redraw failure. {e}");
                                    event_loop_window_target.exit();
                                }
                            }
                        }
                    }
                    _ => {
                        if let Some(window) = window_manager.window_mut(&window_id) {
                            window.handle_event(&event);
                        }
                    }
                }
            }
        }).unwrap();
    }
}

type WindowArgs = (Box<dyn Renderer>, Position, u64);

/// Manages all state required for rendering egui over `Pixels`.
struct EguiWindow<'a> {
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    rpass: RenderPass,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,

    // State for the GUI
    window: Arc<Window>,
    pixels: Pixels<'a>,
    renderer: Box<dyn Renderer>,
}

impl <'a> EguiWindow<'a> {
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
        let window = Arc::new(window);
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.clone());
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
            window.clone(),
            pixels,
            renderer,
        )
    }

    fn new(
        width: u32,
        height: u32,
        scale_factor: f32,
        window: Arc<Window>,
        pixels: pixels::Pixels<'a>,
        renderer: Box<dyn Renderer>,
    ) -> Self {
        let egui_ctx = Context::default();
        let egui_state = egui_winit::State::new(egui_ctx, ViewportId::ROOT, &window, None, None);
        let screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
        };
        let rpass = RenderPass::new(pixels.device(), pixels.render_texture_format(), 1);
        let textures = TexturesDelta::default();

        Self {
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
        let _event_response = self.egui_state.on_window_event(&self.window, event);
    }

    fn draw(&mut self, world: &mut World) -> Result<FlowControl, String> {
        self.renderer.render(world, &mut self.pixels);

        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let mut result = FlowControl::CONTINUE;
        let output = self.egui_state.egui_ctx().run(raw_input, |egui_ctx| {
            result = self.renderer.ui(egui_ctx, world);
        });

        self.textures.append(output.textures_delta);
        self.egui_state.handle_platform_output(
            &self.window,
            output.platform_output,
        );
        self.paint_jobs = self.egui_state.egui_ctx().tessellate(output.shapes, 1.0);

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

struct FlowControl {
    window_args: Option<WindowArgs>,
    should_close_window: bool,
}

impl FlowControl {
    const CONTINUE: Self = Self {
        window_args: None,
        should_close_window: false,
    };
    const CLOSE: Self = Self {
        window_args: None,
        should_close_window: true,
    };

    fn spawn_window(window_args: WindowArgs) -> Self {
        Self {
            window_args: Some(window_args),
            should_close_window: false,
        }
    }
}

struct World {
    nes: Nes,
    config: Config,
    input: WinitInputHelper,
    gilrs: gilrs::Gilrs,
    active_gamepad_id: Option<gilrs::GamepadId>,
}

struct WindowManager<'a> {
    primary_window_id: WindowId,
    windows_by_id: BTreeMap<WindowId, (String, EguiWindow<'a>)>,
    window_names: BTreeSet<String>,
}

impl <'a> WindowManager<'a> {
    pub fn new(primary_window: EguiWindow<'a>, name: String) -> WindowManager<'a> {
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
        if let Some((name, _)) = self.windows_by_id.remove(window_id) {
            self.window_names.remove(&name);
            let primary_removed = *window_id == self.primary_window_id;
            return primary_removed;
        }

        false
    }

    pub fn toggle_pause(&mut self) {
        self.windows_by_id.get_mut(&self.primary_window_id).unwrap().1.renderer.toggle_pause();
    }

    pub fn request_redraws(&self) {
        for (_id, window) in self.windows_by_id.values() {
            window.window.request_redraw();
        }
    }

    pub fn draw(&mut self, world: &mut World, window_id: &WindowId) -> Result<FlowControl, String> {
        let window = self.window_mut(window_id).ok_or("Failed to create window")?;
        window.draw(world)
    }

    pub fn window_mut(&mut self, window_id: &WindowId) -> Option<&mut EguiWindow<'a>> {
        self.windows_by_id
            .get_mut(window_id)
            .map(|(_, window)| window)
    }
}

trait Renderer {
    fn name(&self) -> String;
    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl;
    fn render(&mut self, world: &mut World, pixels: &mut Pixels);
    fn toggle_pause(&mut self) {}
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

struct PrimaryRenderer {
    paused: bool,
}

impl PrimaryRenderer {
    fn new() -> Self {
        Self {
            paused: false,
        }
    }
}

impl Renderer for PrimaryRenderer {
    fn name(&self) -> String {
        "REZNEZ".to_string()
    }

    fn ui(&mut self, ctx: &Context, _world: &mut World) -> FlowControl {
        let mut result = FlowControl::CONTINUE;
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        ui.close_menu();
                        let mut file_dialog = egui_file::FileDialog::open_file(None);
                        file_dialog.open();
                        result = FlowControl::spawn_window((
                            Box::new(LoadRomRenderer::new(file_dialog)) as Box<dyn Renderer>,
                            Position::Physical(PhysicalPosition { x: 850, y: 360 }),
                            2,
                        ));
                    }
                });

                ui.menu_button("Settings", |ui| {
                    if ui.button("Display").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(DisplaySettingsRenderer::new()) as Box<dyn Renderer>,
                            Position::Physical(PhysicalPosition { x: 850, y: 360 }),
                            2,
                        ));
                    }
                });

                ui.menu_button("Debug Windows", |ui| {
                    if ui.button("Status").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(StatusRenderer) as Box<dyn Renderer>,
                            Position::Physical(PhysicalPosition { x: 850, y: 360 }),
                            2,
                        ));
                    }
                    if ui.button("Layers").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(LayersRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 850, y: 50 }),
                            1,
                        ));
                    }
                    if ui.button("Name Tables").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(NameTableRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 1400, y: 50 }),
                            1,
                        ));
                    }
                    if ui.button("Sprites").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(SpritesRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 1400, y: 660 }),
                            6,
                        ));
                    }
                    if ui.button("Pattern Tables").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(PatternTableRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 850, y: 660 }),
                            3,
                        ));
                    }
                    if ui.button("CHR Banks").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(ChrBanksRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 50, y: 50 }),
                            2,
                        ));
                    }
                    if ui.button("Pattern Sources").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(PatternSourceRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 600, y: 200 }),
                            1,
                        ));
                    }
                    if ui.button("Memory Viewer").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(MemoryViewerRenderer),
                            Position::Physical(PhysicalPosition { x: 600, y: 200 }),
                            1,
                        ));
                    }
                    if ui.button("Cartridge Metadata").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(CartridgeMetadataRenderer),
                            Position::Physical(PhysicalPosition { x: 600, y: 200 }),
                            2,
                        ));
                    }
                })
            });
        });

        result
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        if self.paused {
            return;
        }

        let display_frame = |frame: &Frame, mask, _frame_index| {
            frame.copy_to_rgba_buffer(mask, pixels.frame_mut().try_into().unwrap());
        };
        execute_frame(
            &mut world.nes,
            &world.config,
            &events(&world.input, &mut world.gilrs, world.active_gamepad_id),
            display_frame,
        );
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    fn width(&self) -> usize {
        PixelColumn::COLUMN_COUNT
    }

    fn height(&self) -> usize {
        PixelRow::ROW_COUNT
    }
}

struct LoadRomRenderer {
    file_dialog: FileDialog,
}

impl LoadRomRenderer {
    const WIDTH: usize = 300;
    const HEIGHT: usize = 300;

    pub fn new(file_dialog: FileDialog) -> Self {
        Self {
            file_dialog,
        }
    }
}

impl Renderer for LoadRomRenderer {
    fn name(&self) -> String {
        "Load ROM".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl {
        let mut result = FlowControl::CONTINUE;
        egui::CentralPanel::default().show(ctx, |_ui| {
            self.file_dialog.show(ctx);
            if let Some(rom_path) = self.file_dialog.path() && !rom_path.is_dir() {
                world.nes.load_new_config(&Config::with_new_rom(&world.config, rom_path));
                result = FlowControl::CLOSE;
            }
        });

        result
    }

    fn render(&mut self, _world: &mut World, _pixels: &mut Pixels) {
        // Do nothing yet.
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}

struct DisplaySettingsRenderer;

impl DisplaySettingsRenderer {
    const WIDTH: usize = 300;
    const HEIGHT: usize = 300;

    pub fn new() -> Self {
        Self
    }
}

impl Renderer for DisplaySettingsRenderer {
    fn name(&self) -> String {
        "Display Settings".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl {
        let nes = &mut world.nes;
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.checkbox(nes.frame_mut().show_overscan_mut(), "Show overscan?");
                    ui.end_row();
                });
        });

        FlowControl::CONTINUE
    }

    fn render(&mut self, _world: &mut World, _pixels: &mut Pixels) {
        // Do nothing yet.
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}

struct StatusRenderer;

impl StatusRenderer {
    const WIDTH: usize = 300;
    const HEIGHT: usize = 300;
}

impl Renderer for StatusRenderer {
    fn name(&self) -> String {
        "Status".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl {
        let nes = &world.nes;
        let clock = nes.memory().ppu_regs().clock();
        let ppu_regs = nes.memory().ppu_regs();
        let mapper_params = nes.memory().mapper_params();

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
                    ui.label(format!("{}", ppu_regs.x_scroll().to_u8()));
                    ui.end_row();
                    ui.label("Y Scroll");
                    ui.label(format!("{}", ppu_regs.y_scroll().to_u8()));
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
                    ui.label(format!("{:?}", nes.memory().ppu_regs().active_name_table_quadrant()));
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
                    ui.label(format!("{:?}", nes.resolved_metadata().mapper_number));
                    ui.end_row();
                    ui.label("Name Table Mirroring");
                    ui.label(format!("{}", mapper_params.name_table_mirroring()));
                    ui.end_row();
                    ui.label("PRG ROM banks");
                    ui.label(nes.mapper().prg_rom_bank_string(mapper_params));
                    ui.end_row();
                    ui.label("CHR ROM banks");
                    ui.label(nes.mapper().chr_rom_bank_string(mapper_params));
                });
        });

        FlowControl::CONTINUE
    }

    fn render(&mut self, _world: &mut World, _pixels: &mut Pixels) {
        // Do nothing yet.
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
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

    fn ui(&mut self, _ctx: &Context, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        self.buffer
            .place_frame(0, TOP_MENU_BAR_HEIGHT, world.nes.frame());
        self.buffer.place_frame(
            261,
            TOP_MENU_BAR_HEIGHT,
            &world.nes.frame().to_background_only(),
        );

        let (_, mem) = world.nes.ppu_and_memory_mut();

        self.frame.clear();
        mem.oam().only_front_sprites().render(mem, &mut self.frame);
        self.buffer
            .place_frame(0, 245 + TOP_MENU_BAR_HEIGHT, &self.frame);

        self.frame.clear();
        mem.oam().only_back_sprites().render(mem, &mut self.frame);
        self.buffer
            .place_frame(261, 245 + TOP_MENU_BAR_HEIGHT, &self.frame);

        self.buffer.copy_to_rgba_buffer(pixels.frame_mut());
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
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

    fn ui(&mut self, _ctx: &Context, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    #[rustfmt::skip]
    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let x = usize::from(world.nes.memory().ppu_regs().x_scroll().to_u8());
        let y = usize::from(world.nes.memory().ppu_regs().y_scroll().to_u8());
        let mapper = &*world.nes.mapper;
        let mem = &mut world.nes.memory;

        let width = NameTableRenderer::WIDTH;
        let height = NameTableRenderer::HEIGHT;
        // Clear any junk out of the outer border.
        self.buffer.place_wrapping_horizontal_line(0, 0, width, Rgb::new(255, 255, 255));
        self.buffer.place_wrapping_horizontal_line(height, 0, width, Rgb::new(255, 255, 255));
        self.buffer.place_wrapping_vertical_line(0, 0, height, Rgb::new(255, 255, 255));
        self.buffer.place_wrapping_vertical_line(width, 0, height, Rgb::new(255, 255, 255));

        self.frame.set_universal_background_rgb(mem.palette_table().universal_background_rgb());
        let background_table = PatternTable::background_side(mem);
        NameTable::from_mem(mapper, mem, NameTableQuadrant::TopLeft)
            .render(&background_table, &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(1, 1, &self.frame);
        NameTable::from_mem(mapper, mem, NameTableQuadrant::TopRight)
            .render(&background_table, &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(257, 1, &self.frame);
        NameTable::from_mem(mapper, mem, NameTableQuadrant::BottomLeft)
            .render(&background_table, &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(1, 241, &self.frame);
        NameTable::from_mem(mapper, mem, NameTableQuadrant::BottomRight)
            .render(&background_table, &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(257, 241, &self.frame);

        self.buffer.place_wrapping_horizontal_line(y, x, x + 257, Rgb::new(255, 0, 0));
        self.buffer.place_wrapping_horizontal_line(y + 241, x, x + 257, Rgb::new(255, 0, 0));
        self.buffer.place_wrapping_vertical_line(x, y, y + 241, Rgb::new(255, 0, 0));
        self.buffer.place_wrapping_vertical_line(x + 257, y, y + 241, Rgb::new(255, 0, 0));

        self.buffer.copy_to_rgba_buffer(pixels.frame_mut());
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
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

    fn ui(&mut self, _ctx: &Context, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let sprites = world.nes.memory_mut().oam().sprites();
        let mem = world.nes.memory_mut();

        for (index, sprite) in sprites.iter().enumerate() {
            let tile = sprite.render_normal_height(&PatternTable::sprite_side(mem), &mem.palette_table());
            self.buffer.place_tile(
                (8 + 1) * (index % 8),
                (8 + 1) * (index / 8),
                &tile,
            );
        }

        self.buffer.copy_to_rgba_buffer(pixels.frame_mut());
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
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

    fn ui(&mut self, _ctx: &Context, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let mem = world.nes.memory_mut();

        let mut offset = 0;
        for side in [PatternTableSide::Left, PatternTableSide::Right] {
            let palette = if mem.ppu_regs.sprite_table_side() == side {
                mem.palette_table().sprite_palette(PaletteTableIndex::Zero)
            } else {
                mem.palette_table()
                    .background_palette(PaletteTableIndex::Zero)
            };
            for index in 0..=255 {
                PatternTable::from_mem(mem, side).render_background_tile(
                    TileNumber::new(index),
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

        self.buffer.copy_to_rgba_buffer(pixels.frame_mut());
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}

struct ChrBanksRenderer {
    //tile: Tile,
    buffer: DebugBuffer<{ ChrBanksRenderer::WIDTH }, { ChrBanksRenderer::HEIGHT }>,
}

impl ChrBanksRenderer {
    const WIDTH: usize = (8 + 1) * 256;
    const HEIGHT: usize = (8 + 1) * 32;

    fn new() -> ChrBanksRenderer {
        ChrBanksRenderer {
            //tile: Tile::new(),
            buffer: DebugBuffer::new(Rgb::WHITE),
        }
    }
}

impl Renderer for ChrBanksRenderer {
    fn name(&self) -> String {
        "CHR Banks".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    fn render(&mut self, _world: &mut World, pixels: &mut Pixels) {
        // TODO: See if this can be re-enabled or if it isn't generalizable (most likely).
        /*
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
        */

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

        self.buffer.copy_to_rgba_buffer(pixels.frame_mut());
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}

struct PatternSourceRenderer {
    buffer: DebugBuffer<{ PixelColumn::COLUMN_COUNT }, { PixelRow::ROW_COUNT }>,
}

impl PatternSourceRenderer {
    fn new() -> Self {
        Self {
            buffer: DebugBuffer::new(Rgb::WHITE),
        }
    }
}

impl Renderer for PatternSourceRenderer {
    fn name(&self) -> String {
        "Pattern Source".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        self.buffer.place_frame(0, 0, world.nes.ppu().pattern_source_frame());
        self.buffer.copy_to_rgba_buffer(pixels.frame_mut());
    }

    fn width(&self) -> usize {
        PixelColumn::COLUMN_COUNT
    }

    fn height(&self) -> usize {
        PixelRow::ROW_COUNT
    }
}

struct MemoryViewerRenderer;

impl MemoryViewerRenderer {
    const WIDTH: usize = 700;
    const HEIGHT: usize = 400;
}

impl Renderer for MemoryViewerRenderer {
    fn name(&self) -> String {
        "Memory Viewer".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl {
        let nes = &mut world.nes;
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("my_grid")
                    .num_columns(16)
                    .spacing([0.0, 0.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for mem_index in 0..=u16::MAX {
                            let value = nes.mapper().cpu_peek(nes.memory(), CpuAddress::new(mem_index)).resolve(nes.memory().cpu_data_bus);
                            let _ = ui.button(format!("{value:02X}"));
                            if mem_index % 0x10 == 0x0F {
                                ui.end_row();
                            }
                        }
                    });
            })
        });

        FlowControl::CONTINUE
    }

    fn render(&mut self, _world: &mut World, _pixels: &mut Pixels) {
        // Do nothing yet.
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}
struct CartridgeMetadataRenderer;

impl CartridgeMetadataRenderer {
    const WIDTH: usize = 550;
    const HEIGHT: usize = 300;
}

impl Renderer for CartridgeMetadataRenderer {
    fn name(&self) -> String {
        "Status".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl {
        let nes = &world.nes;
        let resolver = nes.metadata_resolver();
        let final_values = resolver.resolve();
        let metadata_sources = [
            &resolver.hard_coded_overrides,
            &resolver.cartridge,
            &resolver.mapper,
            &resolver.database,
            &resolver.database_extension,
            &resolver.defaults(),
        ];

        egui::CentralPanel::default().show(ctx, |ui| {
            fn kib_string(value: u32) -> String {
                if value < KIBIBYTE {
                    value.to_string()
                } else {
                    assert_eq!(value % KIBIBYTE, 0);
                    format!("{}KiB", value / KIBIBYTE)
                }
            }

            egui::Grid::new("my_grid")
                .num_columns(7)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Field");
                    ui.label("ACTUAL");
                    ui.label("Hard-coded Overrides");
                    ui.label("Cartridge");
                    ui.label("Mapper");
                    ui.label("Header Database");
                    ui.label("Database Extension");
                    ui.label("Defaults");
                    ui.end_row();

                    ui.label("Mapper");
                    ui.label(final_values.mapper_number.to_string());
                    for metadata in metadata_sources {
                        ui.label(metadata.mapper_number().map(|m| m.to_string()).unwrap_or("".to_owned()));
                    }
                    ui.end_row();

                    ui.label("Submapper");
                    ui.label(final_values.submapper_number.map(|s| s.to_string()).unwrap_or("N/A".to_owned()));
                    for metadata in metadata_sources {
                        ui.label(metadata.submapper_number().map(|m| m.to_string()).unwrap_or("".to_owned()));
                    }
                    ui.end_row();

                    ui.label("Name Table Mirroring");
                    ui.label(final_values.name_table_mirroring.to_string());
                    for metadata in metadata_sources {
                        ui.label(metadata.name_table_mirroring().map(|m| m.to_string()).unwrap_or("".to_owned()));
                    }
                    ui.end_row();

                    ui.label("Has Persistent Memory");
                    ui.label(final_values.has_persistent_memory.to_string());
                    for metadata in metadata_sources {
                        ui.label(metadata.has_persistent_memory().map(|m| m.to_string()).unwrap_or("".to_owned()));
                    }
                    ui.end_row();

                    ui.label("Console Type");
                    ui.label(final_values.console_type.to_string());
                    for metadata in metadata_sources {
                        ui.label(metadata.console_type().map(|m| m.to_string()).unwrap_or("".to_owned()));
                    }
                    ui.end_row();

                    ui.label("PRG ROM Size");
                    ui.label(kib_string(final_values.prg_rom_size));
                    for metadata in metadata_sources {
                        ui.label(metadata.prg_rom_size().map(kib_string).unwrap_or("".to_owned()));
                    }
                    ui.end_row();

                    ui.label("PRG Work RAM Size");
                    ui.label(kib_string(final_values.prg_work_ram_size));
                    for metadata in metadata_sources {
                        ui.label(metadata.prg_work_ram_size().map(kib_string).unwrap_or("".to_owned()));
                    }
                    ui.end_row();

                    ui.label("PRG Save RAM Size");
                    ui.label(kib_string(final_values.prg_save_ram_size));
                    for metadata in metadata_sources {
                        ui.label(metadata.prg_save_ram_size().map(kib_string).unwrap_or("".to_owned()));
                    }
                    ui.end_row();

                    ui.label("CHR ROM Size");
                    ui.label(kib_string(final_values.chr_rom_size));
                    for metadata in metadata_sources {
                        ui.label(metadata.chr_rom_size().map(kib_string).unwrap_or("".to_owned()));
                    }
                    ui.end_row();

                    ui.label("CHR Work RAM Size");
                    ui.label(kib_string(final_values.chr_work_ram_size));
                    for metadata in metadata_sources {
                        ui.label(metadata.chr_work_ram_size().map(kib_string).unwrap_or("".to_owned()));
                    }
                    ui.end_row();

                    ui.label("CHR Save RAM Size");
                    ui.label(kib_string(final_values.chr_save_ram_size));
                    for metadata in metadata_sources {
                        ui.label(metadata.chr_save_ram_size().map(kib_string).unwrap_or("".to_owned()));
                    }
                    ui.end_row();
                });
        });

        FlowControl::CONTINUE
    }

    fn render(&mut self, _world: &mut World, _pixels: &mut Pixels) {
        // Do nothing yet.
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}

fn events(input: &WinitInputHelper, gilrs: &mut gilrs::Gilrs, active_gamepad_id: Option<GamepadId>) -> Events {
    let mut joypad1_button_statuses = BTreeMap::new();
    let mut joypad2_button_statuses = BTreeMap::new();

    while let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
        assert_eq!(Some(id), active_gamepad_id);
        match event {
            gilrs::EventType::ButtonPressed(_, code) => {
                if let Some(button) = JOY_1_JOYPAD_MAPPINGS.get(&code.into_u32()) {
                    joypad1_button_statuses.insert(*button, ButtonStatus::Pressed);
                }
            }
            gilrs::EventType::ButtonReleased(_, code) => {
                if let Some(button) = JOY_1_JOYPAD_MAPPINGS.get(&code.into_u32()) {
                    joypad1_button_statuses.insert(*button, ButtonStatus::Unpressed);
                }
            }
            _ => {}
        }
    }

    for (&key, &button) in JOY_1_KEYBOARD_MAPPINGS.iter() {
        if input.key_pressed(key) {
            joypad1_button_statuses.insert(button, ButtonStatus::Pressed);
        } else if input.key_released(key) {
            joypad1_button_statuses.insert(button, ButtonStatus::Unpressed);
        };
    }

    for (&key, &button) in JOY_2_KEYBOARD_MAPPINGS.iter() {
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

/*
fn button_pressed(key: input: &WinitInputHelper, gilrs: gilrs::Gilrs, active_gamepad_id: Option<GamepadId>) -> bool {

}
*/