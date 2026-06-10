use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, LazyLock};

use gilrs::GamepadId;

use egui::{ClippedPrimitive, Context, TexturesDelta, ViewportId};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use gilrs;
use log::{info, warn};
use pixels::{Pixels, SurfaceTexture};
use pixels::wgpu::{RenderPassDescriptor, RenderPassColorAttachment, Operations, LoadOp, StoreOp};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, Position};
use winit::event::WindowEvent;
use winit::event_loop::{EventLoop, ActiveEventLoop};
use winit::keyboard::KeyCode;
use winit::window::Icon;
use winit::window::{Window, WindowId};
use winit_input_helper::WinitInputHelper;

use crate::config::Config;
use crate::controller::joypad::{Button, ButtonStatus};
use crate::gui::gui::{Gui, Events};
use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::window_renderers::primary_renderer::PrimaryRenderer;
use crate::gui::world::World;
use crate::nes::Nes;

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

const PRIMARY_WINDOW_SCALE_FACTOR: f32 = 3.0;

pub struct EguiGui<'a> {
    world: World,
    window_manager: WindowManager<'a>,
    keyboard: WinitInputHelper,
    gamepad_handler: gilrs::Gilrs,
    active_gamepad_id: Option<gilrs::GamepadId>,
}

impl <'a> EguiGui<'a> {
    pub fn new(config: Config) -> Self {
        let gamepad_handler = gilrs::Gilrs::new().unwrap();
        let gamepads: Vec<(gilrs::GamepadId, gilrs::Gamepad)> = gamepad_handler.gamepads().collect();
        let active_gamepad_id = gamepads.first().map(|(id, _)| *id);
        if gamepads.len() > 1 {
            warn!("Only one gamepad at a time is currently supported, but multiple are connected. Proceeding with only the first detected gamepad: {active_gamepad_id:?}."
            );
        }

        let events = Events::none();
        Self {
            world: World { nes: None, config, events },
            window_manager: WindowManager::new(),
            keyboard: WinitInputHelper::new(),
            gamepad_handler,
            active_gamepad_id,
        }
    }
}

impl <'a> Gui for EguiGui<'a> {
    fn run(&mut self, nes: Option<Nes>) {
        self.world.nes = nes;
        let event_loop = EventLoop::new().unwrap();
        event_loop.run_app(self).unwrap();
    }
}

impl <'a> ApplicationHandler for EguiGui<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let primary_renderer = Box::new(PrimaryRenderer::new());
        let position = Position::Physical(PhysicalPosition { x: 50, y: 50 });
        self.window_manager.create_window_from_renderer(event_loop, primary_renderer, position, PRIMARY_WINDOW_SCALE_FACTOR as f64);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.window_manager.request_redraws();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                let primary_removed = self.window_manager.remove_window(window_id);
                if primary_removed {
                    log::logger().flush();
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(nes) = &mut self.world.nes {
                    if self.keyboard.key_pressed(KeyCode::F1) {
                        info!("{}", nes.bus().oam);
                    }

                    if self.keyboard.key_pressed(KeyCode::F12) {
                        nes.set_reset_signal();
                    }
                }

                if self.keyboard.key_pressed(KeyCode::Pause)
                    || self.keyboard.key_pressed(KeyCode::KeyP)
                    || self.keyboard.key_pressed(KeyCode::Escape)
                {
                    self.window_manager.toggle_pause();
                }

                self.world.events = poll_button_events(&self.keyboard, &mut self.gamepad_handler, self.active_gamepad_id);

                match self.window_manager.draw(&mut self.world, window_id) {
                    Ok(FlowControl { window_args, should_close_window }) => {
                        if let Some((renderer, position, scale)) = window_args {
                            self.window_manager.create_window_from_renderer(
                                event_loop,
                                renderer,
                                position,
                                scale as f64,
                            );
                        }

                        if should_close_window {
                            self.window_manager.remove_window(window_id);
                        }
                    }
                    Err(e) => {
                        if window_id == self.window_manager.primary_window_id {
                            info!("Closing REZNEZ due to redraw failure. {e}");
                            event_loop.exit();
                        }
                    }
                }
            }
            _ => {
                if let Some(window) = self.window_manager.window_mut(window_id) {
                    window.handle_event(&event);
                }
            }
        }
    }
}

/// Manages all state required for rendering egui over `Pixels`.
struct EguiWindow<'a> {
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    wgpu_renderer: Renderer,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,

    // State for the GUI
    window: Arc<Window>,
    pixels: Pixels<'a>,
    window_renderer: Box<dyn WindowRenderer>,
}

impl<'a> EguiWindow<'a> {
    fn from_active_event_loop(
        event_loop: &ActiveEventLoop,
        scale_factor: f64,
        initial_position: Position,
        renderer: Box<dyn WindowRenderer>,
    ) -> Self {
        let window = {
            let size = LogicalSize::new(
                scale_factor * renderer.width() as f64,
                scale_factor * renderer.height() as f64,
            );
            let window_attributes = Window::default_attributes()
                .with_title(renderer.name())
                .with_window_icon(Some(window_icon()))
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_resizable(false)
                .with_position(initial_position);
            event_loop.create_window(window_attributes).unwrap()
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
        window_renderer: Box<dyn WindowRenderer>,
    ) -> Self {
        let egui_ctx = Context::default();
        egui_extras::install_image_loaders(&egui_ctx);
        let egui_state = egui_winit::State::new(egui_ctx, ViewportId::ROOT, &window, None, None, None);
        let screen_descriptor = ScreenDescriptor {
            pixels_per_point: scale_factor,
            size_in_pixels: [width, height],
        };
        let renderer_options = RendererOptions::default();
        let wgpu_renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), renderer_options);
        let textures = TexturesDelta::default();

