
use crate::{game::{logic::computer::MEMORY_SIZE, render::UI_BASE_DEPTH}, graphics::renderer::{positioning::{PositionMode, RenderBox}, simple_shape_renderer::SimpleShapeRenderer}, ui::{Ui, drag_area::DragArea}};
use nalgebra_glm as glm;

const INFO_WIDTH: f32 = 0.8;
const INFO_HEIGHT: f32 = 1.8;
const INFO_PADDING: f32 = 0.02;
const INFO_MARGIN: f32 = 0.03;
const INSTRUCTION_WIDTH: f32 = INFO_WIDTH - 2.0 * INFO_PADDING;
const INSTRUCTION_HEIGHT: f32 = (INFO_HEIGHT - INFO_PADDING * (MEMORY_SIZE + 1) as f32) / MEMORY_SIZE as f32;

#[derive(Debug, Default)]
pub struct RobotInfo {
    instruction_grab_boxes: [usize; MEMORY_SIZE],
    previous_robot_info_displayed: Option<usize>,
}

impl RobotInfo {
    pub fn send(&self, simple_shape_renderer: &mut SimpleShapeRenderer, ui: &Ui) {
        if self.previous_robot_info_displayed.is_none() { return }

        let top_left = ui.get_ndc_render_box().get_top_left();
        for id in self.instruction_grab_boxes.iter() {
            if let Some(grab_box) = ui.get_drag_area(*id) {
                let render_box = grab_box.get_current_position();
                simple_shape_renderer.send_render_box(
                    &render_box,
                    glm::vec4(200, 180, 50, 255),
                    UI_BASE_DEPTH - 1, 1,
                );
            }
        }
        
        simple_shape_renderer.send_rectangle(
            top_left + glm::vec2(INFO_MARGIN, -INFO_MARGIN),
            glm::vec2(INFO_WIDTH, INFO_HEIGHT),
            PositionMode::TopLeft,
            glm::vec4(20, 20, 40, 255),
            UI_BASE_DEPTH, 1,
        );
    }

    pub fn update(&mut self, robot_info_opened: Option<usize>, ui: &mut Ui) {
        if self.previous_robot_info_displayed != robot_info_opened {
            if self.previous_robot_info_displayed.is_some() {
                for id in self.instruction_grab_boxes {
                    ui.remove_drag_area(id);
                }
            }

            if robot_info_opened.is_some() {
                let top_left = ui.get_ndc_render_box().get_top_left() + glm::vec2(INFO_MARGIN + INFO_PADDING, -INFO_MARGIN - INFO_PADDING);
                for (i, id) in self.instruction_grab_boxes.iter_mut().enumerate() {
                    *id = ui.push_drag_area(DragArea::new(
                        RenderBox::new_simple(
                            top_left - glm::vec2(0.0, (INSTRUCTION_HEIGHT + INFO_PADDING) * i as f32),
                            PositionMode::TopLeft,
                            glm::vec2(INSTRUCTION_WIDTH, INSTRUCTION_HEIGHT)
                        ),
                    ).lock_x());
                }
            }

            self.previous_robot_info_displayed = robot_info_opened;
        }
    }
}
