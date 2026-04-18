
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

impl Into<usize> for RegisterAddress {
    fn into(self) -> usize {
        self.get_raw() as usize
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
