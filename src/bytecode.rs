

#[derive(Debug, Clone, Copy)]
pub enum Comp {
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
}


#[derive(Debug, Clone)]
pub enum Instruction {
    Call(String),
    Return,

    Add,
    Cmp(Comp),

    Jump(usize),
    JumpIfTrue(usize),
    JumpIfFalse(usize),

    Load(usize),
    Store(usize),
    Const(usize),

    Array(usize),
    ArrayGet,
    Push,

    Loop,
    Break,

    // intrinsics
    Len,
    Print,
    Clone,
}
