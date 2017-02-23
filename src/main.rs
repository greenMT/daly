
#[macro_use]
extern crate maplit;

extern crate kaktus;

use kaktus::{PushPop, Stack};

use std::collections::BTreeMap;
use std::cmp::max;

use std::rc::Rc;

// impl From for Value and vice versa
mod conversions;
mod recovery;
mod tracerunner;
mod util;

use recovery::{Guard, FrameInfo};
use tracerunner::Runner;


pub struct Module {
    funcs: BTreeMap<String, Rc<Func>>,
}

#[derive(Debug)]
pub struct Trace {
    pub trace: Vec<TraceInstruction>,
    pub locals: usize,
}

#[derive(Debug, Clone)]
pub struct Func {
    name: String,
    args: usize,
    locals: usize,
    instr: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct FuncInfo {
    name: String,
    args: usize,
    locals: usize,
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
}


#[derive(Debug, Clone)]
pub enum TraceInstruction {
    Add,
    Cmp(Comp),

    Load(usize),
    Store(usize),

    Const(usize),

    Array(usize),
    ArrayGet,
    Push,

    // intrinsics
    Len,
    Print,
    Clone,

    Guard(Guard),
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Usize(usize),
    Array(Vec<usize>),
}


pub struct CallFrame {
    back_ref: (Rc<Func>, usize),
    args: usize,
    locals: Vec<Value>,
}

impl CallFrame {
    fn for_fn(func: &Func, back_ref: (Rc<Func>, usize)) -> Self {
        CallFrame {
            back_ref: back_ref.clone(),
            args: func.args,
            locals: vec![Value::Null; func.args+func.locals],
        }
    }
}



pub struct Interpreter<'a> {
    module: &'a Module,
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
}


impl<'a> Interpreter<'a> {
    fn new(module: &'a Module) -> Self {
        Interpreter {
            module: module,
            stack: Vec::new(),
            frames: Vec::new(),
        }
    }

    fn get_fn(&self, name: &str) -> Rc<Func> {
        self.module.funcs.get(name).unwrap().clone()
    }

    // XXX: why do I return func, pc? shouldn't that be the same as the input?
    fn trace(&mut self, o_func: Rc<Func>, o_pc: usize) -> (Rc<Func>, usize, Trace) {
        use Instruction::*;

        let mut pc = o_pc;
        let mut func = o_func.clone();

        let mut trace = Vec::new();

        // offset from where we can add new locals
        let mut stack_offset = func.args + func.locals;

        let mut call_tree = Stack::root(FrameInfo {
            func: o_func.clone(),
            back_ref: self.frames.last().unwrap().back_ref.clone(),
            offset: 0,
        });

        let mut stack_prefix = 0;
        let mut stack_prefixes = vec![0];
        let mut max_local = 0;

        loop {
            let instr = func.instr[pc].clone();
            // println!("    DO: {:?}", instr);

            pc += 1;
            match instr {
                Loop => {
                    break;
                }

                Break => (),

                Clone => (),

                Const(n) => self.do_const(n),
                Add => self.do_add(),

                Load(idx) => {
                    self.do_load(idx);
                    trace.push(TraceInstruction::Load(stack_prefix + idx));
                    continue;
                }

                Store(idx) => {
                    self.do_store(idx);
                    trace.push(TraceInstruction::Store(stack_prefix + idx));
                    max_local = max(max_local, stack_prefix + idx);
                    continue;

                }

                Print => self.do_print(),

                Array(size) => self.do_array(size),

                Len => self.do_len(),
                Push => self.do_push(),

                ArrayGet => self.do_array_get(),

                Call(ref target) => {
                    let new_func = self.module.funcs.get(target).unwrap().clone();
                    let mut frame = CallFrame::for_fn(&*new_func, (func.clone(), pc));

                    // guard = guard.push(TraceGuard::new(
                    // stack_offset,
                    // new_func.into()));

                    stack_prefixes.push(stack_prefix);
                    stack_prefix = stack_offset;

                    stack_offset += frame.locals.len();

                    // trace.push(Call("xxx".into()));
                    for idx in 0..frame.args {
                        frame.locals[idx] = self.stack.pop().unwrap();
                        trace.push(TraceInstruction::Store(stack_prefix + idx));
                        max_local = max(max_local, stack_prefix + idx);
                    }

                    // XXX: push frame instead?
                    call_tree = call_tree.push(FrameInfo {
                        func: new_func.clone(),
                        back_ref: frame.back_ref.clone(),
                        offset: stack_prefix,
                    });

                    self.frames.push(frame);


                    func = new_func;
                    pc = 0;
                    continue;
                }

                Return => {
                    stack_prefix = stack_prefixes.pop().unwrap();

                    let frame = self.frames.pop();
                    if self.frames.is_empty() {
                        break;
                    }
                    // trace.push(Return);

                    call_tree = call_tree.pop().unwrap();

                    let (f, rpc) = frame.unwrap().back_ref;
                    func = f;
                    pc = rpc;
                    continue;
                }

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

                    let guard = Guard {
                        condition: b,
                        frame: call_tree.clone(),
                        // reverse pc +1 above
                        pc: pc - 1,
                    };
                    trace.push(TraceInstruction::Guard(guard));
                    continue;

                }

                _ => panic!("TODO: {:?}", instr),
            }

            trace.push(TraceInstruction::from(instr));
        }

