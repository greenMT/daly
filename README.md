# daly
Simple VM for a Dyon--subset

To run use:

    cargo run
  
To see what is happening internally, you can enable logging:

    RUST_LOG=daly cargo run 

## Current state

* `main.rs` implements a simple interpreter for some dyon-bytecode. `main()` contains the bytecode of [this](https://github.com/greenMT/example-programs/blob/master/example-programs/dyon/min_loop.dyon) program.

* the interpreter traces execution of loops

* `tracerunner.rs` contains an independent execution engine for generated traces


### Optimisations

**Inlining**
Inlining of function calls is performed. Necessary steps for deoptimisation can be found in `tracerunner::Runner::recover`.
