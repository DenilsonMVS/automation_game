pub mod graphics;
pub mod game;
pub mod open_simplex;
pub mod ui;

use std::num::{NonZeroU32};

use glutin::{context::PossiblyCurrentContext, surface::{GlSurface, Surface, WindowSurface}};
use winit::{event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, keyboard::Key, window::Window};

use crate::{game::{main_scene::MainScene, scene::Scene}, graphics::{App, ApplicationState, renderer::{BlendFactor, Capability, ClearField, DepthFunc, Fence, Renderer, fonts::{Font, Fonts}, simple_shape_renderer::SimpleShapeRenderer}}, ui::{Ui, UiContext}};
use nalgebra_glm as glm;

pub struct MainContext<'a, 'b> {
    simple_shape_renderer: &'a mut SimpleShapeRenderer<'b>,
    font: &'a mut Font<'b>,
    ui: &'a mut Ui,
}

pub struct State<'a> {
    window: Window,
    ui: Ui,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    scene: MainScene<'a>,
    renderer: &'a Renderer,
    fence: Option<Fence>,
    fonts: Fonts<'a>,
    font: Font<'a>,
    simple_shape_renderer: SimpleShapeRenderer<'a>,
}

impl<'a> ApplicationState<'a> for State<'a> {
    fn new(
        window: Window,
        gl_context: PossiblyCurrentContext,
        gl_surface: Surface<WindowSurface>,
        renderer: &'a Renderer,
    ) -> Self {
        renderer.enable(Capability::Multisample);
        renderer.enable(Capability::Blend);
        renderer.enable(Capability::DepthTest);
        renderer.depth_func(DepthFunc::Less);
        renderer.blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);

        renderer.clear_color(&glm::vec4(0.8, 0.8, 0.8, 1.0));

        let scene = MainScene::new(&renderer);

        let fonts = Fonts::new(renderer);
        let font = fonts.new_font(renderer, include_bytes!("../res/fonts/ubuntu_mono.ttf")).unwrap();
        let simple_shape_renderer = SimpleShapeRenderer::new(renderer);

        Self {
            window,
            gl_context,
            gl_surface,
            ui: Ui::new(Self::get_window_size()),
            renderer,
            scene,
            fence: None,
            fonts, font,
            simple_shape_renderer,
        }
    }

    fn event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::Resized(new_size) => {
                self.ui.update_window_size(glm::vec2(
                    new_size.width as f32,
                    new_size.height as f32,
                ));
                if let (Some(w), Some(h)) = (NonZeroU32::new(new_size.width), NonZeroU32::new(new_size.height)) {
                    self.gl_surface.resize(&self.gl_context, w, h);
                }
                unsafe { gl::Viewport(0, 0, new_size.width as i32, new_size.height as i32); }
            },
            WindowEvent::CursorMoved { position, .. } =>
                self.ui.update_mouse_position(glm::vec2(position.x as f32, position.y as f32)),
            WindowEvent::MouseWheel { delta, .. } =>
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => self.ui.on_mouse_scroll(y),
                    _ => {},
                },
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } =>
                self.ui.mouse_click(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.repeat {
                    return;
                }

                if event.state == ElementState::Released {
                    match event.logical_key {
                        Key::Named(winit::keyboard::NamedKey::F11) => {
                            if self.window.fullscreen().is_some() {
                                self.window.set_fullscreen(None);
                            } else {
                                self.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                            }
                        },
                        _ => {},
                    }
                }

                self.ui.on_key_event(event);
            },
            _ => (),
        }
    }

    fn get_window_title() -> &'static str {
        "Automation game"
    }

    fn get_window_size() -> glm::Vec2 {
        glm::vec2(800.0, 600.0)
    }
}

impl<'a> State<'a> {
    fn redraw(&mut self) {
        let mut context= UiContext {
            window: &self.window,
            scene: &mut self.scene,
        };
        self.ui.on_render_start(&mut context);

        self.simple_shape_renderer.reset();
        self.font.reset();

        {
            let mut main_context = MainContext {
                simple_shape_renderer: &mut self.simple_shape_renderer,
                font: &mut self.font,
                ui: &mut self.ui,
            };

            self.scene.game_logic(&mut main_context);

            if let Some(fence) = &self.fence {
                fence.wait();
            }

            self.scene.send(&mut main_context);
            
            self.renderer.clear(&[ClearField::Color, ClearField::Depth]);
            
            self.scene.generate_output(&mut main_context);
        }
        
        self.simple_shape_renderer.finish();
        self.font.finish();

        let proj_matrix = self.ui.get_projection_matrix();
        let vp_matrix = self.ui.get_camera().get_vp_matrix(&proj_matrix);
        self.simple_shape_renderer.draw(self.renderer, &proj_matrix, &vp_matrix);

        self.renderer.depth_mask(false);
        self.fonts.draw(&mut self.font, &proj_matrix);
        self.renderer.depth_mask(true);

        self.fence = Some(self.renderer.new_fence());

        self.gl_surface.swap_buffers(&self.gl_context).unwrap();
        self.window.request_redraw();

        self.ui.on_render_end();
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::<State>::new();
    event_loop.run_app(&mut app).unwrap();
}
