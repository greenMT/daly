
use std::rc::Rc;

use kaktus::PushPop;

use super::{TraceInstruction, Comp, Value, Interpreter, CallFrame, Func};
use recovery::Guard;
use traits::vec::ConvertingStack;


pub struct Runner<'a, 'b: 'a> {
    pub trace: &'a [TraceInstruction],
    pub stack: Vec<Value>,
    pub locals: Vec<Value>,
    pub interp: &'a mut Interpreter<'b>,
}

impl<'a, 'b> Runner<'a, 'b> {
    pub fn new(interp: &'a mut Interpreter<'b>,
               trace: &'a [TraceInstruction],
               n_locals: usize)
               -> Self {
        // we have to copy over current stack frame from interpreter
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
            stack: Vec::new(),
            locals: locals,
        }
    }

    pub fn run(&mut self) -> (Rc<Func>, usize) {
        use TraceInstruction::*;

        let mut pc = 0;
        loop {
            let instr = &self.trace[pc];
            pc = (pc + 1) % self.trace.len();

            info!("TEXEC: {:?}", instr);

            match *instr {
                Add => self.add(),
                Cmp(how) => self.cmp(how),

                Load(idx) => self.load(idx),
                Store(idx) => self.store(idx),
                Const(val) => self.stack.push_from(val),

                ArrayGet => self.array_get(),

                Clone => (),

                Guard(ref guard) => {
                    match self.guard(guard) {
                        Ok(_) => (),
                        Err(recovery) => return recovery,
                    }
                }

                _ => unimplemented!(),
                // Array(usize),
                // Push,
                // Print,
                // Len,
            }

        }
    }

    // XXX: return None guard succeeds
    fn guard(&mut self, guard: &Guard) -> Result<(), (Rc<Func>, usize)> {
        let got = self.stack.pop_into::<bool>();
        if got == guard.condition {
            Ok(())
        } else {
            self.recover(guard);
            Err((guard.frame.func.clone(), guard.pc))
        }
    }

    /// the following things have to be recovered
    /// * stack-frames (call-frames)
    /// * value stack (essentially bool which caused guard to fail)
    fn recover(&mut self, guard: &Guard) {
        // remove the last callframe of the Interpreter
        // it gets replaced with our updated version
        self.interp.frames.pop().unwrap();

        // recover callframes
        // since callframes depend on each other, we start with the one which
        // was created first
        let frames = guard.frame.walk().collect::<Vec<_>>();
        for frame_info in frames.iter().rev() {
            // 1. create a new callframe
            let mut frame = CallFrame::for_fn(&*frame_info.func, frame_info.back_ref.clone());

            // 2. fill it up with locals
            for idx in 0..frame.locals.len() {
                frame.locals[idx] = self.locals[frame_info.offset + idx].clone();
            }

            // 3. add frame to interpreter callframes
            self.interp.frames.push(frame);
        }

        // recover value stack
        self.interp.stack.push_from(!guard.condition);
    }
}

/// normal interpreter functions
impl<'a, 'b> Runner<'a, 'b> {
    fn add(&mut self) {
        let (a, b) = self.stack.pop_2_into::<usize>();
        self.stack.push_from(a + b)
    }

    fn cmp(&mut self, how: Comp) {
        let (left, right) = self.stack.pop_2_into::<usize>();

        let b = match how {
            Comp::Lt => left < right,
            Comp::Le => left <= right,
            _ => panic!("TODO"),
        };

        self.stack.push_from(b);
    }

    fn load(&mut self, idx: usize) {
        let val = self.locals[idx].clone();
        self.stack.push(val);
    }

    fn store(&mut self, idx: usize) {
        self.locals[idx] = self.stack.pop_into();
    }

    fn array_get(&mut self) {
        let index: usize = self.stack.pop_into();
        let xs: Vec<usize> = self.stack.pop_into();
        self.stack.push_from(xs[index]);
    }
}
