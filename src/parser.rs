use super::*;
use super::lexer::*;

#[derive(Clone, Debug)]
pub enum Ast {
    Nonaexpr,
    ILit(i32),
    BLit(bool),
    Binop(TokenType, Box<Ast>, Box<Ast>),
    If(Box<Ast>, Box<Ast>, Box<Ast>),
    Fun(Id, Box<Ast>),
    Var(Id),
    Let(Id, Box<Ast>, Box<Ast>),
    Rec(Id, Box<Ast>, Box<Ast>),
    Loop(Id, Box<Ast>, Box<Ast>),
    Recur(Box<Ast>),
    App(Box<Ast>, Box<Ast>),
    Tuple(Box<Ast>, Box<Ast>),
    Proj(Box<Ast>, Box<Ast>),
}


fn identify(tokenset: &mut TokenSet) -> Id {
    if let Some(id) = tokenset.curid() {
        tokenset.pos += 1;
        Id::from(id)
    } else {
        panic!("We will never get to this process.")
    }
}

fn proj(tokenset: &mut TokenSet, mut ast: Ast) -> Ast {
    while tokenset.consume_ttype(TokenType::Dot) {
        ast = Ast::Proj(Box::new(ast), Box::new(aexpr(tokenset)));
    }
    ast
}

fn aexpr(tokenset: &mut TokenSet) -> Ast {
    match tokenset.curtype() {
        TokenType::ILit => {
            let num = tokenset.curnum();
            tokenset.pos += 1;
            Ast::ILit(num)
        }
        TokenType::Id => {
            let var = tokenset.curid().unwrap();
            tokenset.pos += 1;
            proj(tokenset, Ast::Var(var))
        }
        TokenType::True => {
            tokenset.pos += 1;
            Ast::BLit(true)
        }
        TokenType::False => {
            tokenset.pos += 1;
            Ast::BLit(false)
        }
        TokenType::Lbrac => {
            tokenset.pos += 1;
            let mut ast = expr(tokenset);
            if tokenset.consume_ttype(TokenType::Comma) {
                ast = Ast::Tuple(Box::new(ast), Box::new(expr(tokenset)));
            }
            tokenset.assert_ttype(TokenType::Rbrac);
            proj(tokenset, ast)
        }
        _ => {
            Ast::Nonaexpr
        }
    }
}

fn appexpr(tokenset: &mut TokenSet) -> Ast {
    if tokenset.consume_ttype(TokenType::Recur) {
        return Ast::Recur(Box::new(aexpr(tokenset)));
    }
    let mut ast = aexpr(tokenset);
    loop {
        let ast1 = aexpr(tokenset);
        if let Ast::Nonaexpr = ast1 {
            break;
        }
        ast = Ast::App(Box::new(ast), Box::new(ast1));
    }
    ast
}

fn mexpr(tokenset: &mut TokenSet) -> Ast {
    let mut ast = appexpr(tokenset);
    while tokenset.consume_ttype(TokenType::Mult) {
        ast = Ast::Binop(TokenType::Mult, Box::new(ast), Box::new(aexpr(tokenset)));
    }
    ast
}

fn pexpr(tokenset: &mut TokenSet) -> Ast {
    let mut ast = mexpr(tokenset);
    while tokenset.consume_ttype(TokenType::Plus) {
        ast = Ast::Binop(TokenType::Plus, Box::new(ast), Box::new(pexpr(tokenset)));
    }
    ast
}

fn ltexpr(tokenset: &mut TokenSet) -> Ast {
    let last = pexpr(tokenset);
    if tokenset.consume_ttype(TokenType::Lt) {
        let rast = pexpr(tokenset);
        return Ast::Binop(TokenType::Lt, Box::new(last), Box::new(rast));
    }
    last
}

fn expr(tokenset: &mut TokenSet) -> Ast {
    let ast;
    match tokenset.curtype() {
        TokenType::If => {
            tokenset.pos += 1;
            let cond = expr(tokenset);
            tokenset.assert_ttype(TokenType::Then);
            let then = expr(tokenset);
            tokenset.assert_ttype(TokenType::Else);
            let els = expr(tokenset);
            ast = Ast::If(Box::new(cond), Box::new(then), Box::new(els));
        }
        TokenType::Fun => {
            tokenset.pos += 1;
            let id = identify(tokenset);
            tokenset.assert_ttype(TokenType::Arrow);
            let body = expr(tokenset);
            ast = Ast::Fun(id, Box::new(body));
        }
        TokenType::Let => {
            tokenset.pos += 1;
            match tokenset.curtype() {
                TokenType::Rec => {
                    tokenset.pos += 1;
                    let id = identify(tokenset);
                    tokenset.assert_ttype(TokenType::Assign);
                    let funast = expr(tokenset);
                    if let Ast::Fun(_, _) = funast {
                        tokenset.assert_ttype(TokenType::In);
                        ast = Ast::Rec(id, Box::new(funast), Box::new(expr(tokenset)));
                    } else {
                        panic!("should type fun. {:?}", funast);
                    }
                }
                _ => {
                    let id = identify(tokenset);
                    tokenset.assert_ttype(TokenType::Assign);
                    let ast1 = expr(tokenset);
                    tokenset.assert_ttype(TokenType::In);
                    let ast2 = expr(tokenset);
                    ast = Ast::Let(id, Box::new(ast1), Box::new(ast2))
                }
            }
        }
        TokenType::Loop => {
            tokenset.pos += 1;
            let id = identify(tokenset);
            tokenset.assert_ttype(TokenType::Assign);
            let ast1 = expr(tokenset);
            tokenset.assert_ttype(TokenType::In);
            let ast2 = expr(tokenset);
            ast = Ast::Loop(id, Box::new(ast1), Box::new(ast2))
        }
        _ => {
            ast = ltexpr(tokenset);
        }
    }
    ast
}

pub fn toplevel(mut tokenset: TokenSet) -> Ast {
    let ast = expr(&mut tokenset);
    tokenset.assert_ttype(TokenType::Semisemi);
    ast
}