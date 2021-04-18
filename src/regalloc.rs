use super::*;

pub const REG_SIZE: usize = 7;
pub const A1: i32 = 0;

pub fn regalloc(pg: &mut vm::Program) {
    let mut regs: [i32; 7] = [-1, -1, -1, -1, -1, -1, -1];
    for decl in &mut pg.decls {
        for instr in &mut decl.instrs {
            use vm::Instr::*;
            match instr {
                Mover(r1, r2) | Binop(_, r1, r2) | Read(r1, r2, _) => {
                    r1.set_real(&mut regs);
                    r2.set_real(&mut regs);
                }
                Move(r, _) | Store(_, r) 
                | Load(r, _) | Loadf(r, _) | Br(r, ..) => {
                    r.set_real(&mut regs);
                }
                Malloc(r, args) | Call(r, args)=> {
                    r.set_real(&mut regs);
                    for reg in args {
                        reg.set_real(&mut regs);
                    }
                }
                Ret(r1, r2) => {
                    r1.rm = A1;
                    r2.set_real(&mut regs);
                }
                Kill(r) => {
                    r.kill(&mut regs);
                }
                _ => {}
            }
        }
    }
}