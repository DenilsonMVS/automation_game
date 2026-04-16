use crate::game::{instruction::{Instruction, Syscall, UnpackedInstruction}, memory::{Memory, PROGRAM_COUNTER_IDX}};

const MEMORY_RANK: usize = 4;
const MEMORY_SIZE: usize = 1 << MEMORY_RANK;

#[derive(Debug)]
pub struct Computer {
    registers: Memory<u8, MEMORY_SIZE>,
    instructions: Memory<Instruction, MEMORY_SIZE>,
}

impl Computer {
    pub fn set_instruction(&mut self, idx: usize, instruction: Instruction) {
        self.instructions.write_instruction_memory(idx, instruction);
    }
}

impl Default for Computer {
    fn default() -> Self {
        Self { registers: Memory::new(), instructions: Memory::new() }
    }
}

fn replace_zero(value: u8, replacement: u8) -> u8 {
    if value == 0 {
        replacement
    } else {
        value
    }
}

impl Computer {
    pub fn run_computer(&mut self) -> Option<Syscall> {
        let mut return_sys = None;

        let unpacked_instruction = self.instructions.read_memory(self.registers.read_memory(PROGRAM_COUNTER_IDX)).unpack();
        match unpacked_instruction {
            UnpackedInstruction::Lui(rd, imm) =>
                self.registers.write_memory(rd.get_raw(), imm),
            UnpackedInstruction::AddI(rs, rd, imm) =>
                self.registers.inst1(rs.get_raw(), rd.get_raw(), |x| x.wrapping_add(imm)),
            UnpackedInstruction::SubI(rs, rd, imm) =>
                self.registers.inst1(rs.get_raw(), rd.get_raw(), |x| x.wrapping_sub(imm)),
            UnpackedInstruction::Add(r1, r2, rd) =>
                self.registers.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| a.wrapping_add(b)),
            UnpackedInstruction::Sub(r1, r2, rd) =>
                self.registers.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| a.wrapping_sub(b)),
            UnpackedInstruction::Mul(r1, r2, rd) =>
                self.registers.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| a.wrapping_mul(b)),
            UnpackedInstruction::Div(r1, r2, rd) =>
                self.registers.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| a / replace_zero(b, 1)),
            UnpackedInstruction::Rem(r1, r2, rd) =>
                self.registers.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| a % replace_zero(b, 1)),
            UnpackedInstruction::Seq(r1, r2, rd) =>
                self.registers.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| (a == b) as u8),
            UnpackedInstruction::Slt(r1, r2, rd) =>
                self.registers.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| (a < b) as u8),
            UnpackedInstruction::Sle(r1, r2, rd) =>
                self.registers.inst2(r1.get_raw(), r2.get_raw(), rd.get_raw(), |a, b| (a <= b) as u8),
            UnpackedInstruction::Beq(r1, r2, imm) =>
                self.registers.inst3(r1.get_raw(), r2.get_raw(), PROGRAM_COUNTER_IDX as u16, PROGRAM_COUNTER_IDX as u16, |a, b, c| if a == b { c.wrapping_add(imm) } else { c }),
            UnpackedInstruction::Bne(r1, r2, imm) =>
                self.registers.inst3(r1.get_raw(), r2.get_raw(), PROGRAM_COUNTER_IDX as u16, PROGRAM_COUNTER_IDX as u16, |a, b, c| if a != b { c.wrapping_add(imm) } else { c }),
            UnpackedInstruction::Blt(r1, r2, imm) =>
                self.registers.inst3(r1.get_raw(), r2.get_raw(), PROGRAM_COUNTER_IDX as u16, PROGRAM_COUNTER_IDX as u16, |a, b, c| if a < b { c.wrapping_add(imm) } else { c }),
            UnpackedInstruction::Ble(r1, r2, imm) =>
                self.registers.inst3(r1.get_raw(), r2.get_raw(), PROGRAM_COUNTER_IDX as u16, PROGRAM_COUNTER_IDX as u16, |a, b, c| if a <= b { c.wrapping_add(imm) } else { c }),
            UnpackedInstruction::Sys(sys) =>
                return_sys = Some(sys),
        }

        self.registers.inst1(PROGRAM_COUNTER_IDX, PROGRAM_COUNTER_IDX, |x| x.wrapping_add(1));
        return return_sys;
    }
}
