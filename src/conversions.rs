
// XXX: there might be a macro which implements From/Into for enums

use super::*;

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<Value> for bool {
    fn from(val: Value) -> Self {
        match val {
            Value::Bool(b) => b,
            _ => panic!("unexpeted Array variant"),
        }
    }
}

impl From<Vec<usize>> for Value {
    fn from(xs: Vec<usize>) -> Self {
        Value::Array(xs)
    }
}

impl From<Value> for Vec<usize> {
    fn from(val: Value) -> Self {
        match val {
            Value::Array(xs) => xs,
            _ => panic!("unexpeted Array variant"),
        }
    }
}

impl AsMut<Vec<usize>> for Value {
    fn as_mut(&mut self) -> &mut Vec<usize> {
        match *self {
            Value::Array(ref mut xs) => xs,
            _ => panic!("unexpeted Array variant"),
        }
    }
}


impl From<Value> for usize {
    fn from(val: Value) -> Self {
        match val {
            Value::Usize(n) => n,
            _ => panic!("unexpeted Array variant"),
        }
    }
}

impl From<usize> for Value {
    fn from(n: usize) -> Self {
        Value::Usize(n)
    }
}


impl<'a> From<&'a Instruction> for TraceInstruction {
    fn from(instr: &Instruction) -> TraceInstruction {
        use Instruction as I;
        use TraceInstruction as TI;

        match *instr {
            I::Add => TI::Add,
            I::Cmp(c) => TI::Cmp(c),
            I::Const(c) => TI::Const(c),
            I::Len => TI::Len,
            I::Print => TI::Print,
            I::Clone => TI::Clone,
            I::Array(u) => TI::Array(u),
            I::ArrayGet => TI::ArrayGet,

            _ => panic!("can not convert {:?}", instr),
        }
    }
}
