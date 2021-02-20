extern crate ruscaml;

use std::env;
use std::fs;
use std::io::{BufWriter, Write};

use ruscaml::lexer::*;
use ruscaml::parser::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let content = fs::read_to_string(&args[args.len()-1])?;
    
    let tokens = tokenize(content);
    
    // println!("{:?}", tokens);

    let ast = toplevel(tokens);

    println!("{:?}", ast);

    let mut f = BufWriter::new(fs::File::create("a.s").unwrap());
    
    f.write(b".text\n").unwrap();
    f.write(b".global main\n").unwrap();
    f.write(b"main:\n").unwrap();
    f.write(b"\tpush %rbp\n").unwrap();
    f.write(b"\tmov %rsp, %rbp\n").unwrap();
    f.write(b"\tmov $1, %rax\n").unwrap();
    f.write(b"\tmov %rbp, %rsp\n").unwrap();
    f.write(b"\tpop %rbp\n").unwrap();
    f.write(b"\tret\n").unwrap();

    Ok(())
}
