use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::Mutex;

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

pub static PROGRAM: Lazy<Mutex<String>> = Lazy::new(|| {
    let file: String = env::args().collect::<Vec<String>>().last().unwrap().clone();
    Mutex::new(fs::read_to_string(file).expect("failed to read file."))
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
            end = (*PROGRAM).lock().unwrap().len();
            break;
        }
        if tokenset.tokens[tokenset.pos + i].position.0 == true {
            end = tokenset.tokens[tokenset.pos + i].position.2 - 1;
            break;
        }
    }
    println!("Error: {} Line: {}.", message, tokenset.tokens[tokenset.pos].position.1);
    println!("\t{}", &(*PROGRAM).lock().unwrap()[start..end]);
    print!("\t");
    for _ in 0..tokenset.tokens[tokenset.pos].position.2 - start {
        print!(" ");
    }
    println!("^");
}

pub fn message_error(message: &str) {
    println!("Error: {}", message);
}

#[derive(Debug, Clone)]
struct Env<T, V> {
    vals: HashMap<T, V>,
    prev: Option<Box<Env<T, V>>>
}

impl<T: std::cmp::Eq + std::hash::Hash + std::fmt::Debug, V: std::fmt::Debug> Env<T, V> {
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
    fn addval(&mut self, key: T, value: V) {
        self.vals.insert(key, value);
    }
    fn find(&self, key: &T) -> Option<&V> {
        let mut nenv = self;
        loop {
            if let Some(value) = nenv.vals.get(key) {
                return Some(value);
            } else {
                match nenv.prev {
                    None => { message_error(&format!(" {:?} is not defined. ", key)); panic!("{:?}", &self); return None; }
                    Some(ref next_env) => { nenv = next_env; }
                }
            }
        }
    }
}

impl Env<NV, FV> {
    fn efind(&self, key: &NV) -> FV {
        if let normal::Value::Intv(v) = key {
            return FV::Intv(*v);
        }
        self.find(key).unwrap().clone()
    }
}

impl Env<String, Vec<vm::Byte>> {
    fn is_dummy(&self) -> bool {
        let dm = self.find(&String::from("$$$dummy")).unwrap();
        if dm.len() == 1 && dm[0] == -100 {
            return true;
        } else {
            panic!(" $$$dummy is not defined. ")
        }
    }
}