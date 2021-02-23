use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    Semisemi,
    ILit,
    Plus,
    Mult,
    Lt,
    Arrow,
    Assign,
    Lbrac,
    Rbrac,
    Comma,
    Dot,
    Id,
    If,
    Then,
    Else,
    Fun,
    Let,
    In,
    Rec,
    Loop,
    Recur,
    True,
    False,
}

impl From<&str> for TokenType {
    fn from(s: &str) -> Self {
        match s {
            "if" => TokenType::If,
            "then" => TokenType::Then,
            "else" => TokenType::Else,
            "fun" => TokenType::Fun,
            "let" => TokenType::Let,
            "in" => TokenType::In,
            "rec" => TokenType::Rec,
            "loop" => TokenType::Loop,
            "recur" => TokenType::Recur,
            "true" => TokenType::True,
            "false" => TokenType::False,
            _ => TokenType::Id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tokentype: TokenType,
    pub num: i32,
    pub id: Option<String>,
    pub position: (bool, usize, usize),
}

impl Token {
    pub fn new(tokentype: TokenType, num: i32, id: Option<String>, position: (bool, usize, usize)) -> Self {
        Self {
            tokentype,
            num,
            id,
            position,
        }
    }
}


pub struct TokenSet {
    pub tokens: Vec<Token>,
    pub pos: usize,
}

impl TokenSet {
    pub fn assert_ttype(&mut self, ttype: TokenType) {
        if !self.consume_ttype(ttype) {
            compile_error(&self, "lexer tokenset error");
        }
    }
    pub fn consume_ttype(&mut self, ttype: TokenType) -> bool {
        if ttype == self.tokens[self.pos].tokentype {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    pub fn curtype(&self) -> TokenType {
        self.tokens[self.pos].tokentype
    }
    pub fn curnum(&self) -> i32 {
        self.tokens[self.pos].num
    }
    pub fn curid(&self) -> Option<Id> {
        if let TokenType::Id = self.curtype() {
            self.tokens[self.pos].id.clone()
        } else {
            compile_error(self, "should be identifier.");
            std::process::exit(1);
        }
    }
    pub fn eof(&self) -> bool {
        self.pos+1 == self.tokens.len()
    }
}

fn signal(pos: &mut usize, line: usize, head: bool) -> Option<Token> {
    if &(*PROGRAM)[*pos..*pos+2] == ";;" {
        *pos += 2;
        Some(Token::new(TokenType::Semisemi, -1, None, (head, line, *pos-2)))
    } else if &(*PROGRAM)[*pos..*pos+2] == "->" {
        *pos += 2;
        Some(Token::new(TokenType::Arrow, -1, None, (head, line, *pos-2)))
    } else if &(*PROGRAM)[*pos..*pos+1] == "+" {
        *pos += 1;
        Some(Token::new(TokenType::Plus, -1, None, (head, line, *pos-1)))
    } else if &(*PROGRAM)[*pos..*pos+1] == "*" {
        *pos += 1;
        Some(Token::new(TokenType::Mult, -1, None, (head, line, *pos-1)))
    } else if &(*PROGRAM)[*pos..*pos+1] == "<" {
        *pos += 1;
        Some(Token::new(TokenType::Lt, -1, None, (head, line, *pos-1)))
    } else if  &(*PROGRAM)[*pos..*pos+1] == "=" {
        *pos += 1;
        Some(Token::new(TokenType::Assign, -1 , None, (head, line, *pos-1)))
    } else if &(*PROGRAM)[*pos..*pos+1] == "(" {
        *pos += 1;
        Some(Token::new(TokenType::Lbrac, -1, None, (head, line, *pos-1)))
    } else if &(*PROGRAM)[*pos..*pos+1] == ")" {
        *pos += 1;
        Some(Token::new(TokenType::Rbrac, -1 , None, (head, line, *pos-1)))
    }
    else if &(*PROGRAM)[*pos..*pos+1] == "," {
        *pos += 1;
        Some(Token::new(TokenType::Comma, -1 , None, (head, line, *pos-1)))
    } else if &(*PROGRAM)[*pos..*pos+1] == "." {
        *pos += 1;
        Some(Token::new(TokenType::Dot, -1 , None, (head, line, *pos-1)))
    } else {
        None
    }
}

fn identify(s: &Vec<char>, pos: &mut usize, line: usize, head: bool) -> Option<Token> {
    let start = *pos;
    // first character should be alphabet
    if s[*pos].is_ascii_alphabetic() {
        *pos += 1;
    } else {
        return None;
    }
    while s[*pos].is_ascii_alphabetic() || s[*pos].is_ascii_digit() || s[*pos] == '_' || s[*pos] == '\''{ *pos += 1; }
    if start < *pos {
        Some(Token::new(TokenType::from(&(*PROGRAM)[start..*pos]), -1, Some(String::from(&(*PROGRAM)[start..*pos])), (head, line, start)))
    } else {
        None
    }
}

fn number(s :&Vec<char>, pos: &mut usize, line: usize, head: bool) -> Option<Token> {
    let start = *pos;
    let mut num = 0;
    while s[*pos].is_ascii_digit() { 
        num = num * 10 + (s[*pos] as i32 - 48);
        *pos += 1; 
    }
    if start < *pos {
        Some(Token::new(TokenType::ILit, num, None, (head, line , start)))
    } else {
        None
    }
}

pub fn tokenize() -> TokenSet {
    
    let mut tokens = vec![];
    let mut pos: usize = 0;
    let mut line: usize = 1;
    let mut head = true;
    let pgstr = (*PROGRAM).chars().collect::<Vec<char>>();

    while pos < pgstr.len()-1 {
        
        // identifier
        if let Some(token) = identify(&pgstr, &mut pos, line, head) {
            tokens.push(token);
            head = false;
            continue;
        }

        let nchar = pgstr[pos];
        
        // newline
        if nchar == '\n' {
            line += 1;
            pos += 1;
            head = true;
            continue;
        }

        // space
        if nchar == ' ' {
            pos += 1;
            continue;
        }
        
        // ILit
        if let Some(token) = number(&pgstr, &mut pos, line, head) {
            tokens.push(token);
            head = false;
            continue;
        }

        // signal
        if let Some(token) = signal(&mut pos, line, head) {
            tokens.push(token);
            head = false;
            continue;
        }

        panic!("tokenize error.");
    }
    TokenSet {
        tokens: tokens,
        pos: 0
    }
}