        // println!("{:?}", trace);

        let t = Trace {
            trace: trace,
            locals: max_local + 1,
        };

        (func, pc, t)
    }


    fn run(&mut self) {
        use Instruction::*;

        let main = self.get_fn("main");

        let mut pc = 0;

        self.frames.push(CallFrame::for_fn(&*main, (main.clone(), 0)));

        let mut func = main;

        let mut traces: BTreeMap<usize, Trace> = BTreeMap::new();

        loop {
            let instr = func.instr[pc].clone();
            // println!("I: {:?}", instr);

            pc += 1;
            match instr {
                Loop => {
                    if let Some(trace) = traces.get(&pc) {
                        // println!("{:?}", trace);
                        {
                            let mut runner = Runner::new(self, &trace.trace, trace.locals);
                            let res = runner.run();
                            func = res.0;
                            pc = res.1;
                        }

                        // println!("return from trace to func {:?} pc {:?}", func.name, pc);
                        // println!("STACK: {:?}", self.stack);
                        // println!("FRAME: {:?}", self.frames.last().unwrap().locals);
                        continue;
                    }

                    let res = self.trace(func, pc);
                    func = res.0;
                    pc = res.1;
                    traces.insert(pc, res.2);
                }

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

                    func = new_func.clone();
                    pc = 0;
                }

                Return => {
                    let frame = self.frames.pop();

                    if self.frames.is_empty() {
                        break;
                    }

                    let (f, rpc) = frame.unwrap().back_ref;
                    func = f;
                    pc = rpc;
                }

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
        where T: From<Value>
    {
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
            }.into(),

            "min".into() => Func {
                name: "min".into(),
                args: 2,
                locals: 0,
                instr: vec![Load(1), Load(0), Cmp(self::Comp::Le), JumpIfFalse(6), Load(0), Jump(8), Load(1), Jump(8), Clone, Return]
            }.into(),

            "min_list".into() => Func {
                name: "min_list".into(),
                args: 1,
                locals: 3,
                instr: vec![Load(0), Const(0), ArrayGet, Store(1), Load(0), Len, Store(2), Const(0), Store(3), Loop, Load(2), Load(3), Cmp(Comp::Lt), JumpIfFalse(25), Load(0), Load(3), ArrayGet, Load(1), Call(String::from("min")), Store(1), Load(3), Const(1), Add, Store(3), Jump(9), Break, Load(1), Print, Return],
            }.into(),
        }
    };

    let mut interpreter = Interpreter::new(&prog);

    interpreter.run();
}
