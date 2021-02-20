use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    Semisemi,
    ILit,
    Plus,
    Mult,
    Lt,
    Var,
    If,
    Then,
    Else,
}

impl From<&str> for TokenType {
    fn from(s: &str) -> Self {
        match s {
            "if" => TokenType::If,
            "then" => TokenType::Then,
            "else" => TokenType::Else,
            _ => TokenType::Var
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Token {
    pub tokentype: TokenType,
    pub num: i32,
}

impl Token {
    pub fn new(tokentype: TokenType, num: i32) -> Self {
        Self {
            tokentype,
            num,
        }
    }
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

fn signal(pos: &mut usize) -> Option<Token> {
    if &(*PROGRAM)[*pos..*pos+2] == ";;" {
        *pos += 2;
        Some(Token::new(TokenType::Semisemi, -1))
    } else if &(*PROGRAM)[*pos..*pos+1] == "+" {
        *pos += 1;
        Some(Token::new(TokenType::Plus, -1))
    } else if &(*PROGRAM)[*pos..*pos+1] == "*" {
        *pos += 1;
        Some(Token::new(TokenType::Mult, -1))
    } else if &(*PROGRAM)[*pos..*pos+1] == "<" {
        *pos += 1;
        Some(Token::new(TokenType::Lt, -1))
    } else {
        None
    }
}

fn identify(s: &Vec<char>, pos: &mut usize) -> Option<Token> {
    let start = *pos;
    // first character should be alphabet
    if s[*pos].is_ascii_alphabetic() {
        *pos += 1;
    } else {
        return None;
    }
    while s[*pos].is_ascii_alphabetic() || s[*pos].is_ascii_digit() { *pos += 1; }
    if start < *pos {
        Some(Token::new(TokenType::from(&(*PROGRAM)[start..*pos]), -1))
    } else {
        None
    }
}

fn number(s :&Vec<char>, pos: &mut usize) -> Option<Token> {
    let start = *pos;
    let mut num = 0;
    while s[*pos].is_ascii_digit() { 
        num = num * 10 + (s[*pos] as i32 - 48);
        *pos += 1; 
    }
    if start < *pos {
        Some(Token::new(TokenType::ILit, num))
    } else {
        None
    }
}

pub fn tokenize() -> Vec<Token> {
    
    let mut tokens = vec![];
    let mut pos: usize = 0;
    let pgstr = (*PROGRAM).chars().collect::<Vec<char>>();

    while pos < pgstr.len()-1 {
        
        // identifier
        if let Some(token) = identify(&pgstr, &mut pos) {
            tokens.push(token);
            continue;
        }

        let nchar = pgstr[pos];
        
        // space
        if nchar.is_whitespace() {
            pos += 1;
            continue;
        }
        
        // ILit
        if let Some(token) = number(&pgstr, &mut pos) {
            tokens.push(token);
            continue;
        }

        // signal
        if let Some(token) = signal(&mut pos) {
            tokens.push(token);
            continue;
        }

        panic!("tokenize error.");
    }
    return tokens;
}