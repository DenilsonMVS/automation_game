pub mod graphics;
pub mod game;
pub mod open_simplex;
pub mod ui;
pub mod stable_index_vector;
pub mod kd_tree;

use std::num::{NonZeroU32};

use glutin::{context::PossiblyCurrentContext, surface::{GlSurface, Surface, WindowSurface}};
use winit::{event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, keyboard::Key, window::Window};

use crate::{game::{main_menu::MainMenu, scene::Scene}, graphics::{App, ApplicationState, renderer::{BlendFactor, Capability, ClearField, DepthFunc, Fence, Renderer, fonts::{Font, Fonts}, scissor_group::ScissorStack, simple_shape_renderer::{HasSimpleShapeRenderer, SimpleShapeRenderer}}}, ui::{HasUi, Ui, rendered_button::RenderedButtonContext}};
use nalgebra_glm as glm;

pub struct MainContext<'a, 'b> {
    pub groups: &'a mut ScissorStack<'b>,
    pub simple_shape_renderer: &'a mut SimpleShapeRenderer<'b>,
    pub font: &'a mut Font<'b>,
    pub ui: &'a mut Ui,
}

impl<'a, 'b> MainContext<'a, 'b> {
    pub fn get_button_context<'s>(&'s mut self) -> RenderedButtonContext<'s, 'b> {
        RenderedButtonContext { 
            font: self.font, 
            simple_shape_renderer: self.simple_shape_renderer,
            groups: self.groups,
        }
    }
}

impl<'a, 'b> HasUi for MainContext<'a, 'b> {
    fn get_ui(&mut self) -> &mut Ui {
        self.ui
    }
}

impl<'a, 'b> HasSimpleShapeRenderer<'b> for MainContext<'a, 'b> {
    fn get_simple_shape_renderer(&mut self) -> &mut SimpleShapeRenderer<'b> {
        self.simple_shape_renderer
    }

    fn get_groups(&mut self) -> &mut ScissorStack<'b> {
        self.groups
    }

    fn get(&mut self) -> (&mut SimpleShapeRenderer<'b>, &mut ScissorStack<'b>) {
        (self.simple_shape_renderer, self.groups)
    }
}

pub struct State<'a> {
    window: Window,
    ui: Ui,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    scene: Box<dyn Scene + 'a>,
    renderer: &'a Renderer,
    fence: Option<Fence>,
    fonts: Fonts<'a>,
    font: Font<'a>,
    simple_shape_renderer: SimpleShapeRenderer<'a>,
    groups: ScissorStack<'a>,
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

        let ui = Ui::new(Self::get_window_size());
        let scene = Box::new(MainMenu::new(renderer));

        let fonts = Fonts::new(renderer);
        let font = fonts.new_font(renderer, include_bytes!("../res/fonts/ubuntu_mono.ttf")).unwrap();
        let simple_shape_renderer = SimpleShapeRenderer::new(renderer);

        Self {
            window,
            gl_context,
            gl_surface,
            ui, 
            renderer,
            scene,
            fence: None,
            fonts, font,
            simple_shape_renderer,
            groups: ScissorStack::new(renderer),
        }
    }

    fn event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match &event {
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
                    MouseScrollDelta::LineDelta(_, y) => self.ui.on_mouse_scroll(*y),
                    _ => {},
                },
            WindowEvent::MouseInput { button: MouseButton::Left, state, .. } =>
                match state {
                    ElementState::Pressed => self.ui.mouse_pressed(),
                    ElementState::Released => self.ui.mouse_released(),
                },
            WindowEvent::KeyboardInput { event, .. } => {
                self.ui.on_key_event(event);

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
        self.ui.on_render_start(&self.window);
        self.groups.reset();

        {
            let mut main_context = MainContext {
                simple_shape_renderer: &mut self.simple_shape_renderer,
                font: &mut self.font,
                ui: &mut self.ui,
                groups: &mut self.groups,
            };

            main_context.simple_shape_renderer.reset();
            main_context.font.reset();

            main_context.groups.push(main_context.ui.get_ndc_render_box());

            self.scene.game_logic(&mut main_context);

            if let Some(fence) = &self.fence {
                fence.wait();
            }

            self.scene.send(&mut main_context);
            
            self.renderer.clear(&[ClearField::Color, ClearField::Depth]);
            
            self.scene.generate_output(&mut main_context);
        }

        self.groups.finish();
        self.simple_shape_renderer.finish();
        self.font.finish();

        let proj_matrix = self.ui.get_projection_matrix();

        self.renderer.enable(Capability::ScissorTest);
            self.simple_shape_renderer.draw(self.renderer, &self.ui, &proj_matrix, &self.groups);
            self.renderer.depth_mask(false);
                self.fonts.draw(&mut self.font, &self.groups, &self.ui, &proj_matrix);
                self.simple_shape_renderer.draw_transparent(&self.ui, &self.groups);
            self.renderer.depth_mask(true);
        self.renderer.disable(Capability::ScissorTest);

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
