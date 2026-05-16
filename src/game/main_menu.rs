use crate::{game::{levels::main_menu::create_main_menu_back_view, logic::GameState, render::RenderState, scene::Scene}, graphics::{camera::Camera, renderer::Renderer}};
use nalgebra_glm as glm;

pub struct MainMenu<'a> {
    pub logic: GameState,
    pub render: RenderState<'a>,
}

impl<'a> MainMenu<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let logic = create_main_menu_back_view();
        let render = RenderState::new(renderer, logic.get_world());
        return Self { logic, render };
    }
}

fn get_camera() -> Camera {
    Camera::new(glm::vec2(99.5, 100.0), 0.1)
}

impl<'a> Scene for MainMenu<'a> {
    fn game_logic(&mut self, context: &mut crate::MainContext) {
        self.logic.update(context);
        self.render.update(&mut self.logic, context);
    }

    fn send(&mut self, context: &mut crate::MainContext) {
        let mut camera = get_camera();
        std::mem::swap(&mut context.ui.camera, &mut camera);
        self.render.send(&self.logic, context);
        std::mem::swap(&mut context.ui.camera, &mut camera);
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        let camera = get_camera();
        // let camera = context.ui.camera;
        self.render.draw(&camera.get_vp_matrix(&context.ui.get_projection_matrix()));
    }
}
