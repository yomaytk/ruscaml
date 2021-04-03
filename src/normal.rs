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

pub fn duplicate_var(nv: String) -> String {
    let mut s = String::from("@v");
    s.push_str(&nv[2..]);
    s
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Value {
    Var(Id),
    Intv(i32)
}

impl Value {
    fn ast2value(ast: parser::Ast) -> (Option<Value>, Ast) {
        match ast {
            Ast::ILit(v) => { (Some(Value::Intv(v)), ast) }
            Ast::BLit(v) => { (Some(Value::Intv(if v {1} else {0})), ast) }
            Ast::Var(v) => { (Some(Value::Var(v.clone())), Ast::Var(v)) }
            _ => { (None, ast) }
        }
    }
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

impl Cexp {
    pub fn program_display(self) {
        use Cexp::*;
        match self {
            Val(Value::Var(id)) => { print!("{}", id); }
            Val(Value::Intv(v)) => { print!("{}", v); }
            Binop(tty, val1, val2) => {
                Val(val1).program_display();
                print!("{}", tty.bintype_signal());
                Val(val2).program_display();
            }
            App(val1, val2) => {
                Val(val1).program_display();
                print!(" ");
                Val(val2).program_display();
            }
            If(val, exp1, exp2) => {
                print!("if ");
                Val(val).program_display();
                print!(" then ");
                exp1.program_display();
                print!(" else ");
                exp2.program_display();
            }
            Tuple(val1, val2) => {
                print!("(");
                Val(val1).program_display();
                print!(", ");
                Val(val2).program_display();
                print!(")");
            }
            Proj(val, i) => {
                Val(val).program_display();
                print!(".{}", i);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Exp {
    Compexp(Box<Cexp>),
    Let(Id, Box<Cexp>, Box<Exp>),
    Loop(Id, Box<Cexp>, Box<Exp>),
    Letrec(Id, Id, Box<Exp>, Box<Exp>),
    Recur(Value)
}

impl Exp {
    pub fn program_display(self) {
        use Exp::*;
        match self {
            Compexp(cexp) => {
                cexp.program_display();
            }
            Let(id, cexp, exp) => {
                print!("let {} = ", id);
                cexp.program_display();
                print!(" in\n");
                exp.program_display();
            }
            Loop(id, cexp, exp) => {
                print!("loop {} = ", id);
                cexp.program_display();
                print!(" in\n");
                exp.program_display();
            }
            Letrec(id1, id2, exp1, exp2) => {
                print!("let rec {} = fun ", id1);
                print!("{} -> ", id2);
                match *exp1 {
                    Let(..) | Letrec(..) | Loop(..) => { 
                        print!("\n");
                    }
                    _ => {}
                }
                exp1.program_display(); 
                print!(" in\n");
                exp2.program_display();
            }
            Recur(val) => {
                print!("recur ");
                Cexp::Val(val).program_display();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Bintype {
    Plus,
    Mult,
    Lt
}

impl Bintype {
    pub fn bintype_signal(self) -> char {
        use Bintype::*;
        match self {
            Plus => { '+' }
            Mult => { '*' }
            Lt => { '<' }
        }
    }
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
            let (val1, _ast1) = Value::ast2value(*ast1);
            let (val2, _ast2) = Value::ast2value(*ast2);
            match (val1, val2) {
                (Some(v1), Some(v2)) => {
                    fs[fid].apply()(Cexp::Binop(ttype2btype(ttype), v1, v2))
                }
                (None, Some(_)) => {
                    let nv1 = get_fresh_var();
                    norm_exp(Ast::Let(nv1.clone(), Box::new(_ast1), Box::new(Ast::Binop(ttype, Box::new(Ast::Var(nv1)), Box::new(_ast2)))), fid, fs)
                }
                (Some(_), None) => {
                    let nv2 = get_fresh_var();
                    norm_exp(Ast::Let(nv2.clone(), Box::new(_ast2), Box::new(Ast::Binop(ttype, Box::new(_ast1), Box::new(Ast::Var(nv2))))), fid ,fs)
                }
                (None, None) => {
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
            match Value::ast2value(*ast1) {
                (Some(val1), _) => {
                    Exp::Let(id, Box::new(Cexp::Val(val1)), Box::new(norm_exp(*ast2, fid, fs)))
                }
                (None, _ast1) => {
                    let nast2 = norm_exp(*ast2, fid, fs);
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(id, Box::new(ce), Box::new(nast2)) })));
                    norm_exp(_ast1, fs.len()-1, fs)
                }
            }
        }
        Ast::Rec(id1, id2, ast1, ast2) => {
            Exp::Letrec(id1, id2, Box::new(norm_exp(*ast1, 0, fs)), Box::new(norm_exp(*ast2, fid, fs)))
        }
        Ast::Loop(id, ast1, ast2) => {
            match Value::ast2value(*ast1) {
                (Some(val1), _) => {
                    Exp::Loop(id, Box::new(Cexp::Val(val1)), Box::new(norm_exp(*ast2, fid, fs)))
                }
                (None, _ast1) => {
                    let nast2 = norm_exp(*ast2, fid, fs);
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Loop(id, Box::new(ce), Box::new(nast2)) })));
                    norm_exp(_ast1, fs.len()-1, fs)
                }
            }
        }
        Ast::Recur(ast1) => {
            match Value::ast2value(*ast1) {
                (Some(val1), _) => {
                    Exp::Recur(val1)
                }
                (None, _ast1) => {
                    let nv = get_fresh_var();
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(nv.clone(), Box::new(ce), Box::new(Exp::Recur(Value::Var(nv))))})));
                    norm_exp(_ast1, fs.len()-1, fs)
                }
            }
        }
        Ast::App(ast1, ast2) => {
            let (val1, _ast1) = Value::ast2value(*ast1);
            let (val2, _ast2) = Value::ast2value(*ast2);
            match (val1, val2) {
                (Some(v1), Some(v2)) => {
                    fs[fid].apply()(Cexp::App(v1, v2))
                }
                (Some(v1), _) => {
                    let nv2 = get_fresh_var();
                    let ass_ins = fs[fid].apply()(Cexp::App(v1, Value::Var(nv2.clone())));
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(nv2, Box::new(ce), Box::new(ass_ins))})));
                    norm_exp(_ast2, fs.len()-1, fs)
                }
                (_, Some(v2)) => {
                    let nv1 = get_fresh_var();
                    let ass_ins = fs[fid].apply()(Cexp::App(Value::Var(nv1.clone()), v2));
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(nv1, Box::new(ce), Box::new(ass_ins))})));
                    norm_exp(_ast1, fs.len()-1, fs)
                }
                (None, None) => {
                    let nv1 = get_fresh_var();
                    let nv2 = get_fresh_var();
                    let ass_ins = fs[fid].apply()(Cexp::App(Value::Var(nv1.clone()), Value::Var(nv2.clone())));
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(duplicate_var(nv2), Box::new(ce), Box::new(ass_ins))})));
                    let nast2 = Box::new(norm_exp(_ast2, fs.len()-1, fs));
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(duplicate_var(nv1), Box::new(ce), nast2)})));
                    norm_exp(_ast1, fs.len()-1, fs)
                }
            }
        }
        Ast::Tuple(ast1, ast2) => {
            let (val1, _ast1) = Value::ast2value(*ast1);
            let (val2, _ast2) = Value::ast2value(*ast2);
            match (val1, val2) {
                (Some(v1), Some(v2)) => {
                    fs[fid].apply()(Cexp::Tuple(v1, v2))
                }
                (Some(v1), _) => {
                    let nv2 = get_fresh_var();
                    let ass_ins = fs[fid].apply()(Cexp::Tuple(v1, Value::Var(nv2.clone())));
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(nv2, Box::new(ce), Box::new(ass_ins))})));
                    norm_exp(_ast2, fs.len()-1, fs)
                }
                (_, Some(v2)) => {
                    let nv1 = get_fresh_var();
                    let ass_ins = fs[fid].apply()(Cexp::Tuple(Value::Var(nv1.clone()), v2));
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(nv1, Box::new(ce), Box::new(ass_ins))})));
                    norm_exp(_ast1, fs.len()-1, fs)
                }
                (None, None) => {
                    let nv1 = get_fresh_var();
                    let nv2 = get_fresh_var();
                    let ass_ins = fs[fid].apply()(Cexp::Tuple(Value::Var(nv1.clone()), Value::Var(nv2.clone())));
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(duplicate_var(nv2), Box::new(ce), Box::new(ass_ins))})));
                    let nast2 = Box::new(norm_exp(_ast2, fs.len()-1, fs));
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(duplicate_var(nv1), Box::new(ce), nast2)})));
                    norm_exp(_ast1, fs.len()-1, fs)
                }
            }
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