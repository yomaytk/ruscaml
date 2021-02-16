use std::fs;
use std::io::{BufWriter, Write};

fn main() {
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
}
