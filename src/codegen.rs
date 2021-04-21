use super::*;
use super::vm::*;

macro_rules! emit_reg {
    ($r: ident) => {
        if $r.byte == 4 { format!("w{}", $r.rm) } else { format!("x{}", $r.rm) }
    }
}

pub fn codegen(program: vm::Program) {
    print!(".text\n");
    print!("\t.global main\n");
    for decl in program.decls {
        let mut spofs = 16 * ((decl.vc*4+15)%16);
        print!("{}:\n", decl.funlb);
        if decl.haveapp {
            spofs += 16;
            print!("\tstp, x29, x30, [sp, -{}]!\n", spofs);
            print!("\tmov, x29, sp\n");
        } else if spofs > 0 {
            print!("\tsub sp, sp, #{}\n", spofs);
        }
        for instr in decl.instrs {
            use Instr::*;
            use normal::Bintype::*;
            match instr {
                Move(r, op) => {
                    if let Operand::Intv(v) = op {
                        print!("\tmov {}, #{}\n", emit_reg!(r), v);
                    } else {
                        panic!("codegen Move error. {:?}", op);
                    }
                }
                Mover(r1, r2) => {
                    print!("\tmov {}, {}\n", emit_reg!(r1), emit_reg!(r2));
                }
                Store(ofs, r) => {
                    print!("\tstr {}, [sp, {}]\n", emit_reg!(r), spofs-4*ofs);
                }
                Load(r, ofs) => {
                    print!("\tldr {}, [sp, {}]\n", emit_reg!(r), spofs-4*ofs);
                }
                Loadf(r, id) => {
                    print!("\tadrp {}, {}\n", emit_reg!(r), id);
                    print!("\tadd {}, {}, :lo12:{}\n", emit_reg!(r), emit_reg!(r), id);
                }
                Argst(ofs, op) => {
                    if let Operand::Param(i) = op {
                        print!("\tstr x{}, [sp, {}]\n", i, 8*ofs);
                    } else {
                        panic!("codegen Argst error.");
                    }
                }
                Binop(btype, r1, r2) => {
                    match btype {
                        Plus => {
                            print!("\tadd {}, {}, {}\n", emit_reg!(r1), emit_reg!(r1), emit_reg!(r2));
                        }
                        Mult => {
                            print!("\tmul {}, {}, {}\n", emit_reg!(r1), emit_reg!(r1), emit_reg!(r2));
                        }
                        Lt => {
                            print!("\tcmp {}, {}\n", emit_reg!(r1), emit_reg!(r2));
                            print!("\tcset lt\n");
                            print!("\tand {}, {}\n", emit_reg!(r1), emit_reg!(r1));
                        }
                    }
                }
                Label(lb) => {
                    print!("{}:\n", lb);
                }
                Br(r, lb) => {
                    print!("\tcmp {}, #1\n", emit_reg!(r));
                    print!("\tbeq {}\n", lb);
                }
                Gt(lb) => {
                    print!("\tb {}\n", lb);
                }
                Call(r, args) => {
                    for i in 0..args.len() {
                        print!("\tmov x{}, x{}\n", i, args[i].rm);
                    }
                    print!("\tblr {}\n", emit_reg!(r));
                    print!("\tmov {}, w0\n", emit_reg!(r));
                }
                Ret(r1, r2) => {
                    print!("\tmov {}, {}\n", emit_reg!(r1), emit_reg!(r2));
                }
                Malloc(r, data) => {
                    print!("\tsub sp, sp, #8\n");
                    print!("\tstr x0, [sp, 8]\n");
                    let mut datasize = 0;
                    for d in &data {
                        datasize += d.byte;
                    }
                    print!("\tmov w0, {}\n", datasize);
                    print!("\tbl mymalloc\n");
                    let mut ofs = 0;
                    for d in data {
                        print!("\tstr {}, [x0, {}]\n", emit_reg!(d), ofs);
                        ofs += d.byte;
                    }
                    print!("\tmov x{}, x0\n", r.rm);
                    print!("\tldr x0, [sp, 8]\n");
                    print!("\tadd sp, sp #8\n");    
                }
                Read(r, ofs) => {
                    print!("\tldr x{}, [{}, {}]\n", r.rm, emit_reg!(r), ofs);
                }
                Begin(..) | End(..) | Kill(..) | Dummy => {}
            }
        }
        if decl.haveapp {
            print!("\tldp x29, x30, [sp], {}\n", spofs);
        }
        print!("\tret\n");
    }
}