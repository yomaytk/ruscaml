use super::*;
use super::normal::Bintype;
use regalloc::REG_SIZE;

use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub type Ofs = i32;
type Label = String;

macro_rules! print_reg {
    ($r: ident, $real: expr) => {
        if $real { $r.rm } else { $r.vm }
    };
}

macro_rules! reg_byte {
    ($r: ident) => {
        if $r.byte == 4 { next_stack32() } else { assert_eq!($r.byte, 8); next_stack64() }
    };
}

pub static STACK_POS: Lazy<Mutex<i32>> = Lazy::new(|| { Mutex::new(1) });
pub static FRESH_NUM: Lazy<Mutex<i32>> = Lazy::new(|| { Mutex::new(0) });
pub static LOOP_INFO: Lazy<Mutex<Vec<(Label, i32)>>> = Lazy::new(|| { Mutex::new(vec![]) });
pub static REG_NUM: Lazy<Mutex<i32>> = Lazy::new(|| { Mutex::new(0)} );
pub static HAVE_APP: Lazy<Mutex<bool>> = Lazy::new(|| { Mutex::new(false)} );
pub static MALLOC_INFO: Lazy<Mutex<HashMap<String, Vec<i32>>>> = Lazy::new(|| { Mutex::new(HashMap::new()) });

fn next_stack32() -> i32 {
    let pos = *STACK_POS.lock().unwrap();
    *STACK_POS.lock().unwrap() = pos+1;
    pos
}

