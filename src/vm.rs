use super::*;
use super::normal::Bintype;
use std::sync::Mutex;
use once_cell::sync::Lazy;

type Ofs = i32;
type Label = String;

pub static STACK_POS: Lazy<Mutex<i32>> = Lazy::new(|| { Mutex::new(1) });
pub static FRESH_NUM: Lazy<Mutex<i32>> = Lazy::new(|| { Mutex::new(0) });
pub static LOOP_INFO: Lazy<Mutex<Vec<(Label, i32)>>> = Lazy::new(|| { Mutex::new(vec![]) });

fn next_stack() -> i32 {
    let pos = *STACK_POS.lock().unwrap();
    *STACK_POS.lock().unwrap() = pos+1;
    pos
}

fn next_label() -> Label {
    let fresh_num = *FRESH_NUM.lock().unwrap();
    *FRESH_NUM.lock().unwrap() = fresh_num+1;
    let mut s = Label::from(".L");
    s.push_str(&fresh_num.to_string());
    s
}

fn add_loopinfo(label: Label, ofs: i32) {
    (*LOOP_INFO.lock().unwrap()).push((label, ofs));
}

fn get_loopinfo() -> (Label, Ofs) {
    if let Some(info) = (*LOOP_INFO.lock().unwrap()).pop() {
        info
    } else {
        panic!("get_looplabel error.");
    }
}

#[derive(Clone, Debug)]
pub struct Program(pub Vec<Decl>);

impl Program {
    fn new() -> Self {
        Self(vec![])
    }
    fn add(&mut self, decl: Decl) {
        self.0.push(decl);
    }
    pub fn program_display(self) {
        for decl in self.0 {
            decl.program_display();
        }
    }
}

#[derive(Clone, Debug)]
pub struct Decl{
    pub vc: i32,
    instrs: Vec<Instr>
}

impl Decl {
    fn new(vc: i32, instrs: Vec<Instr>) -> Self {
        Self {
            vc,
            instrs,
        }
    }
    fn addinstr(&mut self, instr: Instr) {
        self.instrs.push(instr);
    }
    fn program_display(self) {
        for instr in self.instrs {
            instr.program_display();
        }
    }
}

#[derive(Debug, Clone)]
enum Operand {
    Param(i32),
    Local(Ofs),
    Proc(Label),
    Intv(i32),
}

