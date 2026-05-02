
use crate::{MainContext, game::{logic::{computer::MEMORY_SIZE, instruction::{Syscall, UnpackedInstruction}, robot::Robot}, render::{dnd::Dnd, instruction_unit::{INSTRUCTION_HEIGHT, INSTRUCTION_WIDTH, InstructionUnit}, rendered_area::{self, RenderedArea}, rendered_button::RenderedButton}, scene::{EventReceiver, EventType, UiElement}}, graphics::renderer::{fonts::Font, positioning::{PositionMode, RenderBox}}, ui::{AreaManager, Ui, drag_area_manager::DragAreaManager, simple_area::SimpleArea}};
use nalgebra_glm as glm;

const BUTTON_AREA_WIDTH: f32 = 0.4;
const BUTTON_HEIGHT: f32 = 0.15;
const BUTTON_WIDTH: f32 = BUTTON_AREA_WIDTH - 4.0 * INFO_PADDING;
const BUTTON_PADDING: f32 = 0.02;
pub const INFO_WIDTH: f32 = 0.7;
pub const INFO_HEIGHT: f32 = 1.6;
pub const INFO_PADDING: f32 = 0.015;
pub const INFO_MARGIN: f32 = 0.02;

pub fn get_background_color() -> glm::U8Vec4 {
    glm::vec4(15, 17, 26, 255)
}

pub fn get_secondary_background_color() -> glm::U8Vec4 {
    glm::vec4(25, 28, 39, 255)
}

pub fn get_empty_color() -> glm::U8Vec4 {
    glm::vec4(18, 20, 30, 255)
}

pub fn get_hovered_color() -> glm::U8Vec4 {
    glm::vec4(45, 48, 59, 255)
}

#[derive(Debug)]
pub struct RobotInfo {
    dnd: Dnd,
    instructions: Vec<InstructionUnit>,
    add_instruction_button: RenderedButton,
    pub robot_button_id: usize,
    container: RenderedArea<DragAreaManager>,
}

impl RobotInfo {
    pub fn new(ui: &mut Ui, robot_button_id: usize, robot: &Robot, depth: u16) -> Self {
        let top_left = ui.get_ndc_render_box().get_top_left();
        let considered_instructions = robot.computer.instructions
            .iter()
            .filter(|instruction| instruction.unpack() != UnpackedInstruction::Sys(Syscall::Nop));

        let dnd = Dnd::new(
            top_left + Self::get_idx_default_offset(0),
            glm::vec2(INSTRUCTION_WIDTH, INSTRUCTION_HEIGHT),
            INFO_PADDING,
            considered_instructions.count(), ui);

        let instructions = robot.computer.instructions
            .iter()
            .filter(|instruction| instruction.unpack() != UnpackedInstruction::Sys(Syscall::Nop))
            .enumerate()
            .map(|(idx, instruction)| InstructionUnit::new(
                *instruction, ui,
                top_left + Self::get_idx_default_offset(idx),
                dnd.get_drag_box_id(idx),
                depth - 2,
            )).collect();

        Self {
            add_instruction_button:
                RenderedButton::new(
                    Self::get_add_instruction_button_box(top_left),
                    depth - 1,
                    glm::vec4(0x5C, 0x6B, 0xC0, 255),
                    ui,
                )
                .with_color(glm::vec4(255, 255, 255, 255))
                .with_hover_bgcolor(glm::vec4(0x6C, 0x7B, 0xD0, 255))
                .with_padding(BUTTON_PADDING)
                .with_content("ADD"),
            dnd,
            instructions,
            robot_button_id,
            container: RenderedArea::new(
                Self::get_outer_area_render_box(top_left),
                depth,
                get_background_color(),
                &mut ui.drag_area_manager,
            ),
        }
    }

    fn get_add_instruction_button_box(top_left: glm::Vec2) -> RenderBox {
        RenderBox::new_simple(
            top_left + glm::vec2(INFO_WIDTH + INFO_MARGIN + INFO_PADDING * 2.0, -INFO_MARGIN - INFO_PADDING),
            PositionMode::TopLeft,
            glm::vec2(BUTTON_WIDTH, BUTTON_HEIGHT),
        )
    }

    fn get_idx_default_offset(idx: usize) -> glm::Vec2 {
        glm::vec2(
            INFO_MARGIN + INFO_PADDING,
            -(INFO_MARGIN + INFO_PADDING + (INSTRUCTION_HEIGHT + INFO_PADDING) * idx as f32),
        )
    }

