use std::collections::{BTreeMap, HashMap};

use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::{LogicalSize, Position, PhysicalPosition};
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use egui::{ClippedMesh, Context, TexturesDelta};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use lazy_static::lazy_static;
use winit::window::Window;

use crate::config::Config;
use crate::controller::joypad::{Button, ButtonStatus};
use crate::gui::gui::{execute_frame, Gui, Events};
use crate::nes::Nes;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::pattern_table::{PatternIndex, Tile, PatternTableSide};
use crate::ppu::pixel_index::{PixelColumn, PixelRow};
use crate::ppu::render::frame::{Frame, DebugBuffer};
use crate::ppu::name_table::name_table_number::NameTableNumber;

const TOP_MENU_BAR_HEIGHT: usize = 24;

lazy_static! {
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
        let mut world = World {nes, config, input};

        let event_loop = EventLoop::new();
        let primary_window = EguiWindow::from_event_loop(
            &event_loop,
            3,
            "REZNEZ",
            Some(Position::Physical(PhysicalPosition {x: 50, y: 50})),
            Box::new(PrimaryPreRender),
        );
        let layers_window = EguiWindow::from_event_loop(
            &event_loop,
            1,
            "Layers",
            Some(Position::Physical(PhysicalPosition {x: 850, y: 50})),
            Box::new(LayersPreRender::new()),
        );
        let name_table_window = EguiWindow::from_event_loop(
            &event_loop,
            1,
            "Name Tables",
            Some(Position::Physical(PhysicalPosition {x: 1400, y: 50})),
            Box::new(NameTablePreRender::new()),
        );
        let pattern_table_window = EguiWindow::from_event_loop(
            &event_loop,
            3,
            "Pattern Tables",
            Some(Position::Physical(PhysicalPosition {x: 850, y: 660})),
            Box::new(PatternTablePreRender::new()),
        );

        let mut windows = BTreeMap::new();
        windows.insert(primary_window.window.id(), primary_window);
        windows.insert(layers_window.window.id(), layers_window);
        windows.insert(name_table_window.window.id(), name_table_window);
        windows.insert(pattern_table_window.window.id(), pattern_table_window);

        event_loop.run(move |event, _, control_flow| {
            if world.input.update(&event) {
                if world.input.key_pressed(VirtualKeyCode::Escape) || world.input.quit() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                for (_id, window) in windows.iter() {
                    window.window.request_redraw();
                }
            }

            match event {
                Event::WindowEvent {event, window_id} => {
                    let window = windows.get_mut(&window_id).unwrap();
                    window.handle_event(&event);
                }
                Event::RedrawRequested(window_id) => {
                    let window = windows.get_mut(&window_id).unwrap();
                    let render_result = window.draw(&mut world);
                    if render_result
                        .map_err(|e| error!("pixels.render() failed: {}", e))
                        .is_err()
                    {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _ => (),
            }
        });
    }
}

/// Manages all state required for rendering egui over `Pixels`.
struct EguiWindow {
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    rpass: RenderPass,
    paint_jobs: Vec<ClippedMesh>,
    textures: TexturesDelta,

    // State for the GUI
    app: App,
    window: Window,
    pixels: Pixels,
    pre_render: Box<dyn PreRender>,
}

impl EguiWindow {
    fn from_event_loop(
        event_loop: &EventLoop<()>,
        scale_factor: u64,
        title: &str,
        initial_position: Option<Position>,
        pre_render: Box<dyn PreRender>,
    ) -> Self {
        let window = {
            let size = LogicalSize::new(
                scale_factor as f64 * pre_render.width() as f64,
                scale_factor as f64 * pre_render.height() as f64,
            );
            let mut builder = WindowBuilder::new()
                .with_title(title)
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_resizable(false);

            if let Some(initial_position) = initial_position {
                builder = builder.with_position(initial_position);
            }

            builder.build(event_loop).unwrap()
        };

        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(
            pre_render.width() as u32,
            pre_render.height() as u32,
            surface_texture,
        ).unwrap();

        EguiWindow::new(window_size.width, window_size.height, scale_factor, window, pixels, pre_render)
    }

    fn new(width: u32, height: u32, scale_factor: f32, window: Window, pixels: pixels::Pixels, pre_render: Box<dyn PreRender>) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = Context::default();
        let egui_state = egui_winit::State::from_pixels_per_point(max_texture_size, scale_factor);
        let screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
        };
        let rpass = RenderPass::new(pixels.device(), pixels.render_texture_format(), 1);
        let textures = TexturesDelta::default();
        let app = App::new();

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            rpass,
            paint_jobs: Vec::new(),
            textures,
            app,
            window,
            pixels,
            pre_render,
        }
    }

    /// Handle input events from the window manager.
    fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        self.egui_state.on_event(&self.egui_ctx, event);
    }

    fn draw(&mut self, world: &mut World) -> Result<(), String> {
        self.pre_render.pre_render(world, &mut self.pixels);

        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            self.app.ui(egui_ctx);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(&self.window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);

        self.pixels.render_with(|encoder, render_target, context| {
            context.scaling_renderer.render(encoder, render_target);
            self.rpass
                .add_textures(&context.device, &context.queue, &self.textures).map_err(|err| err.to_string())?;
            self.rpass.update_buffers(
                &context.device,
                &context.queue,
                &self.paint_jobs,
                &self.screen_descriptor,
            );

            // Record all render passes.
            self.rpass.execute(
                encoder,
                render_target,
                &self.paint_jobs,
                &self.screen_descriptor,
                None,
            ).map_err(|err| err.to_string())?;

            // Cleanup
            let textures = std::mem::take(&mut self.textures);
            Ok(self.rpass.remove_textures(textures).map_err(|err| err.to_string())?)
        }).map_err(|err| err.to_string())
    }
}

