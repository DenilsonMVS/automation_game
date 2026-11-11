use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum InstructionId {
    Lui  = 0,
    Addi = 1,
    Subi = 2,
    Add  = 3,
    Sub  = 4,
    Mul  = 5,
    Div  = 6,
    Rem  = 7,
    Beq  = 8,
    Bne  = 9,
    Blt  = 10,
    Ble  = 11,
    Seq  = 12,
    Slt  = 13,
    Sle  = 14,
    Sys  = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionFormat {
    RdImm8,      // Ex: lui rd, imm8
    RdRsImm4,    // Ex: addi rd, rs, imm4
    RdR1R2,      // Ex: add rd, r1, r2
    R1R2Imm4,    // Ex: beq r1, r2, imm4
    Syscall,     // Ex: sys <syscall_name>
}

impl InstructionId {
    pub fn from_u16(value: u16) -> Self {
        match value & 0xF {
            0  => Self::Lui,
            1  => Self::Addi,
            2  => Self::Subi,
            3  => Self::Add,
            4  => Self::Sub,
            5  => Self::Mul,
            6  => Self::Div,
            7  => Self::Rem,
            8  => Self::Beq,
            9  => Self::Bne,
            10 => Self::Blt,
            11 => Self::Ble,
            12 => Self::Seq,
            13 => Self::Slt,
            14 => Self::Sle,
            _  => Self::Sys,
        }
    }

    pub fn get_format(&self) -> InstructionFormat {
        match self {
            Self::Lui  => InstructionFormat::RdImm8,
            
            Self::Addi | 
            Self::Subi => InstructionFormat::RdRsImm4,
            
            Self::Add | Self::Sub | 
            Self::Mul | Self::Div | 
            Self::Rem | Self::Seq | 
            Self::Slt | Self::Sle => InstructionFormat::RdR1R2,
            
            Self::Beq | Self::Bne | 
            Self::Blt | Self::Ble => InstructionFormat::R1R2Imm4,
            
            Self::Sys  => InstructionFormat::Syscall,
        }
    }

    pub fn get_label_max_len() -> usize {
        4
    }
}

impl fmt::Display for InstructionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Lui  => "lui ",
            Self::Addi => "addi",
            Self::Subi => "subi",
            Self::Add  => "add ",
            Self::Sub  => "sub ",
            Self::Mul  => "mul ",
            Self::Div  => "div ",
            Self::Rem  => "rem ",
            Self::Beq  => "beq ",
            Self::Bne  => "bne ",
            Self::Blt  => "blt ",
            Self::Ble  => "ble ",
            Self::Seq  => "seq ",
            Self::Slt  => "slt ",
            Self::Sle  => "sle ",
            Self::Sys  => "sys ",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterAddress {
    value: u8,
}

impl fmt::Display for RegisterAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x{:<3}", self.value)
    }
}

impl RegisterAddress {
    pub const fn new(address: u8) -> Self {
        Self { value: address & 0xf }
    }

    pub const fn get_raw(&self) -> u16 {
        self.value as u16
    }
}

impl From<RegisterAddress> for usize {
    fn from(reg: RegisterAddress) -> Self {
        reg.value as usize
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
    fn pack(id: InstructionId, p1: u16, p2: u16, p3: u16) -> Self {
        Self {
            data: ((id as u16) << 12) | ((p1 & 0xF) << 8) | ((p2 & 0xF) << 4) | (p3 & 0xF),
        }
    }

    pub fn new_lui(rd: RegisterAddress, value: u16) -> Instruction {
        Self {
            data: ((InstructionId::Lui as u16) << 12) | (rd.get_raw() << 8) | (value & 0xff),
        }
    }

    pub fn new_addi(rd: RegisterAddress, rs: RegisterAddress, value: u16) -> Instruction {
        Self::pack(InstructionId::Addi, rd.get_raw(), rs.get_raw(), value)
    }

    pub fn new_subi(rd: RegisterAddress, rs: RegisterAddress, value: u16) -> Instruction {
        Self::pack(InstructionId::Subi, rd.get_raw(), rs.get_raw(), value)
    }

    pub fn new_add(rd: RegisterAddress, r1: RegisterAddress, r2: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Add, rd.get_raw(), r1.get_raw(), r2.get_raw())
    }

