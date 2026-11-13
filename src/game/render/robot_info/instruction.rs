use crate::{game::{logic::top_level::instruction::{Instruction, Syscall, UnpackedInstruction}, render::robot_info::{ITEM_HEIGHT, ITEM_PADDING}}, graphics::renderer::{fonts::{TextAlign, TextRenderConfig}, positioning::{PositionMode, RenderBox}}, ui::{UiElement, UiState}};
use nalgebra_glm as glm;

impl UnpackedInstruction {
    fn format_assembly(&self) -> String {
        let id = self.get_id();
        match self {
            UnpackedInstruction::Lui(rd, imm) => 
                format!("{} {:<3}{:<3}", id, rd, imm),
            
            UnpackedInstruction::Addi(rd, rs, imm) |
            UnpackedInstruction::Subi(rd, rs, imm) => 
                format!("{} {:<3}{:<3}{:<3}", id, rd, rs, imm),
            
            UnpackedInstruction::Add(rd, r1, r2) |
            UnpackedInstruction::Sub(rd, r1, r2) |
            UnpackedInstruction::Mul(rd, r1, r2) |
            UnpackedInstruction::Div(rd, r1, r2) |
            UnpackedInstruction::Rem(rd, r1, r2) |
            UnpackedInstruction::Seq(rd, r1, r2) |
            UnpackedInstruction::Slt(rd, r1, r2) |
            UnpackedInstruction::Sle(rd, r1, r2) => 
                format!("{} {:<3}{:<3}{:<3}", id, rd, r1, r2),

            UnpackedInstruction::Beq(r1, r2, imm) |
            UnpackedInstruction::Bne(r1, r2, imm) |
            UnpackedInstruction::Blt(r1, r2, imm) |
            UnpackedInstruction::Ble(r1, r2, imm) => 
                format!("{} {:<3}{:<3}{:<3}", id, r1, r2, imm),

            UnpackedInstruction::Sys(Syscall::Nop) => 
                "NOP".to_string(),
            UnpackedInstruction::Sys(syscall) => 
                format!("{} {}", id, syscall),
        }
    }

    fn get_mnemonic_color() -> glm::U8Vec4 {
        glm::vec4(199, 146, 234, 255) // Roxo
    }

    fn get_dest_color() -> glm::U8Vec4 {
        glm::vec4(137, 221, 255, 255) // Azul
    }

    fn get_reg1_color() -> glm::U8Vec4 {
        glm::vec4(240, 113, 120, 255) // Coral
    }

    fn get_reg2_color() -> glm::U8Vec4 {
        glm::vec4(247, 140, 108, 255) // Pêssego
    }

    pub(super) fn get_literal_color() -> glm::U8Vec4 {
        glm::vec4(195, 232, 141, 255) // Verde Lima
    }

    fn format_with_colors<'a>(&self, asm: &'a str) -> [(&'a str, glm::U8Vec4); 4] {
        let mnemonic = Self::get_mnemonic_color();
        let dest = Self::get_dest_color();
        let reg1 = Self::get_reg1_color();
        let reg2 = Self::get_reg2_color();
        let literal = Self::get_literal_color();

        let get_slice = |start: usize, end: usize| -> &'a str {
            let s = start.min(asm.len());
            let e = end.min(asm.len());
            &asm[s..e]
        };

        let nothing = (get_slice(0, 0), glm::vec4(0, 0, 0, 0));

        match self {
            UnpackedInstruction::Lui(_, _) => [
                (get_slice(0, 5), mnemonic),
                (get_slice(5, 9), dest),
                (get_slice(9, 13), literal),
                nothing,
            ],

            UnpackedInstruction::Addi(_, _, _) |
            UnpackedInstruction::Subi(_, _, _) |
            UnpackedInstruction::Add(_, _, _) |
            UnpackedInstruction::Sub(_, _, _) |
            UnpackedInstruction::Mul(_, _, _) |
            UnpackedInstruction::Div(_, _, _) |
            UnpackedInstruction::Rem(_, _, _) |
            UnpackedInstruction::Seq(_, _, _) |
            UnpackedInstruction::Slt(_, _, _) |
            UnpackedInstruction::Sle(_, _, _) => [
                (get_slice(0, 5), mnemonic),
                (get_slice(5, 9), dest),      // rd
                (get_slice(9, 13), reg1),     // r1 / rs
                (get_slice(13, 17), reg2),    // r2 / literal (usando reg2 color para o 3º op)
            ],

            UnpackedInstruction::Beq(_, _, _) |
            UnpackedInstruction::Bne(_, _, _) |
            UnpackedInstruction::Blt(_, _, _) |
            UnpackedInstruction::Ble(_, _, _) => [
                (get_slice(0, 5), mnemonic),
                (get_slice(5, 9), reg1),
                (get_slice(9, 13), reg2),
                (get_slice(13, 17), literal),
            ],

            UnpackedInstruction::Sys(Syscall::Nop) => [
                (get_slice(0, 3), literal),
                nothing,
                nothing,
                nothing,
            ],
            UnpackedInstruction::Sys(_) => [
                (get_slice(0, 5), mnemonic),
                (get_slice(5, asm.len()), literal),
                nothing,
                nothing,
            ],
        }
    }
}

impl UiElement for Instruction {
    fn update(&mut self,
        parent_bbox: &RenderBox,
        parent_depth: u16,
        ctx: &mut UiState,
    ) {
        const FONT_HEIGHT: f32 = ITEM_HEIGHT - ITEM_PADDING * 2.0;

        let depth = parent_depth - 1;
        let unpacked = self.unpack();

        let asm = unpacked.format_assembly();
        let parts = unpacked.format_with_colors(&asm);

        let top_left = parent_bbox.get_top_left() + glm::vec2(ITEM_PADDING, -ITEM_PADDING);
        ctx.renderer.font_renderer.push(
            &TextRenderConfig::new(
                &parts,
                glm::vec2(top_left.x, top_left.y),
                FONT_HEIGHT,
            ).
            with_align(TextAlign::Left)
            .with_depth(depth)
            .with_mode(PositionMode::TopLeft),
            ctx.font, 1,
        );
    }
}
