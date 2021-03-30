use once_cell::sync::Lazy;
use std::sync::Mutex;

use super::*;
use super::normal::{Value, Bintype};

pub static FRESH_COUNT: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));

fn get_fresh_function_var(s: &str, fresh_char: char) -> String {
    let num = *FRESH_COUNT.lock().unwrap();
    *FRESH_COUNT.lock().unwrap() = num + 1;
    let mut ss = String::from("$");
    ss.push(fresh_char);
    ss.push('_');
    let numstr = &num.to_string();
    ss.push_str(s);
    ss.push_str(numstr);
    ss
}

#[derive(Clone, Debug)]
pub enum Cexp {
    Val(Value),
    Binop(Bintype, Value, Value),
    App(Value, Vec<Value>),
    If(Value, Box<Exp>, Box<Exp>),
    Tuple(Vec<Value>),
    Proj(Value, i32)
}

impl Cexp {
    pub fn program_display(self) {
        use Cexp::*;
        match self {
            Val(Value::Var(v)) => { print!("{}", v); }
            Val(Value::Intv(v)) => { print!("{}", v); }
            Binop(tty, val1, val2) => {
                Val(val1).program_display();
                print!("{}", tty.bintype_signal());
                Val(val2).program_display();
            }
            App(val1, mut valls) => {
                Val(val1).program_display();
                print!(" (");
                for i in 0..valls.len() {
                    Val(std::mem::replace(&mut valls[i], Value::Intv(-1))).program_display();
                    if i+1 == valls.len() { break }
                    print!(", ");
                }
                print!(")")
            }
            If(val, exp1, exp2) => {
                print!("if ");
                Val(val).program_display();
                print!(" then ");
                exp1.program_display();
                print!(" else ");
                exp2.program_display();
            }
            Tuple(mut valls) => {
                print!(" (");
                for i in 0..valls.len() {
                    Val(std::mem::replace(&mut valls[i], Value::Intv(-1))).program_display();
                    if i+1 == valls.len() { break }
                    print!(", ");
                }
                print!(")")
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
    Letrec(Id, Vec<Id>, Box<Exp>, Box<Exp>),
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
            Letrec(id1, idls, exp1, exp2) => {
                print!("let rec {} = fun ", id1);
                print!("(");
                for i in 0..idls.len() {
                    print!("{}", idls[i]);
                    if i+1 == idls.len() { break }
                    print!(", ");
                }
                print!(") -> ");
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

fn extract_v(asv: &Vec<String>, val: Value) -> Vec<String> {
    match val {
        Value::Var(v) => {
            if !asv.contains(&v) {
                vec![v.clone()]
            } else {
                vec![]
            }
        }
        Value::Intv(_) => { vec![] }
    }
}

fn sub_find_fv(nmce: normal::Cexp, asv: &mut Vec<String>) -> (Vec<String>, normal::Cexp) {
    use normal::Cexp::*;
    match nmce {
        Val(val) => { (extract_v(&asv, val.clone()), Val(val)) } 
        Binop(btype, val1, val2) => {
            let mut fv = extract_v(&asv, val1.clone());
            fv.append(&mut extract_v(&asv, val2.clone()));
            (fv, Binop(btype, val1, val2))
        }
        App(val1, val2) => {
            let mut fv = extract_v(&asv, val1.clone());
            fv.append(&mut extract_v(&asv, val2.clone()));
            (fv, App(val1, val2))
        }
        Tuple(val1, val2) => {
            let mut fv = extract_v(&asv, val1.clone());
            fv.append(&mut extract_v(&asv, val2.clone()));
            (fv, Tuple(val1, val2))
        }
        If(val, exp1, exp2) => {
            let mut fv = extract_v(&asv, val.clone());
            let (mut fv1, nexp1) = find_fv(*exp1, asv);
            let (mut fv2, nexp2) = find_fv(*exp2, asv);
            fv.append(&mut fv1);
            fv.append(&mut fv2);
            (fv, If(val, Box::new(nexp1), Box::new(nexp2)))
        }
        Proj(val1, c) => {
            (extract_v(&asv, val1.clone()), Proj(val1, c))
        }
    }
}

fn find_fv(normexp: normal::Exp, asv: &mut Vec<String>) -> (Vec<String>, normal::Exp) {
    use normal::Exp::*;

    match normexp {
        Compexp(nmce) => {
            let (fv, _nmce) = sub_find_fv(*nmce, asv);
            (fv, Compexp(Box::new(_nmce)))
        }
        Let(id, nmce, nme) => {
            asv.push(id.clone());
            let (mut fv, _nmce) = sub_find_fv(*nmce, asv);
            let (mut fv2, _nme) = find_fv(*nme, asv);
            fv.append(&mut fv2);
            (fv, Let(id, Box::new(_nmce), Box::new(_nme)))
        }
        Loop(id, nmce, nme) => {
            asv.push(id.clone());
            let (mut fv, _nmce) = sub_find_fv(*nmce, asv);
            let (mut fv2, _nme) = find_fv(*nme, asv);
            fv.append(&mut fv2);
            (fv, Loop(id, Box::new(_nmce), Box::new(_nme)))
        }
        Letrec(id1, id2, nme1, nme2) => {
            asv.push(id1.clone());
            asv.push(id2.clone());
            let (mut fv, _nme1) = find_fv(*nme1, asv);
            let (mut fv2, _nme2) = find_fv(*nme2, asv);
            fv.append(&mut fv2);
            (fv, Letrec(id1, id2, Box::new(_nme1), Box::new(_nme2)))
        }
        Recur(val) => {
            (extract_v(&asv, val.clone()), Recur(val))
        }
    }
}

fn nce2cce(nce: normal::Cexp) -> Cexp {
    use normal::Cexp::*;
    match nce {
        Val(val) => { Cexp::Val(val) }
        Binop(btype, val1, val2) => { Cexp::Binop(btype, val1, val2) }
        App(val1, val2) => { Cexp::App(val1, vec![val2]) }
        Tuple(val1, val2) => { Cexp::Tuple(vec![val1, val2]) }
        Proj(val, c) => { Cexp::Proj(val, c) }
        If(..) => {
            panic!("nce2cce error.")
        }
    }
}

pub fn duplicate_var(nv: String) -> String {
    let mut s = String::from("@v");
    s.push_str(&nv[2..]);
    s
}

fn convert(normexp: normal::Exp, fid: usize, fs: &mut Vec<AsgFun>) -> Exp {
    use normal::Exp::*;
    use normal::Cexp::*;
    match normexp {
        Compexp(nmce) => {
            match *nmce {
                If(val, nme1, nme2) => {
                    fs[fid].apply()(Cexp::If(val, Box::new(convert(*nme1, 0, fs)), Box::new(convert(*nme2, 0, fs))))
                }
                App(Value::Var(v), val2) => {
                    let appv = get_fresh_function_var(&v[..], 'r');
                    Exp::Let(appv.clone(), Box::new(Cexp::Proj(Value::Var(v.clone()), 0)), Box::new(fs[fid].apply()(Cexp::App(Value::Var(appv), vec![Value::Var(v), val2]))))
                }
                ce => {
                    fs[fid].apply()(nce2cce(ce))
                }
            }
        }
        Let(id, nmce, nme) => {
            match *nmce {
                If(val, nme1, nme2) => {
                    Exp::Let(id, Box::new(Cexp::If(val, Box::new(convert(*nme1, 0, fs)), Box::new(convert(*nme2, 0, fs)))), Box::new(convert(*nme, fid, fs)))
                }
                App(Value::Var(v), val2) => {
                    let _cme2 = convert(*nme, fid, fs);
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Let(id, Box::new(ce), Box::new(_cme2)) } )));
                    convert(Compexp(Box::new(App(Value::Var(v), val2))), fs.len()-1, fs)
                }
                ce => {
                    Exp::Let(id, Box::new(nce2cce(ce)), Box::new(convert(*nme, fid, fs)))
                }
            }
        }
        Loop(id, nmce, nme) => {
            match *nmce {
                If(val, nme1, nme2) => {
                    Exp::Loop(id, Box::new(Cexp::If(val, Box::new(convert(*nme1, 0, fs)), Box::new(convert(*nme2, 0, fs)))), Box::new(convert(*nme, fid, fs)))
                }
                App(Value::Var(v), val2) => {
                    let _cme2 = convert(*nme, fid, fs);
                    fs.push(AsgFun::new(Box::new(|ce| { Exp::Loop(id, Box::new(ce), Box::new(_cme2)) } )));
                    convert(Compexp(Box::new(App(Value::Var(v), val2))), fs.len()-1, fs)
                }
                ce => {
                    Exp::Loop(id, Box::new(nce2cce(ce)), Box::new(convert(*nme, fid, fs)))
                }
            }
        }
        Letrec(id1, id2, nme1, nme2) => {
            let mut args = vec![id1.clone(), id2.clone()];
            let (mut fvs, nme1) = find_fv(*nme1, &mut args);
            let mut cfs = vec![AsgFun::new(Box::new(ef))];
            let mut projv = vec![];
            for i in 0..fvs.len() {
                projv.push(Box::new(Cexp::Proj(Value::Var(id1.clone()), i as i32 + 1)));
            }
            for i in 0..fvs.len() {
                let lid = cfs.len()-1;
                let f = cfs[lid].apply();
                let fvsi = fvs[i].clone();
                let proji = projv[i].clone();
                cfs.push(AsgFun::new(Box::new(|ce| { Exp::Let(fvsi, proji, Box::new(f(ce))) })));
            }
            let lid = cfs.len()-1;
            fs.push(std::mem::replace(&mut cfs[lid], AsgFun::new(Box::new(ef))));
            let csexp1 = convert(nme1, fs.len()-1, fs);
            let ffv = get_fresh_function_var(&id1[..], 'b');
            fvs.insert(0, ffv.clone());
            let mut cs_varset = vec![];
            for v in fvs {
                cs_varset.push(Value::Var(v.clone()));
            }
            Exp::Letrec(ffv.clone(), vec![id1.clone(), id2], Box::new(csexp1), Box::new(Exp::Let(id1, Box::new(Cexp::Tuple(cs_varset)), Box::new(convert(*nme2, fid, fs)))))
        }
        Recur(val) => {
            Exp::Recur(val)
        }
    }
}

pub fn closure(normexp: normal::Exp) -> Exp {
    let mut fs = vec![AsgFun::new(Box::new(ef))];
    convert(normexp, 0, &mut fs)
}