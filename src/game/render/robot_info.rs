use std::array;

use winit::window::CursorIcon;
use nalgebra_glm as glm;
use crate::{MainContext, game::{logic::{computer::MEMORY_SIZE, instruction::{Instruction, Syscall, UnpackedInstruction}, memory::PROGRAM_COUNTER_IDX}, render::{INFO_HEIGHT, INFO_WIDTH, go_back_arrow, instruction_editor::{InstructionEditor, PADDING}, instruction_view::InstructionView, memory_visualizer::{self, SpecialHighlight}}}, graphics::renderer::positioning::{PositionMode, RenderBox}, ui::{dnd::Dnd, events::ActionType, rendered_area::RenderedArea, rendered_button::{Content, RenderedButton}, ui_element::UiElement}};

pub fn get_background_color() -> glm::U8Vec4 {
    glm::vec4(8, 17, 25, 255)
}

pub fn get_secondary_background_color() -> glm::U8Vec4 {
    glm::vec4(38, 47, 55, 255)
}

pub fn get_button_bgcolor() -> glm::U8Vec4 {
    glm::vec4(0x5C, 0x6B, 0xC0, 255)
}

pub fn get_button_color() -> glm::U8Vec4 {
    glm::vec4(255, 255, 255, 255)
}

pub fn get_list_bgcolor() -> glm::U8Vec4 {
    glm::vec4(68, 77, 85, 255)
}

const CONTAINER_PADDING: f32 = 0.02;
const BUTTON_PADDING: f32 = 0.03;
const BUTTTON_AREA_WIDTH: f32 = 0.4;
const BUTTON_HEIGHT: f32 = 0.15;
const BUTTON_WIDTH: f32 = BUTTTON_AREA_WIDTH - 2.0 * CONTAINER_PADDING;
const INSTRUCTION_WIDTH: f32 = INFO_WIDTH - BUTTTON_AREA_WIDTH - 2.0 * CONTAINER_PADDING;
const INSTRUCTION_HEIGHT: f32 = (INFO_HEIGHT - (MEMORY_SIZE + 1) as f32 * CONTAINER_PADDING) / MEMORY_SIZE as f32;
const ICON_BUTTON_SIZE: f32 = 0.1;

fn get_button_bbox(bbox: &RenderBox, i: usize) -> RenderBox {
    RenderBox::new_simple(
        bbox.get_top_right() + glm::vec2(-CONTAINER_PADDING, -CONTAINER_PADDING - (CONTAINER_PADDING + BUTTON_HEIGHT) * i as f32),
        PositionMode::TopRight,
        glm::vec2(BUTTON_WIDTH, BUTTON_HEIGHT),
    )
}

fn get_back_button_bbox(bbox: &RenderBox) -> RenderBox {
    RenderBox::new_simple(
        bbox.get_top_left() + glm::vec2(CONTAINER_PADDING, -CONTAINER_PADDING),
        PositionMode::TopLeft,
        glm::vec2(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE),
    )
}

#[derive(Debug)]
struct RobotEdit {
    finish_button_pressed: bool,
    add_instruction_button: RenderedButton,
    finish_button: RenderedButton,
    instructions: Dnd<InstructionEditor>,
    original_instructions: [Instruction; MEMORY_SIZE],
}

impl RobotEdit {
    fn new(instructions: [Instruction; MEMORY_SIZE]) -> Self {
        let mut dnd = Dnd::new(glm::vec2(INSTRUCTION_WIDTH, INSTRUCTION_HEIGHT))
            .with_gap(CONTAINER_PADDING);

        for instruction in instructions {
            let unpacked = instruction.unpack();
            if unpacked == UnpackedInstruction::Sys(Syscall::Nop) {
                continue;
            }

            dnd.push(InstructionEditor::from_instruction(&unpacked));
        }

        Self {
            add_instruction_button: RenderedButton::new(get_button_bgcolor())
                .with_content(Content::Str("ADD"), get_button_color())
                .with_padding(BUTTON_PADDING),
            finish_button: RenderedButton::new(get_button_bgcolor())
                .with_content(Content::Str("FINISH"), get_button_color())
                .with_padding(BUTTON_PADDING),
            instructions: dnd,
            finish_button_pressed: false,
            original_instructions: instructions,
        }
    }

    fn get_code(&self) -> Option<[Instruction; MEMORY_SIZE]> {
        let mut instructions = [Instruction::default(); MEMORY_SIZE];

        for (i, inst) in instructions.iter_mut().enumerate() {
            let editor = self.instructions.get_elements().get(i);
            *inst = match editor {
                Some(editor) => editor.get_instruction()?,
                None => Default::default(),
            };
        }

        Some(instructions)
    }

