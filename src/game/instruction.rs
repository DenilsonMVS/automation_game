
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterAddress {
    value: u8,
}

impl RegisterAddress {
    pub const fn new(address: u8) -> Self {
        Self { value: address & 0xf }
    }

    pub const fn get_raw(&self) -> u16 {
        self.value as u16
    }
}

impl Into<String> for RegisterAddress {
    fn into(self) -> String {
        format!("x{}", self.get_raw())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    data: u16,
}

impl Default for Instruction {
    fn default() -> Self {
        Self::new_sys(Syscall::Nop)
    }
}

impl Instruction {
    pub fn new_lui(write: RegisterAddress, value: u16) -> Instruction {
        const ID: u16 = 0;
        return Self {
            data: (ID << 12) | (write.get_raw() << 8) | (value & 0xff),
        };
    }

    pub fn new_addi(read: RegisterAddress, write: RegisterAddress, value: u16) -> Instruction {
        const ID: u16 = 1;
        return Self {
            data: (ID << 12) | (read.get_raw() << 8) | (write.get_raw() << 4) | (value & 0xf),
        };
    }

    pub fn new_subi(read: RegisterAddress, write: RegisterAddress, value: u16) -> Instruction {
        const ID: u16 = 2;
        return Self {
            data: (ID << 12) | (read.get_raw() << 8) | (write.get_raw() << 4) | (value & 0xf),
        };
    }

    pub fn new_add(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        const ID: u16 = 3;
        return Self {
            data: (ID << 12) | (read1.get_raw() << 8) | (read2.get_raw() << 4) | (write.get_raw()),
        };
    }

    pub fn new_sub(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        const ID: u16 = 4;
        return Self {
            data: (ID << 12) | (read1.get_raw() << 8) | (read2.get_raw() << 4) | (write.get_raw()),
        };
    }

    pub fn new_mul(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        const ID: u16 = 5;
        return Self {
            data: (ID << 12) | (read1.get_raw() << 8) | (read2.get_raw() << 4) | (write.get_raw()),
        };
    }

    pub fn new_div(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        const ID: u16 = 6;
        return Self {
            data: (ID << 12) | (read1.get_raw() << 8) | (read2.get_raw() << 4) | (write.get_raw()),
        };
    }

    pub fn new_rem(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        const ID: u16 = 7;
        return Self {
            data: (ID << 12) | (read1.get_raw() << 8) | (read2.get_raw() << 4) | (write.get_raw()),
        };
    }

    pub fn new_beq(read1: RegisterAddress, read2: RegisterAddress, jump: u16) -> Instruction {
        const ID: u16 = 8;
        return Self {
            data: (ID << 12) | (read1.get_raw() << 8) | (read2.get_raw() << 4) | (jump & 0xf),
        };
    }

    pub fn new_bne(read1: RegisterAddress, read2: RegisterAddress, jump: u16) -> Instruction {
        const ID: u16 = 9;
        return Self {
            data: (ID << 12) | (read1.get_raw() << 8) | (read2.get_raw() << 4) | (jump & 0xf),
        };
    }

    pub fn new_blt(read1: RegisterAddress, read2: RegisterAddress, jump: u16) -> Instruction {
        const ID: u16 = 10;
        return Self {
            data: (ID << 12) | (read1.get_raw() << 8) | (read2.get_raw() << 4) | (jump & 0xf),
        };
    }

    pub fn new_ble(read1: RegisterAddress, read2: RegisterAddress, jump: u16) -> Instruction {
        const ID: u16 = 11;
        return Self {
            data: (ID << 12) | (read1.get_raw() << 8) | (read2.get_raw() << 4) | (jump & 0xf),
        };
    }

    pub fn new_seq(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        const ID: u16 = 12;
        return Self {
            data: (ID << 12) | (read1.get_raw() << 8) | (read2.get_raw() << 4) | (write.get_raw()),
        }
    }

    pub fn new_slt(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        const ID: u16 = 13;
        return Self {
            data: (ID << 12) | (read1.get_raw() << 8) | (read2.get_raw() << 4) | (write.get_raw()),
        }
    }

    pub fn new_sle(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        const ID: u16 = 14;
        return Self {
            data: (ID << 12) | (read1.get_raw() << 8) | (read2.get_raw() << 4) | (write.get_raw()),
        }
    }

    pub fn new_sys(value: Syscall) -> Instruction {
        const ID: u16 = 15;
        return Self {
            data: (ID << 12) | value.to_raw(),
        }
    }