struct App {
    /// Only show the egui window when true.
    window_open: bool,
}

impl App {
    fn new() -> Self {
        Self {window_open: false}
    }

    fn ui(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("About...").clicked() {
                        self.window_open = true;
                        ui.close_menu();
                    }
                })
            });
        });
    }
}

struct World {
    nes: Nes,
    config: Config,
    input: WinitInputHelper,
}

trait PreRender {
    fn pre_render(&mut self, world: &mut World, pixels: &mut Pixels);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

struct PrimaryPreRender;

impl PreRender for PrimaryPreRender {
    fn pre_render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let display_frame = |frame: &Frame, mask, _frame_index| {
            frame.copy_to_rgba_buffer(mask, pixels.get_frame().try_into().unwrap());
        };
        execute_frame(&mut world.nes, &world.config, events(&world.input), display_frame);
    }

    fn width(&self) -> usize {
        PixelColumn::COLUMN_COUNT
    }

    fn height(&self) -> usize {
        PixelRow::ROW_COUNT
    }
}

struct LayersPreRender {
    frame: Frame,
    buffer: DebugBuffer<{LayersPreRender::WIDTH}, {LayersPreRender::HEIGHT}>,
}

impl LayersPreRender {
    const WIDTH: usize = 517;
    const HEIGHT: usize = 485 + TOP_MENU_BAR_HEIGHT;

    fn new() -> LayersPreRender {
        LayersPreRender {
            frame: Frame::new(),
            buffer: DebugBuffer::filled(Rgb::WHITE),
        }
    }
}

impl PreRender for LayersPreRender {
    fn pre_render(&mut self, world: &mut World, pixels: &mut Pixels) {
        self.buffer.place_frame(0, TOP_MENU_BAR_HEIGHT, world.nes.frame());
        self.buffer.place_frame(261, TOP_MENU_BAR_HEIGHT, &world.nes.frame().to_background_only());

        let (ppu, mem) = world.nes.ppu_and_memory_mut();
        let mem = mem.as_ppu_memory();

        self.frame.clear();
        ppu.oam().only_front_sprites().render(&mem, &mut self.frame);
        self.buffer.place_frame(0, 245 + TOP_MENU_BAR_HEIGHT, &self.frame);

        self.frame.clear();
        ppu.oam().only_back_sprites().render(&mem, &mut self.frame);
        self.buffer.place_frame(261, 245 + TOP_MENU_BAR_HEIGHT, &self.frame);

        self.buffer.copy_to_rgba_buffer(pixels.get_frame().try_into().unwrap());
    }

