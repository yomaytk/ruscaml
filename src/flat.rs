use once_cell::sync::Lazy;
use std::sync::Mutex;

use super::*;
use super::normal::Bintype;

pub static PROG: Lazy<Mutex<Program>> = Lazy::new(|| Mutex::new(Program::new()));

#[derive(Clone, Debug)]
pub struct Recdecl(pub Id, pub Vec<Id>, pub Box<Exp>);

impl Recdecl {
    fn new(id: Id, args: Vec<Id>, exp: Exp) -> Self {
        Self(id, args, Box::new(exp))
    }
    pub fn program_display(self) {
        let funid = self.0;
        let args = self.1;
        let body = *self.2;
        print!("let rec {} ", funid);
            print!("(");
            for i in 0..args.len() {
                print!("{}", args[i]);
                if i+1 == args.len() { break }
                print!(", ");
            }
            print!(") = ");
            match body {
                Exp::Let(..) | Exp::Loop(..) => { 
                    print!("\n");
                }
                _ => {}
            }
            body.program_display();
    }
}

#[derive(Clone, Debug)]
pub struct Program{
    pub recs: Vec<Recdecl>   
}

impl Program {
    fn new() -> Self {
        Self {
            recs: vec![]
        }
    }
    fn add(&mut self, decl: Recdecl) {
        self.recs.push(decl)
    }
    pub fn program_display(self) {
        let mut recs = self.recs;
        recs.reverse();
        loop {
            if let Some(decl) = recs.pop() {
                decl.program_display();
            }
            if recs.is_empty() { break; }
            print!(" in\n");
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Value {
    Var(Id),
    Fun(Id),
    Intv(i32)
}

impl Value {
    pub fn nval2fval(nval: NV, valf: bool) -> Value {
        use normal::Value::*;
        match nval {
            Var(s) => {
                if valf { Value::Var(s) }
                else { Value::Fun(s) }
            }
            Intv(v) => { Value::Intv(v) }
        }
    }
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
            Val(Value::Var(id)) | Val(Value::Fun(id)) => { print!("{}", id); }
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
            Recur(val) => {
                print!("recur ");
                Cexp::Val(val).program_display();
            }
        }
    }
}

fn cce2fce(ccexp: closure::Cexp, env: &Env<NV, Value>) -> Cexp {
    use closure::Cexp::*;
    match ccexp {
        Val(val) => { Cexp::Val(env.find(&val)) }
        Binop(btype, val1, val2) => { Cexp::Binop(btype, env.find(&val1), env.find(&val2)) }
        App(val1, vals) => {
            let mut fvals = vec![];
            for val in vals {
                fvals.push(env.find(&val));
            }
            Cexp::App(env.find(&val1), fvals)
        }
        Tuple(vals) => {
            let mut fvals = vec![];
            for val in vals {
                fvals.push(env.find(&val));
            }
            Cexp::Tuple(fvals)
        }
        Proj(val, c) => {
            Cexp::Proj(env.find(&val), c)
        }
        If(..) => {
            panic!("cce2fce error.")
        }
    }
}

fn sub_flatten(ccexp: closure::Cexp, env: &mut Env<NV, Value>) -> Cexp {
    use closure::Cexp::*;
    match ccexp {
        If(val, clexp1, clexp2) => {
            Cexp::If(env.find(&val), Box::new(flatten(*clexp1, env)), Box::new(flatten(*clexp2, env)))
        }
        _ => {
            cce2fce(ccexp, env)
        }
    }
}

fn flatten(clexp: closure::Exp, env: &mut Env<NV, Value>) -> Exp {
    use closure::Exp::*;
    match clexp {
        Compexp(ccexp) => {
            Exp::Compexp(Box::new(sub_flatten(*ccexp, env)))
        }
        Let(id, ccexp, clexp) => {
            env.inc();
            let fcexp = sub_flatten(*ccexp, env);
            env.dec();
            env.addval(NV::Var(id.clone()), true);
            Exp::Let(id, Box::new(fcexp), Box::new(flatten(*clexp, env)))
        }
        Loop(id, ccexp, clexp) => {
            env.inc();
            let fcexp = sub_flatten(*ccexp, env);
            env.dec();
            env.addval(NV::Var(id.clone()), true);
            Exp::Loop(id, Box::new(fcexp), Box::new(flatten(*clexp, env)))
        }
        Letrec(id1, args, clexp1, clexp2) => {
            env.inc();
            for arg in &args {
                env.addval(NV::Var(arg.clone()), true);
            }
            let fclexp1 = flatten(*clexp1, env);
            env.dec();
            PROG.lock().unwrap().add(Recdecl::new(id1.clone(), args, fclexp1));
            env.addval(NV::Var(id1), false);
            flatten(*clexp2, env)
        }
        Recur(val) => {
            Exp::Recur(env.find(&val))
        }
    }
}

pub fn flat(clexp: closure::Exp) -> Program {
    let mut env = Env::new();
    let toplevel = flatten(clexp, &mut env);
    PROG.lock().unwrap().add(Recdecl::new(String::from("main"), vec![], toplevel));
    std::mem::replace(&mut PROG.lock().unwrap(), Program::new())
}