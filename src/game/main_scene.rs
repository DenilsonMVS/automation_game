use winit::event::{ElementState, MouseButton, WindowEvent};

use crate::{game::{logic::GameState, render::RenderState, scene::{EventType, Scene}}, graphics::renderer::Renderer, ui::Ui};

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
    fn handle_event(&mut self, event: &EventType, context: &mut crate::MainContext) {
        match event {
            EventType::Winit(event) => match **event {
                WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } =>
                    self.render.receive_generic_click(),
                _ => {},
            },
            EventType::SimpleAreaPressed(button_id) => {
                self.render.receive_button_click(
                    *button_id,
                    context.ui,
                    self.logic.get_robots().iter()
                        .find(|robot| robot.button_id == *button_id),
                );
                
                self.render.receive_ui_click();
            },
            EventType::FinalEvent => self.render.receive_finish_event(),
            EventType::Grabbed(_) => self.render.receive_ui_click(),
            EventType::Released(id) => {
                self.render.receive_release_event(*id);
                self.render.receive_ui_click();
            },
            EventType::Moved(info) =>
                self.render.receive_move_event(info.as_ref().0, info.as_ref().1),
            EventType::HoverSimpleArea(_) | EventType::HoverDragArea(_) =>
                self.render.receive_hover_event(event),
        }
    }

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
