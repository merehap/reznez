use std::collections::BTreeMap;

use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use egui::{ClippedMesh, Context, TexturesDelta};
use egui_wgpu_backend::{BackendError, RenderPass, ScreenDescriptor};
use pixels::{wgpu, PixelsContext};
use winit::window::Window;

use crate::config::Config;
use crate::gui::gui::{execute_frame, Gui, Events};
use crate::nes::Nes;
use crate::ppu::render::frame::Frame;
use crate::ppu::pixel_index::{PixelColumn, PixelRow};

/*
use std::collections::{BTreeMap, HashMap};
use lazy_static::lazy_static;
lazy_static! {
    static ref JOY_1_BUTTON_MAPPINGS: HashMap<KeyCode, Button> = {
        let mut mappings = HashMap::new();
        mappings.insert(KeyCode::Space,  Button::A);
        mappings.insert(KeyCode::F,      Button::B);
        mappings.insert(KeyCode::RShift, Button::Select);
        mappings.insert(KeyCode::Return, Button::Start);
        mappings.insert(KeyCode::Up,     Button::Up);
        mappings.insert(KeyCode::Down,   Button::Down);
        mappings.insert(KeyCode::Left,   Button::Left);
        mappings.insert(KeyCode::Right,  Button::Right);
        mappings
    };

    static ref JOY_2_BUTTON_MAPPINGS: HashMap<KeyCode, Button> = {
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
    };
}
*/

pub struct EguiGui;

impl EguiGui {
    pub fn new() -> EguiGui {
        EguiGui
    }
}

impl Gui for EguiGui {
    fn run(&mut self, mut nes: Nes, config: Config) {
        let event_loop = EventLoop::new();
        let mut input = WinitInputHelper::new();
        let window = {
            let size = LogicalSize::new(
                3.0 * PixelColumn::COLUMN_COUNT as f64,
                3.0 * PixelRow::ROW_COUNT as f64,
            );
            WindowBuilder::new()
                .with_title("Hello Pixels + egui")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_resizable(false)
                .build(&event_loop)
                .unwrap()
        };

        let (mut pixels, mut egui) = {
            let window_size = window.inner_size();
            let scale_factor = window.scale_factor() as f32;
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
            let pixels = Pixels::new(
                PixelColumn::COLUMN_COUNT as u32,
                PixelRow::ROW_COUNT as u32,
                surface_texture,
            ).unwrap();
            let egui =
                Egui::new(window_size.width, window_size.height, scale_factor, &pixels);

            (pixels, egui)
        };

        event_loop.run(move |event, _, control_flow| {
            // Handle input events
            if input.update(&event) {
                // Close events
                if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                // Update internal state and request a redraw
                window.request_redraw();
            }

            match event {
                Event::WindowEvent { event, .. } => {
                    egui.handle_event(&event);
                }
                Event::RedrawRequested(_) => {
                    let events = events();
                    let display_frame = |frame: &Frame, mask, _frame_index| {
                        frame.copy_to_rgba_buffer(mask, pixels.get_frame().try_into().unwrap());
                    };
                    execute_frame(&mut nes, &config, events, display_frame);

                    egui.prepare(&window);
                    let render_result = pixels.render_with(|encoder, render_target, context| {
                        context.scaling_renderer.render(encoder, render_target);
                        egui.render(encoder, render_target, context)?;
                        Ok(())
                    });

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

fn events() -> Events {
    let joypad1_button_statuses = BTreeMap::new();
    let joypad2_button_statuses = BTreeMap::new();

    Events {
        should_quit: false,
        joypad1_button_statuses,
        joypad2_button_statuses,
    }
}

/// Manages all state required for rendering egui over `Pixels`.
struct Egui {
    // State for egui.
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    rpass: RenderPass,
    paint_jobs: Vec<ClippedMesh>,
    textures: TexturesDelta,

    // State for the GUI
    gui: App,
}

/// Example application state. A real application will need a lot more state than this.
struct App {
    /// Only show the egui window when true.
    window_open: bool,
}

impl Egui {
    fn new(width: u32, height: u32, scale_factor: f32, pixels: &pixels::Pixels) -> Self {
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
        let gui = App::new();

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            rpass,
            paint_jobs: Vec::new(),
            textures,
            gui,
        }
    }

    /// Handle input events from the window manager.
    fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        self.egui_state.on_event(&self.egui_ctx, event);
    }

    fn prepare(&mut self, window: &Window) {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            self.gui.ui(egui_ctx);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) -> Result<(), BackendError> {
        self.rpass
            .add_textures(&context.device, &context.queue, &self.textures)?;
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
        )?;

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
        self.rpass.remove_textures(textures)
    }
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

        egui::Window::new("Hello, egui!")
            .open(&mut self.window_open)
            .show(ctx, |ui| {
                ui.label("This example demonstrates using egui with pixels.");
                ui.label("Made with 💖 in San Francisco!");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x /= 2.0;
                    ui.label("Learn more about egui at");
                    ui.hyperlink("https://docs.rs/egui");
                });
            });
    }
}
