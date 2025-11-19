use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

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
use crate::gui::gui::Gui;
use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::window_renderers::primary_renderer::PrimaryRenderer;
use crate::gui::world::World;
use crate::nes::Nes;

pub struct EguiGui;

impl Gui for EguiGui {
    fn run(&mut self, nes: Option<Nes>, config: Config) {
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
                if let Some(nes) = &mut world.nes {
                    if world.input.key_pressed(KeyCode::F1) {
                        info!("{}", nes.memory().oam);
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
                                if let Some((renderer, position, scale)) = window_args {
                                    window_manager.create_window_from_renderer(
                                        event_loop_window_target,
                                        renderer,
                                        position,
                                        scale,
                                    );
                                }

                                if should_close_window {
                                    window_manager.remove_window(window_id);
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
                        if let Some(window) = window_manager.window_mut(window_id) {
                            window.handle_event(&event);
                        }
                    }
                }
            }
        }).unwrap();
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

impl <'a> EguiWindow<'a> {
    fn from_event_loop(
        event_loop: &EventLoopWindowTarget<()>,
        scale_factor: u64,
        initial_position: Position,
        renderer: Box<dyn WindowRenderer>,
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
        renderer: Box<dyn WindowRenderer>,
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
        renderer: Box<dyn WindowRenderer>,
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

    pub fn remove_window(&mut self, window_id: WindowId) -> bool {
        if let Some((name, _)) = self.windows_by_id.remove(&window_id) {
            self.window_names.remove(&name);
            let primary_removed = window_id == self.primary_window_id;
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

    pub fn draw(&mut self, world: &mut World, window_id: WindowId) -> Result<FlowControl, String> {
        let window = self.window_mut(window_id).ok_or("Failed to create window")?;
        window.draw(world)
    }

    pub fn window_mut(&mut self, window_id: WindowId) -> Option<&mut EguiWindow<'a>> {
        self.windows_by_id
            .get_mut(&window_id)
            .map(|(_, window)| window)
    }
}
/*
fn button_pressed(key: input: &WinitInputHelper, gilrs: gilrs::Gilrs, active_gamepad_id: Option<GamepadId>) -> bool {

}
*/