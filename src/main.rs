
// btreemap! macro
#[macro_use]
extern crate maplit;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate boolinator;
extern crate kaktus;

use std::collections::BTreeMap;
use std::rc::Rc;

use kaktus::{PushPop, Stack};

use bytecode::{Instruction, Comp};
use recovery::{Guard, FrameInfo};
use tracerunner::Runner;
use repr::{CallFrame, Func, InstrPtr, Value};

use traits::vec::ConvertingStack;

mod bytecode;
mod conversions;
mod recovery;
mod tracerunner;
mod traits;
mod repr;


pub type TraceMap = BTreeMap<usize, Trace>;
pub type ModuleMap = BTreeMap<String, Rc<Func>>;


pub struct Module {
    funcs: ModuleMap,
}


#[derive(Debug)]
pub struct Trace {
    pub trace: Vec<TraceInstruction>,
    pub locals_count: usize,
}

impl Trace {
    fn new(trace: Vec<TraceInstruction>, locals_count: usize) -> Self {
        Trace {
            trace: trace,
            locals_count: locals_count,
        }
    }
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



struct TraceDataAllocator {
    total_size: usize,
    offsets: Vec<usize>,
}

impl TraceDataAllocator {
    fn new() -> Self {
        TraceDataAllocator {
            total_size: 0,
            offsets: Vec::new(),
        }
    }

    fn alloc(&mut self, to_allocate: usize) {
        self.offsets.push(self.total_size);
        // reserve space at the end
        self.total_size += to_allocate;
    }

    fn pop(&mut self) {
        self.offsets.pop().unwrap();
    }

    fn current(&self) -> usize {
        *self.offsets.last().unwrap()
    }

    fn at(&self, idx: usize) -> usize {
        self.current() + idx
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
        self.module.funcs[name].clone()
    }

    // XXX: why do I return func, pc? shouldn't that be the same as the input?
    fn trace(&mut self, instr: &InstrPtr) -> (InstrPtr, Trace) {
        use Instruction::*;

        let mut trace = Vec::new();

        let mut call_tree = Stack::root(FrameInfo {
            func: instr.func.clone(),
            back_ref: self.frames.last().unwrap().back_ref.clone(),
            offset: 0,
        });

        let mut locals = TraceDataAllocator::new();
        locals.alloc(instr.func.args_count + instr.func.locals_count);

        let mut next = instr.clone();

        loop {
            let instr = next;
            next = instr.next();

            info!(target: "exec","TRACE: {:?}", instr);

            match *instr {
                Loop => break,

                Break => unimplemented!(),

                Clone => (),

                Const(n) => self.do_const(n),
                Add => self.do_add(),

                Load(idx) => {
                    self.do_load(idx);
                    trace.push(TraceInstruction::Load(locals.at(idx)));
                    continue;
                }

                Store(idx) => {
                    self.do_store(idx);
                    trace.push(TraceInstruction::Store(locals.at(idx)));
                    continue;
                }

                Print => self.do_print(),

                Array(size) => self.do_array(size),

                Len => self.do_len(),
                Push => self.do_push(),

                ArrayGet => self.do_array_get(),

                Call(ref target) => {
                    let new_func = &self.module.funcs[target];
                    let mut frame = CallFrame::for_fn(new_func, next);

                    locals.alloc(frame.locals.len());

                    for idx in 0..frame.args_count {
                        frame.locals[idx] = self.stack.pop().unwrap();
                        trace.push(TraceInstruction::Store(locals.at(idx)));
                    }

                    call_tree = call_tree.push(FrameInfo {
                        func: new_func.clone(),
                        back_ref: frame.back_ref.clone(),
                        offset: locals.current(),
                    });

                    self.frames.push(frame);
                    next = InstrPtr::for_fn(new_func.clone());

                    // don't add Call to trace
                    continue;
                }

                Return => {
                    locals.pop();

                    let frame = self.frames.pop();
                    if self.frames.is_empty() {
                        break;
                    }

                    call_tree = call_tree.pop().unwrap();

                    next = frame.unwrap().back_ref;

                    // don't add Return to trace
                    continue;
                }

                Cmp(how) => self.do_cmp(how),

                Jump(target) => {
                    next = next.jump(target);
                    // skip trace
                    continue;
                }

                JumpIfFalse(target) => {
                    let b: bool = self.stack.pop_into();
                    if !bool::from(b) {
                        next = next.jump(target);
                    }

                    let guard = Guard {
                        condition: b,
                        frame: call_tree.clone(),
                        pc: instr.pc,
                    };
                    trace.push(TraceInstruction::Guard(guard));
                    continue;
                }

                _ => panic!("TODO: {:?}", instr),
            }

            trace.push(TraceInstruction::from(&*instr));
        }

        info!(target: "trace", "{:?}", trace);

        (instr.clone(), Trace::new(trace, locals.total_size))
    }

