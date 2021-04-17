use super::*;

pub const REG_SIZE: usize = 7;
pub const A1: i32 = 8;
pub const A2: i32 = 9;

pub fn regalloc(pg: &mut vm::Program) {
    let mut regs: [i32; 7] = [-1, -1, -1, -1, -1, -1, -1];
    for decl in &mut pg.decls {
        for instr in &mut decl.instrs {
            use vm::Instr::*;
            match instr {
                Move(r, _) => { 
                    r.set_real(&mut regs); 
                }
                Mover(r1, r2) | Binop(_, r1, r2) => {
                    r1.set_real(&mut regs);
                    r2.set_real(&mut regs);
                }
                Store(_, r) => {
                    r.set_real(&mut regs);
                }
                Load(r, _) => {
                    r.set_real(&mut regs);
                }
                Br(r, ..) | Brn(r, ..) | Call(r, ..) | Malloc(r, ..) | Read(r, ..) => {
                    r.set_real(&mut regs);
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