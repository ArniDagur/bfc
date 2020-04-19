use crate::bfir::AstNode;
use std::num::Wrapping;

fn add_instrs_to_c_prog(instrs: &[AstNode], prog: &mut String) {
    for instr in instrs {
        match instr {
            AstNode::Increment { amount, offset, .. } => {
                prog.push_str(&format!("*(ptr + {}) += {};", offset, amount));
            }
            AstNode::PointerIncrement { amount, .. } => {
                prog.push_str(&format!("ptr += {};", amount));
            }
            AstNode::Read { .. } => {
                prog.push_str("scanf(\"%c\", ptr);");
            }
            AstNode::Write { .. } => {
                prog.push_str("printf(\"%c\", *ptr);");
            }
            AstNode::Loop { body, .. } => {
                prog.push_str("while(*ptr) {");
                add_instrs_to_c_prog(&body, prog);
                prog.push_str("}");
            }
            AstNode::Set { amount, offset, .. } => {
                prog.push_str(&format!("*(ptr + {}) = {};", offset, amount));
            }
            AstNode::MultiplyMove { changes, .. } => {
                let mut targets: Vec<_> = changes.keys().collect();
                targets.sort();

                // The original "bfc" documentation talks about guarding
                // this for loop with `if (*ptr != 0) {...`. But the
                // mandelbrot program is faster without the extra branch.
                for target in targets {
                    let factor = *changes.get(target).unwrap();
                    if factor != Wrapping(0) {
                        prog.push_str(&format!("target = ptr + {};", target));
                        prog.push_str(&format!("*target += (*ptr) * {};", factor));
                    }
                }
                prog.push_str("*ptr = 0;");
            }
        }
    }
}

pub fn c_prog_from_instructions(instrs: &[AstNode]) -> String {
    let mut prog =
        "#include<stdio.h>\nint main(){ static char c[30000] = { 0 }, *target, *ptr; ptr=c;"
            .to_owned();

    add_instrs_to_c_prog(&instrs, &mut prog);
    prog += "}";
    prog
}