    fn run(&mut self) {
        use Instruction::*;

        // `main` function has to exist
        let main = self.get_fn("main");

        // a bit awkward, main would return to main
        // maybe it would be better to have Option as back_ref
        self.frames.push(CallFrame::for_fn(&main, InstrPtr::new(main.clone(), 0)));

        let mut traces = TraceMap::new();
        let mut next = InstrPtr::for_fn(main.clone());

        loop {
            // get next instruction
            let instr = next;
            // pre-set next instruction
            next = instr.next();

            info!("E: {:?}", *instr);

            match *instr {
                // XXX: do I care about break here?
                Break | Clone => (),

                // simple dispatch of opcodes to callbacks
                Const(n)    => self.do_const(n),
                Add         => self.do_add(),
                Load(idx)   => self.do_load(idx),
                Store(idx)  => self.do_store(idx),
                Print       => self.do_print(),
                Array(size) => self.do_array(size),
                Len         => self.do_len(),
                Push        => self.do_push(),
                ArrayGet    => self.do_array_get(),
                Cmp(how)    => self.do_cmp(how),

                // XXX: currently there is no threshhold value when to start tracing
                // meaning that tracing starts immediately
                Loop => {
                    // do we already have a trace for this position?
                    if let Some(trace) = traces.get(&instr.pc) {
                        // we need this block, since Runner takes self as &mut
                        {
                            info!("T: running trace @{:}[{:}]", instr.func.name, instr.pc);
                            let mut runner = Runner::new(self, trace);
                            next = runner.run();
                        }
                        info!("T: return from trace to func {:?} pc {:?}", next.func.name, next.pc);
                        info!("T: STACK: {:?}", self.stack);
                        info!("T: FRAME: {:?}", self.frames.last().unwrap().locals);
                        continue;
                    }

                    // no trace found => start tracing (with next instr)
                    let res = self.trace(&next);
                    next = res.0;
                    traces.insert(instr.pc, res.1);
                }

                Call(ref target) => {
                    let new_func = &self.module.funcs[target];
                    let mut frame = CallFrame::for_fn(new_func, next);

                    // pass arguments to function locals
                    for idx in 0..frame.args_count {
                        frame.locals[idx] = self.stack
                            .pop()
                            .expect("Not enough arguments passed");
                    }

                    self.frames.push(frame);
                    next = InstrPtr::for_fn(new_func.clone());
                }

                Return => {
                    // remove latest callframe
                    let old_frame = self.frames
                        .pop()
                        .expect("Return from non existing frame.");

                    // did we return from main function?
                    if self.frames.is_empty() {
                        break;
                    } else {
                        next = old_frame.back_ref;
                    }
                }

                Jump(target) => {
                    next = instr.jump(target);
                }

                JumpIfFalse(target) => {
                    if let false = self.stack.pop_into::<bool>() {
                        next = instr.jump(target);
                    }
                }

                _ => panic!("TODO: {:?}", instr),
            }
        }
    }

    fn do_add(&mut self) {
        let (left, right) = self.stack.pop_2_into::<usize>();
        self.stack.push_from(left + right);
    }

    fn do_push(&mut self) {
        let val = self.stack.pop_into();
        self.stack.last_mut().unwrap().as_mut().push(val);
    }

    fn do_const(&mut self, n: usize) {
        self.stack.push_from(n);
    }

    fn do_load(&mut self, idx: usize) {
        self.stack.push(self.frames.last_mut().unwrap().locals[idx].clone());
    }

    fn do_store(&mut self, idx: usize) {
        self.frames.last_mut().unwrap().locals[idx] = self.stack.pop().unwrap();
    }

    fn do_len(&mut self) {
        let v: Vec<usize> = self.stack.pop_into();
        self.stack.push_from(v.len());
    }

    fn do_print(&mut self) {
        if let Value::Usize(v) = self.stack.pop().unwrap() {
            println!("{:?}", v);
        }
    }

    fn do_array(&mut self, capacity: usize) {
        self.stack.push_from(Vec::with_capacity(capacity));
    }

    fn do_array_get(&mut self) {
        let index: usize = self.stack.pop_into();
        let xs: Vec<usize> = self.stack.pop_into();
        self.stack.push_from(xs[index]);

    }

    fn do_cmp(&mut self, how: Comp) {
        let (left, right) = self.stack.pop_2_into::<usize>();
        self.stack.push_from(match how {
            Comp::Lt => left < right,
            Comp::Le => left <= right,
            _ => panic!("TODO"),
        });
    }
}


fn main() {
    use Instruction::*;

    env_logger::init().unwrap();

    let prog = Module {
        funcs: btreemap!{
            "main".into() => Func {
                name: "main".into(),
                args_count: 0,
                locals_count: 0,
                instrs: vec![Array(8), Const(9), Push, Const(3), Push, Const(4), Push, Const(5), Push, Const(6), Push, Const(1), Push, Const(3), Push, Const(2), Push, Const(4), Push, Call(String::from("min_list")), Return],
            }.into(),

            "min".into() => Func {
                name: "min".into(),
                args_count: 2,
                locals_count: 0,
                instrs: vec![Load(1), Load(0), Cmp(self::Comp::Le), JumpIfFalse(6), Load(0), Jump(8), Load(1), Jump(8), Clone, Return]
            }.into(),

            "min_list".into() => Func {
                name: "min_list".into(),
                args_count: 1,
                locals_count: 3,
                instrs: vec![Load(0), Const(0), ArrayGet, Store(1), Load(0), Len, Store(2), Const(0), Store(3), Loop, Load(2), Load(3), Cmp(Comp::Lt), JumpIfFalse(25), Load(0), Load(3), ArrayGet, Load(1), Call(String::from("min")), Store(1), Load(3), Const(1), Add, Store(3), Jump(9), Break, Load(1), Print, Return],
            }.into(),
        }
    };

    let mut interpreter = Interpreter::new(&prog);
    interpreter.run();
}
