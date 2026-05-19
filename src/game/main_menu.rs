use std::array;

use crate::{game::{levels::main_menu::create_main_menu_back_view, logic::GameState, render::{RenderState, UI_BASE_DEPTH, robot_info::{get_button_bgcolor, get_button_color}}, scene::Scene}, graphics::{camera::Camera, renderer::{Renderer, positioning::{PositionMode, RenderBox}}}, ui::{HasUi, events::ActionType, rendered_button::{Content, RenderedButton}, ui_element::UiElement}};
use nalgebra_glm as glm;

pub struct MainMenu<'a> {
    pub logic: GameState,
    pub render: RenderState<'a>,
    pub buttons: [RenderedButton; 4],
}

impl<'a> MainMenu<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let logic = create_main_menu_back_view();
        let render = RenderState::new(renderer, &logic);
        let contents = ["1", "2", "3", "4"];
        return Self {
            logic, render,
            buttons: array::from_fn(|i| RenderedButton::new(get_button_bgcolor()).with_content(Content::Str(contents[i]), get_button_color())),
        };
    }

    fn get_button_bbox(idx: usize) -> RenderBox {
        let offset = glm::vec2(0.5, -0.5);
        let i = (idx / 2) as f32;
        let j = (idx % 2) as f32;
        let offset = glm::vec2(offset.x * j, offset.y * i);
        RenderBox::new_simple(
            glm::vec2(-0.25, 0.25) + offset,
            PositionMode::Center,
            glm::vec2(0.45, 0.45),
        )
    }
}

fn get_camera() -> Camera {
    Camera::new(glm::vec2(99.0, 100.0), 0.1)
}

impl<'a> Scene for MainMenu<'a> {
    fn game_logic(&mut self, context: &mut crate::MainContext) {
        let mut camera = get_camera();
        std::mem::swap(&mut context.ui.camera, &mut camera);

        self.logic.update(context);
        self.render.update(&mut self.logic, context);

        for (i, button) in self.buttons.iter_mut().enumerate() {

            if matches!(context.ui.events.mouse_actions, ActionType::AreaPress(id, _) if id == button.id) {
                context.save.on_level_finish((i + 1) as u8);
            }

            button.disabled = i as u8 > context.save.get_last_finished_level();
            button.update(
                &Self::get_button_bbox(i),
                UI_BASE_DEPTH,
                context.get_ui(),
            );
        }

        std::mem::swap(&mut context.ui.camera, &mut camera);
    }

    fn send(&mut self, context: &mut crate::MainContext) {
        let mut camera = get_camera();
        std::mem::swap(&mut context.ui.camera, &mut camera);

        self.render.send(&self.logic, context);
        for button in self.buttons.iter_mut() {
            button.send(&mut context.get_button_context());
        }

        std::mem::swap(&mut context.ui.camera, &mut camera);
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        let camera = get_camera();
        // let camera = context.ui.camera;
        self.render.draw(&camera.get_vp_matrix(&context.ui.get_projection_matrix()));
    }
}
