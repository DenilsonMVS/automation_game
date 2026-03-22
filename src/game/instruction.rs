
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    NoOp,
    MoveTop,
    MoveLeft,
    MoveRight,
    MoveBottom,

    Write(u8, u8), // Write(value, write_address)
    Goto(u8), // Goto(instruction)

    // Inst(read_address, write_address)
    Move(u8, u8),
    
    // Inst(read_address_1, read_address_2, write_address)
    Add(u8, u8, u8),
    Sub(u8, u8, u8),
    Mul(u8, u8, u8),
    Div(u8, u8, u8),
    Rem(u8, u8, u8),
    
    And(u8, u8, u8),
    Or(u8, u8, u8),
    Diff(u8, u8, u8),
    Eq(u8, u8, u8),
    Not(u8, u8), // Inst(read_address, write_address)
    Greater(u8, u8, u8),
    GreaterEq(u8, u8, u8),
    Smaller(u8, u8, u8),
    SmallerEq(u8, u8, u8),

    BitAnd(u8, u8, u8),
    BitOr(u8, u8, u8),
    BitXor(u8, u8, u8),
    BitEq(u8, u8, u8),
    BitNot(u8, u8), // Inst(read_address, write_address)

    // Inst(read_address_1, read_address_2, instruction)
    EqJump(u8, u8, u8),
    DiffJump(u8, u8, u8),
    GreaterJump(u8, u8, u8),
    GreaterEqJump(u8, u8, u8),
    SmallerJump(u8, u8, u8),
    SmallerEqJump(u8, u8, u8),
}

impl Default for Instruction {
    fn default() -> Self {
        Self::NoOp
    }
}
