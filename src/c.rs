use crate::bfir::AstNode;
use std::io::prelude::Write;
use std::num::Wrapping;
use std::process::{Command, Stdio};

fn add_instrs_to_c_prog(instrs: &[AstNode], prog: &mut String) {
    for instr in instrs {
        match instr {
            AstNode::Increment { amount, offset, .. } => {
                prog.push_str(&format!("*(ptr + {}) += {};", offset, amount));
            }
            AstNode::PointerIncrement { amount, .. } => {
                if *amount < 0 {
                    prog.push_str(&format!(
                        "if ((ptr + {}) < c) {{ raise(SIGSEGV); }};",
                        amount
                    ));
                } else if *amount > 0 {
                    prog.push_str(&format!(
                        "if ((ptr + {}) >= (c + NUM_CELLS)) {{ raise(SIGSEGV); }};",
                        amount
                    ));
                }
                if *amount != 0 {
                    prog.push_str(&format!("ptr += {};", amount));
                }
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
        "#include<stdio.h>\n#include<signal.h>\n#define NUM_CELLS 30000\nint main(){ static char c[NUM_CELLS] = { 0 }, *target, *ptr; ptr=c;"
            .to_owned();

    add_instrs_to_c_prog(&instrs, &mut prog);
    prog += "}";
    prog
}
pub fn compile_c_program(c_program: &str, output: &str, opt_level: u8, native: bool) {
    let mut args = vec!["-x", "c", "-"];
    // Optimization level
    let opt_level_arg = format!("-O{}", opt_level);
    args.push(&opt_level_arg);
    // Output
    let output_arg = format!("-o{}", output);
    args.push(&output_arg);
    // Build for native architecture
    if native {
        args.push("-march=native")
    }

    let mut cc = Command::new("cc")
        .args(&args)
        .stdin(Stdio::piped())
        .spawn()
        .expect("Could not start C compiler");

    let cc_stdin = cc
        .stdin
        .as_mut()
        .expect("Could not get stdin of C compiler");
    cc_stdin
        .write_all(c_program.as_bytes())
        .expect("Failed to write to C compiler");
}
