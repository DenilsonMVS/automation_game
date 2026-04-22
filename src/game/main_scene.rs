use crate::{game::{logic::GameState, render::RenderState, scene::Scene}, graphics::renderer::Renderer};

pub struct MainScene<'a> {
    logic: GameState,
    render: RenderState<'a>,
}

impl<'a> MainScene<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let logic = GameState::new();
        let render = RenderState::new(renderer, logic.get_world());
        return Self { logic, render };
    }
}

impl<'a> Scene for MainScene<'a> {    
    fn game_logic(&mut self, context: &mut crate::MainContext) {
        self.logic.update(context);
    }

    fn send(&mut self, context: &mut crate::MainContext) {
        self.render.send(&self.logic, context);
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        self.render.draw(&context.ui.get_camera().get_vp_matrix(&context.ui.get_projection_matrix()));
    }
}