impl Operand {
    fn program_display(self) {
        use Operand::*;
        match self {
            Param(c) => { print!(" param({})", c); }
            Local(c) => { print!(" local({})", c); }
            Proc(lb) => { print!(" labimm {}", lb); }
            Intv(c) => { print!(" imm({})", c); }
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum Instr {
    Move(Ofs, Operand),
    Binop(Ofs, Bintype, Operand, Operand),
    Label(Label),
    Br(Operand, Label),
    Brn(Operand, Label),
    Gt(Label),
    Call(Ofs, Operand, Vec<Operand>),
    Ret(Operand),
    Malloc(Ofs, Vec<Operand>),
    Read(Ofs, Operand, i32),
    Begin(Label),
    End(Label),
}

impl Instr {
    fn program_display(self) {
        use Instr::*;
        match self {
            Move(c, op1) => { 
                print!(" local({}) <-", c);
                op1.program_display();
                print!("\n");
            }
            Binop(c, btype, op1, op2) => {
                print!(" local({}) <- {}(", c, btype.bintype_signal());
                op1.program_display();
                print!(", ");
                op2.program_display();
                print!(")\n");
            }
            Label(lb) => { print!("{}:\n", lb) }
            Br(op, lb) => { 
                print!(" if");
                op.program_display();
                print!(" then goto {}\n", lb); 
            }
            Brn(op, lb) => { 
                print!(" if not");
                op.program_display();
                print!(" then goto {}\n", lb); 
            }
            Gt(lb) => { print!(" goto {}", lb); }
            Call(c, op_f, mut ops) => {
                print!(" local({}) <-", c);
                op_f.program_display();
                ops.reverse();
                print!("(");
                loop {
                    if let Some(op) = ops.pop() {
                        op.program_display();
                    }
                    if ops.is_empty() { 
                        print!(")\n");
                        break; 
                    }
                    print!(", ");
                }
            }
            Ret(op) => { 
                print!("return(");
                op.program_display();
                print!(")\n");
            }
            Malloc(c, mut ops) => {
                print!("new {} [", c);
                loop {
                    if let Some(op) = ops.pop() {
                        op.program_display();
                    }
                    if ops.is_empty(){
                        print!("]\n");
                        break;
                    }
                    print!(", ");
                }
            }
            Read(c, op, i) => {
                print!("read {} #{}(", c, i);
                op.program_display();
                print!(")\n");
            }
            Begin(..) | End(..) => { print!(" Begin or End is unimplemented. "); }
        }
    }
}

fn trans_value(fval: flat::Value, env: &Env<String, i32>) -> Operand {
    use flat::Value::*;
    match fval {
        Var(id) => { Operand::Local(env.find(&id)) }
        Fun(id) => { Operand::Proc(id) }
        Intv(v) => { Operand::Intv(v) }
    }
}

fn trans_cexp(fcexp: flat::Cexp, decl: &mut Decl, env: &mut Env<String, i32>) -> Operand {
    use flat::Cexp::*;
    match fcexp {
        Val(val) => { trans_value(val, env) }
        Binop(btype, val1, val2) => {
            let ofs = next_stack();
            decl.addinstr(Instr::Binop(ofs, btype, trans_value(val1, env), trans_value(val2, env)));
            Operand::Local(ofs)
        }
        App(val, vals) => {
            let mut args = vec![];
            for val in vals {
                args.push(trans_value(val, env));
            }
            let ofs = next_stack();
            decl.addinstr(Instr::Call(ofs, trans_value(val, env), args));
            Operand::Local(ofs)
        }
        If(val, fexp1, fexp2) => {
            let t_e = next_label();
            decl.addinstr(Instr::Brn(trans_value(val, env), t_e.clone()));
            env.inc();
            trans_exp(*fexp1, decl, env);
            env.dec();
            decl.addinstr(Instr::Label(t_e.clone()));
            env.inc();
            trans_exp(*fexp2, decl, env);
            env.dec();
            Operand::Local(-1)
        }
        Tuple(vals) => {
            let mut data = vec![];
            for val in vals {
                data.push(trans_value(val, env));
            }
            let ofs = next_stack();
            decl.addinstr(Instr::Malloc(ofs, data));
            Operand::Local(ofs)
        }
        Proj(val, c) => {
            let ofs = next_stack();
            decl.addinstr(Instr::Read(ofs, trans_value(val, env), c));
            Operand::Local(ofs)
        }
    }
}

fn trans_exp(fexp: flat::Exp, decl: &mut Decl, env: &mut Env<String, i32>) {
    use flat::Exp::*;
    match fexp {
        Compexp(fcexp) => {
            trans_cexp(*fcexp, decl, env);
        }
        Let(id, fcexp, fexp) => {
            let op = trans_cexp(*fcexp, decl, env);
            let ofs = next_stack();
            decl.addinstr(Instr::Move(ofs, op));
            env.addval(id, ofs);
            trans_exp(*fexp, decl, env);
        }
        Loop(id, fcexp, fexp) => {
            let loop_l = next_label();
            decl.addinstr(Instr::Label(loop_l.clone()));
            let id_ofs = next_stack();
            add_loopinfo(loop_l, id_ofs);
            let op = trans_cexp(*fcexp, decl, env);
            decl.addinstr(Instr::Move(id_ofs, op));
            env.addval(id, id_ofs);
            trans_exp(*fexp, decl, env);
        }
        Recur(val) => {
            let (loop_l, loop_ofs) = get_loopinfo();
            decl.addinstr(Instr::Move(loop_ofs, trans_value(val, env)));
            decl.addinstr(Instr::Gt(loop_l));
        }
    }
}

pub fn trans_pg(pg: flat::Program) -> Program {
    let mut env = Env::new();
    let mut program = Program::new();
    for flat::Recdecl(funame, args, body) in pg.recs {
        let mut decl = Decl::new(0, vec![]);
        decl.addinstr(Instr::Label(funame));
        env.inc();
        let mut pari = 1;
        for arg in args {
            let ofs = next_stack();
            decl.addinstr(Instr::Move(ofs, Operand::Param(pari)));
            env.addval(arg, ofs);
            pari += 1;
        }
        trans_exp(*body, &mut decl, &mut env);
        env.dec();
        decl.vc = *STACK_POS.lock().unwrap()-1;
        *STACK_POS.lock().unwrap() = 1;
        program.add(decl);
    }
    program
}