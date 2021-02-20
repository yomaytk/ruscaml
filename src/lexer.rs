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
    pub fn assert_ttype(&mut self, ttype: TokenType) {
        assert_eq!(self.tokenvec[self.pos].tokentype, ttype);
        self.pos += 1;
    }
    pub fn consume_ttype(&mut self, ttype: TokenType) -> bool {
        if ttype == self.tokenvec[self.pos].tokentype {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    pub fn curtype(&self) -> TokenType {
        self.tokenvec[self.pos].tokentype
    }
    pub fn curnum(&self) -> i32 {
        self.tokenvec[self.pos].num
    }
    pub fn eof(&self) {
        assert_eq!(self.pos+1, self.tokenvec.len());
    }
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

pub fn tokenize(program: String) -> Vec<Token> {
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