    fn get_dnd_bbox(bbox: &RenderBox) -> RenderBox {
        RenderBox::new_simple(
            bbox.get_top_left() + glm::vec2(CONTAINER_PADDING, - 2.0 * CONTAINER_PADDING - ICON_BUTTON_SIZE),
            PositionMode::TopLeft,
            glm::vec2(INFO_WIDTH - BUTTON_WIDTH - 2.0 * CONTAINER_PADDING, INFO_HEIGHT - 2.0 * CONTAINER_PADDING),
        )
    }
}

impl<'a, 'b> UiElement<MainContext<'a, 'b>, MainContext<'a, 'b>, ()> for RobotEdit {
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut MainContext<'a, 'b>) {
        self.finish_button_pressed = false;
        if let ActionType::AreaPress(id, _) = context.ui.events.mouse_actions {
            if id == self.add_instruction_button.id && self.instructions.len() < MEMORY_SIZE {
                self.instructions.push(InstructionEditor::new());
            } else if let Some(idx) = self.instructions
                .get_elements()
                .iter()
                .enumerate()
                .find(|(_, el)| el.get_delete_button_id() == id)
                .map(|(idx, _)| idx)
            {
                self.instructions.remove(idx);
            } else if id == self.finish_button.id {
                self.finish_button_pressed = true;
            }
        }

        self.instructions.update(&Self::get_dnd_bbox(bbox), depth - 1, context);
        
        self.finish_button.disabled = self.get_code().is_none();
        self.finish_button.update(
            &get_button_bbox(bbox, 1),
            depth - 1, context.ui,
        );

        self.add_instruction_button.update(
            &get_button_bbox(bbox, 0),
            depth - 1, context.ui,
        );
    }

    fn send(&mut self, context: &mut MainContext<'a, 'b>) {
        self.instructions.send(context);
        self.add_instruction_button.send(&mut context.get_button_context());
        self.finish_button.send(&mut context.get_button_context());
    }
}

#[derive(Debug)]
struct RobotView {
    instructions: [InstructionView; MEMORY_SIZE],
    edit_button: RenderedButton,
}

impl RobotView {
    fn new(instructions: [Instruction; MEMORY_SIZE]) -> Self {
        Self {
            instructions: array::from_fn(|i| InstructionView::new(instructions[i])),
            edit_button: RenderedButton::new(get_button_bgcolor())
                .with_content(Content::Str("EDIT"), get_button_color())
                .with_padding(BUTTON_PADDING),
        }
    }

    fn get_instruction_bbox(bbox: &RenderBox, i: usize) -> RenderBox {
        RenderBox::new_simple(
            bbox.get_top_left() + glm::vec2(CONTAINER_PADDING, - 2.0 * CONTAINER_PADDING - ICON_BUTTON_SIZE - (INSTRUCTION_HEIGHT + CONTAINER_PADDING) * i as f32),
            PositionMode::TopLeft,
            glm::vec2(INSTRUCTION_WIDTH, INSTRUCTION_HEIGHT),
        )
    }

    fn update_execution_state(&mut self, executed_instruction: usize) {
        for (i, inst) in self.instructions.iter_mut().enumerate() {
            inst.update_execution_state(i == executed_instruction);
        }
    }
}

impl<'a, 'b> UiElement<MainContext<'a, 'b>, MainContext<'a, 'b>, ()> for RobotView {
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut MainContext<'a, 'b>) {
        self.edit_button.update(&get_button_bbox(bbox, 0), depth - 1, context.ui);

        for (i, inst) in self.instructions.iter_mut().enumerate() {
            inst.update(&Self::get_instruction_bbox(bbox, i), depth - 1, context);
        }
    }

    fn send(&mut self, context: &mut MainContext<'a, 'b>) {
        self.edit_button.send(&mut context.get_button_context());
        for inst in self.instructions.iter_mut() {
            inst.send(context);
        }
    }
}

#[derive(Debug)]
enum RobotInfoMode {
    View(RobotView),
    Edit(RobotEdit),
}

#[derive(Debug)]
pub struct RobotInfo {
    pub robot_id: u32,
    outer_container: RenderedArea,
    mode: RobotInfoMode,
    back_button: RenderedButton,
}

impl RobotInfo {
    pub fn new(robot_id: u32, instructions: [Instruction; MEMORY_SIZE]) -> Self {
        Self {
            robot_id,
            outer_container: RenderedArea::new(CursorIcon::Default, get_background_color()),
            mode: RobotInfoMode::View(RobotView::new(instructions)),
            back_button: RenderedButton::new(get_button_bgcolor())
                .with_content(Content::Icon(go_back_arrow::send), get_button_color())
                .with_padding(BUTTON_PADDING),
        }
    }

    pub fn finished(&self) -> Option<[Instruction; MEMORY_SIZE]> {
        if let RobotInfoMode::Edit(edit) = &self.mode && edit.finish_button_pressed {
            edit.get_code()
        } else {
            None
        }
    }

