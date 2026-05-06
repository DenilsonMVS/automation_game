use crate::{MainContext, game::{logic::{computer::MEMORY_SIZE, instruction::{InstructionFormat, InstructionId, RegisterAddress, Syscall, UnpackedInstruction}}, render::{colored_text::{ColoredTextButton, ColoredTextView}, robot_info::{get_list_bgcolor, get_secondary_background_color}}}, graphics::renderer::{fonts::Font, positioning::{PositionMode, RenderBox}}, ui::{Ui, dnd::HasId, menu::Menu, rendered_button::{RenderedButton, RenderedButtonContext}, simple_area_manager::AreaId, ui_element::UiElement}};
use nalgebra_glm as glm;

const PADDING: f32 = 0.01;

fn get_delete_button_color() -> glm::U8Vec4 {
    glm::vec4(200, 50, 60, 255)
}

#[derive(Debug)]
pub enum OperandState {
    RdImm8 {
        rd: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        imm: Menu<ColoredTextButton<u8>, ColoredTextView<u8>>,
    },
    RsRdImm4 {
        rd: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        rs: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        imm: Menu<ColoredTextButton<u8>, ColoredTextView<u8>>,
    },
    R1R2Rd {
        rd: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        r1: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        r2: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
    },
    R1R2Imm4 {
        r1: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        r2: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        imm: Menu<ColoredTextButton<u8>, ColoredTextView<u8>>,
    },
    Sys {
        sys: Menu<ColoredTextButton<Syscall>, ColoredTextView<Syscall>>,
    },
}

#[derive(Debug)]
pub struct InstructionEditor {
    id: AreaId,
    bbox: RenderBox,
    depth: u16,
    opcode_menu: Menu<ColoredTextButton<InstructionId>, ColoredTextView<InstructionId>>,
    operands: Option<OperandState>,
    delete_button: RenderedButton,
}

impl InstructionEditor {
    const WIDTH_OPCODE: f32 = 4.0;
    const WIDTH_REG: f32 = 3.0;
    const WIDTH_IMM: f32 = 3.0;
    const WIDTH_SYS: f32 = 12.0;
    const CHAR_MARGIN: f32 = 1.0;

    pub fn new() -> Self {
        let opcodes = (0..16)
            .map(InstructionId::from_u16)
            .map(|opcode| ColoredTextButton::new(
                opcode,
                UnpackedInstruction::get_mnemonic_color(),
                get_secondary_background_color(),
            ))
            .collect();
        Self {
            id: AreaId::default(),
            bbox: Default::default(),
            depth: 0,
            opcode_menu: Menu::new(opcodes).with_gap(PADDING).with_open_bgcolor(get_list_bgcolor()),
            delete_button: RenderedButton::new(get_delete_button_color()),
            operands: None,
        }
    }

