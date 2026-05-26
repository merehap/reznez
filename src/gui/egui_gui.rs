use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, LazyLock};

use gilrs::GamepadId;

use crate::controller::joypad::{Button, ButtonStatus};

use egui::{ClippedPrimitive, Context, TexturesDelta, ViewportId};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use gilrs;
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

const PRIMARY_WINDOW_SCALE_FACTOR: f32 = 3.0;

pub struct EguiGui;

impl Gui for EguiGui {
    fn run(&mut self, nes: Option<Nes>, config: Config) {
        let input = WinitInputHelper::new();

        let gilrs = gilrs::Gilrs::new().unwrap();
        let gamepads: Vec<(gilrs::GamepadId, gilrs::Gamepad)> =
            gilrs.gamepads().collect();
            if gamepads.len() > 1 {
                info!("More than one gamepad connected. Using the first detected gamepad.");
            }
            
        let active_gamepad_id = gamepads.first().map(|(id, _)| *id);

        let events = Events::none();
        let mut world = World { nes, config, input, gilrs, active_gamepad_id, events };
        let event_loop = EventLoop::new().unwrap();

        let primary_renderer = Box::new(PrimaryRenderer::new());
        let primary_window_name = primary_renderer.name();
        let primary_window = EguiWindow::from_event_loop(
            &event_loop,
            PRIMARY_WINDOW_SCALE_FACTOR as f64,
            Position::Physical(PhysicalPosition { x: 50, y: 50 }),
            primary_renderer,
        );
        let mut window_manager = WindowManager::new(primary_window, primary_window_name);

        event_loop
            .run(move |event, event_loop_window_target| {
                let updated = world.input.update(&event);
                if updated {
                    if let Some(nes) = &mut world.nes {
                        if world.input.key_pressed(KeyCode::F1) {
                            info!("{}", nes.bus().oam);
                        }

                        if world.input.key_pressed(KeyCode::F12) {
                            nes.set_reset_signal();
                        }
                    }

                    if world.input.key_pressed(KeyCode::Pause)
                        || world.input.key_pressed(KeyCode::KeyP)
                        || world.input.key_pressed(KeyCode::Escape)
                    {
                        window_manager.toggle_pause();
                    }

                    world.events = poll_button_events(&world.input, &mut world.gilrs, world.active_gamepad_id);

                    window_manager.request_redraws();
                }

                if let Event::WindowEvent { event, window_id } = event {
                    match event {
                        WindowEvent::CloseRequested => {
                            let primary_removed = window_manager.remove_window(window_id);
                            if primary_removed {
                                log::logger().flush();
                                event_loop_window_target.exit();
                            }
                        }
                        WindowEvent::RedrawRequested => {
                            match window_manager.draw(&mut world, window_id) {
                                Ok(FlowControl { window_args, should_close_window }) => {
                                    if let Some((renderer, position, scale)) = window_args
                                    {
                                        window_manager.create_window_from_renderer(
                                            event_loop_window_target,
                                            renderer,
                                            position,
                                            scale as f64,
                                        );
                                    }

                                    if should_close_window {
                                        window_manager.remove_window(window_id);
                                    }
                                }
                                Err(e) => {
                                    if window_id == window_manager.primary_window_id {
                                        info!(
                                            "Closing REZNEZ due to redraw failure. {e}"
                                        );
                                        event_loop_window_target.exit();
                                    }
                                }
                            }
                        }
                        _ => {
                            if let Some(window) = window_manager.window_mut(window_id) {
                                window.handle_event(&event);
                            }
                        }
                    }
                }
            })
            .unwrap();
    }
}

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
    renderer: Box<dyn WindowRenderer>,
}

impl<'a> EguiWindow<'a> {
    fn from_event_loop(
        event_loop: &EventLoopWindowTarget<()>,
        scale_factor: f64,
        initial_position: Position,
        renderer: Box<dyn WindowRenderer>,
    ) -> Self {
        let window = {
            let size = LogicalSize::new(
                scale_factor * renderer.width() as f64,
                scale_factor * renderer.height() as f64,
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
        renderer: Box<dyn WindowRenderer>,
    ) -> Self {
        let egui_ctx = Context::default();
        let egui_state =
            egui_winit::State::new(egui_ctx, ViewportId::ROOT, &window, None, None);
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
        let mut raw_input = self.egui_state.take_egui_input(&self.window);

        raw_input.viewports.iter_mut().for_each(|viewport| {
            // Hack around bug with scale factor causing egui to crash in fonts lookup
            viewport.1.native_pixels_per_point = Some(PRIMARY_WINDOW_SCALE_FACTOR);
        });

        let mut result = FlowControl::CONTINUE;
        let output = self.egui_state.egui_ctx().run(raw_input, |egui_ctx| {
            result = self.renderer.ui(egui_ctx, world);
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

struct WindowManager<'a> {
    primary_window_id: WindowId,
    windows_by_id: BTreeMap<WindowId, (String, EguiWindow<'a>)>,
    window_names: BTreeSet<String>,
}

impl<'a> WindowManager<'a> {
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
        renderer: Box<dyn WindowRenderer>,
        position: Position,
        scale: f64,
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
            .renderer
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
