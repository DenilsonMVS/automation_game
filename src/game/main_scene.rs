use crate::{MainContext, game::{logic::GameState, render::RenderState, scene::{EventReceiver, EventType, Scene}}, graphics::renderer::Renderer, ui::Ui};

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
    fn on_simple_area_pressed(&mut self, id: usize, context: &mut MainContext<'a, 'b>) {
        self.render.receive_button_click(
            id,
            context.ui,
            context.font,
            self.logic.get_robots().iter()
                .find(|robot| robot.button_id == id),
        );
    }

    fn on_released(&mut self, id: usize, _context: &mut MainContext<'a, 'b>) {
        self.render.receive_release_event(id);
    }

    fn on_moved(&mut self, id: usize, movement: nalgebra_glm::Vec2, _context: &mut MainContext<'a, 'b>) {
        self.render.receive_move_event(id, movement);
    }

    fn on_hover_drag(&mut self, id: usize, _context: &mut MainContext<'a, 'b>) {
        self.render.receive_hover_event(&EventType::HoverDragArea(id));
    }

    fn on_hover_simple(&mut self, id: usize, _context: &mut MainContext<'a, 'b>) {
        self.render.receive_hover_event(&&EventType::HoverSimpleArea(id));
    }

    fn on_click_outside(&mut self, _context: &mut MainContext<'a, 'b>) {
        self.render.receive_click_outside();
    }
}
