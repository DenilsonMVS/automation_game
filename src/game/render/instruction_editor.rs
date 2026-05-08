use crate::{MainContext, game::{logic::{computer::MEMORY_SIZE, instruction::{InstructionFormat, InstructionId, RegisterAddress, Syscall, UnpackedInstruction}}, render::{colored_text::{ColoredTextButton, ColoredTextView}, robot_info::{get_list_bgcolor, get_secondary_background_color}}}, graphics::renderer::{fonts::Font, positioning::{PositionMode, RenderBox}}, ui::{Ui, dnd::HasId, menu::Menu, rendered_button::{RenderedButton, RenderedButtonContext}, simple_area_manager::AreaId, text_input::{CharFilter, TextInput, ValidateInput}, ui_element::UiElement}};
use nalgebra_glm as glm;

const PADDING: f32 = 0.01;

fn get_delete_button_color() -> glm::U8Vec4 {
    glm::vec4(200, 50, 60, 255)
}

fn get_scroll_bar_color() -> glm::U8Vec4 {
    glm::vec4(30, 30, 50, 255)
}

#[derive(Debug, Default)]
pub struct U8Input;

impl CharFilter for U8Input {
    fn filter(&self, c: char) -> bool {
        c.is_numeric()
    }
}

impl ValidateInput for U8Input {
    fn validate(&self, input: &str) -> bool {
        input.parse::<u8>().is_ok()
    }
}

#[derive(Debug, Default)]
pub struct U4Input;

impl CharFilter for U4Input {
    fn filter(&self, c: char) -> bool {
        c.is_numeric()
    }
}

impl ValidateInput for U4Input {
    fn validate(&self, input: &str) -> bool {
        matches!(input.parse::<u8>(), Ok(value) if value < (1 << 4))
    }
}

