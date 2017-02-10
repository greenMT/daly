
#[macro_use] extern crate maplit;

use std::collections::BTreeMap;

pub struct Module {
    funcs: BTreeMap<String, Func>,
}

pub struct Func {
    name: String,
    args: usize,
    locals: usize,
    instr: Vec<Instruction>,
}

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

    Guard(bool),
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Usize(usize),
    Array(Vec<usize>),
}

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


// usize
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

pub struct CallFrame<'a> {
    back_ref: (&'a Func, usize),
    args: usize,
    locals: Vec<Value>,
}

impl<'a> CallFrame<'a> {
    fn for_fn(func: &Func, back_ref: (&'a Func, usize)) -> Self {
        CallFrame {
            back_ref: back_ref,
            args: func.args,
            locals: vec![Value::Null; func.args+func.locals] }
    }
}

pub struct Interpreter<'a> {
    module: &'a Module,
    stack: Vec<Value>,
    frames: Vec<CallFrame<'a>>,
}


impl<'a> Interpreter<'a> {
    fn new(module: &'a Module) -> Self {
        Interpreter {
            module: module,
            stack: Vec::new(),
            frames: Vec::new(),
        }
    }

    fn get_fn(&self, name: &str) -> &'a Func {
        self.module.funcs.get(name).unwrap()
    }

    fn trace(&mut self, o_func: &'a Func, o_pc: usize) -> (&'a Func, usize) {
        use Instruction::*;

        let mut pc = o_pc;
        let mut func = o_func;

        let mut trace = Vec::new();
        let mut stack_size = 0;
        loop {
            let instr = &func.instr[pc];
            // println!("{:?}", instr);

            pc += 1;
            match *instr {
                Loop => {
                    break;
                },

                Break => (),

                Clone => (),

                Const(n) => self.do_const(n),
                Add => self.do_add(),

                Load(idx) => self.do_load(idx),
                Store(idx) => self.do_store(idx),

                Print => self.do_print(),

                Array(size) => self.do_array(size),

                Len => self.do_len(),
                Push => self.do_push(),

                ArrayGet => self.do_array_get(),

                Call(ref target) => {
                    let new_func = self.module.funcs.get(target).unwrap();
                    let mut frame = CallFrame::for_fn(new_func, (func, pc));

                    for idx in 0..frame.args {
                        frame.locals[idx] = self.stack.pop().unwrap();
                    }

                    self.frames.push(frame);

                    func = new_func;
                    pc = 0;
                    continue;
                },

                Return => {
                    let frame = self.frames.pop();

                    if self.frames.is_empty() {
                        break;
                    }

                    let (f, rpc) = frame.unwrap().back_ref;
                    func = f;
                    pc = rpc;
                    continue;
                },

                Cmp(how) => self.do_cmp(how),

                Jump(target) => {
                    pc = target;
                    // don't trace
                    continue;
                }

                JumpIfFalse(target) => {
                    let b: bool = self.pop();
                    if !bool::from(b) {
                        pc = target;
                    }

                    trace.push(Guard(b));
                    continue;

                }

                _ => panic!("TODO: {:?}", instr),
            }

            trace.push(instr.clone());
        }

        println!("{:?}", trace);

        (func, pc)
    }

    fn run(&mut self) {
        use Instruction::*;

        let main = self.get_fn("main");

        let mut pc = 0;

        self.frames.push(CallFrame::for_fn(&main, (&main, 0)));

        let mut func = main;

        loop {
            let instr = &func.instr[pc];
            // println!("{:?}", instr);

            pc += 1;
            match *instr {
                Loop => {
                    let res = self.trace(func, pc);
                    func = res.0;
                    pc = res.1;
                },

                Break => (),

                Clone => (),

                Const(n) => self.do_const(n),
                Add => self.do_add(),

                Load(idx) => self.do_load(idx),
                Store(idx) => self.do_store(idx),

                Print => self.do_print(),

                Array(size) => self.do_array(size),

                Len => self.do_len(),
                Push => self.do_push(),

                ArrayGet => self.do_array_get(),

                Call(ref target) => {
                    let new_func = self.module.funcs.get(target).unwrap();
                    let mut frame = CallFrame::for_fn(new_func, (func, pc));

                    for idx in 0..frame.args {
                        frame.locals[idx] = self.stack.pop().unwrap();
                    }

                    self.frames.push(frame);

                    func = new_func;
                    pc = 0;
                },

                Return => {
                    let frame = self.frames.pop();

                    if self.frames.is_empty() {
                        break;
                    }

                    let (f, rpc) = frame.unwrap().back_ref;
                    func = f;
                    pc = rpc;
                },

                Cmp(how) => self.do_cmp(how),

                Jump(target) => {
                    pc = target;
                }

                JumpIfFalse(target) => {
                    if !bool::from(self.stack.pop().unwrap()) {
                        pc = target;
                    }
                }

                _ => panic!("TODO: {:?}", instr),

            }
        }
    }

    fn push_stack<T: Into<Value>>(&mut self, val: T) {
        self.stack.push(val.into());
    }

    fn pop<T>(&mut self) -> T
    where T: From<Value>  {
        self.stack.pop().unwrap().into()
    }

    fn do_add(&mut self) {
        let left = self.pop::<usize>();
        let right = self.pop::<usize>();

        self.push_stack(left + right);
    }

    fn do_push(&mut self) {
        let val = self.pop();
        self.stack.last_mut().unwrap().as_mut().push(val);
    }

    fn do_const(&mut self, n: usize) {
        self.stack.push(n.into());
    }

    fn do_load(&mut self, idx: usize) {
        self.stack.push(self.frames.last_mut().unwrap().locals[idx].clone());
    }

    fn do_store(&mut self, idx: usize) {
        self.frames.last_mut().unwrap().locals[idx] = self.stack.pop().unwrap();
    }

    fn do_len(&mut self) {
        let v: Vec<usize> = self.pop();
        self.stack.push(v.len().into());
    }

    fn do_print(&mut self) {
        if let Value::Usize(v) = self.stack.pop().unwrap() {
            println!("{:?}", v);
        }
    }

    fn do_array(&mut self, capacity: usize) {
        self.stack.push(Vec::with_capacity(capacity).into());
    }

    fn do_array_get(&mut self) {
        let index: usize = self.pop();
        let xs: Vec<usize> = self.pop();
        self.stack.push(xs[index].into());

    }

    fn do_cmp(&mut self, how: Comp) {
        let left: usize = self.pop();
        let right: usize = self.pop();

        let b = match how {
            Comp::Lt => left < right,
            Comp::Le => left <= right,
            _ => panic!("TODO"),
        };

        self.stack.push(b.into());
    }
}

