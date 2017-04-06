
use std::rc::Rc;
use std::ops::Deref;

use bytecode::Instruction;

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub args_count: usize,
    pub locals_count: usize,
    pub instrs: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Usize(usize),
    Array(Vec<usize>),
}


pub struct CallFrame {
    pub back_ref: InstrPtr,
    pub args_count: usize,
    pub locals: Vec<Value>,
}

impl CallFrame {
    pub fn for_fn(func: &Func, back_ref: InstrPtr) -> Self {
        CallFrame {
            back_ref: back_ref,
            args_count: func.args_count,
            locals: vec![Value::Null; func.args_count + func.locals_count],
        }
    }
}


#[derive(Debug, Clone)]
pub struct InstrPtr {
    pub func: Rc<Func>,
    pub pc: usize,
}

impl InstrPtr {
    pub fn new(func: Rc<Func>, pc: usize) -> Self {
        InstrPtr {
            func: func,
            pc: pc,
        }
    }

    pub fn for_fn(func: Rc<Func>) -> Self {
        InstrPtr::new(func, 0)
    }

    pub fn next(&self) -> Self {
        InstrPtr::new(self.func.clone(), self.pc + 1)
    }

    pub fn jump(&self, target: usize) -> Self {
        InstrPtr::new(self.func.clone(), target)
    }

    pub fn pc(&self) -> usize {
        self.pc
    }
}

impl Deref for InstrPtr {
    type Target = Instruction;

    fn deref(&self) -> &Instruction {
        &self.func.instrs[self.pc]
    }
}
