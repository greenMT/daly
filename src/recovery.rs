
use std::rc::Rc;

use kaktus::Stack;
use super::Func;

use std::fmt;

// #[derive(Debug, Clone)]
// pub struct Func {
//     pub name: String,
//     // number of arguments
//     pub args: usize,
//     // number of local variables (excluding args)
//     pub locals: usize,
//     pub instr: Vec<Instruction>,
// }


pub struct FrameInfo {
    pub func: Rc<Func>,
    pub back_ref: (Rc<Func>, usize),
    pub offset: usize,
}

#[derive(Clone)]
pub struct Guard {
    // condition guard protects
    pub condition: bool,

    pub frame: Stack<FrameInfo>,
    // instruction in frame.instructions
    pub pc: usize, /* offset of locals to trace vars
                    * pub locals_offset: usize, */
}


impl fmt::Debug for Guard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-{:?}-", self.condition)
    }
}
