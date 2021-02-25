use once_cell::sync::Lazy;
use std::env;
use std::fs;

pub mod lexer;
pub mod parser;

use lexer::*;

type Id = String;

pub static PROGRAM: Lazy<String> = Lazy::new(|| {
    let file: String = env::args().collect::<Vec<String>>().last().unwrap().clone();
    fs::read_to_string(file).expect("failed to read file.")
});

pub fn compile_error(tokenset: &TokenSet, message: &str) {
    let mut start: usize = 0;
    let mut end: usize = std::usize::MAX;

    for i in 0..std::usize::MAX {
        if tokenset.pos - i == 0 || tokenset.tokens[tokenset.pos-i].position.0 == true {
            start = tokenset.tokens[tokenset.pos - i].position.2;
            break;
        }
    }
    for i in 0..std::usize::MAX {
        if tokenset.pos + i == tokenset.tokens.len()-1 {
            end = (*PROGRAM).len();
            break;
        }
        if tokenset.tokens[tokenset.pos + i].position.0 == true {
            end = tokenset.tokens[tokenset.pos + i].position.2 - 1;
            break;
        }
    }
    println!("Error: {} Line: {}.", message, tokenset.tokens[tokenset.pos].position.1);
    println!("\t{}", &(*PROGRAM)[start..end]);
    print!("\t");
    for _ in 0..tokenset.tokens[tokenset.pos].position.2 - start {
        print!(" ");
    }
    println!("^");
}

pub fn message_error(message: &str) {
    println!("{}", message);
}