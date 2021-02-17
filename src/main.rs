use std::env;
use std::fs;
use std::io::{BufWriter, Write};

#[derive(Clone, Copy, Debug, PartialEq)]
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

pub struct TokenVec {
    pub tokenvec: Vec<Token>,
    pub pos: usize,
}

impl TokenVec {
    fn assert_ttype(&mut self, ttype: TokenType) {
        assert_eq!(self.tokenvec[self.pos].tokentype, ttype);
        self.pos += 1;
    }
    fn consume_ttype(&mut self, ttype: TokenType) -> bool {
        if ttype == self.tokenvec[self.pos].tokentype {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn curtype(&self) -> TokenType {
        self.tokenvec[self.pos].tokentype
    }
    fn curnum(&self) -> i32 {
        self.tokenvec[self.pos].num
    }
    fn eof(&self) {
        assert_eq!(self.pos+1, self.tokenvec.len());
    }
}

pub enum Expr {
    LtExpr,
}

#[derive(Clone, Debug)]
pub enum Ast {
    IntV(i32),
    Binop(TokenType, Box<Ast>, Box<Ast>),
}

fn reserved_token(s: &str, pos: &mut usize) -> Option<TokenType> {
    if &s[0..1] == "+" {
        *pos += 1;
        Some(TokenType::Plus)
    } else if &s[0..1] == "*" {
        *pos += 1;
        Some(TokenType::Mult)
    } else if &s[0..1] == "<" {
        *pos += 1;
        Some(TokenType::Lt)
    } else if &s[0..2] == ";;" {
        *pos += 2;
        Some(TokenType::Semisemi)
    } else {
        None
    }
}

fn tokenize(program: String) -> Vec<Token> {
    let mut tokens = vec![];
    let mut pos: usize = 0;
    while pos < program.len()-1 {
        // reservec token type
        if let Some(tokentype) = reserved_token(&program[pos..], &mut pos) {
            tokens.push(
                Token {
                    tokentype,
                    num: 0,
                }
            );
            continue;
        }
        // not reserved
        let nchar = &program[pos..pos+1];
        match nchar {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                tokens.push(
                    Token {
                        tokentype: TokenType::ILit,
                        num: (&nchar).parse().unwrap()
                    }
                );
                pos += 1;
            }
            " " => {
                pos += 1;
            }
            _ => {
                panic!("tokenize error.")
            }
        }
    }
    return tokens;
}

fn aexpr(tokenvec: &mut TokenVec) -> Ast {
    match tokenvec.curtype() {
        TokenType::ILit => {
            let num = tokenvec.curnum();
            tokenvec.pos += 1;
            Ast::IntV(num)
        }
        _ => {
            panic!("aexpr error.")
        }
    }
}

fn mexpr(tokenvec: &mut TokenVec) -> Ast {
    let mut ast = aexpr(tokenvec);
    while tokenvec.consume_ttype(TokenType::Mult) {
        ast = Ast::Binop(TokenType::Mult, Box::new(ast), Box::new(aexpr(tokenvec)));
    }
    ast
}

fn pexpr(tokenvec: &mut TokenVec) -> Ast {
    let mut ast = mexpr(tokenvec);
    while tokenvec.consume_ttype(TokenType::Plus) {
        ast = Ast::Binop(TokenType::Plus, Box::new(ast), Box::new(pexpr(tokenvec)));
    }
    ast
}

fn ltexpr(tokenvec: &mut TokenVec) -> Ast {
    let last = pexpr(tokenvec);
    if tokenvec.consume_ttype(TokenType::Lt) {
        let rast = pexpr(tokenvec);
        return Ast::Binop(TokenType::Lt, Box::new(last), Box::new(rast));
    }
    last
}

fn expr(tokenvec: &mut TokenVec) -> Ast {
    let ast = ltexpr(tokenvec);
    tokenvec.eof();
    ast
}

fn toplevel(tokens: Vec<Token>) -> Ast {
    let mut tokenvec = TokenVec {
        tokenvec: tokens,
        pos: 0
    };
    let ast = expr(&mut tokenvec);
    tokenvec.assert_ttype(TokenType::Semisemi);
    ast
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let content = fs::read_to_string(&args[1])?;
    
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
