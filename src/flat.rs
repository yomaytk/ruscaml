use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;

use super::*;
use super::normal::Bintype;

pub static PROG: Lazy<Mutex<Program>> = Lazy::new(|| Mutex::new(Program::new()));

#[derive(Clone, Debug)]
struct Recdecl(Id, Vec<Id>, Box<Exp>);

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
pub struct Program(Vec<Recdecl>);

impl Program {
    fn new() -> Self {
        Self(vec![])
    }
    fn add(&mut self, decl: Recdecl) {
        self.0.push(decl)
    }
    pub fn program_display(self) {
        let mut decls = self.0;
        decls.reverse();
        loop {
            if let Some(decl) = decls.pop() {
                decl.program_display();
            }
            if decls.is_empty() { break; }
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
    fn nval2fval(nval: normal::Value, valf: bool) -> Value {
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

#[derive(Debug, Clone)]
struct Env {
    vals: HashMap<normal::Value, Value>,
    prev: Option<Box<Env>>
}

impl Env {
    fn new() -> Self {
        Self {
            vals: HashMap::new(),
            prev: None
        }
    }
    fn inc(&mut self) {
        let curenv = std::mem::replace(self, Env::new());
        *self = Self {
            vals: HashMap::new(),
            prev: Some(Box::new(curenv))
        }
    }
    fn dec(&mut self) {
        let env = std::mem::replace(&mut (*self).prev, None);
        *self = *env.unwrap()
    }
    fn find(&self, tval: &normal::Value) -> Value {
        if let normal::Value::Intv(v) = tval {
            return Value::Intv(*v);
        }
        let mut nenv = self;
        loop {
            if let Some(val) = nenv.vals.get(tval) {
                return val.clone()
            } else {
                match nenv.prev {
                    None => { panic!("cannot find variable from Env. : {:?}, variable: {:?}", self, tval); }
                    Some(ref next_env) => { nenv = next_env; }
                }
            }
        }
    }
    fn addval(&mut self, val: normal::Value, valf: bool) {
        self.vals.insert(val.clone(), Value::nval2fval(val, valf));
    }
}

fn cce2fce(ccexp: closure::Cexp, env: &Env) -> Cexp {
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

fn sub_flatten(ccexp: closure::Cexp, env: &mut Env) -> Cexp {
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

fn flatten(clexp: closure::Exp, env: &mut Env) -> Exp {
    use closure::Exp::*;
    match clexp {
        Compexp(ccexp) => {
            Exp::Compexp(Box::new(sub_flatten(*ccexp, env)))
        }
        Let(id, ccexp, clexp) => {
            env.inc();
            let fcexp = sub_flatten(*ccexp, env);
            env.dec();
            env.addval(normal::Value::Var(id.clone()), true);
            Exp::Let(id, Box::new(fcexp), Box::new(flatten(*clexp, env)))
        }
        Loop(id, ccexp, clexp) => {
            env.inc();
            let fcexp = sub_flatten(*ccexp, env);
            env.dec();
            env.addval(normal::Value::Var(id.clone()), true);
            Exp::Loop(id, Box::new(fcexp), Box::new(flatten(*clexp, env)))
        }
        Letrec(id1, args, clexp1, clexp2) => {
            env.inc();
            for arg in &args {
                env.addval(normal::Value::Var(arg.clone()), true);
            }
            let fclexp1 = flatten(*clexp1, env);
            env.dec();
            PROG.lock().unwrap().add(Recdecl::new(id1.clone(), args, fclexp1));
            env.addval(normal::Value::Var(id1), false);
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
    PROG.lock().unwrap().add(Recdecl::new(String::from("_toplevel"), vec![], toplevel));
    std::mem::replace(&mut PROG.lock().unwrap(), Program::new())
}