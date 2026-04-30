use crate::{MainContext, game::{logic::GameState, render::{RenderEventContext, RenderState}, scene::{EventReceiver, EventType, Scene}}, graphics::renderer::Renderer, ui::Ui};

pub struct MainScene<'a> {
    pub logic: GameState,
    pub render: RenderState<'a>,
}

impl<'a> MainScene<'a> {
    pub fn new(renderer: &'a Renderer, ui: &mut Ui) -> Self {
        let logic = GameState::new(ui);
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

impl<'a, 'b, 'c> EventReceiver<MainContext<'a, 'b>> for MainScene<'c> {
    fn propagate_children(&mut self, event: &EventType, context: &mut MainContext<'a, 'b>) {
        let mut context = RenderEventContext {
            ui: context.ui,
            simple_shape_renderer: context.simple_shape_renderer,
            font: context.font,
            robot: if let EventType::SimpleAreaPressed(id) = event {
                self.logic.get_robots().iter()
                    .find(|robot| robot.button_id == *id)
            } else {
                None
            }
        };

        self.render.on_event(event, &mut context);
    }
}
