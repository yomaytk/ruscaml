use super::*;
use super::normal::Bintype;
use std::sync::Mutex;
use once_cell::sync::Lazy;

type Ofs = i32;
type Label = String;

pub static STACK_POS: Lazy<Mutex<i32>> = Lazy::new(|| { Mutex::new(1) });
pub static FRESH_NUM: Lazy<Mutex<i32>> = Lazy::new(|| { Mutex::new(0) });
pub static LOOP_INFO: Lazy<Mutex<Vec<(Label, i32)>>> = Lazy::new(|| { Mutex::new(vec![]) });
pub static REG_NUM: Lazy<Mutex<i32>> = Lazy::new(|| { Mutex::new(0)} );

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

fn next_regnum() -> i32 {
    let nreg = *REG_NUM.lock().unwrap();
    *REG_NUM.lock().unwrap() = nreg+1;
    nreg
}

#[derive(Clone, Debug)]
pub struct Program {
    pub decls: Vec<Decl>,
    pub ret: Instr
}

impl Program {
    fn new() -> Self {
        Self {
            decls: vec![],
            ret: Instr::Dummy,
        }
    }
    fn add(&mut self, decl: Decl) {
        self.decls.push(decl);
    }
    pub fn program_display(self) {
        for decl in self.decls {
            decl.program_display();
        }
        self.ret.program_display();
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
pub enum Operand {
    Param(i32),
    Local(Ofs),
    Proc(Label),
    Intv(i32),
}

#[derive(Debug, Clone, Copy)]
pub struct Reg {
    pub vm: i32,
    pub rm: i32
}

impl Reg {
    fn new() -> Self {
        Self {
            vm: next_regnum(),
            rm: 0,
        }
    }
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
pub enum Instr {
    Move(Reg, Operand),
    Mover(Reg, Reg),
    Store(i32, Reg),
    Argst(i32, Operand),
    Binop(Reg, Bintype, Operand, Operand),
    Label(Label),
    Br(Reg, Operand, Label),
    Brn(Reg, Operand, Label),
    Gt(Label),
    Call(Reg, Operand, Vec<Operand>),
    Ret(Reg),
    Malloc(Reg, Vec<Operand>),
    Read(Reg, Operand, i32),
    Begin(Label),
    End(Label),
    Dummy,
}

impl Instr {
    fn program_display(self) {
        use Instr::*;
        match self {
            Move(r, op1) => { 
                print!(" r{} <-", r.vm);
                op1.program_display();
                print!("\n");
            }
            Mover(r1, r2) => {
                print!(" r{} <- r{}\n", r1.vm, r2.vm);
            }
            Store(ofs, r) => {
                print!(" Local({}) <- r{}\n", ofs, r.vm);
            }
            Argst(ofs, op) => {
                print!(" r{} <- Param(", ofs);
                op.program_display();
                print!(" )\n");
            }
            Binop(r, btype, op1, op2) => {
                print!(" r{} <- {}(", r.vm, btype.bintype_signal());
                op1.program_display();
                print!(", ");
                op2.program_display();
                print!(")\n");
            }
            Label(lb) => { print!("{}:\n", lb) }
            Br(r, op, lb) => { 
                print!(" if r{} <- ", r.vm);
                op.program_display();
                print!(" then goto {}\n", lb); 
            }
            Brn(r, op, lb) => { 
                print!(" if not r{} <- ", r.vm);
                op.program_display();
                print!(" then goto {}\n", lb); 
            }
            Gt(lb) => { print!(" goto {}", lb); }
            Call(r, op_f, mut ops) => {
                print!(" r{} <-", r.vm);
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
            Ret(r) => { 
                print!("return(r{})", r.vm);
            }
            Malloc(r, mut ops) => {
                print!("r{} <- new [", r.vm);
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
            Read(r, op, i) => {
                print!("read r{} #{}(", r.vm, i);
                op.program_display();
                print!(")\n");
            }
            Begin(..) | End(..) => { print!(" Begin or End is unimplemented. "); }
            Dummy => {}
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

fn trans_cexp(fcexp: flat::Cexp, decl: &mut Decl, env: &mut Env<String, i32>) -> Reg {
    use flat::Cexp::*;
    match fcexp {
        Val(val) => {
            let r1 = Reg::new();
            let op = trans_value(val, env);
            decl.addinstr(Instr::Move(r1, op));
            r1
        }
        Binop(btype, val1, val2) => {
            let r1 = Reg::new();
            decl.addinstr(Instr::Binop(r1, btype, trans_value(val1, env), trans_value(val2, env)));
            r1
        }
        App(val, vals) => {
            let mut args = vec![];
            for val in vals {
                args.push(trans_value(val, env));
            }
            let r1 = Reg::new();
            decl.addinstr(Instr::Call(r1, trans_value(val, env), args));
            r1
        }
        If(val, fexp1, fexp2) => {
            let t_e1 = next_label();
            let t_e2 = next_label();
            let r1 = Reg::new();
            decl.addinstr(Instr::Brn(r1, trans_value(val, env), t_e1.clone()));
            env.inc();
            let r2 = trans_exp(*fexp1, decl, env);
            decl.addinstr(Instr::Mover(r1, r2));
            decl.addinstr(Instr::Gt(t_e2.clone()));
            env.dec();
            decl.addinstr(Instr::Label(t_e1.clone()));
            env.inc();
            let r3 = trans_exp(*fexp2, decl, env);
            decl.addinstr(Instr::Mover(r1, r3));
            decl.addinstr(Instr::Label(t_e2.clone()));
            env.dec();
            r1
        }
        Tuple(vals) => {
            let mut data = vec![];
            for val in vals {
                data.push(trans_value(val, env));
            }
            let r1 = Reg::new();
            decl.addinstr(Instr::Malloc(r1, data));
            r1
        }
        Proj(val, c) => {
            let r1 = Reg::new();
            decl.addinstr(Instr::Read(r1, trans_value(val, env), c));
            r1
        }
    }
}

fn trans_exp(fexp: flat::Exp, decl: &mut Decl, env: &mut Env<String, i32>) -> Reg {
    use flat::Exp::*;
    match fexp {
        Compexp(fcexp) => {
            trans_cexp(*fcexp, decl, env)
        }
        Let(id, fcexp, fexp) => {
            let r1 = trans_cexp(*fcexp, decl, env);
            let ofs = next_stack();
            decl.addinstr(Instr::Store(ofs, r1));
            env.addval(id, ofs);
            trans_exp(*fexp, decl, env)
        }
        Loop(id, fcexp, fexp) => {
            let loop_l = next_label();
            decl.addinstr(Instr::Label(loop_l.clone()));
            let id_ofs = next_stack();
            add_loopinfo(loop_l, id_ofs);
            let r1 = trans_cexp(*fcexp, decl, env);
            decl.addinstr(Instr::Store(id_ofs, r1));
            env.addval(id, id_ofs);
            trans_exp(*fexp, decl, env)
        }
        Recur(val) => {
            let (loop_l, loop_ofs) = get_loopinfo();
            let r1 = trans_cexp(flat::Cexp::Val(val), decl, env);
            decl.addinstr(Instr::Store(loop_ofs, r1));
            decl.addinstr(Instr::Gt(loop_l));
            r1
        }
    }
}

pub fn trans_pg(pg: flat::Program) -> Program {
    let mut env = Env::new();
    let mut program = Program::new();
    for flat::Recdecl(funame, args, body) in pg.recs {
        let mut decl = Decl::new(0, vec![]);
        decl.addinstr(Instr::Label(funame.clone()));
        env.inc();
        let mut pari = 1;
        for arg in args {
            let ofs = next_stack();
            decl.addinstr(Instr::Argst(ofs, Operand::Param(pari)));
            env.addval(arg, ofs);
            pari += 1;
        }
        let r1 = trans_exp(*body, &mut decl, &mut env);
        env.dec();
        decl.vc = *STACK_POS.lock().unwrap()-1;
        if "_toplevel" == &funame[..] {
            program.ret = Instr::Ret(r1);
        }
        program.add(decl);
        *STACK_POS.lock().unwrap() = 1;
    }
    program
}