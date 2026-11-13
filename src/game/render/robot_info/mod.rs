use crate::{game::{logic::top_level::{computer::MEMORY_SIZE, memory::PROGRAM_COUNTER_IDX, robot::RobotLogic}, render::robot_info::register::Register}, graphics::renderer::{Renderer, fonts::font::Font, positioning::{PositionMode, RenderBox}}, ui::{Ui, UiElement, UiNode, UiState, layout::Layout, rect::Rect, table::Table}};
use nalgebra_glm as glm;

pub mod instruction;
pub mod register;

const UI_BASE_DEPTH: u16 = 50000;
const LINES_SPACING: f32 = 0.01;
const ITEM_HEIGHT: f32 = 0.06;
const ITEM_PADDING: f32 = 0.005;
const ITEM_WIDTH: f32 = 0.5;

pub fn get_background_color() -> glm::U8Vec4 {
    glm::vec4(8, 17, 25, 255)
}

pub fn get_secondary_background_color() -> glm::U8Vec4 {
    glm::vec4(38, 47, 55, 255)
}

pub fn get_instruction_being_executed_color() -> glm::U8Vec4 {
    glm::vec4(38, 47, 100, 255)
}

pub fn get_reg_separator_color() -> glm::U8Vec4 {
    glm::vec4(188, 197, 55, 255)
}

#[derive(Debug)]
pub struct InstructionBox {
    idx: usize,
}

impl InstructionBox {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

impl UiElement for InstructionBox {
    fn update(&mut self,
        parent_bbox: &RenderBox,
        parent_depth: u16,
        ctx: &mut UiState,
    ) {
        let pc_value = ctx.watched_robot.as_ref()
            .map(|robot| robot.computer.get_data().get(PROGRAM_COUNTER_IDX))
            .flatten()
            .cloned()
            .map(|pc| pc % MEMORY_SIZE as u8);

        let color = if Some(self.idx as u8) == pc_value {
            get_instruction_being_executed_color()
        } else {
            get_secondary_background_color()
        };

        let depth = parent_depth - 1;
        ctx.renderer.simple_shape_renderer.send_render_box(
            parent_bbox,
            color,
            depth,
            1,
        );

        if let Some(mut inst) = ctx.watched_robot.as_ref()
            .map(|robot| robot.computer.instructions.get().get(self.idx))
            .flatten()
            .cloned()
        {
            let depth = depth - 1;
            inst.update(parent_bbox, depth, ctx);
        }
    }
}

pub struct RobotInfo {
    robot_idx: usize,
    root: Rect,
}

impl RobotInfo {
    pub fn new(robot_idx: usize, ui: &mut Ui) -> Self {
        let registers = (0..MEMORY_SIZE)
            .into_iter()
            .map(|i| UiNode::Register(Register::new(i)))
            .collect();

        let reg_table = Table::new(
            2,
            glm::vec2(0.09, 0.06),
            0.005,
            get_reg_separator_color(),
            registers, ui,
        )
            .with_general_anchor(PositionMode::BottomLeft)
            .with_offset(glm::vec2(0.005, 0.005));

        let instructions = (0..MEMORY_SIZE)
            .into_iter()
            .map(|i|
                UiNode::InstructionBox(InstructionBox::new(i)))
            .collect();   

        let inst_table = Table::new(
            16,
            glm::vec2(ITEM_WIDTH, ITEM_HEIGHT),
            LINES_SPACING,
            glm::vec4(0, 0, 0, 0),
            instructions,
            ui
        );

        Self {
            robot_idx,
            root: Rect::new(
                Layout::new(glm::vec2(1.0, 1.6))
                    .with_offset(glm::vec2(0.015, -0.015)),
                get_background_color(),
                ui.new_id(),
                vec![
                    UiNode::Table(reg_table),
                    UiNode::Table(inst_table),
                ],
            ),
        }
    }

    pub fn get_robot_idx(&self) -> usize {
        self.robot_idx
    }

    pub fn update(
        &mut self,
        ui: &mut Ui,
        renderer: &mut Renderer,
        font: &mut Font,
        watched_robots: &[RobotLogic],
    ) {
        let screen_bbox = ui.get_screen_bbox();
        let mut ctx = UiState {
            renderer, ui, font,
            watched_robot: watched_robots.get(0).cloned(),
        };
        self.root.update(&screen_bbox, UI_BASE_DEPTH, &mut ctx);
    }
}
