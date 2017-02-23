
use std::rc::Rc;
use super::{TraceInstruction, Comp, Value, Interpreter, CallFrame, Func};
use util::Stack;
use recovery::Guard;
use kaktus::PushPop;

pub struct Runner<'a, 'b: 'a> {
    pub trace: &'a [TraceInstruction],
    pub stack: Stack,
    pub locals: Vec<Value>,
    pub interp: &'a mut Interpreter<'b>,
}

impl<'a, 'b> Runner<'a, 'b> {
    pub fn new(interp: &'a mut Interpreter<'b>,
               trace: &'a [TraceInstruction],
               n_locals: usize)
               -> Self {
        let mut locals = vec![Value::Null; n_locals];
        {
            let interp_locals = &interp.frames.last().unwrap().locals;
            for idx in 0..interp_locals.len() {
                locals[idx] = interp_locals[idx].clone();
            }
        }

        Runner {
            interp: interp,
            trace: trace,
            stack: Stack::new(),
            locals: locals,
        }
    }

    pub fn run(&mut self) -> (Rc<Func>, usize) {
        use TraceInstruction::*;

        let mut pc = 0;
        loop {
            let instr = &self.trace[pc];
            pc = (pc + 1) % self.trace.len();

            // println!("    RUN: {:?}", instr);

            match *instr {
                Add => self.add(),
                Cmp(how) => self.cmp(how),

                Load(idx) => self.load(idx),
                Store(idx) => self.store(idx),
                Const(val) => self.stack.push(val),

                ArrayGet => self.array_get(),

                Clone => (),

                Guard(ref guard) => {
                    match self.guard(guard) {
                        Some(pc) => return pc,
                        None => (),
                    }
                },

                _ => unimplemented!(),
                // NOT needed
                // Array(usize),
                // Push,
                // Print,
                // Len,
            }

        }
    }


    fn add(&mut self) {
        let (a, b) = self.stack.pop_2::<usize>();
        self.stack.push(a + b)
    }

    fn cmp(&mut self, how: Comp) {
        let (left, right) = self.stack.pop_2::<usize>();

        let b = match how {
            Comp::Lt => left < right,
            Comp::Le => left <= right,
            _ => panic!("TODO"),
        };

        self.stack.push(b);
    }

    fn load(&mut self, idx: usize) {
        let val = self.locals[idx].clone();
        self.stack.push(val);

        // println!("STACK: {:?}", self.stack.stack);
    }

    fn store(&mut self, idx: usize) {
        self.locals[idx] = self.stack.pop();
    }

    fn array_get(&mut self) {
        let index: usize = self.stack.pop();
        let xs: Vec<usize> = self.stack.pop();
        self.stack.push(xs[index]);
    }

    fn guard(&mut self, guard: &Guard) -> Option<(Rc<Func>, usize)> {
        let got = self.stack.pop::<bool>();
        if got == guard.condition {
            None
        } else {
            self.recover(guard);
            Some((guard.frame.func.clone(), guard.pc))
        }
    }

    fn recover(&mut self, guard: &Guard) {
        // self.stack
        // stack frames
        // let mut frames = Vec::new();
        let chain = guard.frame.walk().collect::<Vec<_>>();

        // let mut last = &chain[0];//(*guard.frame).clone();

        // remove the last call frame of the Interpreter
        // it gets replaced with our updated version
        self.interp.frames.pop().unwrap();

        for info in chain.iter().rev() {
            // first get an empty call frame
            let mut frame = CallFrame::for_fn(&*info.func, info.back_ref.clone());

            // fill it up with values
            for idx in 0..frame.locals.len() {
                frame.locals[idx] = self.locals[info.offset + idx].clone();
            }
            // println!("{:?}", frame.locals);

            // and back to the Interpreter
            self.interp.frames.push(frame);
        }

        // push
        self.interp.push_stack(!guard.condition);
    }
}
