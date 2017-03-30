
use std::rc::Rc;
use std::fmt;

use kaktus::Stack;

use super::Func;
use repr::InstrPtr;


pub struct FrameInfo {
    pub func: Rc<Func>,
    pub back_ref: InstrPtr,
    // offset of inlined values
    pub offset: usize,
}


#[derive(Clone)]
pub struct Guard {
    // condition guard protects
    pub condition: bool,
    // frame information to recover from
    pub frame: Stack<FrameInfo>,
    // pc position where execution can continue
    pub pc: usize,
}

impl fmt::Debug for Guard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-{:?}-", self.condition)
    }
}
