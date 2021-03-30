extern crate ruscaml;

use ruscaml::lexer::*;
use ruscaml::parser::*;
use ruscaml::normal::*;
use ruscaml::closure::*;

// use std::io::{BufWriter, Write};
// use std::env;
// use std::fs;

fn main() {
    
    // let args: Vec<String> = env::args().collect();
    
    let tokenset = tokenize();
    
    // println!("{:?}", tokenset.tokens);

    let ast = parse(tokenset);

    let norm_ast = normalize(ast);
    // norm_ast.program_display();
    
    let closed_norm = closure(norm_ast);
    closed_norm.program_display();
        
    // let mut f = BufWriter::new(fs::File::create("a.s").unwrap());
    
    // f.write(b".text\n").unwrap();
    // f.write(b".global main\n").unwrap();
    // f.write(b"main:\n").unwrap();
    // f.write(b"\tpush %rbp\n").unwrap();
    // f.write(b"\tmov %rsp, %rbp\n").unwrap();
    // f.write(b"\tmov $1, %rax\n").unwrap();
    // f.write(b"\tmov %rbp, %rsp\n").unwrap();
    // f.write(b"\tpop %rbp\n").unwrap();
    // f.write(b"\tret\n").unwrap();
}
