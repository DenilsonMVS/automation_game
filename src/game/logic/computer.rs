use crate::{game::logic::{instruction::{Instruction, RegisterAddress, Syscall, UnpackedInstruction}, memory::{GenericMemory, InstructionMemory, PROGRAM_COUNTER_IDX, RegisterMemory}}};


const MEMORY_RANK: usize = 4;
pub const MEMORY_SIZE: usize = 1 << MEMORY_RANK;

pub const X: [RegisterAddress; 16] = [
    RegisterAddress::new(0),
    RegisterAddress::new(1),
    RegisterAddress::new(2),
    RegisterAddress::new(3),
    RegisterAddress::new(4),
    RegisterAddress::new(5),
    RegisterAddress::new(6),
    RegisterAddress::new(7),
    RegisterAddress::new(8),
    RegisterAddress::new(9),
    RegisterAddress::new(10),
    RegisterAddress::new(11),
    RegisterAddress::new(12),
    RegisterAddress::new(13),
    RegisterAddress::new(14),
    RegisterAddress::new(15),
];

pub const X0: RegisterAddress = X[0]; 
pub const X1: RegisterAddress = X[1]; 
pub const X2: RegisterAddress = X[2]; 
pub const X3: RegisterAddress = X[3]; 
pub const X4: RegisterAddress = X[4]; 
pub const X5: RegisterAddress = X[5]; 
pub const X6: RegisterAddress = X[6]; 
pub const X7: RegisterAddress = X[7]; 
pub const X8: RegisterAddress = X[8]; 
pub const X9: RegisterAddress = X[9]; 
pub const X10: RegisterAddress = X[10]; 
pub const X11: RegisterAddress = X[11]; 
pub const X12: RegisterAddress = X[12]; 
pub const X13: RegisterAddress = X[13]; 
pub const X14: RegisterAddress = X[14]; 
pub const X15: RegisterAddress = X[15];

pub const PC: RegisterAddress = X[PROGRAM_COUNTER_IDX];


#[derive(Debug)]
pub struct Computer {
    pub registers: RegisterMemory<u8, MEMORY_SIZE>,
    pub instructions: InstructionMemory<MEMORY_SIZE>,
}


fn replace_zero(value: u8, replacement: u8) -> u8 {
    if value == 0 {
        replacement
    } else {
        value
    }
}

impl Computer {
    pub fn new(instructions: [Instruction; MEMORY_SIZE]) -> Self {
        Self {
            registers: RegisterMemory::new([0; MEMORY_SIZE]),
            instructions: InstructionMemory::new(instructions),
        }
    }

    pub fn run_computer(&mut self) -> Option<Syscall> {
        let mut return_sys = None;

        let unpacked_instruction = self.instructions.read(self.registers.read(PC)).unpack();
        match unpacked_instruction {
            UnpackedInstruction::Lui(rd, imm) =>
                self.registers.write(rd, imm),
            UnpackedInstruction::Addi(rs, rd, imm) =>
                self.registers.inst1(rs, rd, |x| x.wrapping_add(imm)),
            UnpackedInstruction::Subi(rs, rd, imm) =>
                self.registers.inst1(rs, rd, |x| x.wrapping_sub(imm)),
            UnpackedInstruction::Add(r1, r2, rd) =>
                self.registers.inst2(r1, r2, rd, |a, b| a.wrapping_add(b)),
            UnpackedInstruction::Sub(r1, r2, rd) =>
                self.registers.inst2(r1, r2, rd, |a, b| a.wrapping_sub(b)),
            UnpackedInstruction::Mul(r1, r2, rd) =>
                self.registers.inst2(r1, r2, rd, |a, b| a.wrapping_mul(b)),
            UnpackedInstruction::Div(r1, r2, rd) =>
                self.registers.inst2(r1, r2, rd, |a, b| a / replace_zero(b, 1)),
            UnpackedInstruction::Rem(r1, r2, rd) =>
                self.registers.inst2(r1, r2, rd, |a, b| a % replace_zero(b, 1)),
            UnpackedInstruction::Seq(r1, r2, rd) =>
                self.registers.inst2(r1, r2, rd, |a, b| (a == b) as u8),
            UnpackedInstruction::Slt(r1, r2, rd) =>
                self.registers.inst2(r1, r2, rd, |a, b| (a < b) as u8),
            UnpackedInstruction::Sle(r1, r2, rd) =>
                self.registers.inst2(r1, r2, rd, |a, b| (a <= b) as u8),
            UnpackedInstruction::Beq(r1, r2, imm) =>
                self.registers.inst3(r1, r2, PC, PC, |a, b, c| if a == b { c.wrapping_add(imm) } else { c }),
            UnpackedInstruction::Bne(r1, r2, imm) =>
                self.registers.inst3(r1, r2, PC, PC, |a, b, c| if a != b { c.wrapping_add(imm) } else { c }),
            UnpackedInstruction::Blt(r1, r2, imm) =>
                self.registers.inst3(r1, r2, PC, PC, |a, b, c| if a < b { c.wrapping_add(imm) } else { c }),
            UnpackedInstruction::Ble(r1, r2, imm) =>
                self.registers.inst3(r1, r2, PC, PC, |a, b, c| if a <= b { c.wrapping_add(imm) } else { c }),
            UnpackedInstruction::Sys(sys) =>
                return_sys = Some(sys),
        }

        self.registers.inst1(PC, PC, |x| x.wrapping_add(1));
        return return_sys;
    }

    pub fn write_reg(&mut self, address: RegisterAddress, value: u8) {
        self.registers.write(address, value);
    }

    pub fn read_reg(&mut self, address: RegisterAddress) -> u8 {
        self.registers.read(address)
    }

    pub fn get(&self) -> &[Instruction] {
        self.instructions.get()
    }

    pub fn get_data(&self) -> &[u8; MEMORY_SIZE] {
        self.registers.get()
    }
}
