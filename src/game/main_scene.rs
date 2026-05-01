use crate::{MainContext, game::{logic::GameState, render::{RenderEventContext, RenderState}, scene::{EventReceiver, EventType, Scene}}, graphics::renderer::Renderer};

pub struct MainScene<'a> {
    pub logic: GameState,
    pub render: RenderState<'a>,
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
        self.render.update(&self.logic, context);
    }

    fn send(&mut self, context: &mut crate::MainContext) {
        self.render.send(&self.logic, context);
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        self.render.draw(&context.ui.get_camera().get_vp_matrix(&context.ui.get_projection_matrix()));
    }
}

impl<'a, 'b, 'c> EventReceiver<MainContext<'a, 'b>> for MainScene<'c> {
    fn propagate_children(&mut self, event: &EventType, context: &mut MainContext<'a, 'b>) {
        let mut context = RenderEventContext {
            ui: context.ui,
            simple_shape_renderer: context.simple_shape_renderer,
            font: context.font,
            robots: self.logic.get_robots()
        };

        self.render.on_event(event, &mut context);
    }
}