fn next_stack64() -> i32 {
    let pos = *STACK_POS.lock().unwrap();
    *STACK_POS.lock().unwrap() = pos+2;
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

fn malloc_info_dummy() -> Option<Vec<i32>> {
    let res = (*MALLOC_INFO).lock().unwrap().remove("dummy");
    res
}

#[derive(Clone, Debug)]
pub struct Program {
    pub decls: Vec<Decl>,
}

impl Program {
    fn new() -> Self {
        Self {
            decls: vec![],
        }
    }
    fn add(&mut self, decl: Decl) {
        self.decls.push(decl);
    }
    pub fn program_display(self, real: bool) {
        for decl in self.decls {
            decl.program_display(real);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Decl{
    pub funlb: Label,
    pub vc: i32,
    pub instrs: Vec<Instr>,
    pub haveapp: bool,
}

impl Decl {
    fn new(funlb: Label, vc: i32, instrs: Vec<Instr>, haveapp: bool) -> Self {
        Self {
            funlb,
            vc,
            instrs,
            haveapp,
        }
    }
    fn addinstr(&mut self, instr: Instr) {
        self.instrs.push(instr);
    }
    fn program_display(self, real: bool) {
        for instr in self.instrs {
            instr.program_display(real);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operand {
    Param(i32),
    Local(Ofs, i32),
    Proc(Label),
    Intv(i32),
}

#[derive(Debug, Clone, Copy)]
pub struct Reg {
    pub vm: i32,
    pub rm: i32,
    pub byte: i32,
}

impl Reg {
    fn new(byte: i32) -> Self {
        Self {
            vm: next_regnum(),
            rm: -1,
            byte,
        }
    }
    pub fn set_real(&mut self, regs: &mut [i32; REG_SIZE]) {
        // find real register already allocated
        for i in 0..REG_SIZE {
            if regs[i] == self.vm {
                self.rm = i as i32;
                return;
            }
        }
        // allocate real register
        for i in 0..REG_SIZE {
            if regs[i] == -1 {
                regs[i] = self.vm;
                self.rm = i as i32;
                return;
            }
        }
        // message_error("There are enough registers.");
    }
    pub fn kill(&mut self, regs: &mut [i32; REG_SIZE]) {
        for i in 0..REG_SIZE {
            if regs[i] == self.vm {
                self.rm = i as i32;
                regs[i] = -1;
                return;
            }
        }
        message_error("register cannot be killed.")
    }
}
impl Operand {
    fn program_display(self) {
        use Operand::*;
        match self {
            Param(c) => { print!(" param({})", c); }
            Local(c, _) => { print!(" local({})", c); }
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
    Load(Reg, i32),
    Loadf(Reg, Label),
    Argst(i32, Operand),
    Binop(Bintype, Reg, Reg),
    Label(Label),
    Br(Reg, Label),
    Gt(Label),
    Call(Reg, Vec<Reg>),
    Ret(Reg, Reg),
    Malloc(Reg, Vec<Reg>),
    Read(Reg, i32),
    Begin(Label),
    End(Label),
    Kill(Reg),
    Dummy,
}

// should use macro
impl Instr {
    fn program_display(self, real: bool) {
        use Instr::*;
        match self {
            Move(r, op1) => {
                print!(" r{} <-", print_reg!(r, real));
                op1.program_display();
                print!("\n");
            }
            Mover(r1, r2) => {
                print!(" r{} <- r{}\n", print_reg!(r1, real), print_reg!(r2, real));
            }
            Store(ofs, r) => {
                print!(" local({}) <- r{}\n", ofs, print_reg!(r, real));
            }
            Load(r, ofs) => {
                print!(" r{} <- local({})\n", print_reg!(r, real), ofs);
            }
            Loadf(r, id) => {
                print!(" r{} <- :{}\n", print_reg!(r, real), id);
            }
            Argst(ofs, op) => {
                print!(" local({}) <- Param(", ofs);
                op.program_display();
                print!(" )\n");
            }
            Binop(btype, r1, r2) => {
                print!(" r{} <- {}(r{}, r{})\n", print_reg!(r1, real), btype.bintype_signal(), print_reg!(r1, real), print_reg!(r2, real));
            }
            Label(lb) => { print!("{}:\n", lb) }
            Br(r, lb) => { 
                print!(" if r{} then goto {}\n", print_reg!(r, real), lb);
            }
            Gt(lb) => { print!(" goto {}\n", lb); }
            Call(r, mut args) => {
                print!(" r{} ", print_reg!(r, real));
                args.reverse();
                print!("(");
                loop {
                    if let Some(rx) = args.pop() {
                        print!(" r{}", print_reg!(rx, real));
                    }
                    if args.is_empty() { 
                        print!(" )\n");
                        break; 
                    }
                    print!(",");
                }
            }
            Ret(r1, r2) => { 
                print!(" r{} <- r{}\n", print_reg!(r1, real), print_reg!(r2, real));
                print!(" return(r{})\n", print_reg!(r1, real));
            }
            Malloc(r, mut datas) => {
                print!("r{} <- new [", print_reg!(r, real));
                loop {
                    if let Some(rx) = datas.pop() {
                        print!(" r{}", rx.rm);
                    }
                    if datas.is_empty(){
                        print!(" ]\n");
                        break;
                    }
                    print!(",");
                }
            }
            Read(r, ofs) => {
                print!("read r{} <- #{}( r{} )\n", print_reg!(r, real), ofs, print_reg!(r, real));
            }
            Kill(r) => {
                print!("kill r{}\n", print_reg!(r, real));
            }
            Begin(..) | End(..) => { print!(" Begin or End is unimplemented. "); }
            Dummy => {}
        }
    }
}

fn trans_value(fval: flat::Value, env: &Env<String, (Ofs, i32)>) -> Operand {
    use flat::Value::*;
    match fval {
        Var(id) => { 
            let (ofs, b4) = env.find(&id);
            Operand::Local(ofs, b4)
        }
        Fun(id) => { Operand::Proc(id) }
        Intv(v) => { Operand::Intv(v) }
    }
}

fn value2reg(decl: &mut Decl, val: flat::Value, env: &Env<String, (Ofs, i32)>) -> Reg {
    let op = trans_value(val, env);
    use Operand::*;
    match op {
        Local(ofs, byte) => {
            let r = Reg::new(byte);
            decl.addinstr(Instr::Load(r, ofs));
            r
        }
        Intv(v) => {
            let r = Reg::new(4);
            decl.addinstr(Instr::Move(r, Intv(v)));
            r
        }
        Proc(id) => {
            let r = Reg::new(4);
            decl.addinstr(Instr::Loadf(r, id));
            r
        }
        Param(..) => {
            panic!("value2reg error.");
        }
    }
}

fn trans_cexp(fcexp: flat::Cexp, decl: &mut Decl, env: &mut Env<String, (Ofs, i32)>) -> Reg {
    use flat::Cexp::*;
    match fcexp {
        Val(val) => {
            value2reg(decl, val, env)
        }
        Binop(btype, val1, val2) => {
            let r1 = value2reg(decl, val1, env);
            let r2 = value2reg(decl, val2, env);
            decl.addinstr(Instr::Binop(btype, r1, r2));
            decl.addinstr(Instr::Kill(r2));
            r1
        }
        App(val, vals) => {
            let mut args = vec![];
            for val in vals {
                let r = value2reg(decl, val, env);
                args.push(r);
            }
            let r1 = value2reg(decl, val, env);
            decl.addinstr(Instr::Call(r1, args.clone()));
            for arg in args {
                decl.addinstr(Instr::Kill(arg));
            }
            *HAVE_APP.lock().unwrap() = true;
            r1
        }
        If(val, fexp1, fexp2) => {
            let t_e1 = next_label();
            let t_e2 = next_label();
            let r1 = value2reg(decl, val, env);
            decl.addinstr(Instr::Br(r1, t_e1.clone()));
            env.inc();
            let r2 = trans_exp(*fexp2, decl, env);
            decl.addinstr(Instr::Mover(r1, r2));
            decl.addinstr(Instr::Kill(r2));
            decl.addinstr(Instr::Gt(t_e2.clone()));
            env.dec();
            decl.addinstr(Instr::Label(t_e1.clone()));
            env.inc();
            let r3 = trans_exp(*fexp1, decl, env);
            decl.addinstr(Instr::Mover(r1, r3));
            decl.addinstr(Instr::Kill(r3));
            decl.addinstr(Instr::Label(t_e2.clone()));
            env.dec();
            r1
        }
        Tuple(vals) => {
            let mut data = vec![];
            let mut bsizes = vec![];
            for val in vals {
                let mut r = value2reg(decl, val, env);
                r.byte = 8;
                bsizes.push(r.byte);
                data.push(r);
            }
            let r1 = Reg::new(8);
            decl.addinstr(Instr::Malloc(r1, data.clone()));
            for d in data {
                decl.addinstr(Instr::Kill(d));
            }
            // assert!(!(*MALLOC_INFO).lock().unwrap().contains_key("dummy"));
            (*MALLOC_INFO).lock().unwrap().insert(String::from("dummy"), bsizes);
            r1
        }
        Proj(val, c) => {
            let r = value2reg(decl, val, env);
            decl.addinstr(Instr::Read(r, c));
            r
        }
    }
}

fn trans_exp(fexp: flat::Exp, decl: &mut Decl, env: &mut Env<String, (Ofs, i32)>) -> Reg {
    use flat::Exp::*;
    match fexp {
        Compexp(fcexp) => {
            trans_cexp(*fcexp, decl, env)
        }
        Let(id, fcexp, fexp) => {
            let r1 = trans_cexp(*fcexp, decl, env);
            let ofs = reg_byte!(r1);
            if let Some(bsizes) = malloc_info_dummy() {
                (*MALLOC_INFO).lock().unwrap().insert(id.clone(), bsizes);
            }
            decl.addinstr(Instr::Store(ofs, r1));
            decl.addinstr(Instr::Kill(r1));
            env.addval(id, (ofs, r1.byte));
            trans_exp(*fexp, decl, env)
        }
        Loop(id, fcexp, fexp) => {
            let loop_l = next_label();
            decl.addinstr(Instr::Label(loop_l.clone()));
            let r1 = trans_cexp(*fcexp, decl, env);
            let id_ofs = reg_byte!(r1);
            add_loopinfo(loop_l, id_ofs);
            decl.addinstr(Instr::Store(id_ofs, r1));
            decl.addinstr(Instr::Kill(r1));
            env.addval(id, (id_ofs, r1.byte));
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
        let mut decl = Decl::new(funame, 0, vec![], false);
        env.inc();
        let mut pari = 0;
        for arg in args {
            let ofs = next_stack64();
            decl.addinstr(Instr::Argst(ofs, Operand::Param(pari)));
            env.addval(arg, (ofs, 8));
            pari += 1;
        }
        let r1 = trans_exp(*body, &mut decl, &mut env);
        env.dec();
        decl.vc = *STACK_POS.lock().unwrap()-1;
        let mut ra1 = Reg::new(4);
        ra1.byte = r1.byte;
        decl.addinstr(Instr::Ret(ra1, r1));
        decl.addinstr(Instr::Kill(r1));
        decl.haveapp = *HAVE_APP.lock().unwrap();
        program.add(decl);
        *STACK_POS.lock().unwrap() = 0;
    }
    program
}