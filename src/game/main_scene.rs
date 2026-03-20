use crate::{game::{robot::Robot, scene::Scene}, graphics::renderer::positioning::PositionMode};
use nalgebra_glm as glm;

#[derive(Debug)]
pub struct MainScene {
    robot: Robot,
}

impl MainScene {
    pub fn new() -> Self {
        Self { 
            robot: Robot::new(),
        }
    }
}

impl Scene for MainScene {
    fn game_logic(&mut self, context: &mut crate::MainContext) {
        
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        let simple_shape_renderer = &mut context.simple_shape_renderer;
        self.robot.draw(simple_shape_renderer);
    }

    fn handle_input(&mut self, context: &mut crate::MainContext) {
        
    }
}