    fn get_operand_bboxes(bbox: &RenderBox, format: InstructionFormat, font: &Font) -> [RenderBox; 3] {
        let opcode_bbox = Self::get_opcode_bbox(bbox, font);
        let height = opcode_bbox.get_size().y;
        let char_len = font.get_monospace_width(height);
        let mut current_pos = opcode_bbox.get_top_right() + glm::vec2(Self::CHAR_MARGIN * char_len, 0.0);
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

    fn manage_operand_menus(&mut self, ui: &mut Ui, depth: u16, format: InstructionFormat, bboxes: [RenderBox; 3]) {
        let d_col = UnpackedInstruction::get_dest_color();
        let r1_col = UnpackedInstruction::get_reg1_color();
        let r2_col = UnpackedInstruction::get_reg2_color();
        let l_col = UnpackedInstruction::get_literal_color();
        let bg = get_secondary_background_color();
        let list_bg = get_list_bgcolor();

        let generate_reg_opts = |color, bgcolor| (0..MEMORY_SIZE)
            .map(|i| RegisterAddress::new(i as u8))
            .map(|reg| ColoredTextButton::new(reg, color, bgcolor))
            .collect::<Vec<_>>();
        
        let generate_imms = |size, color, bgcolor| (0..size)
            .map(|reg| ColoredTextButton::new(reg as u8, color, bgcolor))
            .collect::<Vec<_>>();

        let needs_init = match (&self.operands, format) {
            (Some(OperandState::RdImm8 { .. }), InstructionFormat::RdImm8) => false,
            (Some(OperandState::RsRdImm4 { .. }), InstructionFormat::RsRdImm4) => false,
            (Some(OperandState::R1R2Rd { .. }), InstructionFormat::R1R2Rd) => false,
            (Some(OperandState::R1R2Imm4 { .. }), InstructionFormat::R1R2Imm4) => false,
            (Some(OperandState::Sys { .. }), InstructionFormat::Syscall) => false,
            _ => true,
        };

        let generate_syscalls = |color, bgcolor| {
            vec![
                Syscall::Nop, Syscall::MoveLeft, Syscall::MoveRight, Syscall::MoveTop, 
                Syscall::MoveBottom, Syscall::Mine, Syscall::Display, Syscall::Scan, 
                Syscall::Stored, Syscall::MaxStorage
            ]
            .into_iter()
            .map(|s| ColoredTextButton::new(s, color, bgcolor))
            .collect::<Vec<_>>()
        };

        if needs_init {
            self.operands = Some(match format {
                InstructionFormat::RdImm8 => OperandState::RdImm8 {
                    rd: Menu::new(generate_reg_opts(d_col, bg)).with_gap(PADDING).with_open_bgcolor(list_bg),
                    imm: Menu::new(generate_imms(256, l_col, bg)).with_gap(PADDING).with_open_bgcolor(list_bg),
                },
                InstructionFormat::RsRdImm4 => OperandState::RsRdImm4 {
                    rd: Menu::new(generate_reg_opts(d_col, bg)).with_gap(PADDING).with_open_bgcolor(list_bg),
                    rs: Menu::new(generate_reg_opts(r1_col, bg)).with_gap(PADDING).with_open_bgcolor(list_bg),
                    imm: Menu::new(generate_imms(16, l_col, bg)).with_gap(PADDING).with_open_bgcolor(list_bg),
                },
                InstructionFormat::R1R2Rd => OperandState::R1R2Rd {
                    rd: Menu::new(generate_reg_opts(d_col, bg)).with_gap(PADDING).with_open_bgcolor(list_bg),
                    r1: Menu::new(generate_reg_opts(r1_col, bg)).with_gap(PADDING).with_open_bgcolor(list_bg),
                    r2: Menu::new(generate_reg_opts(r2_col, bg)).with_gap(PADDING).with_open_bgcolor(list_bg),
                },
                InstructionFormat::R1R2Imm4 => OperandState::R1R2Imm4 {
                    r1: Menu::new(generate_reg_opts(r1_col, bg)).with_gap(PADDING).with_open_bgcolor(list_bg),
                    r2: Menu::new(generate_reg_opts(r2_col, bg)).with_gap(PADDING).with_open_bgcolor(list_bg),
                    imm: Menu::new(generate_imms(16, l_col, bg)).with_gap(PADDING).with_open_bgcolor(list_bg),
                },
                InstructionFormat::Syscall => OperandState::Sys {
                    sys: Menu::new(generate_syscalls(l_col, bg)).with_gap(PADDING).with_open_bgcolor(list_bg),
                },
            });
        }

        if let Some(state) = &mut self.operands {
            match state {
                OperandState::RdImm8 { rd, imm } => {
                    rd.update(&bboxes[0], depth, ui);
                    imm.update(&bboxes[1], depth, ui);
                }
                OperandState::RsRdImm4 { rd, rs, imm } => {
                    rd.update(&bboxes[0], depth, ui);
                    rs.update(&bboxes[1], depth, ui);
                    imm.update(&bboxes[2], depth, ui);
                }
                OperandState::R1R2Rd { rd, r1, r2 } => {
                    rd.update(&bboxes[0], depth, ui);
                    r1.update(&bboxes[1], depth, ui);
                    r2.update(&bboxes[2], depth, ui);
                }
                OperandState::R1R2Imm4 { r1, r2, imm } => {
                    r1.update(&bboxes[0], depth, ui);
                    r2.update(&bboxes[1], depth, ui);
                    imm.update(&bboxes[2], depth, ui);
                }
                OperandState::Sys { sys } => {
                    sys.update(&bboxes[0], depth, ui);
                }
            }
        }
    }

    fn get_delete_button_bbox(bbox: &RenderBox) -> RenderBox {
        let height = bbox.get_size().y - PADDING * 2.0;
        RenderBox::new_simple(
            bbox.get_top_left() + glm::vec2(PADDING, -PADDING),
            PositionMode::TopLeft,
            glm::vec2(height, height),
        )
    }

    fn get_opcode_bbox(bbox: &RenderBox, font: &Font) -> RenderBox {
        let delete_button_bbox = Self::get_delete_button_bbox(bbox);
        let height = delete_button_bbox.get_size().y;
        let amount_chars = Self::WIDTH_OPCODE;
        let char_width = font.get_monospace_width(height);
        RenderBox::new_simple(
            delete_button_bbox.get_top_right() + glm::vec2(PADDING * 2.0, 0.0),
            PositionMode::TopLeft,
            glm::vec2(char_width * amount_chars, height),
        )
    }

    pub fn get_delete_button_id(&self) -> AreaId {
        self.delete_button.id
    }
}

impl HasId for InstructionEditor {
    fn get_id(&self) -> AreaId {
        self.id
    }

    fn update_id(&mut self, id: AreaId) {
        self.id = id;
    }
}

impl<'a, 'b> UiElement<MainContext<'a, 'b>, MainContext<'a, 'b>, ()> for InstructionEditor {
    fn update(&mut self, bbox: &RenderBox, depth: u16, context: &mut MainContext<'a, 'b>) {
        self.bbox = *bbox;
        self.depth = depth;

        self.delete_button.update(&Self::get_delete_button_bbox(bbox), depth - 1, context.ui);
        self.opcode_menu.update(&Self::get_opcode_bbox(bbox, context.font), depth - 1, context.ui);
    
        if let Some(key) = self.opcode_menu.get_selected_key() {
            let format = key.get_format();
            let bboxes = Self::get_operand_bboxes(bbox, format, context.font);

            self.manage_operand_menus(context.ui, self.depth + 1, format, bboxes);
        } else {
            self.operands = None;
        }
    }

    fn send(&mut self, context: &mut MainContext) {
        let mut button_context = RenderedButtonContext {
            font: context.font,
            simple_shape_renderer: context.simple_shape_renderer,
            groups: context.groups
        };
        self.delete_button.send(&mut button_context);

        match &mut self.operands {
            Some(OperandState::RdImm8 { rd, imm }) => { rd.send(context); imm.send(context); }
            Some(OperandState::RsRdImm4 { rd, rs, imm }) => { rd.send(context); rs.send(context); imm.send(context); }
            Some(OperandState::R1R2Rd { rd, r1, r2 }) => { rd.send(context); r1.send(context); r2.send(context); }
            Some(OperandState::R1R2Imm4 { r1, r2, imm }) => { r1.send(context); r2.send(context); imm.send(context); }
            Some(OperandState::Sys { sys }) => { sys.send(context); }
            None => {}
        }

        self.opcode_menu.send(context);

        context.simple_shape_renderer.send_render_box(&self.bbox, get_secondary_background_color(), self.depth, None, context.groups);
    }
}
