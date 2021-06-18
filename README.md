# ruscaml
## Overview

ruscaml is a tiny Mincaml compiler by Rust. This compiler can interpret the some basic syntax of Mincaml.
<br>
<br>
ruscaml has some stages, and here is an overview of the internals.
<br>
1. compiles input program to AST.(use recursive descent parsing.)
2. convert AST to normal form which limits the expressions that can be written as expressions that are bound to variables by let and loop expressions.
3. apply closure transform to canonical form and convert to closed normal form.
4. smooths closed normal forms and removes the nesting of let rec syntax.
5. convert to virtual machine code, assuming there are innumerable physical registers.
6. allocate physical registers.
7. Output arm64 assembly code.

## Run

Build.

    $ cargo build  
 
Run main test of `./test/test.ml`.

    $ cargo test -- --nocapture

