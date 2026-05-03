use crate::MainContext;

pub trait Scene {
    fn game_logic(&mut self, context: &mut MainContext);
    fn send(&mut self, context: &mut MainContext);
    fn generate_output(&mut self, context: &mut MainContext);
}
