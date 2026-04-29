use crate::{MainContext, game::{logic::{computer::MEMORY_SIZE, instruction::{Instruction, InstructionFormat, InstructionId, RegisterAddress, Syscall, UnpackedInstruction}}, render::{UI_BASE_DEPTH, menu::Menu, robot_info::{INFO_HEIGHT, INFO_PADDING, INFO_WIDTH, get_secondary_background_color}}, scene::EventType}, graphics::renderer::{fonts::{Font, TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}}, ui::{Ui, simple_area::SimpleArea}};
use nalgebra_glm as glm;

pub const INSTRUCTION_WIDTH: f32 = INFO_WIDTH - 2.0 * INFO_PADDING;
pub const INSTRUCTION_HEIGHT: f32 = (INFO_HEIGHT - INFO_PADDING * (MEMORY_SIZE + 1) as f32) / MEMORY_SIZE as f32;
const INSTRUCTION_PADDING: f32 = 0.01;
const DELETE_BUTTON_SIZE: f32 = INSTRUCTION_HEIGHT - 2.0 * INSTRUCTION_PADDING;


#[derive(Debug)]
pub enum OperandState {
    RdImm8 { rd: Menu<RegisterAddress>, imm: Menu<u8> },
    RsRdImm4 { rd: Menu<RegisterAddress>, rs: Menu<RegisterAddress>, imm: Menu<u8> },
    R1R2Rd { rd: Menu<RegisterAddress>, r1: Menu<RegisterAddress>, r2: Menu<RegisterAddress> },
    R1R2Imm4 { r1: Menu<RegisterAddress>, r2: Menu<RegisterAddress>, imm: Menu<u8> },
    Sys { sys: Menu<Syscall> },
}

#[derive(Debug)]
pub struct UnfinishedInstruction {
    opcode: Menu<InstructionId>,
    operands: Option<OperandState>,
}

impl UnfinishedInstruction {
    const WIDTH_OPCODE: f32 = 4.0;
    const WIDTH_REG: f32 = 3.0;
    const WIDTH_IMM: f32 = 3.0;
    const WIDTH_SYS: f32 = 10.0;
    const CHAR_MARGIN: f32 = 1.0;

    fn new(ui: &mut Ui, delete_button_bbox: &RenderBox, font: &Font) -> Self {
        Self {
            opcode: Menu::new(
                Self::get_opcode_bbox(delete_button_bbox, font, delete_button_bbox.get_size().y),
                glm::vec2(0.4, delete_button_bbox.get_size().y),
                (0..16)
                    .map(|id| (InstructionId::from_u16(id), UnpackedInstruction::get_mnemonic_color()))
                    .collect(),
                ui,
            ),
            operands: None,
        }
    }

    // --- Métodos de BBox (Mantidos como você definiu) ---

    fn get_opcode_bbox(delete_button_bbox: &RenderBox, font: &Font, height: f32) -> RenderBox {
        let char_len = font.get_monospace_width(height);
        RenderBox::new_simple(
            delete_button_bbox.get_top_right() + glm::vec2(2.0 * INSTRUCTION_PADDING, 0.0),
            PositionMode::TopLeft,
            glm::vec2(Self::WIDTH_OPCODE * char_len, height),
        )
    }

    fn get_operand_bboxes(delete_button_bbox: &RenderBox, format: InstructionFormat, font: &Font, height: f32) -> [RenderBox; 3] {
        let char_len = font.get_monospace_width(height);
        let mut current_pos = Self::get_opcode_bbox(delete_button_bbox, font, height).get_top_right() + glm::vec2(Self::CHAR_MARGIN * char_len, 0.0);
        let mut bboxes = [RenderBox::default(); 3];
        let widths = match format {
            InstructionFormat::RdImm8    => [Self::WIDTH_REG, Self::WIDTH_IMM, 0.0],
            InstructionFormat::RsRdImm4  => [Self::WIDTH_REG, Self::WIDTH_REG, Self::WIDTH_IMM],
            InstructionFormat::R1R2Rd    => [Self::WIDTH_REG, Self::WIDTH_REG, Self::WIDTH_REG],
            InstructionFormat::R1R2Imm4  => [Self::WIDTH_REG, Self::WIDTH_REG, Self::WIDTH_IMM],
            InstructionFormat::Syscall   => [Self::WIDTH_SYS, 0.0, 0.0],
        };

        for i in 0..3 {
            let w = widths[i];
            if w > 0.0 {
                bboxes[i] = RenderBox::new_simple(current_pos, PositionMode::TopLeft, glm::vec2(w * char_len, height));
                current_pos.x += (w + Self::CHAR_MARGIN) * char_len;
            }
        }
        bboxes
    }

