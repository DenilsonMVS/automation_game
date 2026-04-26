use core::fmt;
use nalgebra_glm as glm;

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

#[derive(Debug, PartialEq, Eq)]
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
    pub fn format_assembly(&self) -> String {
        match self {
            UnpackedInstruction::Lui(rd, imm) => 
                format!("lui  {:<3}{:<3}", rd, imm),
            
            UnpackedInstruction::AddI(rs, rd, imm) => 
                format!("addi {:<3}{:<3}{:<3}", rd, rs, imm),
            
            UnpackedInstruction::SubI(rs, rd, imm) => 
                format!("subi {:<3}{:<3}{:<3}", rd, rs, imm),
            
            UnpackedInstruction::Add(r1, r2, rd) => 
                format!("add  {:<3}{:<3}{:<3}", rd, r1, r2),
            
            UnpackedInstruction::Sub(r1, r2, rd) => 
                format!("sub  {:<3}{:<3}{:<3}", rd, r1, r2),
            
            UnpackedInstruction::Mul(r1, r2, rd) => 
                format!("mul  {:<3}{:<3}{:<3}", rd, r1, r2),
            
            UnpackedInstruction::Div(r1, r2, rd) => 
                format!("div  {:<3}{:<3}{:<3}", rd, r1, r2),
            
            UnpackedInstruction::Rem(r1, r2, rd) => 
                format!("rem  {:<3}{:<3}{:<3}", rd, r1, r2),

            UnpackedInstruction::Beq(r1, r2, imm) => 
                format!("beq  {:<3}{:<3}{:<3}", r1, r2, imm),
            
            UnpackedInstruction::Bne(r1, r2, imm) => 
                format!("bne  {:<3}{:<3}{:<3}", r1, r2, imm),
            
            UnpackedInstruction::Blt(r1, r2, imm) => 
                format!("blt  {:<3}{:<3}{:<3}", r1, r2, imm),
            
            UnpackedInstruction::Ble(r1, r2, imm) => 
                format!("ble  {:<3}{:<3}{:<3}", r1, r2, imm),

            UnpackedInstruction::Seq(r1, r2, rd) => 
                format!("seq  {:<3}{:<3}{:<3}", rd, r1, r2),
            
            UnpackedInstruction::Slt(r1, r2, rd) => 
                format!("slt  {:<3}{:<3}{:<3}", rd, r1, r2),
            
            UnpackedInstruction::Sle(r1, r2, rd) => 
                format!("sle  {:<3}{:<3}{:<3}", rd, r1, r2),

            UnpackedInstruction::Sys(syscall) => 
                format!("sys  {}", syscall),
        }
    }

    pub fn format_with_colors<'a>(&self, asm: &'a str) -> [(&'a str, glm::U8Vec4); 4] {
        let mnemonic = glm::vec4(199, 146, 234, 255); // Roxo
        let dest     = glm::vec4(137, 221, 255, 255); // Azul
        let reg1     = glm::vec4(240, 113, 120, 255); // Coral
        let reg2     = glm::vec4(247, 140, 108, 255); // Pêssego
        let literal  = glm::vec4(195, 232, 141, 255); // Verde Lima

        let get_slice = |start: usize, end: usize| -> &'a str {
            let s = start.min(asm.len());
            let e = end.min(asm.len());
            &asm[s..e]
        };

        match self {
            UnpackedInstruction::Lui(_, _) => [
                (get_slice(0, 5),  mnemonic), // "lui  "
                (get_slice(5, 9),  dest),     // "x14 " (3+1)
                (get_slice(9, 13), literal),  // "1   "
                (get_slice(0, 0), literal),
            ],

            UnpackedInstruction::AddI(_, _, _) | 
            UnpackedInstruction::SubI(_, _, _) |
            UnpackedInstruction::Add(_, _, _)  |
            UnpackedInstruction::Sub(_, _, _)  |
            UnpackedInstruction::Mul(_, _, _)  |
            UnpackedInstruction::Div(_, _, _)  |
            UnpackedInstruction::Rem(_, _, _)  |
            UnpackedInstruction::Seq(_, _, _)  |
            UnpackedInstruction::Slt(_, _, _)  |
            UnpackedInstruction::Sle(_, _, _) => [
                (get_slice(0, 5),   mnemonic), // "addi "
                (get_slice(5, 9),   dest),     // "x3  "
                (get_slice(9, 13),  reg1),     // "x15 "
                (get_slice(13, 17), literal),  // "0   "
            ],

            UnpackedInstruction::Beq(_, _, _) |
            UnpackedInstruction::Bne(_, _, _) |
            UnpackedInstruction::Blt(_, _, _) |
            UnpackedInstruction::Ble(_, _, _) => [
                (get_slice(0, 5),   mnemonic),
                (get_slice(5, 9),   reg1),
                (get_slice(9, 13),  reg2),
                (get_slice(13, 17), literal),
            ],

            UnpackedInstruction::Sys(_) => [
                (get_slice(0, 5), mnemonic),
                (get_slice(5, asm.len()), literal),
                (get_slice(0, 0), literal),
                (get_slice(0, 0), literal),
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
