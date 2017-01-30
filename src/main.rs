
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

#[derive(Debug)]
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

pub struct Interpreter;


impl Interpreter {

    fn run(&mut self, module: &Module) {
        use Instruction::*;

        let main = module.funcs.get("main").unwrap();

        let mut stack = Vec::new();
        let mut frames = Vec::new();
        let mut pc = 0;

        frames.push(CallFrame::for_fn(&main, (&main, 0)));

        let mut func = main;
        loop {
            let instr = &func.instr[pc];
            // println!("{:?}", instr);

            pc += 1;
            match *instr {
                Loop | Break => (),
                Clone => (),

                Const(n) => stack.push(Value::Usize(n)),

                Add => {
                    let left: usize = stack.pop().unwrap().into();
                    let right: usize = stack.pop().unwrap().into();

                    stack.push((left + right).into());
                }

                Load(idx) => {
                    stack.push(frames.last_mut().unwrap().locals[idx].clone());
                }

                Store(idx) => {
                    frames.last_mut().unwrap().locals[idx] = stack.pop().unwrap();
                }

                Print => {
                    if let Value::Usize(v) = stack.pop().unwrap() {
                        println!("{:?}", v);
                    }
                },

                Array(size) => {
                    stack.push(Vec::with_capacity(size).into());
                },

                Len => {
                    let v: Vec<usize> = stack.pop().unwrap().into();
                    stack.push(v.len().into());
                }

                Push => {
                    // let mut v: &Vec<usize> = &mut
                    let val = stack.pop().unwrap().into();
                    stack.last_mut().unwrap().as_mut().push(val);
                }

                ArrayGet => {
                    let index: usize = stack.pop().unwrap().into();
                    let xs: Vec<usize> = stack.pop().unwrap().into();
                    stack.push(xs[index].into());
                }

                Call(ref target) => {
                    let new_func = module.funcs.get(target).unwrap();
                    let mut frame = CallFrame::for_fn(new_func, (func, pc));

                    for idx in 0..frame.args {
                        frame.locals[idx] = stack.pop().unwrap();
                    }

                    frames.push(frame);

                    func = new_func;
                    pc = 0;
                },

                Return => {
                    let frame = frames.pop();

                    if frames.is_empty() {
                        break;
                    }

                    let (f, rpc) = frame.unwrap().back_ref;
                    func = f;
                    pc = rpc;
                },

                Cmp(how) => {
                    let left: usize = stack.pop().unwrap().into();
                    let right: usize = stack.pop().unwrap().into();

                    let b = match how {
                        Comp::Lt => left < right,
                        Comp::Le => left <= right,
                        _ => panic!("TODO"),
                    };

                    stack.push(b.into());
                },

                Jump(target) => {
                    pc = target;
                }

                JumpIfFalse(target) => {
                    if !bool::from(stack.pop().unwrap()) {
                        pc = target;
                    }
                }

                _ => panic!("TODO: {:?}", instr),

            }
        }
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
                instr: vec![Load(0), Const(0), ArrayGet, Store(1), Load(0), Len, Store(2), Const(0), Store(3), Load(2), Load(3), Cmp(Comp::Lt), JumpIfFalse(24), Load(0), Load(3), ArrayGet, Load(1), Call(String::from("min")), Store(1), Load(3), Const(1), Add, Store(3), Jump(9), Load(1), Print, Return],
            }
        }
    };

    let mut interpreter = Interpreter;

    interpreter.run(&prog);
}
