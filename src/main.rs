#![warn(trivial_numeric_casts)]
// option_unwrap_used is specific to clippy. However, we don't want to
// add clippy to the build requirements, so we build without it and
// ignore any warnings about rustc not recognising clippy's lints.
#![allow(unknown_lints)]
// TODO: enable this warning and cleanup.
#![allow(option_unwrap_used)]

//! bfc is a highly optimising compiler for BF.

extern crate ansi_term;
extern crate getopts;
extern crate itertools;
// extern crate llvm_sys;
#[cfg(test)]
extern crate pretty_assertions;
#[cfg(test)]
extern crate quickcheck;
extern crate tempfile;

#[macro_use]
extern crate matches;

use structopt::StructOpt;

use crate::diagnostics::{Info, Level};

use std::fs::File;
use std::io::prelude::Read;
use std::path::PathBuf;
use std::process::exit;

mod bfir;
mod bounds;
mod diagnostics;
mod execution;
mod peephole;

mod c;

#[cfg(test)]
mod peephole_tests;
#[cfg(test)]
mod soundness_tests;

// TODO: return a Vec<Info> that may contain warnings or errors,
// instead of printing in lots of different place shere.
fn compile_file(
    path: &str,
    output: &str,
    opt_level: u8,
    native: bool,
    dump_ir: bool,
    dump_c: bool,
) -> Result<(), String> {
    let src = match slurp_file_to_string(path) {
        Ok(src) => src,
        Err(info) => {
            return Err(format!("{}", info));
        }
    };

    let mut instrs = match bfir::parse(&src) {
        Ok(instrs) => instrs,
        Err(parse_error) => {
            let info = Info {
                level: Level::Error,
                filename: path.to_owned(),
                message: parse_error.message,
                position: Some(parse_error.position),
                source: Some(src),
            };
            return Err(format!("{}", info));
        }
    };

    if opt_level != 0 {
        // let pass_specification = matches.opt_str("passes");
        let pass_specification = None;
        let (opt_instrs, warnings) = peephole::optimize(instrs, &pass_specification);
        instrs = opt_instrs;

        for warning in warnings {
            let info = Info {
                level: Level::Warning,
                filename: path.to_owned(),
                message: warning.message,
                position: warning.position,
                source: Some(src.clone()),
            };
            eprintln!("{}", info);
        }
    }

    if dump_ir {
        for instr in &instrs {
            println!("{}", instr);
        }
        return Ok(());
    }

    let (state, execution_warning) = if opt_level == 2 {
        execution::execute(&instrs, 10_000_000)
    } else {
        let mut init_state = execution::ExecutionState::initial(&instrs[..]);
        // TODO: this will crash on the empty program.
        init_state.start_instr = Some(&instrs[0]);
        (init_state, None)
    };
    if let Some(execution_warning) = execution_warning {
        let info = Info {
            level: Level::Warning,
            filename: path.to_owned(),
            message: execution_warning.message,
            position: execution_warning.position,
            source: Some(src),
        };
        eprintln!("{}", info);
    }

    let c_program = c::c_prog_from_instructions(&instrs);
    if dump_c {
        println!("{}", c_program);
        return Ok(());
    }
    c::compile_c_program(&c_program, output, opt_level, native);

    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bfc", about = "Optimizing brainfuck compiler")]
struct Opt {
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    debug: bool,

    /// print BF IR generated
    #[structopt(long = "dump-ir")]
    dump_ir: bool,

    /// print C generated
    #[structopt(long = "dump-c")]
    dump_c: bool,

    /// optimize level (0 to 2)
    #[structopt(short = "O", default_value = "2")]
    opt_level: u8,

    /// build for the native architecture
    #[structopt(long = "native")]
    native: bool,

    /// strip symbols from the binary (default: yes)
    #[structopt(long = "strip")]
    strip: bool,

    // output binary
    #[structopt(short = "o", default_value = "a.out", parse(from_os_str))]
    output: PathBuf,

    // TODO: Replace with Vec<PathBuf>
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    if opt.opt_level > 3 {
        eprintln!("Optimization level must be one of: 0, 1, 2");
        exit(1);
    }

    match compile_file(
        opt.file.to_str().unwrap(),
        opt.output.to_str().unwrap(),
        opt.opt_level,
        opt.native,
        opt.dump_ir,
        opt.dump_c,
    ) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(2);
        }
    }
}

/// Read the contents of the file at path, and return a string of its
/// contents. Return a diagnostic if we can't open or read the file.
fn slurp_file_to_string(path: &str) -> Result<String, Info> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(message) => {
            return Err(Info {
                level: Level::Error,
                filename: path.to_owned(),
                message: format!("{}", message),
                position: None,
                source: None,
            });
        }
    };

    let mut contents = String::new();

    match file.read_to_string(&mut contents) {
        Ok(_) => Ok(contents),
        Err(message) => Err(Info {
            level: Level::Error,
            filename: path.to_owned(),
            message: format!("{}", message),
            position: None,
            source: None,
        }),
    }
}