    // --- Lógica de Update e Gerenciamento ---

    pub fn update(&mut self, ui: &mut Ui, delete_button_bbox: &RenderBox, font: &Font) {
        let height = delete_button_bbox.get_size().y;
        self.opcode.update(Self::get_opcode_bbox(delete_button_bbox, font, height).get_top_left(), ui);

        if let Some(op_id) = self.opcode.get_selected_value() {
            let format = op_id.get_format();
            let bboxes = Self::get_operand_bboxes(delete_button_bbox, format, font, height);
            let entry_size = glm::vec2(0.3, height);

            self.manage_operand_menus(ui, format, bboxes, entry_size);
        } else {
            self.operands = None; // Se não tem opcode selecionado, limpa tudo
        }
    }

    fn manage_operand_menus(&mut self, ui: &mut Ui, format: InstructionFormat, bboxes: [RenderBox; 3], entry_size: glm::Vec2) {
        let d_col = UnpackedInstruction::get_dest_color();
        let r1_col = UnpackedInstruction::get_reg1_color();
        let r2_col = UnpackedInstruction::get_reg2_color();
        let l_col = UnpackedInstruction::get_literal_color();
        let reg_opts = |c| (0..MEMORY_SIZE).map(|i| (RegisterAddress::new(i as u8), c)).collect();

        match format {
            InstructionFormat::RdImm8 => {
                if let Some(OperandState::RdImm8 { rd, imm }) = &mut self.operands {
                    rd.update(bboxes[0].get_top_left(), ui);
                    imm.update(bboxes[1].get_top_left(), ui);
                } else {
                    self.operands = Some(OperandState::RdImm8 {
                        rd: Menu::new(bboxes[0], entry_size, reg_opts(d_col), ui),
                        imm: Menu::new(bboxes[1], entry_size, (0..256).map(|i|(i as u8, l_col)).collect(), ui),
                    });
                }
            }
            InstructionFormat::RsRdImm4 => {
                if let Some(OperandState::RsRdImm4 { rd, rs, imm }) = &mut self.operands {
                    rd.update(bboxes[0].get_top_left(), ui);
                    rs.update(bboxes[1].get_top_left(), ui);
                    imm.update(bboxes[2].get_top_left(), ui);
                } else {
                    self.operands = Some(OperandState::RsRdImm4 {
                        rd: Menu::new(bboxes[0], entry_size, reg_opts(d_col), ui),
                        rs: Menu::new(bboxes[1], entry_size, reg_opts(r1_col), ui),
                        imm: Menu::new(bboxes[2], entry_size, (0..16).map(|i|(i as u8, l_col)).collect(), ui),
                    });
                }
            }
            InstructionFormat::R1R2Rd => {
                if let Some(OperandState::R1R2Rd { rd, r1, r2 }) = &mut self.operands {
                    rd.update(bboxes[0].get_top_left(), ui);
                    r1.update(bboxes[1].get_top_left(), ui);
                    r2.update(bboxes[2].get_top_left(), ui);
                } else {
                    self.operands = Some(OperandState::R1R2Rd {
                        rd: Menu::new(bboxes[0], entry_size, reg_opts(d_col), ui),
                        r1: Menu::new(bboxes[1], entry_size, reg_opts(r1_col), ui),
                        r2: Menu::new(bboxes[2], entry_size, reg_opts(r2_col), ui),
                    });
                }
            }
            InstructionFormat::R1R2Imm4 => {
                if let Some(OperandState::R1R2Imm4 { r1, r2, imm }) = &mut self.operands {
                    r1.update(bboxes[0].get_top_left(), ui);
                    r2.update(bboxes[1].get_top_left(), ui);
                    imm.update(bboxes[2].get_top_left(), ui);
                } else {
                    self.operands = Some(OperandState::R1R2Imm4 {
                        r1: Menu::new(bboxes[0], entry_size, reg_opts(r1_col), ui),
                        r2: Menu::new(bboxes[1], entry_size, reg_opts(r2_col), ui),
                        imm: Menu::new(bboxes[2], entry_size, (0..16).map(|i|(i as u8, l_col)).collect(), ui),
                    });
                }
            }
            InstructionFormat::Syscall => {
                if let Some(OperandState::Sys { sys }) = &mut self.operands {
                    sys.update(bboxes[0].get_top_left(), ui);
                } else {
                    let opts = vec![Syscall::Nop, Syscall::MoveLeft, Syscall::MoveRight, Syscall::MoveTop, Syscall::MoveBottom, Syscall::Mine, Syscall::Display, Syscall::Scan, Syscall::Stored, Syscall::MaxStorage]
                        .into_iter().map(|s|(s, l_col)).collect();
                    self.operands = Some(OperandState::Sys { sys: Menu::new(bboxes[0], entry_size, opts, ui) });
                }
            }
        }
    }

