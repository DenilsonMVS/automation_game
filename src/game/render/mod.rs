use crate::{MainContext, game::{logic::{GameState, world::World}, render::world::ViewWorld}, graphics::renderer::Renderer};
use nalgebra_glm as glm;

pub mod display_module;
pub mod robot;
pub mod world;

pub struct RenderState<'a> {
    world: ViewWorld<'a>
}

impl<'a> RenderState<'a> {
    pub fn new(renderer: &'a Renderer, logic_world: &World) -> Self {
        Self { world: ViewWorld::new(renderer, logic_world) }
    }

    pub fn send(&mut self, logic_state: &GameState, context: &mut MainContext) {
        let simple_shape_renderer = &mut context.simple_shape_renderer;
        let font = &mut context.font;

        for robot in logic_state.get_robots().iter() {
            robot.send(simple_shape_renderer, font, context.camera);
        }

        self.world.send(logic_state.get_world());
    }

    pub fn draw(&self, proj_matrix: &glm::Mat4) {
        self.world.draw(proj_matrix);
    }
}