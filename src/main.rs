
pub mod graphics;
pub mod sounds;
pub mod game;
pub mod open_simplex;

use std::time;

use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use nalgebra_glm::{self as glm, Mat4};

use crate::{game::{graph::GraphRenderer, main_scene::MainScene, scene::Scene}, graphics::{camera::Camera, renderer::{BlendFactor, Capability, ClearField, DepthFunc, Renderer, fonts::{Font, Fonts}, positioning::screen_pos_to_world_pos, simple_shape_renderer::SimpleShapeRenderer}}};

fn get_window_size(window: &PWindow) -> glm::Vec2 {
    let (x, y) = window.get_size();
    return glm::vec2(x as f32, y as f32);
}

fn get_projection_matrix(window_size: glm::Vec2) -> glm::Mat4 {
    let ratio = window_size.x / window_size.y;

    let projection = if window_size.x > window_size.y {
        glm::ortho(-ratio, ratio, -1.0, 1.0, -1.0, 1.0)
    } else {
        let inv_ratio = window_size.y / window_size.x;
        glm::ortho(-1.0, 1.0, -inv_ratio, inv_ratio, -1.0, 1.0)
    };

    return projection;
}

fn get_mouse_pos(window: &PWindow, window_size: glm::Vec2, i_projection_matrix: &glm::Mat4) -> glm::Vec2 {
    let (x, y) = window.get_cursor_pos();
    return screen_pos_to_world_pos(glm::vec2(x as f32, y as f32), window_size, i_projection_matrix);
}


#[allow(dead_code)]
pub struct MainContext<'a, 'b> {
    // 'b current frame lifetime
    pub glfw: &'b mut Glfw,
    pub window: &'b mut PWindow,
    pub events: &'b mut GlfwReceiver<(f64, WindowEvent)>,

    // 'a long lifetime
    pub renderer: &'a Renderer,
    pub simple_shape_renderer: &'b mut SimpleShapeRenderer<'a>,
    pub graph_renderer: &'b GraphRenderer<'a>,
    pub font: &'b mut Font<'a>,
    
    pub proj_matrix: Mat4,
    pub i_proj_matrix: Mat4,
    pub vp_matrix: Mat4,
    pub window_size: glm::Vec2,
    pub mouse_pos: glm::Vec2,
    pub delta: f32,
    pub halt: bool,
}

fn main() {

    let mut setup = graphics::Graphics::new(glm::U32Vec2::new(800, 600), "Automation game").unwrap();
    let (glfw, window, events, renderer) = setup.get();

    renderer.enable(Capability::Multisample);
    renderer.enable(Capability::Blend);
    renderer.enable(Capability::DepthTest);
    renderer.depth_func(DepthFunc::Less);

    renderer.clear_color(&glm::vec4(0.8, 0.8, 0.8, 0.0));

    let fonts = Fonts::new(renderer);
    let mut font = fonts.new_font(renderer, include_bytes!("../res/fonts/ubuntu_mono.ttf")).unwrap();
    let mut simple_shape_renderer: SimpleShapeRenderer<'_> = SimpleShapeRenderer::new(renderer);
    let graph_renderer = GraphRenderer::new(renderer).unwrap();
    
    let mut camera = Camera::new();
    camera.handle_scroll(-10.0);

    let mut scene = MainScene::new(renderer);
    let mut previous_frame = time::Instant::now();
    let mut is_fullscreen = false;
    let mut halt = true;

    while !window.should_close() {

        let now = time::Instant::now();
        let delta = now.duration_since(previous_frame).as_secs_f32();
        previous_frame = now;

        let window_size = get_window_size(window);
        unsafe { gl::Viewport(0, 0, window_size.x as i32, window_size.y as i32); }

        let proj_matrix = get_projection_matrix(window_size);
        let view_matrix = camera.get_view_matrix();
        let vp_matrix = view_matrix * proj_matrix;
        let i_proj_matrix = glm::inverse(&proj_matrix);
        let mouse_pos = get_mouse_pos(window, window_size, &i_proj_matrix);

        
        glfw.poll_events();
        let messages: Vec<(f64, WindowEvent)> = glfw::flush_messages(events).collect();
        for (_, event) in messages.iter() {
            match event {
                WindowEvent::Scroll(_, scroll) => camera.handle_scroll(*scroll as f32),
                WindowEvent::Key(glfw::Key::Space, _, glfw::Action::Press, _) => halt = !halt,
                WindowEvent::Key(glfw::Key::F11, _, glfw::Action::Press, _) => {
                    if is_fullscreen {
                        window.set_monitor(
                            glfw::WindowMode::Windowed,
                            100, 100,
                            800, 600,
                            None,
                        );
                    } else {
                        glfw.with_primary_monitor(|_, monitor| {
                            if let Some(m) = monitor {
                                let video_mode = m.get_video_mode().unwrap();
                                window.set_monitor(
                                    glfw::WindowMode::FullScreen(m),
                                    0, 0,
                                    video_mode.width, 
                                    video_mode.height,
                                    Some(video_mode.refresh_rate)
                                );
                            }
                        });
                    }

                    is_fullscreen = !is_fullscreen;
                },
                _ => {}
            }
        }
        camera.handle_keyboard_input(window, delta);


        simple_shape_renderer.reset();
        font.reset();

        let fence;

        {
            let mut main_context = MainContext {
                glfw, window, events, renderer,
                proj_matrix,
                i_proj_matrix,
                vp_matrix,
                window_size,
                mouse_pos,
                simple_shape_renderer: &mut simple_shape_renderer,
                graph_renderer: &graph_renderer,
                font: &mut font,
                delta,
                halt,
            };

            scene.handle_input(&mut main_context);
            scene.game_logic(&mut main_context);

            fence = renderer.new_fence();

            scene.send(&mut main_context);

            renderer.clear(&[ClearField::Color, ClearField::Depth]);

            scene.generate_output(&mut main_context);
        }

        simple_shape_renderer.finish();
        font.finish();

        simple_shape_renderer.draw(renderer, &proj_matrix, &vp_matrix);

        renderer.blend_func(BlendFactor::One, BlendFactor::OneMinusSrc1Color);
        renderer.depth_mask(false);
        fonts.draw(&mut font, &proj_matrix, &vp_matrix);

        fence.wait();

        renderer.depth_mask(true);
        renderer.blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);

        window.swap_buffers();
    }
}
