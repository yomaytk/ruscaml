use super::*;
use super::parser::*;

use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static FRESH_COUNT: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(1));
pub static TARGET_EXP: Lazy<Mutex<Exp>> = Lazy::new(|| Mutex::new(Exp::Compexp(Box::new(Cexp::Val(Value::Intv(1))))));

pub fn get_fresh_var() -> String {
    let num = *(FRESH_COUNT).lock().unwrap();
    *(FRESH_COUNT).lock().unwrap() = num + 1;
    let mut s = String::from("@v");
    let c = std::char::from_digit(num as u32, 10).unwrap();
    s.push(c);
    s
}

#[derive(Clone, Debug)]
pub enum Value {
    Var(Id),
    Intv(i32)
}

#[derive(Clone, Debug)]
pub enum Cexp {
    Val(Value),
    Binop(Bintype, Value, Value),
    App(Value, Value),
    If(Value, Box<Exp>, Box<Exp>),
    Tuple(Value, Value),
    Proj(Value, i32)
}

#[derive(Clone, Debug)]
pub enum Exp {
    Compexp(Box<Cexp>),
    Let(Id, Box<Cexp>, Box<Exp>),
    Loop(Id, Box<Cexp>, Box<Exp>),
    Letrec(Id, Id, Box<Exp>, Box<Exp>),
    Recur(Value)
}

#[derive(Clone, Debug)]
pub enum Bintype {
    Plus,
    Mult,
    Lt
}

fn ttype2btype(ttype: TokenType) -> Bintype {
    match ttype {
        TokenType::Plus => { Bintype::Plus }
        TokenType::Mult => { Bintype::Mult }
        TokenType::Lt => { Bintype::Lt }
        _ => { panic!("ttyep2btype error.") }
    }
}