    fn get_idx_drag_area(idx: usize, top_left: glm::Vec2) -> RenderBox {
        RenderBox::new_simple(
            top_left + Self::get_idx_default_offset(idx),
            PositionMode::TopLeft,
            glm::vec2(INSTRUCTION_WIDTH, INSTRUCTION_HEIGHT),
        )
    }

    fn get_outer_area_render_box(top_left: glm::Vec2) -> RenderBox {
        RenderBox::new_simple(
            top_left + glm::vec2(INFO_MARGIN, -INFO_MARGIN),
            PositionMode::TopLeft,
            glm::vec2(INFO_WIDTH + BUTTON_AREA_WIDTH, INFO_HEIGHT),
        )
    }
}

impl<'a, 'b> UiElement<MainContext<'a, 'b>, MainContext<'a, 'b>, MainContext<'a, 'b>> for RobotInfo {
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut MainContext<'a, 'b>) {
        self.dnd.top_left_position = bbox.get_top_left() + Self::get_idx_default_offset(0);
        self.dnd.update(context.ui);
        self.container.update(
            &Self::get_outer_area_render_box(bbox.get_top_left()),
            depth,
            &mut context.ui.drag_area_manager,
        );

        self.add_instruction_button.update(
            &Self::get_add_instruction_button_box(bbox.get_top_left()),
            depth - 1,
            context.ui,
        );

        for (i, (bbox, is_grabbed)) in self.dnd.get_unit_positions().enumerate() {
            let depth = if is_grabbed { depth - 12 } else { depth - 2 };
            context.ui.drag_area_manager.update(
                self.instructions[i].get_id(),
                SimpleArea::new(bbox, depth)
            );

            self.instructions[i].update(
                &bbox,
                depth,
                context,
            );
        }
    }

    fn send(&mut self, context: &mut MainContext<'a, 'b>) {
        let top_left = context.ui.get_ndc_render_box().get_top_left();
        for instruction in self.instructions.iter_mut() {
            instruction.send(context);
        }

        if let Some(container) = self.container.get_area(&context.ui.drag_area_manager) {
            for i in 0..MEMORY_SIZE {
                let slot_box = Self::get_idx_drag_area(i, top_left);
                context.simple_shape_renderer.send_render_box(
                    &slot_box,
                    get_empty_color(),
                    container.depth - 1, 1,
                );
            }
        }

        self.add_instruction_button.send(context);

        let mut context = rendered_area::SendContext {
            manager: &context.ui.drag_area_manager,
            simple_shape_renderer: context.simple_shape_renderer,
        };
        self.container.send(&mut context);
    }
}

pub struct RobotInfoEventContext<'a, 'b> {
    pub ui: &'a mut Ui,
    pub font: &'a mut Font<'b>,
}

impl<'a, 'b> EventReceiver<RobotInfoEventContext<'a, 'b>> for RobotInfo {
    fn on_simple_area_pressed(&mut self, id: usize, context: &mut RobotInfoEventContext<'a, 'b>) {
        let ui = &mut context.ui;
        let font = &mut context.font;

        let top_left = ui.get_ndc_render_box().get_top_left();

        if
            id == self.add_instruction_button.get_id() &&
            self.instructions.len() < MEMORY_SIZE &&
            let Some(depth) = self.container.get_area(&ui.drag_area_manager).map(|area| area.depth)
        {
            self.dnd.push(ui);
            self.instructions.push(InstructionUnit::new_blank(
                ui,
                font,
                top_left + Self::get_idx_default_offset(self.instructions.len()),
                self.dnd.get_drag_box_id(self.dnd.len() - 1),
                depth - 2
            ));
        }
        
        if let Some(instruction_to_remove) = self.instructions
            .iter()
            .enumerate()
            .find(|(_, instruction)| instruction.delete_button.get_id() == id)
            .map(|(i, _)| i)
        {
            self.dnd.remove_original(instruction_to_remove);
            self.instructions.remove(instruction_to_remove);
        }
    }

    fn propagate_children(&mut self, event: &EventType, context: &mut RobotInfoEventContext<'a, 'b>) {
        for instruction in self.instructions.iter_mut() {
            instruction.on_event(event, context.ui);
        }

        self.dnd.on_event(event, &mut ());
        self.add_instruction_button.on_event(event,&mut ());
    }
}