    pub fn unpack(&self) -> UnpackedInstruction {
        let id = (self.data >> 12) & 0xF;
        
        let r1 = RegisterAddress::new(((self.data >> 8) & 0xF) as u8);
        let r2 = RegisterAddress::new(((self.data >> 4) & 0xF) as u8);
        let r3 = RegisterAddress::new((self.data & 0xF) as u8);
        
        let imm4 = (self.data & 0xF) as u8;
        let imm8 = (self.data & 0xFF) as u8;
        let imm12 = self.data & 0xFFF;

        match id {
            0  => UnpackedInstruction::Lui(r1, imm8),
            1  => UnpackedInstruction::AddI(r1, r2, imm4),
            2  => UnpackedInstruction::SubI(r1, r2, imm4),
            3  => UnpackedInstruction::Add(r1, r2, r3),
            4  => UnpackedInstruction::Sub(r1, r2, r3),
            5  => UnpackedInstruction::Mul(r1, r2, r3),
            6  => UnpackedInstruction::Div(r1, r2, r3),
            7  => UnpackedInstruction::Rem(r1, r2, r3),
            8  => UnpackedInstruction::Beq(r1, r2, imm4),
            9  => UnpackedInstruction::Bne(r1, r2, imm4),
            10 => UnpackedInstruction::Blt(r1, r2, imm4),
            11 => UnpackedInstruction::Ble(r1, r2, imm4),
            12 => UnpackedInstruction::Seq(r1, r2, r3),
            13 => UnpackedInstruction::Slt(r1, r2, r3),
            14 => UnpackedInstruction::Sle(r1, r2, r3),
            15 => UnpackedInstruction::Sys(Syscall::from_raw(imm12)),
            _  => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum UnpackedInstruction {
    Lui(RegisterAddress, u8),           // (write, imm8)
    AddI(RegisterAddress, RegisterAddress, u8), // (read, write, imm4)
    SubI(RegisterAddress, RegisterAddress, u8), // (read, write, imm4)
    Add(RegisterAddress, RegisterAddress, RegisterAddress), // (r1, r2, dest)
    Sub(RegisterAddress, RegisterAddress, RegisterAddress),
    Mul(RegisterAddress, RegisterAddress, RegisterAddress),
    Div(RegisterAddress, RegisterAddress, RegisterAddress),
    Rem(RegisterAddress, RegisterAddress, RegisterAddress),
    Beq(RegisterAddress, RegisterAddress, u8), // (r1, r2, jump)
    Bne(RegisterAddress, RegisterAddress, u8),
    Blt(RegisterAddress, RegisterAddress, u8),
    Ble(RegisterAddress, RegisterAddress, u8),
    Seq(RegisterAddress, RegisterAddress, RegisterAddress), // (r1, r2, dest)
    Slt(RegisterAddress, RegisterAddress, RegisterAddress),
    Sle(RegisterAddress, RegisterAddress, RegisterAddress),
    Sys(Syscall),
}

impl UnpackedInstruction {
    pub fn get_visualization(&self) -> [String; 4] {
        match *self {
            Self::Lui(rd, imm) =>
                ["LUI".into(), rd.into(), format!("{}", imm), "".into()],
            Self::AddI(rs, rd, imm) =>
                ["ADDI".into(), rs.into(), rd.into(), format!("{}", imm)],
            Self::SubI(rs, rd, imm) =>
                ["SUBI".into(), rs.into(), rd.into(), format!("{}", imm)],
            Self::Add(r1, r2, rd) =>
                ["ADD".into(), r1.into(), r2.into(), rd.into()],
            Self::Sub(r1, r2, rd) =>
                ["SUB".into(), r1.into(), r2.into(), rd.into()],
            Self::Mul(r1, r2, rd) =>
                ["MUL".into(), r1.into(), r2.into(), rd.into()],
            Self::Div(r1, r2, rd) =>
                ["DIV".into(), r1.into(), r2.into(), rd.into()],
            Self::Rem(r1, r2, rd) =>
                ["REM".into(), r1.into(), r2.into(), rd.into()],
            Self::Seq(r1, r2, rd) =>
                ["SEQ".into(), r1.into(), r2.into(), rd.into()],
            Self::Slt(r1, r2, rd) =>
                ["SLT".into(), r1.into(), r2.into(), rd.into()],
            Self::Sle(r1, r2, rd) =>
                ["SLE".into(), r1.into(), r2.into(), rd.into()],
            Self::Beq(r1, r2, imm) =>
                ["BEQ".into(), r1.into(), r2.into(), format!("{}", imm)],
            Self::Bne(r1, r2, imm) =>
                ["BNE".into(), r1.into(), r2.into(), format!("{}", imm)],
            Self::Blt(r1, r2, imm) =>
                ["BLT".into(), r1.into(), r2.into(), format!("{}", imm)],
            Self::Ble(r1, r2, imm) =>
                ["BLE".into(), r1.into(), r2.into(), format!("{}", imm)],
            Self::Sys(sys) =>
                if sys == Syscall::Nop {
                    ["NOP".into(), "".into(), "".into(), "".into()]
                } else {
                    ["SYS".into(), format!("{}", sys.to_raw()), "".into(), "".into()]
                },
        }
    }

    pub fn get_simplified_visualization(&self) -> String {
        match *self {
            Self::Lui(rd, imm) =>
                format!("x{} = {}", rd.get_raw(), imm),
            Self::AddI(rs, rd, imm) =>
                format!("x{} = x{} + {}", rd.get_raw(), rs.get_raw(), imm),
            Self::SubI(rs, rd, imm) =>
                format!("x{} = x{} - {}", rd.get_raw(), rs.get_raw(), imm),
            Self::Add(r1, r2, rd) =>
                format!("x{} = x{} + x{}", rd.get_raw(), r1.get_raw(), r2.get_raw()),
            Self::Sub(r1, r2, rd) =>
                format!("x{} = x{} - x{}", rd.get_raw(), r1.get_raw(), r2.get_raw()),
            Self::Mul(r1, r2, rd) =>
                format!("x{} = x{} * x{}", rd.get_raw(), r1.get_raw(), r2.get_raw()),
            Self::Div(r1, r2, rd) =>
                format!("x{} = x{} / x{}", rd.get_raw(), r1.get_raw(), r2.get_raw()),
            Self::Rem(r1, r2, rd) =>
                format!("x{} = x{} % x{}", rd.get_raw(), r1.get_raw(), r2.get_raw()),
            Self::Seq(r1, r2, rd) =>
                format!("x{} = x{} == x{}", rd.get_raw(), r1.get_raw(), r2.get_raw()),
            Self::Slt(r1, r2, rd) =>
                format!("x{} = x{} < x{}", rd.get_raw(), r1.get_raw(), r2.get_raw()),
            Self::Sle(r1, r2, rd) =>
                format!("x{} = x{} <= x{}", rd.get_raw(), r1.get_raw(), r2.get_raw()),
            Self::Beq(r1, r2, imm) =>
                format!("if x{} == x{}, skip {}", r1.get_raw(), r2.get_raw(), imm),
            Self::Bne(r1, r2, imm) =>
                format!("if x{} != x{}, skip {}", r1.get_raw(), r2.get_raw(), imm),
            Self::Blt(r1, r2, imm) =>
                format!("if x{} < x{}, skip {}", r1.get_raw(), r2.get_raw(), imm),
            Self::Ble(r1, r2, imm) =>
                format!("if x{} <= x{}, skip {}", r1.get_raw(), r2.get_raw(), imm),
            Self::Sys(imm) =>
                match imm {
                    Syscall::MoveBottom => "BOT".into(),
                    Syscall::MoveLeft => "LEFT".into(),
                    Syscall::MoveRight => "RIGHT".into(),
                    Syscall::MoveTop => "TOP".into(),
                    Syscall::Mine => "MINE".into(),
                    Syscall::Display => "DISPLAY".into(),
                    _ => "NOP".into(),
                }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syscall {
    Nop,
    MoveLeft,
    MoveRight,
    MoveTop,
    MoveBottom,
    Mine,
    Display,
    Scan,
    Stored,
    MaxStorage,
}

impl Syscall {
    pub fn from_raw(value: u16) -> Self {
        match value {
            1 => Self::MoveLeft,
            2 => Self::MoveRight,
            3 => Self::MoveTop,
            4 => Self::MoveBottom,
            5 => Self::Mine,
            6 => Self::Display,
            7 => Self::Scan,
            8 => Self::Stored,
            9 => Self::MaxStorage,
            _ => Self::Nop,
        }
    }

    pub fn to_raw(&self) -> u16 {
        match self {
            Self::MoveLeft => 1,
            Self::MoveRight => 2,
            Self::MoveTop => 3,
            Self::MoveBottom => 4,
            Self::Mine => 5,
            Self::Display => 6,
            Self::Scan => 7,
            Self::Stored => 8,
            Self::MaxStorage => 9,
            Self::Nop => 0,
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_address_masking() {
        // O endereço deve ser mascarado para 4 bits (0-15)
        let reg = RegisterAddress::new(17); // 10001 em binário
        assert_eq!(reg.get_raw(), 1);
    }

    #[test]
    fn test_lui_encoding_decoding() {
        let rd = RegisterAddress::new(5);
        let imm = 200;
        let inst = Instruction::new_lui(rd, imm as u16);
        
        if let UnpackedInstruction::Lui(unpacked_rd, unpacked_imm) = inst.unpack() {
            assert_eq!(unpacked_rd.get_raw(), 5);
            assert_eq!(unpacked_imm, 200);
        } else {
            panic!("Deveria ser uma instrução LUI");
        }
    }

    #[test]
    fn test_addi_encoding_decoding() {
        let rs = RegisterAddress::new(1);
        let rd = RegisterAddress::new(2);
        let imm = 10; // ADDI usa imm4
        let inst = Instruction::new_addi(rs, rd, imm);
        
        if let UnpackedInstruction::AddI(r_rs, r_rd, r_imm) = inst.unpack() {
            assert_eq!(r_rs.get_raw(), 1);
            assert_eq!(r_rd.get_raw(), 2);
            assert_eq!(r_imm, 10);
        } else {
            panic!("Deveria ser uma instrução ADDI");
        }
    }

    #[test]
    fn test_add_encoding_decoding() {
        let r1 = RegisterAddress::new(10);
        let r2 = RegisterAddress::new(11);
        let rd = RegisterAddress::new(12);
        let inst = Instruction::new_add(r1, r2, rd);
        
        if let UnpackedInstruction::Add(unpacked_r1, unpacked_r2, unpacked_rd) = inst.unpack() {
            assert_eq!(unpacked_r1.get_raw(), 10);
            assert_eq!(unpacked_r2.get_raw(), 11);
            assert_eq!(unpacked_rd.get_raw(), 12);
        } else {
            panic!("Deveria ser uma instrução ADD");
        }
    }

    #[test]
    fn test_beq_encoding_decoding() {
        let r1 = RegisterAddress::new(1);
        let r2 = RegisterAddress::new(2);
        let jump = 15;
        let inst = Instruction::new_beq(r1, r2, jump);
        
        if let UnpackedInstruction::Beq(u_r1, u_r2, u_jump) = inst.unpack() {
            assert_eq!(u_r1.get_raw(), 1);
            assert_eq!(u_r2.get_raw(), 2);
            assert_eq!(u_jump, 15);
        } else {
            panic!("Deveria ser uma instrução BEQ");
        }
    }

    #[test]
    fn test_syscall_conversion() {
        let sys = Syscall::Mine;
        let raw = sys.to_raw();
        assert_eq!(raw, 5);
        assert_eq!(Syscall::from_raw(5), Syscall::Mine);
        
        // Teste de Nop / Inválido
        assert_eq!(Syscall::from_raw(99), Syscall::Nop);
    }

    #[test]
    fn test_visualization_strings() {
        let inst = Instruction::new_add(
            RegisterAddress::new(1), 
            RegisterAddress::new(2), 
            RegisterAddress::new(3)
        );
        let vis = inst.unpack().get_visualization();
        assert_eq!(vis[0], "ADD");
        assert_eq!(vis[1], "x1");
        assert_eq!(vis[2], "x2");
        assert_eq!(vis[3], "x3");

        let simple_vis = inst.unpack().get_simplified_visualization();
        assert_eq!(simple_vis, "x3 = x1 + x2");
    }

    #[test]
    fn test_nop_visualization() {
        let inst = Instruction::new_sys(Syscall::Nop);
        let simple_vis = inst.unpack().get_simplified_visualization();
        assert_eq!(simple_vis, "NOP");
    }

    #[test]
    fn test_addi_truncation() {
        let rs = RegisterAddress::new(1);
        let rd = RegisterAddress::new(2);
        // 16 é 10000 em binário. Com a máscara & 0xf, deve virar 0.
        let inst = Instruction::new_addi(rs, rd, 16);
        
        if let UnpackedInstruction::AddI(_, _, imm) = inst.unpack() {
            assert_eq!(imm, 0, "O imediato de ADDI deve ser limitado a 4 bits (max 15)");
        }
    }
}
