use std::collections::{HashMap, HashSet};

use crate::{game::{logic::{GameState, computer::MEMORY_SIZE, instruction::Instruction, robot::Robot, target::Targets, world::{Chunk, GridPosition}}, render::{RenderState, UI_BASE_DEPTH, robot_info::{get_button_bgcolor, get_button_color}, targets::get_target_text_color}, scene::Scene}, graphics::renderer::{Renderer, fonts::{TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}}, stable_index_vector::StableIndexVector, ui::{events::ActionType, rendered_button::{Content, RenderedButton}, simple_area_manager::AreaId, ui_element::UiElement}};
use nalgebra_glm as glm;

fn create_first_level() -> GameState {
    let mut robots = StableIndexVector::with_capacity(1);
    robots.push(Robot::new(glm::vec2(1.0, 1.0), [Instruction::default(); MEMORY_SIZE]));

    let mut grid = [[GridPosition::None; _]; _];

    grid[1][0] = GridPosition::Obstacle;
    grid[1][6] = GridPosition::Obstacle;
    for i in 0..7 {
        grid[0][i] = GridPosition::Obstacle;
        grid[2][i] = GridPosition::Obstacle;
    }

    let targets = Targets::new(HashSet::from([glm::vec2(5, 1)]));
    let chunk = Chunk::new(grid,  vec![], robots, targets);

    let mut chunks = HashMap::new();
    chunks.insert(glm::vec2(0, 0), chunk);

    GameState::new(chunks)
}

pub struct FirstLevel<'a> {
    pub logic: GameState,
    pub render: RenderState<'a>,
    pub next_button: RenderedButton,
}

impl<'a> FirstLevel<'a> {
    pub fn new(renderer: &'a Renderer) -> Self {
        let logic = create_first_level();
        let render = RenderState::new(renderer, &logic);
        Self {
            logic,
            render,
            next_button: RenderedButton::new(get_button_bgcolor())
                .with_padding(0.01)
                .with_content(Content::Str("Next"), get_button_color()),
        }
    }

    fn get_next_button_bbox(ndc_bbox: &RenderBox) -> RenderBox {
        RenderBox::new_simple(
            ndc_bbox.get_bottom_right() + glm::vec2(-0.02, 0.02),
            PositionMode::BottomRight,
            glm::vec2(0.6, 0.25),
        )
    }
}

impl<'a> Scene for FirstLevel<'a> {
    fn game_logic(&mut self, context: &mut crate::MainContext) {
        self.logic.update(context);
        self.render.update(&mut self.logic, context, true);
    }

    fn send(&mut self, context: &mut crate::MainContext) {
        let ndc_bbox = context.ui.get_ndc_render_box();
        
        let (reached, total) = self.logic.get_target_counts();
        if reached == total {
            if matches!(context.ui.events.mouse_actions, ActionType::AreaPress(id, _) if id == self.next_button.id) {
                todo!();
            }

            self.next_button.update(&Self::get_next_button_bbox(&ndc_bbox), UI_BASE_DEPTH, context.ui);
            self.next_button.send(&mut context.get_button_context());
        } else {
            self.next_button.id = AreaId::default();
        }

        let ndc_bbox = context.ui.get_ndc_render_box();
        let text = format!("{}/{}", reached, total);
        context.font.add_text(
            &TextRenderConfig::new(
                &[(&text, get_target_text_color())],
                ndc_bbox.get_top_right() - glm::vec2(0.02, 0.02),
                0.15,
            )
            .set_align(TextAlign::Right)
            .set_depth(UI_BASE_DEPTH)
            .set_mode(PositionMode::TopRight),
            context.groups,
        );

        self.render.send(&self.logic, context);
    }

    fn generate_output(&mut self, context: &mut crate::MainContext) {
        self.render.draw(&context.ui.camera.get_vp_matrix(&context.ui.get_projection_matrix()));
    }
}