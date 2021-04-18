use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::fs;

pub mod lexer;
pub mod parser;
pub mod normal;
pub mod closure;
pub mod flat;
pub mod vm;
pub mod regalloc;
pub mod codegen;

use lexer::*;

type Id = String;
type NV = normal::Value;
type FV = flat::Value;

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
    println!("Error: {}", message);
}

trait Environment<T, V, U> {
    fn find(&self, key : &T) -> V;
    fn addval(&mut self, key: T, _: U);
}

#[derive(Debug, Clone)]
struct Env<T, V> {
    vals: HashMap<T, V>,
    prev: Option<Box<Env<T, V>>>
}

impl<T, V> Env<T, V> {
    fn new() -> Self {
        Self {
            vals: HashMap::new(),
            prev: None
        }
    }
    fn inc(&mut self) {
        let curenv = std::mem::replace(self, Env::new());
        *self = Self {
            vals: HashMap::new(),
            prev: Some(Box::new(curenv))
        }
    }
    fn dec(&mut self) {
        let env = std::mem::replace(&mut (*self).prev, None);
        *self = *env.unwrap()
    }
}

impl Environment<NV, FV, bool> for Env<NV, FV> {
    fn find(&self, key: &NV) -> FV {
        if let normal::Value::Intv(v) = key {
            return FV::Intv(*v);
        }
        let mut nenv = self;
        loop {
            if let Some(value) = nenv.vals.get(key) {
                return value.clone()
            } else {
                match nenv.prev {
                    None => { panic!("cannot find variable from Env. : {:?}, variable: {:?}", self, key); }
                    Some(ref next_env) => { nenv = next_env; }
                }
            }
        }
    }
    fn addval(&mut self, key: NV, tf: bool) {
        self.vals.insert(key.clone(), FV::nval2fval(key, tf));
    }
}

impl Environment<String, i32, i32> for Env<String, i32> {
    fn find(&self, key: &String) -> i32 {
        let mut nenv = self;
        loop {
            if let Some(value) = nenv.vals.get(key) {
                return *value;
            } else {
                match nenv.prev {
                    None => { panic!(" {} is not defined. ", key); }
                    Some(ref next_env) => { nenv = next_env; }
                }
            }
        }
    }
    fn addval(&mut self, key: String, value: i32) {
        self.vals.insert(key, value);
    }
}