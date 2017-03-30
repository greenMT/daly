
use boolinator::Boolinator;
use kaktus::PushPop;

use super::{TraceInstruction, Comp, Value, Interpreter, CallFrame, Trace};
use recovery::Guard;
use traits::vec::ConvertingStack;
use repr::InstrPtr;

pub struct Runner<'a, 'b: 'a> {
    pub trace: &'a [TraceInstruction],
    pub stack: Vec<Value>,
    pub locals: Vec<Value>,
    pub interp: &'a mut Interpreter<'b>,
}

impl<'a, 'b> Runner<'a, 'b> {
    pub fn new(interp: &'a mut Interpreter<'b>, trace: &'a Trace) -> Self {
        // we have to copy over current stack frame from interpreter
        let mut locals = vec![Value::Null; trace.locals_count];
        {
            let interp_locals = &interp.frames.last().unwrap().locals;
            for idx in 0..interp_locals.len() {
                locals[idx] = interp_locals[idx].clone();
            }
        }

        Runner {
            interp: interp,
            trace: &trace.trace,
            stack: Vec::new(),
            locals: locals,
        }
    }

    pub fn run(&mut self) -> InstrPtr {
        use TraceInstruction::*;

        let mut pc = 0;
        loop {
            let instr = &self.trace[pc];
            pc = (pc + 1) % self.trace.len();

            info!("TEXEC: {:?}", instr);

            match *instr {
                Add        => self.add(),
                Cmp(how)   => self.cmp(how),
                Load(idx)  => self.load(idx),
                Store(idx) => self.store(idx),
                ArrayGet   => self.array_get(),
                Const(val) => self.stack.push_from(val),
                Clone      => {}

                Guard(ref guard) => {
                    if let Err(recovery) = self.check_guard(guard) {
                        return recovery;
                    }
                }

                // these opcodes are not needed for example
                Array(_) | Push | Print | Len => unimplemented!(),
            }
        }
    }

    fn check_guard(&mut self, guard: &Guard) -> Result<(), InstrPtr> {
        let check = self.stack.pop_into::<bool>() == guard.condition;
        check.ok_or_else(||{
                self.recover(guard);
                InstrPtr::new(guard.frame.func.clone(), guard.pc)
            })
    }

    /// Recovery (aka Blackholing)
    ///
    /// Execution has reached a point, where the trace isn't valid anymore.
    /// The goal is to return to the interpreter, but the state has to be
    /// recovered first.
    ///
    /// The following states have to be recovered:
    ///     * stack-frames (call-frames)
    ///       The failed guard might have failed within an inlined function call.
    ///       Thus, we have to reconstruct all missing callframes, before the
    ///       the interpreter can gain back control.
    ///       Second, we also have to consider the frame where the loop resides
    ///       in, since state might have also has changed there.
    ///
    ///     * value stack
    ///       Also the operand stack has to be recovered.
    ///       Foremost, the condition, which caused the guard to fail, has to be
    ///       restored.
    ///       TODO: Are there other values which might have to be recovered?
    fn recover(&mut self, guard: &Guard) {
        // remove the last callframe of the Interpreter
        // it gets replaced with our updated version
        self.interp.frames.pop().unwrap();

        // recover callframes
        let frames = guard.frame.walk().collect::<Vec<_>>();

        // since callframes depend on each other, we start with the one which
        // was created first (least-recent frame) `.rev()` ensures that
        for frame_info in frames.iter().rev() {
            // 1. create a new callframe to push
            let mut frame = CallFrame::for_fn(
                &frame_info.func,
                frame_info.back_ref.clone());

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

// normal interpreter functions
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
