use enum_dispatch::enum_dispatch;

use crate::MainContext;

#[enum_dispatch]
pub trait Scene {
    fn game_logic(&mut self, context: &mut MainContext);
    fn send(&mut self, context: &mut MainContext);
    fn generate_output(&mut self, context: &mut MainContext);
}