        Self {
            egui_state,
            screen_descriptor,
            wgpu_renderer,
            paint_jobs: Vec::new(),
            textures,
            window,
            pixels,
            window_renderer,
        }
    }

    /// Handle input events from the window manager.
    fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        let _event_response = self.egui_state.on_window_event(&self.window, event);
    }

    fn draw(&mut self, world: &mut World) -> Result<FlowControl, String> {
        self.window_renderer.render(world, &mut self.pixels);

        // Run the egui frame and create all paint jobs to prepare for rendering.
        let mut raw_input = self.egui_state.take_egui_input(&self.window);

        raw_input.viewports.iter_mut().for_each(|viewport| {
            // Hack around bug with scale factor causing egui to crash in fonts lookup
            viewport.1.native_pixels_per_point = Some(PRIMARY_WINDOW_SCALE_FACTOR);
        });

        let mut result = FlowControl::CONTINUE;
        let output = self.egui_state.egui_ctx().run_ui(raw_input, |ui| {
            result = self.window_renderer.ui(self.egui_state.egui_ctx(), ui, world);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(&self.window, output.platform_output);
        self.paint_jobs = self
            .egui_state
            .egui_ctx()
            .tessellate(output.shapes, PRIMARY_WINDOW_SCALE_FACTOR);

        self.pixels
            .render_with(|encoder, render_target, context| {
                context.scaling_renderer.render(encoder, render_target);
                for (id, delta) in &self.textures.set {
                    self.wgpu_renderer.update_texture(&context.device, &context.queue, *id, delta);
                }
                self.wgpu_renderer.update_buffers(
                    &context.device,
                    &context.queue,
                    encoder,
                    &self.paint_jobs,
                    &self.screen_descriptor,
                );

                {
                    let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("egui"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: render_target,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Load,
                                store: StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    }).forget_lifetime();

                    // Record all render passes.
                    self.wgpu_renderer.render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
                }

                // Cleanup
                let textures = std::mem::take(&mut self.textures);
                for id in &textures.free {
                    self.wgpu_renderer.free_texture(id);
                }

                Ok(())
            })
            .map_err(|err| err.to_string())?;

        Ok(result)
    }
}

struct WindowManager<'a> {
    primary_window_id: WindowId,
    windows_by_id: BTreeMap<WindowId, (String, EguiWindow<'a>)>,
    window_names: BTreeSet<String>,
}

impl<'a> WindowManager<'a> {
    pub fn new() -> WindowManager<'a> {
        WindowManager {
            primary_window_id: WindowId::dummy(),
            windows_by_id: BTreeMap::new(),
            window_names: BTreeSet::new(),
        }
    }

    pub fn create_window_from_renderer(
        &mut self,
        event_loop: &ActiveEventLoop,
        renderer: Box<dyn WindowRenderer>,
        position: Position,
        scale: f64,
    ) {
        let name = renderer.name();
        if self.window_names.contains(&name) {
            return;
        }

        self.window_names.insert(name.clone());

        let window = EguiWindow::from_active_event_loop(event_loop, scale, position, renderer);
        self.windows_by_id
            .insert(window.window.id(), (name, window));
    }

    pub fn remove_window(&mut self, window_id: WindowId) -> bool {
        if let Some((name, _)) = self.windows_by_id.remove(&window_id) {
            self.window_names.remove(&name);
            let primary_removed = window_id == self.primary_window_id;
            return primary_removed;
        }

        false
    }

    pub fn toggle_pause(&mut self) {
        self.windows_by_id
            .get_mut(&self.primary_window_id)
            .unwrap()
            .1
            .window_renderer
            .toggle_pause();
    }

    pub fn request_redraws(&self) {
        for (_id, window) in self.windows_by_id.values() {
            window.window.request_redraw();
        }
    }

    pub fn draw(
        &mut self,
        world: &mut World,
        window_id: WindowId,
    ) -> Result<FlowControl, String> {
        let window = self
            .window_mut(window_id)
            .ok_or("Failed to create window")?;
        window.draw(world)
    }

    pub fn window_mut(&mut self, window_id: WindowId) -> Option<&mut EguiWindow<'a>> {
        self.windows_by_id
            .get_mut(&window_id)
            .map(|(_, window)| window)
    }
}

fn poll_button_events(input: &WinitInputHelper, gilrs: &mut gilrs::Gilrs, active_gamepad_id: Option<GamepadId>) -> Events {
    let mut joypad1_button_statuses = BTreeMap::new();
    let mut joypad2_button_statuses = BTreeMap::new();

    while let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
        if Some(id) != active_gamepad_id {
            warn!("Event won't be processed from ignored gamepad {id:?}: {event:?}");
            continue;
        }
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
        }
    }

    for (&key, &button) in JOY_2_KEYBOARD_MAPPINGS.iter() {
        if input.key_pressed(key) {
            joypad2_button_statuses.insert(button, ButtonStatus::Pressed);
        } else if input.key_released(key) {
            joypad2_button_statuses.insert(button, ButtonStatus::Unpressed);
        }
    }

    Events {
        // Quit-handling is done by winit.
        should_quit: false,
        joypad1_button_statuses,
        joypad2_button_statuses,
    }
}

fn window_icon() -> Icon {
    let image_bytes = include_bytes!("assets/reznez_logo.png");
    let image = image::load_from_memory(image_bytes)
        .unwrap()
        .into_rgba8();
    let (width, height) = image.dimensions();

    Icon::from_rgba(image.into_raw(), width, height).unwrap()
}