#[derive(Debug)]
pub enum OperandState {
    RdImm8 {
        rd: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        imm: TextInput<U8Input>,
    },
    RdRsImm4 {
        rd: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        rs: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        imm: TextInput<U4Input>,
    },
    RdR1R2 {
        rd: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        r1: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        r2: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
    },
    R1R2Imm4 {
        r1: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        r2: Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        imm: TextInput<U4Input>,
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

    fn create_opcode_options() -> Vec<ColoredTextButton<InstructionId>> {
        (0..16)
            .map(InstructionId::from_u16)
            .map(|opcode| ColoredTextButton::new(
                opcode,
                UnpackedInstruction::get_mnemonic_color(),
                get_secondary_background_color(),
            ))
            .collect()
    }

    fn create_opcode_menu() -> Menu<ColoredTextButton<InstructionId>, ColoredTextView<InstructionId>> {
        let opcodes = Self::create_opcode_options();
        Menu::new(opcodes).with_gap(PADDING)
            .with_open_bgcolor(get_list_bgcolor())
            .with_maximum_displayed(8)
            .with_scroll_bar(0.02, get_scroll_bar_color())
    }

    pub fn new() -> Self {
        Self {
            id: AreaId::default(),
            bbox: Default::default(),
            depth: 0,
            opcode_menu: Self::create_opcode_menu(),
            delete_button: RenderedButton::new(get_delete_button_color()),
            operands: None,
        }
    }

    pub fn from_instruction(inst: &UnpackedInstruction) -> Self {
        Self {
            id: AreaId::default(),
            bbox: Default::default(),
            depth: 0,
            opcode_menu: Self::create_opcode_menu()
                .with_display_value(ColoredTextView::new(
                    inst.get_id(),
                    UnpackedInstruction::get_mnemonic_color(),
                    get_secondary_background_color(),
                )),
            delete_button: RenderedButton::new(get_delete_button_color()),
            operands: Some(Self::create_operands_from_instruction(inst)),
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
            InstructionFormat::RdRsImm4  => [Self::WIDTH_REG, Self::WIDTH_REG, Self::WIDTH_IMM],
            InstructionFormat::RdR1R2    => [Self::WIDTH_REG, Self::WIDTH_REG, Self::WIDTH_REG],
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

    fn generate_reg_opts(color: glm::U8Vec4, bgcolor: glm::U8Vec4) -> Vec<ColoredTextButton<RegisterAddress>> {
        (0..MEMORY_SIZE)
            .map(|i| RegisterAddress::new(i as u8))
            .map(|reg| ColoredTextButton::new(reg, color, bgcolor))
            .collect()
    }

    fn generate_syscalls() -> Vec<ColoredTextButton<Syscall>> {
        [
            Syscall::Nop, Syscall::MoveLeft, Syscall::MoveRight, Syscall::MoveTop, 
            Syscall::MoveBottom, Syscall::Mine, Syscall::Display, Syscall::Scan, 
            Syscall::Stored, Syscall::MaxStorage,
        ]
        .iter()
        .map(|s| ColoredTextButton::new(*s, UnpackedInstruction::get_literal_color(), get_secondary_background_color()))
        .collect()
    }

    fn create_rdimm8() -> (
        Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        TextInput<U8Input>
    ) {
        let d_col = UnpackedInstruction::get_dest_color();
        let l_col = UnpackedInstruction::get_literal_color();
        let bg = get_secondary_background_color();
        let empty_text_bgcolor = glm::vec4(255, 255, 255, 20);
        let text_hover_color = glm::vec4(255, 255, 255, 40);
        let list_bg = get_list_bgcolor();
        (
            Menu::new(Self::generate_reg_opts(d_col, bg))
                .with_gap(PADDING)
                .with_open_bgcolor(list_bg)
                .with_maximum_displayed(8)
                .with_scroll_bar(0.02, get_scroll_bar_color()),
            TextInput::new(empty_text_bgcolor, l_col, U8Input)
                .with_hover_bgcolor(text_hover_color)
                .with_max_len(3),
        )
    }

    fn create_r1r2imm4() -> (
        Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        TextInput<U4Input>,
    ) {
        let l_col = UnpackedInstruction::get_literal_color();
        let bg = get_secondary_background_color();
        let r1_col = UnpackedInstruction::get_reg1_color();
        let empty_text_bgcolor = glm::vec4(255, 255, 255, 20);
        let text_hover_color = glm::vec4(255, 255, 255, 40);
        let list_bg = get_list_bgcolor();
        let r2_col = UnpackedInstruction::get_reg2_color();
        (
            Menu::new(Self::generate_reg_opts(r1_col, bg))
                .with_gap(PADDING)
                .with_open_bgcolor(list_bg)
                .with_maximum_displayed(8)
                .with_scroll_bar(0.02, get_scroll_bar_color()),
            Menu::new(Self::generate_reg_opts(r2_col, bg))
                .with_gap(PADDING)
                .with_open_bgcolor(list_bg)
                .with_maximum_displayed(8)
                .with_scroll_bar(0.02, get_scroll_bar_color()),
            TextInput::new(empty_text_bgcolor, l_col, U4Input)
                .with_hover_bgcolor(text_hover_color)
                .with_max_len(3),
        )
    }

    fn create_syscall() -> Menu<ColoredTextButton<Syscall>, ColoredTextView<Syscall>> {
        let list_bg = get_list_bgcolor();
        Menu::new(Self::generate_syscalls())
            .with_gap(PADDING)
            .with_open_bgcolor(list_bg)
            .with_maximum_displayed(8)
            .with_scroll_bar(0.02, get_scroll_bar_color())
    }

    fn create_rd_rs_imm4() -> (
        Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        TextInput<U4Input>
    ) {
        let d_col = UnpackedInstruction::get_dest_color();
        let r1_col = UnpackedInstruction::get_reg1_color();
        let l_col = UnpackedInstruction::get_literal_color();
        let bg = get_secondary_background_color();
        let list_bg = get_list_bgcolor();
        
        (
            Menu::new(Self::generate_reg_opts(d_col, bg))
                .with_open_bgcolor(list_bg)
                .with_scroll_bar(0.02, get_scroll_bar_color()),
            Menu::new(Self::generate_reg_opts(r1_col, bg))
                .with_open_bgcolor(list_bg)
                .with_scroll_bar(0.02, get_scroll_bar_color()),
            TextInput::new(glm::vec4(255, 255, 255, 20), l_col, U4Input)
                .with_max_len(3),
        )
    }

    fn create_rd_r1_r2() -> (
        Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
        Menu<ColoredTextButton<RegisterAddress>, ColoredTextView<RegisterAddress>>,
    ) {
        let d_col = UnpackedInstruction::get_dest_color();
        let r1_col = UnpackedInstruction::get_reg1_color();
        let r2_col = UnpackedInstruction::get_reg2_color();
        let bg = get_secondary_background_color();
        let list_bg = get_list_bgcolor();
        (
            Menu::new(Self::generate_reg_opts(d_col, bg)).with_open_bgcolor(list_bg),
            Menu::new(Self::generate_reg_opts(r1_col, bg)).with_open_bgcolor(list_bg),
            Menu::new(Self::generate_reg_opts(r2_col, bg)).with_open_bgcolor(list_bg),
        )
    }

    fn create_rd_display(value: RegisterAddress) -> ColoredTextView<RegisterAddress> {
        ColoredTextView::new(value,
            UnpackedInstruction::get_dest_color(),
            get_secondary_background_color(),
        )
    }

    fn create_r1_display(value: RegisterAddress) -> ColoredTextView<RegisterAddress> {
        ColoredTextView::new(value,
            UnpackedInstruction::get_reg1_color(),
            get_secondary_background_color(),
        )
    }

    fn create_r2_display(value: RegisterAddress) -> ColoredTextView<RegisterAddress> {
        ColoredTextView::new(value,
            UnpackedInstruction::get_reg2_color(),
            get_secondary_background_color(),
        )
    }

    fn create_rs_display(value: RegisterAddress) -> ColoredTextView<RegisterAddress> {
        Self::create_r1_display(value)
    }

    fn create_sys_display(sys: Syscall) -> ColoredTextView<Syscall> {
        ColoredTextView::new(sys,
            UnpackedInstruction::get_literal_color(),
            get_secondary_background_color(),
        )
    }

    fn create_operands_from_instruction(inst: &UnpackedInstruction) -> OperandState {
        use UnpackedInstruction::*;
        match inst {
            Lui(rdv, immv) => {
                let (rd, imm) = Self::create_rdimm8();
                OperandState::RdImm8 {
                    rd: rd.with_display_value(Self::create_rd_display(*rdv)),
                    imm: imm.with_content(immv.to_string()),
                }
            },

            Addi(rsv, rdv, immv) | Subi(rsv, rdv, immv) => {
                let (rd, rs, imm) = Self::create_rd_rs_imm4();
                OperandState::RdRsImm4 {
                    rd: rd.with_display_value(Self::create_rd_display(*rdv)),
                    rs: rs.with_display_value(Self::create_rs_display(*rsv)),
                    imm: imm.with_content(immv.to_string()),
                }
            },

            Add(r1v, r2v, rdv) | Sub(r1v, r2v, rdv) | Mul(r1v, r2v, rdv) | Div(r1v, r2v, rdv) |
            Rem(r1v, r2v, rdv) | Seq(r1v, r2v, rdv) | Slt(r1v, r2v, rdv) | Sle(r1v, r2v, rdv) => {
                let (rd, r1, r2) = Self::create_rd_r1_r2();
                OperandState::RdR1R2 {
                    rd: rd.with_display_value(Self::create_rd_display(*rdv)),
                    r1: r1.with_display_value(Self::create_r1_display(*r1v)),
                    r2: r2.with_display_value(Self::create_r2_display(*r2v)),
                }
            },

            Beq(r1v, r2v, immv) | Bne(r1v, r2v, immv) | Blt(r1v, r2v, immv) | Ble(r1v, r2v, immv) => {
                let (r1, r2, imm) = Self::create_r1r2imm4();
                OperandState::R1R2Imm4 {
                    r1: r1.with_display_value(Self::create_r1_display(*r1v)),
                    r2: r2.with_display_value(Self::create_r2_display(*r2v)),
                    imm: imm.with_content(immv.to_string()),
                }
            },

            Sys(sysv) => {
                OperandState::Sys {
                    sys: Self::create_syscall().with_display_value(Self::create_sys_display(*sysv)),
                }
            },
        }
    }

    fn manage_operand_menus(&mut self, ui: &mut Ui, depth: u16, format: InstructionFormat, bboxes: [RenderBox; 3]) {
        let bg = get_secondary_background_color();
        let empty_bg = glm::vec4(255, 255, 255, 20);
        
        let needs_init = match (&self.operands, format) {
            (Some(OperandState::RdImm8 { .. }), InstructionFormat::RdImm8) => false,
            (Some(OperandState::RdRsImm4 { .. }), InstructionFormat::RdRsImm4) => false,
            (Some(OperandState::RdR1R2 { .. }), InstructionFormat::RdR1R2) => false,
            (Some(OperandState::R1R2Imm4 { .. }), InstructionFormat::R1R2Imm4) => false,
            (Some(OperandState::Sys { .. }), InstructionFormat::Syscall) => false,
            _ => true,
        };

        if needs_init {
            self.operands = Some(match format {
                InstructionFormat::RdImm8 => { let (rd, imm) = Self::create_rdimm8(); OperandState::RdImm8 { rd, imm } },
                InstructionFormat::RdRsImm4 => { let (rd, rs, imm) = Self::create_rd_rs_imm4(); OperandState::RdRsImm4 { rd, rs, imm } },
                InstructionFormat::RdR1R2 => { let (rd, r1, r2) = Self::create_rd_r1_r2(); OperandState::RdR1R2 { rd, r1, r2 } },
                InstructionFormat::R1R2Imm4 => { let (r1, r2, imm) = Self::create_r1r2imm4(); OperandState::R1R2Imm4 { r1, r2, imm } },
                InstructionFormat::Syscall => OperandState::Sys { sys: Self::create_syscall() },
            });
        }

        if let Some(state) = &mut self.operands {
            match state {
                OperandState::RdImm8 { rd, imm } => {
                    rd.update(&bboxes[0], depth, ui);
                    imm.update(&bboxes[1], depth, ui);
                    imm.bgcolor = if imm.get_content().is_some() { bg } else { empty_bg };
                },
                OperandState::RdRsImm4 { rd, rs, imm } => {
                    rd.update(&bboxes[0], depth, ui);
                    rs.update(&bboxes[1], depth, ui);
                    imm.update(&bboxes[2], depth, ui);
                    imm.bgcolor = if imm.get_content().is_some() { bg } else { empty_bg };
                },
                OperandState::RdR1R2 { rd, r1, r2 } => {
                    rd.update(&bboxes[0], depth, ui);
                    r1.update(&bboxes[1], depth, ui);
                    r2.update(&bboxes[2], depth, ui);
                },
                OperandState::R1R2Imm4 { r1, r2, imm } => {
                    r1.update(&bboxes[0], depth, ui);
                    r2.update(&bboxes[1], depth, ui);
                    imm.update(&bboxes[2], depth, ui);
                    imm.bgcolor = if imm.get_content().is_some() { bg } else { empty_bg };
                },
                OperandState::Sys { sys } => sys.update(&bboxes[0], depth, ui),
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

            self.manage_operand_menus(context.ui, self.depth - 1, format, bboxes);
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
            Some(OperandState::RdImm8 {rd, imm }) => {
                rd.send(context);
                imm.send(&mut context.get_button_context());
            },
            Some(OperandState::RdRsImm4 { rd, rs, imm }) => {
                rd.send(context);
                rs.send(context);
                imm.send(&mut context.get_button_context());
            },
            Some(OperandState::RdR1R2 { rd, r1, r2 }) => {
                rd.send(context);
                r1.send(context);
                r2.send(context);
            },
            Some(OperandState::R1R2Imm4 { r1, r2, imm }) => {
                r1.send(context);
                r2.send(context);
                imm.send(&mut context.get_button_context());
            }
            Some(OperandState::Sys { sys }) => {
                sys.send(context);
            }
            None => {}
        }

        self.opcode_menu.send(context);

        context.simple_shape_renderer.send_render_box(&self.bbox, get_secondary_background_color(), self.depth, None, context.groups);
    }
}
