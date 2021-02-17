use std::env;
use std::fs;
use std::io::{BufWriter, Write};

#[derive(Clone, Copy, Debug)]
pub enum TokenType {
    Semisemi,
    ILit,
    Plus,
    Mult,
    Lt,
}

#[derive(Clone, Copy, Debug)]
pub struct Token {
    pub tokentype: TokenType,
    pub num: i32,
}

pub enum Expr {
    LtExpr,
}

pub enum AstType {
    Ilit(i32),
    Binop(BinType, Expr, Expr)
}

#[derive(Clone, Copy)]
pub enum BinType {
    Plus,
    Mult,
    Lt,
}

fn ToTokenType(c: &str) -> TokenType {
    match c {
       "+" => TokenType::Plus,
       "-" => TokenType::Mult,
       "<" => TokenType::Lt,
       _ => panic!("ToBinType error.")
    }
}

fn tokenize(mut program: String) -> Vec<Token> {
    let mut tokens = vec![];
    while program.len() > 0 {
        let nchar = program.drain(..1).collect::<String>();
        match &nchar[..] {
            "+" | "-" | "<" => {
                tokens.push(
                    Token {
                        tokentype: ToTokenType(&nchar),
                        num: -1,
                    }
                );
            }
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                tokens.push(
                    Token {
                        tokentype: TokenType::ILit,
                        num: (&nchar).parse().unwrap()
                    }
                )
            }
            " " => {}
            _ => {
                panic!("tokenize error.")
            }
        }
    }
    return tokens;
}

// fn toplevel() {

// }

fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let content = fs::read_to_string(&args[1])?;
    
    let _tokens = tokenize(content);

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