    // --- Delegação de Eventos ---

    pub fn send(&mut self, context: &mut MainContext) {
        self.opcode.send(context);
        match &mut self.operands {
            Some(OperandState::RdImm8 { rd, imm }) => { rd.send(context); imm.send(context); }
            Some(OperandState::RsRdImm4 { rd, rs, imm }) => { rd.send(context); rs.send(context); imm.send(context); }
            Some(OperandState::R1R2Rd { rd, r1, r2 }) => { rd.send(context); r1.send(context); r2.send(context); }
            Some(OperandState::R1R2Imm4 { r1, r2, imm }) => { r1.send(context); r2.send(context); imm.send(context); }
            Some(OperandState::Sys { sys }) => { sys.send(context); }
            None => {}
        }
    }

    pub fn receive_hover_event(&mut self, id: usize) {
        self.opcode.receive_hover_event(id);
        match &mut self.operands {
            Some(OperandState::RdImm8 { rd, imm }) => { rd.receive_hover_event(id); imm.receive_hover_event(id); }
            Some(OperandState::RsRdImm4 { rd, rs, imm }) => { rd.receive_hover_event(id); rs.receive_hover_event(id); imm.receive_hover_event(id); }
            Some(OperandState::R1R2Rd { rd, r1, r2 }) => { rd.receive_hover_event(id); r1.receive_hover_event(id); r2.receive_hover_event(id); }
            Some(OperandState::R1R2Imm4 { r1, r2, imm }) => { r1.receive_hover_event(id); r2.receive_hover_event(id); imm.receive_hover_event(id); }
            Some(OperandState::Sys { sys }) => { sys.receive_hover_event(id); }
            None => {}
        }
    }

    pub fn receive_simple_area_click(&mut self, id: usize, ui: &mut Ui) {
        self.opcode.receive_simple_area_click(id, ui);
        match &mut self.operands {
            Some(OperandState::RdImm8 { rd, imm }) => { rd.receive_simple_area_click(id, ui); imm.receive_simple_area_click(id, ui); }
            Some(OperandState::RsRdImm4 { rd, rs, imm }) => { rd.receive_simple_area_click(id, ui); rs.receive_simple_area_click(id, ui); imm.receive_simple_area_click(id, ui); }
            Some(OperandState::R1R2Rd { rd, r1, r2 }) => { rd.receive_simple_area_click(id, ui); r1.receive_simple_area_click(id, ui); r2.receive_simple_area_click(id, ui); }
            Some(OperandState::R1R2Imm4 { r1, r2, imm }) => { r1.receive_simple_area_click(id, ui); r2.receive_simple_area_click(id, ui); imm.receive_simple_area_click(id, ui); }
            Some(OperandState::Sys { sys }) => { sys.receive_simple_area_click(id, ui); }
            None => {}
        }
    }
}

#[derive(Debug)]
pub enum InstructionConfig {
    FinishedInstruction(Instruction),
    UnfinishedInstruction(UnfinishedInstruction),
}

#[derive(Debug)]
pub struct InstructionUnit {
    config: InstructionConfig,
    pub delete_button_id: usize,
    drag_area_id: usize,
    hovering_instruction: bool,
    hovering_delete_button: bool,
}

impl InstructionUnit {
    pub fn new(instruction: Instruction, ui: &mut Ui, unit_top_left_position: glm::Vec2, drag_area_id: usize) -> Self {
        Self {
            config: InstructionConfig::FinishedInstruction(instruction),
            delete_button_id: ui.simple_area_manager.push_simple_area(SimpleArea::new_button(
                Self::get_delete_button_box(unit_top_left_position),
                UI_BASE_DEPTH - 2,
            )),
            hovering_delete_button: false,
            hovering_instruction: false,
            drag_area_id,
        }
    }