    fn get_mem_bbox(bbox: &RenderBox) -> RenderBox {
        RenderBox::new_simple(
            bbox.get_bottom_right() + glm::vec2(-PADDING, PADDING),
            PositionMode::BottomRight,
            glm::vec2(BUTTTON_AREA_WIDTH - PADDING * 2.0, 0.3),
        )
    }
}

impl UnpackedInstruction {
    fn get_highlights(&self) -> SpecialHighlight {
        let mut highlight = SpecialHighlight {
            r1: None,
            r2: None,
            rd: None,
        };

        match self {
            UnpackedInstruction::Lui(rd, _) => {
                highlight.rd = Some(*rd);
            }

            UnpackedInstruction::Addi(rd, rs, _) | 
            UnpackedInstruction::Subi(rd, rs, _) => {
                highlight.rd = Some(*rd);
                highlight.r1 = Some(*rs);
            }

            UnpackedInstruction::Add(rd, r1, r2) |
            UnpackedInstruction::Sub(rd, r1, r2) |
            UnpackedInstruction::Mul(rd, r1, r2) |
            UnpackedInstruction::Div(rd, r1, r2) |
            UnpackedInstruction::Rem(rd, r1, r2) |
            UnpackedInstruction::Seq(rd, r1, r2) |
            UnpackedInstruction::Slt(rd, r1, r2) |
            UnpackedInstruction::Sle(rd, r1, r2) => {
                highlight.rd = Some(*rd);
                highlight.r1 = Some(*r1);
                highlight.r2 = Some(*r2);
            }

            UnpackedInstruction::Beq(r1, r2, _) |
            UnpackedInstruction::Bne(r1, r2, _) |
            UnpackedInstruction::Blt(r1, r2, _) |
            UnpackedInstruction::Ble(r1, r2, _) => {
                highlight.r1 = Some(*r1);
                highlight.r2 = Some(*r2);
            }

            UnpackedInstruction::Sys(_) => {}
        }

        highlight
    }
}

pub struct RobotInfoContext<'a, 'b, 'c> {
    pub mc: &'c mut MainContext<'a, 'b>,
    pub data: &'c [u8; MEMORY_SIZE],
}

impl<'a, 'b, 'c> UiElement<RobotInfoContext<'a, 'b, 'c>, RobotInfoContext<'a, 'b, 'c>, ()> for RobotInfo {
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut RobotInfoContext) {
        match &mut self.mode {
            RobotInfoMode::Edit(edit) => {
                if matches!(context.mc.ui.events.mouse_actions, ActionType::AreaPress(id, _) if id == self.back_button.id) {
                    self.mode = RobotInfoMode::View(RobotView::new(edit.original_instructions));
                }
            },
            RobotInfoMode::View(view) => {
                if matches!(context.mc.ui.events.mouse_actions, ActionType::AreaPress(id, _) if id == view.edit_button.id) {
                    let instructions: [_; MEMORY_SIZE] = array::from_fn(|i| view.instructions[i].get_instruction());
                    self.mode = RobotInfoMode::Edit(RobotEdit::new(instructions));
                }
            },
        }

        self.back_button.disabled = matches!(self.mode, RobotInfoMode::View(_));
        match &mut self.mode {
            RobotInfoMode::Edit(edit) => {
                edit.update(bbox, depth, context.mc);
            },
            RobotInfoMode::View(view) => {
                view.update(bbox, depth, context.mc);
                view.update_execution_state((context.data[PROGRAM_COUNTER_IDX] as usize) % MEMORY_SIZE);
            }
        }
        self.outer_container.update(bbox, depth, context.mc.ui);
        self.back_button.update(&get_back_button_bbox(bbox), depth - 1, context.mc.ui);
    }

    fn send(&mut self, context: &mut RobotInfoContext<'a, 'b, 'c>) {
        let mut main_context = &mut context.mc;
        self.back_button.send(&mut main_context.get_button_context());

        let executed_instruction = (context.data[PROGRAM_COUNTER_IDX] & 0xf) as usize;
        let instruction = match &self.mode {
            RobotInfoMode::Edit(edit) => edit.original_instructions[executed_instruction],
            RobotInfoMode::View(view) => view.instructions[executed_instruction].get_instruction(),
        }.unpack();

        memory_visualizer::send(
            main_context.simple_shape_renderer,
            main_context.font,
            main_context.groups,
            context.data,
            &Self::get_mem_bbox(&self.outer_container.bbox),
            self.outer_container.depth - 1,
            instruction.get_highlights(),
        );

        match &mut self.mode {
            RobotInfoMode::Edit(edit) => edit.send(&mut main_context),
            RobotInfoMode::View(view) => view.send(&mut main_context),
        }
        self.outer_container.send(&mut (main_context.simple_shape_renderer, main_context.groups));
    }
}
