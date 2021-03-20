use super::*;
use super::parser::*;

use once_cell::sync::Lazy;
use std::sync::Mutex;
// use std::fmt;

pub static FRESH_COUNT: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));

fn get_fresh_var() -> String {
    let num = *(FRESH_COUNT).lock().unwrap();
    *(FRESH_COUNT).lock().unwrap() = num + 1;
    let mut s = String::from("@v");
    let numstr = &num.to_string();
    s.push_str(numstr);
    s
}

fn duplicate_var(nv: String) -> String {
    let mut s = String::from("@v");
    s.push_str(&nv[2..]);
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

fn norm_exp(ast: Ast, fid: usize, fs: &mut Vec<AsgFun>) -> Exp {
    match ast {
        Ast::ILit(v) => {
            fs[fid].apply()(Cexp::Val(Value::Intv(v)))
        }
        Ast::BLit(v) => {
            fs[fid].apply()(Cexp::Val(Value::Intv(if v {1} else {0})))
        }
        Ast::Binop(ttype, ast1, ast2) => {
            match (*ast1, *ast2) {
                (Ast::Var(id1), Ast::Var(id2)) => {
                    fs[fid].apply()(Cexp::Binop(ttype2btype(ttype), Value::Var(id1), Value::Var(id2)))
                }
                (_ast1, _ast2) => {
                    let nv1 = get_fresh_var();
                    let nv2 = get_fresh_var();
                    norm_exp(Ast::Let(nv1.clone(), Box::new(_ast1), Box::new(Ast::Let(nv2.clone(), Box::new(_ast2), Box::new(Ast::Binop(ttype, Box::new(Ast::Var(nv1)), Box::new(Ast::Var(nv2))))))), fid, fs)
                }
            }
        }
        Ast::If(ast1, ast2, ast3) => {
            let nv = get_fresh_var();
            let ass_ins = fs[fid].apply()(Cexp::If(Value::Var(nv.clone()), Box::new(norm_exp(*ast2, 0, fs)), Box::new(norm_exp(*ast3, 0, fs))));
            fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(duplicate_var(nv), Box::new(ce), Box::new(ass_ins))})));
            norm_exp(*ast1, fs.len()-1, fs)
        }
        Ast::Fun(id, ast1) => {
            let nv = get_fresh_var();
            norm_exp(Ast::Rec(nv.clone(), id, Box::new(*ast1), Box::new(Ast::Var(nv))), fid, fs)
        }
        Ast::Var(id) => {
            fs[fid].apply()(Cexp::Val(Value::Var(id)))
        }
        Ast::Let(id, ast1, ast2) => {
            match *ast1 {
                Ast::ILit(v) => { Exp::Let(id, Box::new(Cexp::Val(Value::Intv(v))), Box::new(norm_exp(*ast2, fid, fs))) }
                Ast::BLit(v) => { Exp::Let(id, Box::new(Cexp::Val(Value::Intv(if v {1} else {0}))), Box::new(norm_exp(*ast2, fid, fs))) }
                Ast::Var(v) => { Exp::Let(id, Box::new(Cexp::Val(Value::Var(v))), Box::new(norm_exp(*ast2, fid, fs))) }
                _ => {
                    let nv = get_fresh_var();
                    let nast2 = Box::new(norm_exp(Ast::Let(id, Box::new(Ast::Var(nv.clone())), ast2), fid, fs));
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(duplicate_var(nv), Box::new(ce), nast2)})));
                    norm_exp(*ast1, fs.len()-1, fs)
                }
            }
        }
        Ast::Rec(id1, id2, ast1, ast2) => {
            Exp::Letrec(id1, id2, Box::new(norm_exp(*ast1, 0, fs)), Box::new(norm_exp(*ast2, fid, fs)))
        }
        Ast::Loop(id, ast1, ast2) => {
            match *ast1 {
                Ast::ILit(v) => { Exp::Loop(id, Box::new(Cexp::Val(Value::Intv(v))), Box::new(norm_exp(*ast2, fid, fs))) }
                Ast::BLit(v) => { Exp::Loop(id, Box::new(Cexp::Val(Value::Intv(if v {1} else {0}))), Box::new(norm_exp(*ast2, fid, fs))) }
                Ast::Var(v) => { Exp::Loop(id, Box::new(Cexp::Val(Value::Var(v))), Box::new(norm_exp(*ast2, fid, fs))) }
                _ => {
                    let nv = get_fresh_var();
                    let nast2 = norm_exp(Ast::Loop(id, Box::new(Ast::Var(nv.clone())), ast2), fid, fs);
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Loop(duplicate_var(nv), Box::new(ce), Box::new(nast2)) })));
                    norm_exp(*ast1, fs.len()-1, fs)
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
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(nv.clone(), Box::new(ce), Box::new(Exp::Recur(Value::Var(nv))))})));
                    norm_exp(*ast1, fs.len()-1, fs)
                }
            }
        }
        Ast::App(ast1, ast2) => {
            let nv1 = get_fresh_var();
            let nv2 = get_fresh_var();
            let ass_ins = fs[fid].apply()(Cexp::App(Value::Var(nv1.clone()), Value::Var(nv2.clone())));
            fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(duplicate_var(nv2), Box::new(ce), Box::new(ass_ins))})));
            let nast2 = Box::new(norm_exp(*ast2, fs.len()-1, fs));
            fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(duplicate_var(nv1), Box::new(ce), nast2)})));
            norm_exp(*ast1, fs.len()-1, fs)
        }
        Ast::Tuple(ast1, ast2) => {
            let nv1 = get_fresh_var();
            let nv2 = get_fresh_var();
            let ass_ins = fs[fid].apply()(Cexp::Tuple(Value::Var(nv1.clone()), Value::Var(nv2.clone())));
            fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(duplicate_var(nv2), Box::new(ce), Box::new(ass_ins))})));
            let nast2 = Box::new(norm_exp(*ast2, fs.len()-1, fs));
            fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(duplicate_var(nv1), Box::new(ce), nast2)})));
            norm_exp(*ast1, fs.len()-1, fs)
        }
        Ast::Proj(ast1, v) => {
            let nv = get_fresh_var();
            let ass_ins = fs[fid].apply()(Cexp::Proj(Value::Var(nv.clone()), v));
            fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(duplicate_var(nv), Box::new(ce), Box::new(ass_ins))})));
            norm_exp(*ast1, fs.len()-1, fs)
        }
        Ast::Nonaexpr => {
            panic!("There shouldn't be Nonaexpr in Ast.");
        }
    }
}

fn ef(cexp: Cexp) -> Exp {
    Exp::Compexp(Box::new(cexp))
}

struct AsgFun(Box<dyn FnOnce(Cexp) -> Exp>);

impl AsgFun {
    fn new(f: Box<dyn FnOnce(Cexp) -> Exp>) -> Self {
        Self(f)
    }
    fn apply(&mut self) -> Box<dyn FnOnce(Cexp) -> Exp> {
        std::mem::replace(&mut self.0, Box::new(ef))
    }
}

pub fn normalize(ast: Ast) -> Exp {
    let mut fs = vec![(AsgFun::new(Box::new(ef)))];
    norm_exp(ast, 0, &mut fs)
}