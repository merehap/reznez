use std::collections::{BTreeMap, HashMap};

use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
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
use crate::ppu::render::frame::Frame;
use crate::ppu::pixel_index::{PixelColumn, PixelRow};

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
            PixelColumn::COLUMN_COUNT,
            PixelRow::ROW_COUNT,
            3,
            "REZNEZ",
            Box::new(PrimaryPreRender),
        );

        let mut windows = BTreeMap::new();
        windows.insert(primary_window.window.id(), primary_window);

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
        width: usize,
        height: usize,
        scale_factor: u64,
        title: &str,
        pre_render: Box<dyn PreRender>,
    ) -> Self {
        let window = {
            let size = LogicalSize::new(
                scale_factor as f64 * width as f64,
                scale_factor as f64 * height as f64,
            );
            WindowBuilder::new()
                .with_title(title)
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_resizable(false)
                .build(event_loop)
                .unwrap()
        };

        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(
            width as u32,
            height as u32,
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
    fn pre_render(&self, world: &mut World, pixels: &mut Pixels);
}

struct PrimaryPreRender;

impl PreRender for PrimaryPreRender {
    fn pre_render(&self, world: &mut World, pixels: &mut Pixels) {
        let display_frame = |frame: &Frame, mask, _frame_index| {
            frame.copy_to_rgba_buffer(mask, pixels.get_frame().try_into().unwrap());
        };
        execute_frame(&mut world.nes, &world.config, events(&world.input), display_frame);
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
