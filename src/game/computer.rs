use crate::game::{instruction::{Instruction, RegisterAddress, Syscall, UnpackedInstruction}, memory::{Memory, PROGRAM_COUNTER_IDX}};

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

#[derive(Debug)]
pub struct Computer {
    registers: Memory<u8, MEMORY_SIZE>,
    instructions: Memory<Instruction, MEMORY_SIZE>,
}

impl Computer {
    pub fn set_instruction(&mut self, idx: usize, instruction: Instruction) {
        self.instructions.write_instruction_memory(idx, instruction);
    }

    pub fn with_instructions(mut self, instructions: &[Instruction; MEMORY_SIZE]) -> Self {
        for i in 0..MEMORY_SIZE {
            self.instructions.write_instruction_memory(i, instructions[i]);
        }
        return self;
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

    pub fn write_reg(&mut self, address: RegisterAddress, value: u8) {
        self.registers.write_memory(address.get_raw(), value);
    }

    pub fn read_reg(&mut self, address: RegisterAddress) -> u8 {
        self.registers.read_memory(address.get_raw())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::instruction::{Instruction, Syscall};

    #[test]
    fn test_initial_state() {
        let computer = Computer::default();
        // O PC (Program Counter) deve começar em 0
        // No seu código, PROGRAM_COUNTER_IDX é 1
        assert_eq!(computer.registers.read_memory(PROGRAM_COUNTER_IDX), 0);
        // Registrador x0 deve ser sempre 0
        assert_eq!(computer.registers.read_memory(0usize), 0);
    }

    #[test]
    fn test_lui_and_addi() {
        let mut computer = Computer::default();
        let instructions = [
            Instruction::new_lui(X[2], 100),       // x2 = 100 (LUI aceita até 255)
            Instruction::new_addi(X[2], X[3], 15),  // x3 = x2 + 15 (Máximo permitido em 4 bits)
            Instruction::new_sys(Syscall::Nop),
        ];
        
        for (i, &inst) in instructions.iter().enumerate() {
            computer.set_instruction(i, inst);
        }

        computer.run_computer(); // x2 = 100
        computer.run_computer(); // x3 = 100 + 15
        
        assert_eq!(computer.read_reg(X[3]), 115);
    }

    #[test]
    fn test_zero_register_immutability() {
        let mut computer = Computer::default();
        // Tenta escrever 255 no registrador x0 usando LUI
        computer.set_instruction(0, Instruction::new_lui(X[0], 255));
        
        computer.run_computer();
        
        // x0 deve permanecer 0 (regra comum de arquiteturas RISC)
        assert_eq!(computer.read_reg(X[0]), 0);
    }

    #[test]
    fn test_branch_taken() {
        let mut computer = Computer::default();
        // Se x1 == x0 (0 == 0), pula 2 instruções
        computer.set_instruction(0, Instruction::new_beq(X[1], X[0], 2));
        computer.set_instruction(1, Instruction::new_lui(X[2], 1)); // Será pulada
        computer.set_instruction(2, Instruction::new_lui(X[2], 2)); // Será pulada
        computer.set_instruction(3, Instruction::new_lui(X[3], 10)); // Destino do pulo
        
        computer.run_computer(); // Executa BEQ
        
        // PC estava em 0. BEQ somou +1 (padrão) e +2 (pulo) = 3
        assert_eq!(computer.read_reg(RegisterAddress::new(PROGRAM_COUNTER_IDX as u8)), 3);
        
        computer.run_computer(); // Executa x3 = 10
        assert_eq!(computer.read_reg(X[3]), 10);
        assert_eq!(computer.read_reg(X[2]), 0); // x2 nunca foi alterado
    }

    #[test]
    fn test_wrapping_arithmetic() {
        let mut computer = Computer::default();
        computer.write_reg(X[3], 200);
        computer.write_reg(X[2], 100);
        // 200 + 100 em u8 deve resultar em 44 (300 % 256)
        computer.set_instruction(0, Instruction::new_add(X[3], X[2], X[4]));
        
        computer.run_computer();
        assert_eq!(computer.read_reg(X[4]), 44);
    }

    #[test]
    fn test_syscall_return() {
        let mut computer = Computer::default();
        computer.set_instruction(0, Instruction::new_sys(Syscall::Mine));
        
        let result = computer.run_computer();
        
        // Verifica se o syscall foi propagado para fora do computador
        assert_eq!(result, Some(Syscall::Mine));
    }

    #[test]
    fn test_division_by_zero() {
        let mut computer = Computer::default();
        computer.write_reg(X[4], 10); // Usamos x4 para o valor 10
        computer.write_reg(X[2], 0);  // Divisor 0
        // O PC (x1) começa em 0 por padrão, então vai ler a instrução no 0
        computer.set_instruction(0, Instruction::new_div(X[4], X[2], X[3]));
        
        computer.run_computer();
        assert_eq!(computer.read_reg(X[3]), 10); // 10 / 1 = 10
    }

    #[test]
    fn test_manual_pc_set_via_api() {
        let mut computer = Computer::default();
        
        // Escrevemos uma instrução lá na frente, no endereço 10
        computer.set_instruction(10, Instruction::new_lui(X[2], 42));
        
        // Manipulamos o PC (X[1]) via API para apontar para o 10
        computer.write_reg(X[1], 10);
        
        // Executa a instrução que está no 10
        computer.run_computer();
        
        assert_eq!(computer.read_reg(X[2]), 42);
        // O PC deve ser 11 agora (10 + 1 do incremento automático)
        assert_eq!(computer.read_reg(X[1]), 11);
    }

    #[test]
    fn test_pc_manipulation_by_instruction() {
        let mut computer = Computer::default();
        
        // Instrução no endereço 0: ADDI x1, x1, 5 (PC = PC + 5)
        // Como o PC é o x1, isso é um "jump relativo" manual
        computer.set_instruction(0, Instruction::new_addi(X[1], X[1], 5));
        
        // Instrução no endereço 6 (0 + 5 do ADDI + 1 do incremento automático)
        computer.set_instruction(6, Instruction::new_lui(X[3], 255));
        
        computer.run_computer(); // Executa o ADDI no x1
        
        // O PC deve ter saltado para 6
        assert_eq!(computer.read_reg(X[1]), 6);
        
        computer.run_computer(); // Executa o LUI no endereço 6
        assert_eq!(computer.read_reg(X[3]), 255);
    }

    #[test]
    fn test_pc_wrapping() {
        let mut computer = Computer::default();
        
        // Coloca o PC no limite máximo do u8
        computer.write_reg(X[1], 255);
        
        // Instrução qualquer no endereço 255
        computer.set_instruction(255, Instruction::new_sys(Syscall::Nop));
        
        computer.run_computer();
        
        // O PC deve dar a volta (wrapping) e retornar para 0
        assert_eq!(computer.read_reg(X[1]), 0);
    }
}