    fn width(&self) -> usize {
        LayersPreRender::WIDTH
    }

    fn height(&self) -> usize {
        LayersPreRender::HEIGHT
    }
}

struct NameTablePreRender {
    frame: Frame,
    buffer: DebugBuffer<{NameTablePreRender::WIDTH}, {NameTablePreRender::HEIGHT}>,
}

impl NameTablePreRender {
    const WIDTH: usize = 517;
    const HEIGHT: usize = 485 + TOP_MENU_BAR_HEIGHT;

    fn new() -> NameTablePreRender {
        NameTablePreRender {
            frame: Frame::new(),
            buffer: DebugBuffer::filled(Rgb::WHITE),
        }
    }
}

impl PreRender for NameTablePreRender {
    fn pre_render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let mem = world
            .nes
            .memory_mut()
            .as_ppu_memory();

        self.frame.set_universal_background_rgb(mem.palette_table().universal_background_rgb());
        mem.name_table(NameTableNumber::Zero)
            .render(&mem.background_pattern_table(), &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(0, TOP_MENU_BAR_HEIGHT, &self.frame);
        mem.name_table(NameTableNumber::One)
            .render(&mem.background_pattern_table(), &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(261, TOP_MENU_BAR_HEIGHT, &self.frame);
        mem.name_table(NameTableNumber::Two)
            .render(&mem.background_pattern_table(), &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(0, 245 + TOP_MENU_BAR_HEIGHT, &self.frame);
        mem.name_table(NameTableNumber::Three)
            .render(&mem.background_pattern_table(), &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(261, 245 + TOP_MENU_BAR_HEIGHT, &self.frame);
        self.buffer.copy_to_rgba_buffer(pixels.get_frame().try_into().unwrap());
    }

    fn width(&self) -> usize {
        NameTablePreRender::WIDTH
    }

    fn height(&self) -> usize {
        NameTablePreRender::HEIGHT
    }
}

struct PatternTablePreRender {
    tile: Tile,
    buffer: DebugBuffer<{PatternTablePreRender::WIDTH}, {PatternTablePreRender::HEIGHT}>,
}

impl PatternTablePreRender {
    const WIDTH: usize = 2 * (8 + 1) * 16 + 10;
    const HEIGHT: usize = (8 + 1) * 16 + TOP_MENU_BAR_HEIGHT / 3;

    fn new() -> PatternTablePreRender {
        PatternTablePreRender {
            tile: Tile::new(),
            buffer: DebugBuffer::filled(Rgb::WHITE),
        }
    }
}

impl PreRender for PatternTablePreRender {
    fn pre_render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let mem = world
            .nes
            .memory_mut()
            .as_ppu_memory();

        let mut offset = 0;
        for side in [PatternTableSide::Left, PatternTableSide::Right] {
            for index in 0..=255 {
                let palette = if mem.regs().sprite_table_side() == side {
                    mem.palette_table().sprite_palette(PaletteTableIndex::Zero)
                } else {
                    mem.palette_table().background_palette(PaletteTableIndex::Zero)
                };

                mem.pattern_table(side).render_background_tile(
                    PatternIndex::new(index), palette, &mut self.tile);
                self.buffer.place_tile(
                    (8 + 1) * (index as usize % 16) + offset,
                    (8 + 1) * (index as usize / 16) + TOP_MENU_BAR_HEIGHT / 3,
                    &self.tile,
                );
            }

            offset += (8 + 1) * 16 + 10;
        }

        self.buffer.copy_to_rgba_buffer(pixels.get_frame().try_into().unwrap());
    }

    fn width(&self) -> usize {
        PatternTablePreRender::WIDTH
    }

    fn height(&self) -> usize {
        PatternTablePreRender::HEIGHT
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