fn main() {
    use Instruction::*;
    let prog = Module {
        funcs: btreemap!{
            "main".into() => Func {
                name: "main".into(),
                args: 0,
                locals: 0,
                instr: vec![Array(8), Const(9), Push, Const(3), Push, Const(4), Push, Const(5), Push, Const(6), Push, Const(1), Push, Const(3), Push, Const(2), Push, Const(4), Push, Call(String::from("min_list")), Return],
            },

            "min".into() => Func {
                name: "min".into(),
                args: 2,
                locals: 0,
                instr: vec![Load(1), Load(0), Cmp(self::Comp::Le), JumpIfFalse(6), Load(0), Jump(8), Load(1), Jump(8), Clone, Return]
            },

            "min_list".into() => Func {
                name: "print".into(),
                args: 1,
                locals: 3,
                instr: vec![Load(0), Const(0), ArrayGet, Store(1), Load(0), Len, Store(2), Const(0), Store(3), Loop, Load(2), Load(3), Cmp(Comp::Le), JumpIfFalse(25), Load(0), Load(3), ArrayGet, Load(1), Call(String::from("min")), Store(1), Load(3), Const(1), Add, Store(3), Jump(9), Break, Load(1), Print, Return],
            }
        }
    };

    let mut interpreter = Interpreter::new(&prog);

    interpreter.run();
}
