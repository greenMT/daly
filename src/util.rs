

use super::Value;

pub struct Stack {
    pub stack: Vec<Value>,
}

impl Stack {
    pub fn new() -> Self {
        Stack { stack: Vec::new() }
    }

    pub fn pop<T>(&mut self) -> T
        where T: From<Value>
    {
        self.stack.pop().unwrap().into()
    }

    pub fn pop_2<T>(&mut self) -> (T, T)
        where T: From<Value>
    {
        (self.stack.pop().unwrap().into(), self.stack.pop().unwrap().into())
    }

    pub fn push<T: Into<Value>>(&mut self, val: T) {
        self.stack.push(val.into());
    }
}