    pub fn new_blank(ui: &mut Ui, font: &Font, unit_top_left_position: glm::Vec2, drag_area_id: usize) -> Self {
        Self {
            config: InstructionConfig::UnfinishedInstruction(UnfinishedInstruction::new(
                ui,
                &Self::get_menu_button_box(Self::get_delete_button_box(unit_top_left_position).get_top_left()),
                font,
            )),
            delete_button_id: ui.simple_area_manager.push_simple_area(SimpleArea::new_button(
                Self::get_delete_button_box(unit_top_left_position),
                UI_BASE_DEPTH - 2,
            )),
            hovering_delete_button: false,
            hovering_instruction: false,
            drag_area_id,
        }
    }

    pub fn receive_hover_event(&mut self, event: &EventType) {
        match event {
            EventType::HoverDragArea(id) => {
                if *id == self.drag_area_id {
                    self.hovering_instruction = true;
                }
            },
            EventType::HoverSimpleArea(id) => {
                if *id == self.delete_button_id {
                    self.hovering_delete_button = true;
                } else if let InstructionConfig::UnfinishedInstruction(inst) = &mut self.config {
                    inst.receive_hover_event(*id);
                }
            },
            _ => {}
        }
    }

    pub fn update(&mut self, unit_top_left_position: glm::Vec2, ui: &mut Ui, font: &Font) {
        ui.simple_area_manager.update_simple_area(
            self.delete_button_id,
            SimpleArea::new_button(
                Self::get_delete_button_box(unit_top_left_position),
                UI_BASE_DEPTH - 2,
            ),
        );

        if let InstructionConfig::UnfinishedInstruction(inst) = &mut self.config {
            inst.update(
                ui,
                &Self::get_delete_button_box(unit_top_left_position),
                font,
            );
        }
    }

    pub fn send(
        &mut self,
        context: &mut MainContext,
        is_grabbed: bool,
        bbox: &RenderBox,
    ) {
        let extra_depth = if is_grabbed { 10 } else { 0 };
        let unit_top_left_position = bbox.get_top_left();

        let delete_button_color = if self.hovering_delete_button {
            glm::vec4(230, 80, 80, 255)
        } else if self.hovering_instruction {
            glm::vec4(200, 50, 50, 255)
        } else {
            glm::vec4(5, 8, 19, 255)
        };

        self.hovering_delete_button = false;
        self.hovering_instruction = false;

        if let InstructionConfig::UnfinishedInstruction(inst) = &mut self.config {
            inst.send(context);
        }

        let delete_button_bbox = Self::get_delete_button_box(unit_top_left_position);
        context.simple_shape_renderer.send_render_box(
            &delete_button_bbox,
            delete_button_color,
            UI_BASE_DEPTH - 5 - extra_depth,
            1,
        );

        context.simple_shape_renderer.send_render_box(
            &bbox,
            get_secondary_background_color(),
            UI_BASE_DEPTH - 2 - extra_depth, 1,
        );

        if let InstructionConfig::FinishedInstruction(inst) = self.config {
            let inst = inst.unpack();
            let text_box = delete_button_bbox.apply_offset(glm::vec2(DELETE_BUTTON_SIZE + INSTRUCTION_PADDING * 2.0, 0.0));
            let formatted_text = inst.format_assembly();
            let color_formatted = inst.format_with_colors(&formatted_text);

            context.font.add_text(&TextRenderConfig::new(
                &color_formatted,
                text_box.get_top_left(),
                text_box.get_size().y)
                .set_mode(PositionMode::TopLeft)
                .set_align(TextAlign::Left)
                .set_depth(UI_BASE_DEPTH - 3 - extra_depth),
            );
        }
    }

    fn get_delete_button_box(unit_top_left_position: glm::Vec2) -> RenderBox {
        RenderBox::new_simple(
            unit_top_left_position + glm::vec2(INSTRUCTION_PADDING, -INSTRUCTION_PADDING),
            PositionMode::TopLeft,
            glm::vec2(DELETE_BUTTON_SIZE, DELETE_BUTTON_SIZE),
        )
    }

    fn get_menu_button_box(delete_button_top_left: glm::Vec2) -> RenderBox {
        RenderBox::new_simple(
            delete_button_top_left + glm::vec2(3.0 * INSTRUCTION_PADDING + DELETE_BUTTON_SIZE, -INSTRUCTION_PADDING),
            PositionMode::TopLeft,
            glm::vec2(DELETE_BUTTON_SIZE, DELETE_BUTTON_SIZE),
        )
    }

    pub fn receive_simple_area_click(&mut self, id: usize, ui: &mut Ui) {
        if let InstructionConfig::UnfinishedInstruction(inst) = &mut self.config {
            inst.receive_simple_area_click(id, ui);
        }
    }
}
