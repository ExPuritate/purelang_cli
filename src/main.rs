#![feature(once_cell_try)]
#![feature(error_iter)]

mod compile;
mod run;

use clap::{Args, Parser, Subcommand};
use std::fmt::Display;

#[derive(Parser, Clone, Debug)]
struct CliArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug)]
enum Command {
    #[command(name = "compile")]
    Compile(CompileArgs),
    #[command(name = "run")]
    Run(RunArgs),
}

#[derive(Args, Clone, Debug)]
struct CompileArgs {
    #[arg(long)]
    #[arg(default_value_t = match std::env::var("PURELANG_HOME") {
        Ok(val) => format!("{val}/Native/CompileService.module"),
        Err(_) => "CompileService".to_owned(),
    })]
    core: String,
    #[arg(long)]
    compilers: Vec<String>,
    #[arg(long, short = 's')]
    sources: Vec<String>,
}

#[derive(Args, Clone, Debug)]
struct RunArgs {
    #[arg(long)]
    #[arg(default_value_t = match std::env::var("PURELANG_HOME") {
        Ok(val) => format!("{val}/Native/Runtime.module"),
        Err(_) => "Runtime".to_owned(),
    })]
    core: String,
    #[arg(long = "assembly")]
    main_assembly_name: String,
    #[arg(long = "class")]
    main_class_name: String,
    #[arg(long)]
    assemblies: Vec<String>,
    #[arg(trailing_var_arg = true)]
    arguments: Vec<String>,
}

#[derive(thiserror::Error, Debug)]
pub struct Utf8Error;

impl Display for Utf8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

fn main() -> global::Result<()> {
    let args = CliArgs::try_parse()?;
    match args.command {
        Command::Compile(args) => compile::handle(args),
        Command::Run(args) => run::handle(args),
    }
}
