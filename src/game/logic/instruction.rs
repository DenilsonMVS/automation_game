use core::fmt;
use nalgebra_glm as glm;

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
    RsRdImm4,    // Ex: addi rd, rs, imm4
    R1R2Rd,      // Ex: add rd, r1, r2
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
            Self::Subi => InstructionFormat::RsRdImm4,
            
            Self::Add | Self::Sub | 
            Self::Mul | Self::Div | 
            Self::Rem | Self::Seq | 
            Self::Slt | Self::Sle => InstructionFormat::R1R2Rd,
            
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
    fn pack(id: InstructionId, r1: u16, r2: u16, r3_or_imm: u16) -> Self {
        Self {
            data: ((id as u16) << 12) | ((r1 & 0xF) << 8) | ((r2 & 0xF) << 4) | (r3_or_imm & 0xF),
        }
    }

    pub fn new_lui(write: RegisterAddress, value: u16) -> Instruction {
        Self {
            data: ((InstructionId::Lui as u16) << 12) | (write.get_raw() << 8) | (value & 0xff),
        }
    }

    pub fn new_addi(read: RegisterAddress, write: RegisterAddress, value: u16) -> Instruction {
        Self::pack(InstructionId::Addi, read.get_raw(), write.get_raw(), value)
    }

    pub fn new_subi(read: RegisterAddress, write: RegisterAddress, value: u16) -> Instruction {
        Self::pack(InstructionId::Subi, read.get_raw(), write.get_raw(), value)
    }

    pub fn new_add(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Add, read1.get_raw(), read2.get_raw(), write.get_raw())
    }

    pub fn new_sub(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Sub, read1.get_raw(), read2.get_raw(), write.get_raw())
    }

    pub fn new_mul(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Mul, read1.get_raw(), read2.get_raw(), write.get_raw())
    }

    pub fn new_div(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Div, read1.get_raw(), read2.get_raw(), write.get_raw())
    }

    pub fn new_rem(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Rem, read1.get_raw(), read2.get_raw(), write.get_raw())
    }

    pub fn new_beq(read1: RegisterAddress, read2: RegisterAddress, jump: u16) -> Instruction {
        Self::pack(InstructionId::Beq, read1.get_raw(), read2.get_raw(), jump)
    }

    pub fn new_bne(read1: RegisterAddress, read2: RegisterAddress, jump: u16) -> Instruction {
        Self::pack(InstructionId::Bne, read1.get_raw(), read2.get_raw(), jump)
    }

    pub fn new_blt(read1: RegisterAddress, read2: RegisterAddress, jump: u16) -> Instruction {
        Self::pack(InstructionId::Blt, read1.get_raw(), read2.get_raw(), jump)
    }

    pub fn new_ble(read1: RegisterAddress, read2: RegisterAddress, jump: u16) -> Instruction {
        Self::pack(InstructionId::Ble, read1.get_raw(), read2.get_raw(), jump)
    }

    pub fn new_seq(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Seq, read1.get_raw(), read2.get_raw(), write.get_raw())
    }

    pub fn new_slt(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Slt, read1.get_raw(), read2.get_raw(), write.get_raw())
    }

    pub fn new_sle(read1: RegisterAddress, read2: RegisterAddress, write: RegisterAddress) -> Instruction {
        Self::pack(InstructionId::Sle, read1.get_raw(), read2.get_raw(), write.get_raw())
    }

    pub fn new_sys(value: Syscall) -> Instruction {
        Self {
            data: ((InstructionId::Sys as u16) << 12) | value.to_raw(),
        }
    }

    pub fn unpack(&self) -> UnpackedInstruction {
        let id = InstructionId::from_u16(self.data >> 12);
        
        let r1 = RegisterAddress::new(((self.data >> 8) & 0xF) as u8);
        let r2 = RegisterAddress::new(((self.data >> 4) & 0xF) as u8);
        let r3 = RegisterAddress::new((self.data & 0xF) as u8);
        
        let imm4 = (self.data & 0xF) as u8;
        let imm8 = (self.data & 0xFF) as u8;
        let imm12 = self.data & 0xFFF;

        match id {
            InstructionId::Lui  => UnpackedInstruction::Lui(r1, imm8),
            InstructionId::Addi => UnpackedInstruction::Addi(r1, r2, imm4),
            InstructionId::Subi => UnpackedInstruction::Subi(r1, r2, imm4),
            InstructionId::Add  => UnpackedInstruction::Add(r1, r2, r3),
            InstructionId::Sub  => UnpackedInstruction::Sub(r1, r2, r3),
            InstructionId::Mul  => UnpackedInstruction::Mul(r1, r2, r3),
            InstructionId::Div  => UnpackedInstruction::Div(r1, r2, r3),
            InstructionId::Rem  => UnpackedInstruction::Rem(r1, r2, r3),
            InstructionId::Beq  => UnpackedInstruction::Beq(r1, r2, imm4),
            InstructionId::Bne  => UnpackedInstruction::Bne(r1, r2, imm4),
            InstructionId::Blt  => UnpackedInstruction::Blt(r1, r2, imm4),
            InstructionId::Ble  => UnpackedInstruction::Ble(r1, r2, imm4),
            InstructionId::Seq  => UnpackedInstruction::Seq(r1, r2, r3),
            InstructionId::Slt  => UnpackedInstruction::Slt(r1, r2, r3),
            InstructionId::Sle  => UnpackedInstruction::Sle(r1, r2, r3),
            InstructionId::Sys  => UnpackedInstruction::Sys(Syscall::from_raw(imm12)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnpackedInstruction {
    Lui(RegisterAddress, u8),
    Addi(RegisterAddress, RegisterAddress, u8),
    Subi(RegisterAddress, RegisterAddress, u8),
    Add(RegisterAddress, RegisterAddress, RegisterAddress),
    Sub(RegisterAddress, RegisterAddress, RegisterAddress),
    Mul(RegisterAddress, RegisterAddress, RegisterAddress),
    Div(RegisterAddress, RegisterAddress, RegisterAddress),
    Rem(RegisterAddress, RegisterAddress, RegisterAddress),
    Beq(RegisterAddress, RegisterAddress, u8),
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

    pub fn format_assembly(&self) -> String {
        let id = self.get_id();
        match self {
            UnpackedInstruction::Lui(rd, imm) => 
                format!("{} {:<3}{:<3}", id, rd, imm),
            
            UnpackedInstruction::Addi(rs, rd, imm) |
            UnpackedInstruction::Subi(rs, rd, imm) => 
                format!("{} {:<3}{:<3}{:<3}", id, rd, rs, imm),
            
            UnpackedInstruction::Add(r1, r2, rd) |
            UnpackedInstruction::Sub(r1, r2, rd) |
            UnpackedInstruction::Mul(r1, r2, rd) |
            UnpackedInstruction::Div(r1, r2, rd) |
            UnpackedInstruction::Rem(r1, r2, rd) |
            UnpackedInstruction::Seq(r1, r2, rd) |
            UnpackedInstruction::Slt(r1, r2, rd) |
            UnpackedInstruction::Sle(r1, r2, rd) => 
                format!("{} {:<3}{:<3}{:<3}", id, rd, r1, r2),

            UnpackedInstruction::Beq(r1, r2, imm) |
            UnpackedInstruction::Bne(r1, r2, imm) |
            UnpackedInstruction::Blt(r1, r2, imm) |
            UnpackedInstruction::Ble(r1, r2, imm) => 
                format!("{} {:<3}{:<3}{:<3}", id, r1, r2, imm),

            UnpackedInstruction::Sys(syscall) => 
                format!("{} {}", id, syscall),
        }
    }

    pub fn get_mnemonic_color() -> glm::U8Vec4 {
        glm::vec4(199, 146, 234, 255) // Roxo
    }

    pub fn get_dest_color() -> glm::U8Vec4 {
        glm::vec4(137, 221, 255, 255) // Azul
    }

    pub fn get_reg1_color() -> glm::U8Vec4 {
        glm::vec4(240, 113, 120, 255) // Coral
    }

    pub fn get_reg2_color() -> glm::U8Vec4 {
        glm::vec4(247, 140, 108, 255) // Pêssego
    }

    pub fn get_literal_color() -> glm::U8Vec4 {
        glm::vec4(195, 232, 141, 255) // Verde Lima
    }

    pub fn format_with_colors<'a>(&self, asm: &'a str) -> [(&'a str, glm::U8Vec4); 4] {
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
                (get_slice(0, 5), mnemonic), // "lui "
                (get_slice(5, 9), dest),     // "x14 "
                (get_slice(9, 13), literal), // "1 "
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
                (get_slice(0, 5), mnemonic),  // "addi "
                (get_slice(5, 9), dest),      // "x3 "
                (get_slice(9, 13), reg1),     // "x15 "
                (get_slice(13, 17), literal), // "0 "
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

            UnpackedInstruction::Sys(_) => [
                (get_slice(0, 5), mnemonic),
                (get_slice(5, asm.len()), literal),
                nothing,
                nothing,
            ],
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
