use super::lexer::*;

#[derive(Clone, Debug)]
pub enum Ast {
    IntV(i32),
    Binop(TokenType, Box<Ast>, Box<Ast>),
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

pub fn toplevel(tokens: Vec<Token>) -> Ast {
    let mut tokenvec = TokenVec {
        tokenvec: tokens,
        pos: 0
    };
    let ast = expr(&mut tokenvec);
    tokenvec.assert_ttype(TokenType::Semisemi);
    ast
}