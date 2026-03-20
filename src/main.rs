
pub mod graphics;
pub mod sounds;

use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use nalgebra_glm::{self as glm, Mat4};

use crate::graphics::renderer::{BlendFactor, Capability, ClearField, DepthFunc, Renderer, positioning::{PositionMode, screen_pos_to_world_pos}, simple_shape_renderer::SimpleShapeRenderer};

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
    
    pub proj_matrix: Mat4,
    pub i_proj_matrix: Mat4,
    pub now: std::time::Instant,
    pub window_size: glm::Vec2,
    pub mouse_pos: glm::Vec2,
}

fn main() {

    let mut setup = graphics::Graphics::new(glm::U32Vec2::new(800, 600), "Automation game").unwrap();
    let (glfw, window, events, renderer) = setup.get();

    renderer.enable(Capability::Multisample);
    renderer.enable(Capability::Blend);
    renderer.blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);

    renderer.enable(Capability::DepthTest);
    renderer.depth_func(DepthFunc::Less);

    renderer.clear_color(&glm::vec4(0.1, 0.12, 0.2, 0.0));

    let mut simple_shape_renderer = SimpleShapeRenderer::new(renderer);
    
    while !window.should_close() {

        let window_size = get_window_size(window);
        unsafe { gl::Viewport(0, 0, window_size.x as i32, window_size.y as i32); }

        let proj_matrix = get_projection_matrix(window_size);
        let i_proj_matrix = glm::inverse(&proj_matrix);
        let mouse_pos = get_mouse_pos(window, window_size, &i_proj_matrix);

        // let mut main_context = MainContext {
        //     glfw, window, events, renderer,
        //     proj_matrix,
        //     i_proj_matrix,
        //     now: time::Instant::now(),
        //     window_size,
        //     mouse_pos,
        // };

        glfw.poll_events();

        simple_shape_renderer.reset();
        simple_shape_renderer.send_rectangle(
            glm::vec2(0.0, 0.0),
            glm::vec2(0.5, 0.3),
            PositionMode::Center,
            glm::vec4(200, 50, 50, 255),
            50,
        );
        simple_shape_renderer.send_rectangle(
            glm::vec2(0.1, 0.1),
            glm::vec2(0.5, 0.3),
            PositionMode::Center,
            glm::vec4(200, 250, 50, 255),
            51,
        );

        simple_shape_renderer.send_symmetrical_triangle(
            glm::vec2(0.4, 0.4),
            glm::vec2(0.25, 0.25),
            0.1, glm::vec4(200, 250, 250, 200),
            10,
        );

        simple_shape_renderer.send_arbitrary_triangle(
            &[
                glm::vec2(-0.5, -0.5),
                glm::vec2( 0.0,  0.5),
                glm::vec2( 0.5, -0.5),
            ],
            glm::vec4(1, 250, 100, 100),
            1,
        );

        simple_shape_renderer.send_ellipse(
            glm::vec2(-0.8, 0.8),
            glm::vec2(0.2, 0.1),
            PositionMode::Center,
            glm::vec4(10, 10, 10, 255),
            100,
        );

        simple_shape_renderer.send_rotated_ellipse(
            glm::vec2(0.8, 0.8),
            glm::vec2(0.2, 0.1),
            PositionMode::Center,
            glm::vec2(0.5, -0.5),
            glm::vec4(10, 10, 10, 255),
            100,
        );
        
        renderer.clear(&[ClearField::Color, ClearField::Depth]);

        simple_shape_renderer.finish();
        simple_shape_renderer.draw(&proj_matrix);

        window.swap_buffers();
    }
}