fn norm_exp(ast: Ast, f: impl FnOnce(Cexp) -> Exp) -> Exp {
    match ast {
        Ast::ILit(v) => {
            f(Cexp::Val(Value::Intv(v)))
        }
        Ast::BLit(v) => {
            f(Cexp::Val(Value::Intv(if v {1} else {0})))
        }
        Ast::Binop(ttype, ast1, ast2) => {
            match (*ast1, *ast2) {
                (Ast::Var(id1), Ast::Var(id2)) => {
                    f(Cexp::Binop(ttype2btype(ttype), Value::Var(id1), Value::Var(id2)))
                }
                (_ast1, _ast2) => {
                    let nv1 = get_fresh_var();
                    let nv2 = get_fresh_var();
                    norm_exp(Ast::Let(nv1.clone(), Box::new(_ast1), Box::new(Ast::Let(nv2.clone(), Box::new(_ast2), Box::new(Ast::Binop(ttype, Box::new(Ast::Var(nv1)), Box::new(Ast::Var(nv2))))))), f)
                }
            }
        }
        Ast::If(ast1, ast2, ast3) => {
            let nv = get_fresh_var();
            *(TARGET_EXP).lock().unwrap() = Exp::Let(nv.clone(), Box::new(ce), Box::new(f(Cexp::If(Value::Var(nv), Box::new(norm_exp(*ast2, ef)), Box::new(norm_exp(*ast3, ef))))));
            norm_exp(*ast1, ef)
        }
        Ast::Fun(id, ast1) => {
            let nv = get_fresh_var();
            norm_exp(Ast::Rec(nv.clone(), id, Box::new(*ast1), Box::new(Ast::Var(nv))), f)
        }
        Ast::Var(id) => {
            f(Cexp::Val(Value::Var(id)))
        }
        Ast::Let(id, ast1, ast2) => {
            match *ast1 {
                Ast::ILit(v) => { Exp::Let(id, Box::new(Cexp::Val(Value::Intv(v))), Box::new(norm_exp(*ast2, f))) }
                Ast::BLit(v) => { Exp::Let(id, Box::new(Cexp::Val(Value::Intv(if v {1} else {0}))), Box::new(norm_exp(*ast2, f))) }
                Ast::Var(v) => { Exp::Let(id, Box::new(Cexp::Val(Value::Var(v))), Box::new(norm_exp(*ast2, f))) }
                _ => {
                    let nv = get_fresh_var();
                    norm_exp(*ast1, |ce| { Exp::Let(nv.clone(), Box::new(ce), Box::new(norm_exp(Ast::Let(id, Box::new(Ast::Var(nv)), ast2), f))) })
                }
            }
        }
        Ast::Rec(id1, id2, ast1, ast2) => {
            Exp::Letrec(id1, id2, Box::new(norm_exp(*ast1, ef)), Box::new(norm_exp(*ast2, f)))
        }
        Ast::Loop(id, ast1, ast2) => {
            match *ast1 {
                Ast::ILit(v) => { Exp::Loop(id, Box::new(Cexp::Val(Value::Intv(v))), Box::new(norm_exp(*ast2, f))) }
                Ast::BLit(v) => { Exp::Loop(id, Box::new(Cexp::Val(Value::Intv(if v {1} else {0}))), Box::new(norm_exp(*ast2, f))) }
                Ast::Var(v) => { Exp::Loop(id, Box::new(Cexp::Val(Value::Var(v))), Box::new(norm_exp(*ast2, f))) }
                _ => {
                    let nv = get_fresh_var();
                    norm_exp(*ast1, |ce| { Exp::Loop(nv.clone(), Box::new(ce), Box::new(norm_exp(Ast::Loop(id, Box::new(Ast::Var(nv)), ast2), f))) })
                }
            }
        }
        Ast::Recur(ast1) => {
            match *ast1 {
                Ast::ILit(v) => { Exp::Recur(Value::Intv(v)) }
                Ast::BLit(v) => { Exp::Recur(Value::Intv(if v {1} else {0})) }
                Ast::Var(v) => { Exp::Recur(Value::Var(v)) }
                _ => {
                    let nv = get_fresh_var();
                    norm_exp(*ast1, |ce| { Exp::Let(nv.clone(), Box::new(ce), Box::new(Exp::Recur(Value::Var(nv))))} )
                }
            }
        }
        Ast::App(ast1, ast2) => {
            let nv1 = get_fresh_var();
            let nv2 = get_fresh_var();
            norm_exp(*ast1, |ce1| { Exp::Let(nv1.clone(), Box::new(ce1), Box::new(norm_exp(*ast2, |ce2| { Exp::Let(nv2.clone(), Box::new(ce2), Box::new(f(Cexp::App(Value::Var(nv1), Value::Var(nv2))))) }))) })
        }
        Ast::Tuple(ast1, ast2) => {
            let nv1 = get_fresh_var();
            let nv2 = get_fresh_var();
            norm_exp(*ast1, |ce1| { Exp::Let(nv1.clone(), Box::new(ce1), Box::new(norm_exp(*ast2, |ce2| { Exp::Let(nv2.clone(), Box::new(ce2), Box::new(f(Cexp::Tuple(Value::Var(nv1), Value::Var(nv2))))) }))) })
        }
        Ast::Proj(ast1, v) => {
            let nv = get_fresh_var();
            norm_exp(*ast1, |ce| { Exp::Let(nv.clone(), Box::new(ce), Box::new(f(Cexp::Proj(Value::Var(nv), v)))) })
        }
        Ast::Nonaexpr => {
            panic!("There shouldn't be Nonaexpr in Ast.");
        }
    }
}

fn ef(cexp: Cexp) -> Exp {
    Exp::Compexp(Box::new(cexp))
}

// fn if_f(ce: Cexp, nv: String, ast2: Ast, ast3: Ast, f: impl FnOnce(Cexp) -> Exp) -> impl FnOnce(Cexp) -> Exp {
//     Exp::Let(nv.clone(), Box::new(ce), Box::new(f(Cexp::If(Value::Var(nv), Box::new(norm_exp(ast2, ef)), Box::new(norm_exp(ast3, ef))))))
// }

pub fn normalize(ast: Ast) -> Exp{
    norm_exp(ast, ef)
}