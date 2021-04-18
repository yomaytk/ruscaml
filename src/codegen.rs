use super::*;
use super::vm::*;

pub fn codegen(program: vm::Program) {
    for decl in program.decls {
        print!("{}:\n", decl.funlb);
        print!("\tpush {{r7}}\n");
        print!("\tsub sp, sp, #{}\n", decl.vc*4 + 8);
        print!("\tadd r7, sp, #0\n");
        for instr in decl.instrs {
            use Instr::*;
            use normal::Bintype::*;
            match instr {
                Move(r, op) => {
                    if let Operand::Intv(v) = op {
                        print!("\tmovs r{}, #{}\n", r.rm, v);
                    } else {
                        panic!("codegen Move error. {:?}", op);
                    }
                }
                Mover(r1, r2) => {
                    print!("\tmov r{}, r{}\n", r1.rm, r2.rm);
                }
                Store(ofs, r) => {
                    print!("\tstr r{}, [r7, #{}]\n", r.rm, 4*ofs+8);
                }
                Load(r, ofs) => {
                    print!("\tldr r{}, [r7, #{}]\n", r.rm, 4*ofs+8);
                }
                Loadf(r, id) => {
                    print!("\tmovw r{}, #:lower16:{}\n", r.rm, id);
                    print!("\tmovw r{}, #:upper16:{}\n", r.rm, id);
                }
                Argst(ofs, op) => {
                    if let Operand::Param(i) = op {
                        print!("\tstr r{}, [r7, #{}]\n", i, 4*ofs+8);
                    } else {
                        panic!("codegen Argst error.");
                    }
                }
                Binop(btype, r1, r2) => {
                    match btype {
                        Plus => {
                            print!("\tadd r{}, r{}, r{}\n", r1.rm, r1.rm, r2.rm);
                        }
                        Mult => {
                            print!("\tmul r{}, r{}, r{}\n", r1.rm, r1.rm, r2.rm);
                        }
                        Lt => {
                            print!("\tcmp r{}, r{}\n", r1.rm, r2.rm);
                            print!("\tite lt\n");
                            print!("\tmovlt r{}, #1\n", r1.rm);
                            print!("\tmovge, r{}, #0\n", r2.rm);
                            print!("\tuxtb r{}, r{}\n", r1.rm, r1.rm);
                        }
                    }
                }
                Label(lb) => {
                    print!("{}:\n", lb);
                }
                Br(r, lb) => {
                    print!("\tcmp r{}, #1\n", r.rm);
                    print!("\tbeq {}\n", lb);
                }
                Gt(lb) => {
                    print!("\tb {}\n", lb);
                }
                Call(r, args) => {
                    print!("\tpush r0\n");
                    print!("\tpush r1\n");
                    for i in 0..2 {
                        print!("\tmov r{}, r{}\n", i, args[i].rm);
                    }
                    print!("\tblx r{}\n", r.rm);
                }
                Ret(r1, r2) => {
                    print!("\tmov r{}, r{}\n", r1.rm, r2.rm);
                }
                Malloc(r, data) => {
                    print!("\tpush r0\n");
                    print!("\tmov r0, {}\n", data.len()*4);
                    print!("\tbl mymalloc\n");
                    for i in 0..data.len() {
                        print!("\tstr r0, [r{}, #{}]\n", data[i].rm, i*4);
                    }
                    print!("\tmov r{}, r0\n", r.rm);
                    print!("\tldr r0, [sp], #4\n");
                }
                Read(r1, r2, ofs) => {
                    print!("\tldr r{}, [r{}, #{}]\n", r1.rm, r2.rm, ofs*4);
                }
                Begin(..) | End(..) | Kill(..) | Dummy => {}
            }
        }
        print!("\tadd r7, r7, #{}\n", decl.vc*4+8);
        print!("\tmov sp, r7\n");
        print!("\tldr r7, [sp]\n");
        print!("\tbx lr\n");
    }
}