    pub fn new_sub(rd: RegisterAddress, r1: RegisterAddress, r2: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Sub, rd.get_raw(), r1.get_raw(), r2.get_raw())
    }

    pub fn new_mul(rd: RegisterAddress, r1: RegisterAddress, r2: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Mul, rd.get_raw(), r1.get_raw(), r2.get_raw())
    }

    pub fn new_div(rd: RegisterAddress, r1: RegisterAddress, r2: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Div, rd.get_raw(), r1.get_raw(), r2.get_raw())
    }

    pub fn new_rem(rd: RegisterAddress, r1: RegisterAddress, r2: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Rem, rd.get_raw(), r1.get_raw(), r2.get_raw())
    }

    pub fn new_beq(r1: RegisterAddress, r2: RegisterAddress, jump: u16) -> Instruction {
        Self::pack(InstructionId::Beq, r1.get_raw(), r2.get_raw(), jump)
    }

    pub fn new_bne(r1: RegisterAddress, r2: RegisterAddress, jump: u16) -> Instruction {
        Self::pack(InstructionId::Bne, r1.get_raw(), r2.get_raw(), jump)
    }

    pub fn new_blt(r1: RegisterAddress, r2: RegisterAddress, jump: u16) -> Instruction {
        Self::pack(InstructionId::Blt, r1.get_raw(), r2.get_raw(), jump)
    }

    pub fn new_ble(r1: RegisterAddress, r2: RegisterAddress, jump: u16) -> Instruction {
        Self::pack(InstructionId::Ble, r1.get_raw(), r2.get_raw(), jump)
    }

    pub fn new_seq(rd: RegisterAddress, r1: RegisterAddress, r2: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Seq, rd.get_raw(), r1.get_raw(), r2.get_raw())
    }

    pub fn new_slt(rd: RegisterAddress, r1: RegisterAddress, r2: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Slt, rd.get_raw(), r1.get_raw(), r2.get_raw())
    }

    pub fn new_sle(rd: RegisterAddress, r1: RegisterAddress, r2: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Sle, rd.get_raw(), r1.get_raw(), r2.get_raw())
    }

    pub fn new_sys(value: Syscall) -> Instruction {
        Self {
            data: ((InstructionId::Sys as u16) << 12) | value.to_raw(),
        }
    }

    pub fn unpack(&self) -> UnpackedInstruction {
        let id = InstructionId::from_u16(self.data >> 12);
        
        let p1 = RegisterAddress::new(((self.data >> 8) & 0xF) as u8);
        let p2 = RegisterAddress::new(((self.data >> 4) & 0xF) as u8);
        let p3 = RegisterAddress::new((self.data & 0xF) as u8);
        
        let imm4 = (self.data & 0xF) as u8;
        let imm8 = (self.data & 0xFF) as u8;
        let imm12 = self.data & 0xFFF;

        match id {
            InstructionId::Lui  => UnpackedInstruction::Lui(p1, imm8),
            InstructionId::Addi => UnpackedInstruction::Addi(p1, p2, imm4),
            InstructionId::Subi => UnpackedInstruction::Subi(p1, p2, imm4),
            InstructionId::Add  => UnpackedInstruction::Add(p1, p2, p3),
            InstructionId::Sub  => UnpackedInstruction::Sub(p1, p2, p3),
            InstructionId::Mul  => UnpackedInstruction::Mul(p1, p2, p3),
            InstructionId::Div  => UnpackedInstruction::Div(p1, p2, p3),
            InstructionId::Rem  => UnpackedInstruction::Rem(p1, p2, p3),
            InstructionId::Beq  => UnpackedInstruction::Beq(p1, p2, imm4),
            InstructionId::Bne  => UnpackedInstruction::Bne(p1, p2, imm4),
            InstructionId::Blt  => UnpackedInstruction::Blt(p1, p2, imm4),
            InstructionId::Ble  => UnpackedInstruction::Ble(p1, p2, imm4),
            InstructionId::Seq  => UnpackedInstruction::Seq(p1, p2, p3),
            InstructionId::Slt  => UnpackedInstruction::Slt(p1, p2, p3),
            InstructionId::Sle  => UnpackedInstruction::Sle(p1, p2, p3),
            InstructionId::Sys  => UnpackedInstruction::Sys(Syscall::from_raw(imm12)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnpackedInstruction {
    Lui(RegisterAddress, u8),
    Addi(RegisterAddress, RegisterAddress, u8),             // rd, rs, imm
    Subi(RegisterAddress, RegisterAddress, u8),             // rd, rs, imm
    Add(RegisterAddress, RegisterAddress, RegisterAddress), // rd, r1, r2
    Sub(RegisterAddress, RegisterAddress, RegisterAddress),
    Mul(RegisterAddress, RegisterAddress, RegisterAddress),
    Div(RegisterAddress, RegisterAddress, RegisterAddress),
    Rem(RegisterAddress, RegisterAddress, RegisterAddress),
    Beq(RegisterAddress, RegisterAddress, u8),              // r1, r2, imm
    Bne(RegisterAddress, RegisterAddress, u8),
    Blt(RegisterAddress, RegisterAddress, u8),
    Ble(RegisterAddress, RegisterAddress, u8),
    Seq(RegisterAddress, RegisterAddress, RegisterAddress),
    Slt(RegisterAddress, RegisterAddress, RegisterAddress),
    Sle(RegisterAddress, RegisterAddress, RegisterAddress),
    Sys(Syscall),
}

impl UnpackedInstruction {
    pub fn get_id(&self) -> InstructionId {
        match self {
            Self::Lui(..)  => InstructionId::Lui,
            Self::Addi(..) => InstructionId::Addi,
            Self::Subi(..) => InstructionId::Subi,
            Self::Add(..)  => InstructionId::Add,
            Self::Sub(..)  => InstructionId::Sub,
            Self::Mul(..)  => InstructionId::Mul,
            Self::Div(..)  => InstructionId::Div,
            Self::Rem(..)  => InstructionId::Rem,
            Self::Beq(..)  => InstructionId::Beq,
            Self::Bne(..)  => InstructionId::Bne,
            Self::Blt(..)  => InstructionId::Blt,
            Self::Ble(..)  => InstructionId::Ble,
            Self::Seq(..)  => InstructionId::Seq,
            Self::Slt(..)  => InstructionId::Slt,
            Self::Sle(..)  => InstructionId::Sle,
            Self::Sys(..)  => InstructionId::Sys,
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
    Release,
}

impl fmt::Display for Syscall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Syscall::Nop => "nop",
            Syscall::MoveLeft => "move_left",
            Syscall::MoveRight => "move_right",
            Syscall::MoveTop => "move_top",
            Syscall::MoveBottom => "move_bottom",
            Syscall::Mine => "mine",
            Syscall::Display => "display",
            Syscall::Scan => "scan",
            Syscall::Stored => "stored",
            Syscall::MaxStorage => "max_storage",
            Syscall::Release => "release",
        };
        write!(f, "{}", name)
    }
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
            10 => Self::Release,
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
            Self::Release => 10,
            Self::Nop => 0,
        }
    }
}
