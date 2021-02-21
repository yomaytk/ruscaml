use once_cell::sync::Lazy;
use std::env;
use std::fs;

pub mod lexer;
pub mod parser;

type Id = String;

pub static PROGRAM: Lazy<String> = Lazy::new(|| {
    let file: String = env::args().collect::<Vec<String>>().last().unwrap().clone();
    fs::read_to_string(file).expect("failed to read file.")
});